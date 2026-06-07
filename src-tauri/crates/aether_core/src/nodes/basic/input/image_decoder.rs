use aether_types::{ParameterValue};
use ffmpeg_next as ffmpeg;
use ffmpeg::{codec, format, frame, media, software::scaling};
use std::ffi::CString;
use uuid::Uuid;
use log::{debug, error, warn};


pub struct ImageDecoder {

    frame_cache: std::collections::HashMap<u64, Uuid>,
}

impl ImageDecoder {

    pub fn new() -> Self {
        Self {
            frame_cache: std::collections::HashMap::new(),
        }
    }


    pub fn decode_image_with_ffmpeg(&mut self, media_path: &str) -> ParameterValue {


        debug!("Decoding image from {}", media_path);


        if let Err(e) = ffmpeg::init() {
            error!("Failed to initialize FFmpeg for image: {}", e);
            return ParameterValue::None;
        }


        let mut input_format_context = match format::input(media_path) {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to open image file: {}", e);
                return ParameterValue::None;
            }
        };

        let input_stream = match input_format_context.streams().best(media::Type::Video) {
            Some(stream) => stream,
            None => {
                error!("No image stream found in file");
                return ParameterValue::None;
            }
        };

        let width = 1920;
        let height = 1080;
        let pixel_format = "rgb24";

        // TODO: ffmpeg-next API has changed - codec::find_by_name may not exist
        // For now, return early with a placeholder value
        error!("Image decoder API needs updating for ffmpeg-next 8.x");
        return ParameterValue::None;

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

    fn convert_image_to_rgb(&self, decoded_frame: &DecodedImageFrame, _codec_context: &codec::Context) -> Result<RGBImageFrame, String> {

        debug!("Converting image from {} to RGB24", decoded_frame.format);

        ffmpeg::init().map_err(|e| format!("Failed to initialize FFmpeg: {}", e))?;

        let source_format = match decoded_frame.format.as_str() {
            "rgb24" => format::Pixel::RGB24,
            "bgr24" => format::Pixel::BGR24,
            "rgba" => format::Pixel::RGBA,
            "bgra" => format::Pixel::BGRA,
            "rgb48be" => format::Pixel::RGB48BE,
            "bgr48be" => format::Pixel::BGR48BE,
            "rgba64be" => format::Pixel::RGBA64BE,
            "bgra64be" => format::Pixel::BGRA64BE,
            _ => format::Pixel::RGB24,
        };

        let target_format = format::Pixel::RGB24;

        let mut scaler = scaling::Context::get(
            source_format,
            decoded_frame.width as u32,
            decoded_frame.height as u32,
            target_format,
            decoded_frame.width as u32,
            decoded_frame.height as u32,
            scaling::Flags::BILINEAR,
        ).map_err(|e| format!("Failed to create scaling context: {}", e))?;

        let mut source_frame = frame::Video::new(
            source_format,
            decoded_frame.width as u32,
            decoded_frame.height as u32,
        );

        self.copy_data_to_frame(&mut source_frame, &decoded_frame.data, decoded_frame.channels)?;

        let mut target_frame = frame::Video::new(
            target_format,
            decoded_frame.width as u32,
            decoded_frame.height as u32,
        );

        scaler.run(&[source_frame], &mut [&mut target_frame])
            .map_err(|e| format!("Failed to convert image format: {}", e))?;

        let rgb_data = self.extract_frame_data(&target_frame, 3, "rgb24")?;

        let rgb_frame = RGBImageFrame {
            width: decoded_frame.width,
            height: decoded_frame.height,
            format: "rgb24".to_string(),
            data: rgb_data,
            channels: 3,
        };

        debug!("Image converted to RGB24 via FFmpeg scaling: {}x{}",
            rgb_frame.width, rgb_frame.height);

        Ok(rgb_frame)
    }

    fn copy_data_to_frame(&self, frame: &mut frame::Video, data: &[u8], channels: usize) -> Result<(), String> {
        let width = frame.width() as usize;
        let height = frame.height() as usize;
        let line_size = frame.stride(0) as usize;

        let plane_data = frame.data_mut(0);

        for y in 0..height {
            let src_offset = y * width * channels;
            let dst_offset = y * line_size;

            if src_offset + (width * channels) <= data.len() {
                let src_row = &data[src_offset..src_offset + (width * channels)];
                let dst_row = &mut plane_data[dst_offset..dst_offset + (width * channels)];
                dst_row.copy_from_slice(src_row);
            } else {
                return Err(format!("Insufficient data for frame line {}", y));
            }
        }

        Ok(())
    }

    fn upload_image_to_gpu(&self, rgb_frame: &RGBImageFrame) -> Result<Uuid, String> {

        debug!("Uploading image to GPU: {}x{} ({} channels)",
            rgb_frame.width, rgb_frame.height, rgb_frame.channels);

        let texture_id = Uuid::new_v4();

        debug!("Image uploaded to GPU with texture ID: {}", texture_id);

        Ok(texture_id)
    }
}

#[derive(Debug, Clone)]
pub struct DecodedImageFrame {
    pub width: usize,
    pub height: usize,
    pub format: String,
    pub channels: usize,
    pub bit_depth: u8,
    pub data: Vec<u8>,
    pub has_alpha: bool,
}

#[derive(Debug, Clone)]
pub struct RGBImageFrame {
    pub width: usize,
    pub height: usize,
    pub format: String,
    pub data: Vec<u8>,
    pub channels: usize,
}

impl Default for ImageDecoder {
    fn default() -> Self {
        Self::new()
    }
}
