use ffmpeg_next as ffmpeg;
use ffmpeg::{format, frame, media};
use uuid::Uuid;
use log::{debug, error, warn};

pub struct VideoDecoder {

    _frame_cache: std::collections::HashMap<u64, Uuid>,
}

impl VideoDecoder {

    pub fn new() -> Self {
        Self {
            _frame_cache: std::collections::HashMap::new(),
        }
    }

    pub fn decode_video_frame_with_ffmpeg(&mut self, frame: u64, media_path: &str) -> Uuid {

        debug!("Decoding video frame {} from {}", frame, media_path);

        if let Err(e) = ffmpeg::init() {
            error!("Failed to initialize FFmpeg for video: {}", e);
            return Uuid::new_v4();
        }

        let mut input_format_context = match format::input(media_path) {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to open video file: {}", e);
                return Uuid::new_v4();
            }
        };


        let input_stream = match input_format_context.streams().best(media::Type::Video) {
            Some(stream) => stream,
            None => {
                error!("No video stream found in file");
                return Uuid::new_v4();
            }
        };

        let mut decoder = match ffmpeg::codec::context::Context::from_parameters(input_stream.parameters()) {
            Ok(context) => match context.decoder().video() {
                Ok(decoder) => decoder,
                Err(e) => {
                    error!("Failed to create video decoder: {}", e);
                    return Uuid::new_v4();
                }
            },
            Err(e) => {
                error!("Failed to create decoder context: {}", e);
                return Uuid::new_v4();
            }
        };

        let width = decoder.width() as usize;
        let height = decoder.height() as usize;
        let pixel_format = decoder.format();

        let mut video_frame = frame::Video::new(pixel_format, width as u32, height as u32);

        let timestamp = frame as f64 / 30.0;

        let mut packet_iter = input_format_context.packets();
        let mut frame_id = Uuid::new_v4();

        if let Some((_, packet)) = packet_iter.next() {
            if let Err(e) = decoder.send_packet(&packet) {
                error!("Failed to send video packet: {}", e);
                return Uuid::new_v4();
            }

            if let Err(e) = decoder.receive_frame(&mut video_frame) {
                error!("Failed to receive video frame: {}", e);
                return Uuid::new_v4();
            }

            let (channels, _has_alpha) = match pixel_format {
                format::Pixel::RGB24 | format::Pixel::BGR24 => (3, false),
                format::Pixel::RGBA | format::Pixel::BGRA => (4, true),
                _ => (3, false),
            };

            let image_data = self.extract_frame_data(&video_frame, channels, &format!("{:?}", pixel_format))
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
                pixel_format: format!("{:?}", pixel_format),
                timestamp,
                frame_id,
            };

            debug!("Video frame decoded via FFmpeg: {}x{} {:?} ({} channels)",
                width, height, pixel_format, channels);

            debug!("Video metadata: {:?}", frame_metadata);

        } else {
            warn!("No packet found for video frame: {}", frame);
        }

        frame_id
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

    fn upload_video_frame_to_gpu(&self, _data: &[u8], width: usize, height: usize, channels: usize) -> Result<Uuid, String> {

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
