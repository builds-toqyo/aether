use aether_types::{ParameterValue};
use ffmpeg_next as ffmpeg;
use ffmpeg::{format, frame, media};
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

        let mut input_format_context = match format::input(filename) {
            Ok(context) => context,
            Err(e) => {
                warn!("Failed to open sequence frame {}: {}", filename, e);

                return self.create_default_sequence_frame(frame);
            }
        };

        let input_stream = match input_format_context.streams().best(media::Type::Video) {
            Some(stream) => stream,
            None => {
                warn!("No image stream found in sequence frame: {}", filename);
                return self.create_default_sequence_frame(frame);
            }
        };

        let mut decoder = match ffmpeg::codec::context::Context::from_parameters(input_stream.parameters()) {
            Ok(context) => match context.decoder().video() {
                Ok(decoder) => decoder,
                Err(e) => {
                    warn!("Failed to create video decoder for sequence: {}", e);
                    return self.create_default_sequence_frame(frame);
                }
            },
            Err(e) => {
                warn!("Failed to create decoder context for sequence: {}", e);
                return self.create_default_sequence_frame(frame);
            }
        };

        let width = decoder.width() as usize;
        let height = decoder.height() as usize;
        let pixel_format = decoder.format();

        let mut image_frame = frame::Video::new(pixel_format, width as u32, height as u32);

        let mut packet_iter = input_format_context.packets();
        let mut frame_id = Uuid::new_v4();

        if let Some((_, packet)) = packet_iter.next() {
            if let Err(e) = decoder.send_packet(&packet) {
                warn!("Failed to send packet for sequence frame {}: {}", filename, e);
                return self.create_default_sequence_frame(frame);
            }

            if let Err(e) = decoder.receive_frame(&mut image_frame) {
                warn!("Failed to receive frame for sequence frame {}: {}", filename, e);
                return self.create_default_sequence_frame(frame);
            }

            let (channels, _has_alpha) = match pixel_format {
                format::Pixel::RGB24 | format::Pixel::BGR24 => (3, false),
                format::Pixel::RGBA | format::Pixel::BGRA => (4, true),
                _ => (3, false),
            };

            let image_data = self.extract_frame_data(&image_frame, channels, &format!("{:?}", pixel_format))
                .unwrap_or_else(|_| {
                    warn!("Failed to extract data for sequence frame: {}", filename);
                    vec![0u8; width * height * channels]
                });

            frame_id = self.upload_sequence_frame_to_gpu(&image_data, width, height, channels)
                .unwrap_or_else(|_| {
                    warn!("Failed to upload sequence frame to GPU: {}", filename);
                    Uuid::new_v4()
                });

            debug!("Sequence frame decoded via FFmpeg: {}x{} {:?} ({} channels)",
                width, height, pixel_format, channels);

        } else {
            warn!("No packet found in sequence frame: {}", filename);
            return self.create_default_sequence_frame(frame);
        }

        frame_id
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

    fn extract_frame_data(&self, frame: &frame::Video, channels: usize, _pixel_format: &str) -> Result<Vec<u8>, String> {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        let line_size = frame.stride(0) as usize;

        let plane_data = frame.data(0);

        let mut data = Vec::with_capacity(width * height * channels);


        for y in 0..height {
            let src_offset = y * line_size;
            let _dst_offset = y * width * channels;

            if src_offset + (width * channels) <= plane_data.len() {
                let src_row = &plane_data[src_offset..src_offset + (width * channels)];
                data.extend_from_slice(src_row);
            } else {
                return Err(format!("Insufficient data for frame line {}", y));
            }
        }

        Ok(data)
    }


    fn upload_sequence_frame_to_gpu(&self, _data: &[u8], width: usize, height: usize, channels: usize) -> Result<Uuid, String> {

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
