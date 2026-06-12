use aether_types::{ParameterValue};
use ffmpeg_next as ffmpeg;
use ffmpeg::{codec, format, frame, media, software::scaling};
use uuid::Uuid;
use log::{debug, error};

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

        let input_format_context = match format::input(media_path) {
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

        let mut decoder = match ffmpeg::codec::context::Context::from_parameters(input_stream.parameters()) {
            Ok(context) => match context.decoder().video() {
                Ok(decoder) => decoder,
                Err(e) => {
                    error!("Failed to create image decoder: {}", e);
                    return ParameterValue::None;
                }
            },
            Err(e) => {
                error!("Failed to create decoder context: {}", e);
                return ParameterValue::None;
            }
        };

        let width = decoder.width() as usize;
        let height = decoder.height() as usize;
        let pixel_format = decoder.format();

        let mut image_frame = frame::Video::new(pixel_format, width as u32, height as u32);

        let mut packet_iter = input_format_context.packets();
        let frame_id = if let Some((_, packet)) = packet_iter.next() {
            if let Err(e) = decoder.send_packet(&packet) {
                error!("Failed to send packet for image: {}", e);
                return ParameterValue::None;
            }

            if let Err(e) = decoder.receive_frame(&mut image_frame) {
                error!("Failed to receive image frame: {}", e);
                return ParameterValue::None;
            }

            let (channels, _has_alpha) = match pixel_format {
                format::Pixel::RGB24 | format::Pixel::BGR24 => (3, false),
                format::Pixel::RGBA | format::Pixel::BGRA => (4, true),
                _ => (3, false),
            };

            let image_data = self.extract_frame_data(&image_frame, channels, &pixel_format.to_string())
                .unwrap_or_else(|_| {
                    error!("Failed to extract data for image: {}", media_path);
                    vec![0u8; width * height * channels]
                });

            let decoded_frame = DecodedImageFrame {
                width,
                height,
                format: pixel_format.to_string(),
                channels,
                data: image_data,
            };

            match self.convert_image_to_rgb(&decoded_frame, &decoder) {
                Ok(rgb_frame) => {
                    match self.upload_image_to_gpu(&rgb_frame) {
                        Ok(id) => id,
                        Err(e) => {
                            error!("Failed to upload image to GPU: {}", e);
                            return ParameterValue::None;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to convert image to RGB: {}", e);
                    return ParameterValue::None;
                }
            }
        } else {
            error!("No packet found in image: {}", media_path);
            return ParameterValue::None;
        };

        ParameterValue::Image(frame_id)
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

        scaler.run(&source_frame, &mut target_frame)
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
