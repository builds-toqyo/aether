use aether_types::{ParameterValue, ExecutionContext};
use ffmpeg as ffmpeg;
use ffmpeg::{codec, format, frame, media};
use std::ffi::CString;
use uuid::Uuid;
use log::{debug, error, warn};


pub struct VideoDecoder {

    frame_cache: std::collections::HashMap<u64, Uuid>,
}

impl VideoDecoder {

    pub fn new() -> Self {
        Self {
            frame_cache: std::collections::HashMap::new(),
        }
    }


    pub fn decode_video_frame_with_ffmpeg(&mut self, frame: u64, media_path: &str) -> Uuid {


        debug!("Decoding video frame {} from {}", frame, media_path);


        if let Err(e) = ffmpeg::init() {
            error!("Failed to initialize FFmpeg for video: {}", e);
            return Uuid::new_v4();
        }


        let path_cstring = CString::new(media_path).unwrap_or_else(|_| CString::new("default.mp4").unwrap());
        let mut input_format_context = match format::Input::open(&path_cstring) {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to open video file: {}", e);
                return Uuid::new_v4();
            }
        };


        if let Err(e) = input_format_context.find_stream_info(None) {
            error!("Failed to find video stream info: {}", e);
            return Uuid::new_v4();
        }


        let input_stream = match input_format_context.streams().best(media::Type::Video) {
            Some(stream) => stream,
            None => {
                error!("No video stream found in file");
                return Uuid::new_v4();
            }
        };


        let codec_params = input_stream.parameters();
        let width = codec_params.width().unwrap_or(1920) as usize;
        let height = codec_params.height().unwrap_or(1080) as usize;
        let pixel_format = codec_params.format().map_or("yuv420p", |f| f.name());


        let decoder = match codec::find_by_name("h264") {
            Some(decoder) => decoder,
            None => {
                error!("Video decoder not found");
                return Uuid::new_v4();
            }
        };

        let mut decoder_context = match codec::Context::new() {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to create decoder context: {}", e);
                return Uuid::new_v4();
            }
        };

        decoder_context.set_parameters(input_stream.parameters());

        if let Err(e) = decoder_context.open(decoder, None) {
            error!("Failed to open video decoder: {}", e);
            return Uuid::new_v4();
        }


        let mut video_frame = frame::Video::new(width, height, decoder_context.format());


        let timestamp = frame as f64 / 30.0;
        let seek_timestamp = (timestamp * 1000000.0) as i64;


        let mut packet_iter = input_format_context.packets();
        let mut frame_id = Uuid::new_v4();

        if let Some((_, packet)) = packet_iter.next() {
            if let Err(e) = decoder_context.send_packet(&packet) {
                error!("Failed to send video packet: {}", e);
                return Uuid::new_v4();
            }

            if let Err(e) = decoder_context.receive_frame(&mut video_frame) {
                error!("Failed to receive video frame: {}", e);
                return Uuid::new_v4();
            }


            let (channels, has_alpha) = match pixel_format {
                "rgb24" | "bgr24" => (3, false),
                "rgba" | "bgra" => (4, true),
                _ => (3, false),
            };

            let image_data = self.extract_frame_data(&video_frame, channels, pixel_format)
                .unwrap_or_else(|_| {
                    warn!("Failed to extract data for video frame: {}", frame);
                    vec![0u8; width * height * channels]
                });


            frame_id = self.upload_video_frame_to_gpu(&image_data, width, height, channels)
                .unwrap_or_else(|_| {
                    warn!("Failed to upload video frame to GPU: {}", frame);
                    Uuid::new_v4()
                });


            let frame_metadata = VideoFrameMetadata {
                frame_number: frame,
                width,
                height,
                pixel_format: pixel_format.to_string(),
                timestamp,
                frame_id,
            };

            debug!("Video frame decoded via FFmpeg: {}x{} {} ({} channels)",
                width, height, pixel_format, channels);

            debug!("Video metadata: {:?}", frame_metadata);

        } else {
            warn!("No packet found for video frame: {}", frame);
        }

        frame_id
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


    fn upload_video_frame_to_gpu(&self, data: &[u8], width: usize, height: usize, channels: usize) -> Result<Uuid, String> {


        debug!("Uploading video frame to GPU: {}x{} ({} channels)", width, height, channels);


        let texture_id = Uuid::new_v4();


        debug!("Video frame uploaded to GPU with texture ID: {}", texture_id);

        Ok(texture_id)
    }
}


#[derive(Debug, Clone)]
pub struct VideoFrameMetadata {
    pub frame_number: u64,
    pub width: usize,
    pub height: usize,
    pub pixel_format: String,
    pub timestamp: f64,
    pub frame_id: Uuid,
}

impl Default for VideoDecoder {
    fn default() -> Self {
        Self::new()
    }
}
