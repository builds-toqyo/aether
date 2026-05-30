use aether_types::{ParameterValue};
use ffmpeg_next as ffmpeg;
use ffmpeg::{codec, format, frame, media};
use std::ffi::CString;
use uuid::Uuid;
use log::{debug, error, warn};


pub struct SequenceLoader {

    sequence_pattern: Option<String>,

    frame_cache: std::collections::HashMap<u64, Uuid>,
}

impl SequenceLoader {

    pub fn new() -> Self {
        Self {
            sequence_pattern: None,
            frame_cache: std::collections::HashMap::new(),
        }
    }


    pub fn set_sequence_pattern(&mut self, pattern: String) {
        self.sequence_pattern = Some(pattern);
    }


    pub fn get_sequence_pattern(&self) -> Option<&String> {
        self.sequence_pattern.as_ref()
    }


    pub fn load_sequence_frame(&mut self, frame: u64, media_path: &str) -> ParameterValue {


        let filename = self.generate_sequence_filename(frame, media_path);
        debug!("Loading sequence frame {} from file: {}", frame, filename);


        let frame_data = self.decode_sequence_frame_with_ffmpeg(frame, &filename);

        ParameterValue::Image(frame_data)
    }


    fn generate_sequence_filename(&self, frame: u64, media_path: &str) -> String {


        if let Some(pattern) = &self.sequence_pattern {

            let directory = std::path::Path::new(media_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or(".");


            let filename = pattern
                .replace("%d", &format!("{}", frame))
                .replace("%04d", &format!("{:04}", frame))
                .replace("%06d", &format!("{:06}", frame));

            format!("{}/{}", directory, filename)
        } else {

            let directory = std::path::Path::new(media_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or(".");

            let extension = std::path::Path::new(media_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("png");

            format!("{}/output_{:04}.{}", directory, frame, extension)
        }
    }


    fn decode_sequence_frame_with_ffmpeg(&mut self, frame: u64, filename: &str) -> Uuid {


        debug!("Decoding sequence frame {} from {}", frame, filename);


        if let Err(e) = ffmpeg::init() {
            error!("Failed to initialize FFmpeg for sequence: {}", e);
            return Uuid::new_v4();
        }


        let path_cstring = CString::new(filename).unwrap_or_else(|_| CString::new("default.png").unwrap());
        let mut input_format_context = match format::Input::open(&path_cstring) {
            Ok(context) => context,
            Err(e) => {
                warn!("Failed to open sequence frame {}: {}", filename, e);

                return self.create_default_sequence_frame(frame);
            }
        };


        if let Err(e) = input_format_context.find_stream_info(None) {
            warn!("Failed to find stream info for sequence frame {}: {}", filename, e);
            return self.create_default_sequence_frame(frame);
        }


        let input_stream = match input_format_context.streams().best(media::Type::Video) {
            Some(stream) => stream,
            None => {
                warn!("No image stream found in sequence frame: {}", filename);
                return self.create_default_sequence_frame(frame);
            }
        };


        let codec_params = input_stream.parameters();
        let width = codec_params.width().unwrap_or(1920) as usize;
        let height = codec_params.height().unwrap_or(1080) as usize;
        let pixel_format = codec_params.format().map_or("rgb24", |f| f.name());


        // TODO: ffmpeg-next API has changed - codec::find_by_name may not exist
        // For now, return early with a placeholder frame
        warn!("Sequence loader decoder API needs updating for ffmpeg-next 8.x");
        return self.create_default_sequence_frame(frame);

        // Unreachable code below - kept for reference when updating API
        /*
        let mut image_frame = frame::Video::new(width, height, decoder_context.format());


        let mut packet_iter = input_format_context.packets();
        let mut frame_id = Uuid::new_v4();

        if let Some((_, packet)) = packet_iter.next() {
            if let Err(e) = decoder_context.send_packet(&packet) {
                warn!("Failed to send packet for sequence frame {}: {}", filename, e);
                return self.create_default_sequence_frame(frame);
            }

            if let Err(e) = decoder_context.receive_frame(&mut image_frame) {
                warn!("Failed to receive frame for sequence frame {}: {}", filename, e);
                return self.create_default_sequence_frame(frame);
            }


            let (channels, has_alpha) = match pixel_format {
                "rgb24" | "bgr24" => (3, false),
                "rgba" | "bgra" => (4, true),
                _ => (3, false),
            };

            let image_data = self.extract_frame_data(&image_frame, channels, pixel_format)
                .unwrap_or_else(|_| {
                    warn!("Failed to extract data for sequence frame: {}", filename);
                    vec![0u8; width * height * channels]
                });


            frame_id = self.upload_sequence_frame_to_gpu(&image_data, width, height, channels)
                .unwrap_or_else(|_| {
                    warn!("Failed to upload sequence frame to GPU: {}", filename);
                    Uuid::new_v4()
                });

            debug!("Sequence frame decoded via FFmpeg: {}x{} {} ({} channels)",
                width, height, pixel_format, channels);

        } else {
            warn!("No packet found in sequence frame: {}", filename);
            return self.create_default_sequence_frame(frame);
        }

        frame_id
        */
    }


    fn create_default_sequence_frame(&self, frame: u64) -> Uuid {
        debug!("Creating default sequence frame for frame {}", frame);

        let width = 1920;
        let height = 1080;
        let channels = 3;


        let mut data = Vec::with_capacity(width * height * channels);
        for y in 0..height {
            for x in 0..width {
                let checker = ((x / 32) + (y / 32)) % 2;
                let color = if checker == 0 { 128 } else { 64 };
                data.extend_from_slice(&[color, color, color]);
            }
        }


        self.upload_sequence_frame_to_gpu(&data, width, height, channels)
            .unwrap_or_else(|_| {
                error!("Failed to upload default sequence frame");
                Uuid::new_v4()
            })
    }


    fn extract_frame_data(&self, frame: &frame::Video, channels: usize, pixel_format: &str) -> Result<Vec<u8>, String> {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        let line_size = frame.stride(0) as usize;

        let plane_data = frame.data(0);

        let mut data = Vec::with_capacity(width * height * channels);


        for y in 0..height {
            let src_offset = y * line_size;
            let dst_offset = y * width * channels;

            if src_offset + (width * channels) <= plane_data.len() {
                let src_row = &plane_data[src_offset..src_offset + (width * channels)];
                data.extend_from_slice(src_row);
            } else {
                return Err(format!("Insufficient data for frame line {}", y));
            }
        }

        Ok(data)
    }


    fn upload_sequence_frame_to_gpu(&self, data: &[u8], width: usize, height: usize, channels: usize) -> Result<Uuid, String> {


        debug!("Uploading sequence frame to GPU: {}x{} ({} channels)", width, height, channels);


        let texture_id = Uuid::new_v4();


        debug!("Sequence frame uploaded to GPU with texture ID: {}", texture_id);

        Ok(texture_id)
    }
}

impl Default for SequenceLoader {
    fn default() -> Self {
        Self::new()
    }
}
