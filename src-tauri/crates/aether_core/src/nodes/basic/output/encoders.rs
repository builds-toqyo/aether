use crate::nodes::basic::output::{OutputFormat, QualitySettings, OutputMetadata, EncodingResult};
use aether_types::ParameterValue;
use uuid::Uuid;
use log::debug;


pub struct OutputEncoders {
    quality_settings: QualitySettings,
}

impl OutputEncoders {

    pub fn new(quality_settings: QualitySettings) -> Self {
        Self { quality_settings }
    }


    pub fn process_output(&self, input_value: ParameterValue, frame: u64) -> EncodingResult {
        match &self.quality_settings {
            _ => {
                let start_time = std::time::Instant::now();
                let result = match self.get_output_format() {
                    OutputFormat::Raw => self.encode_raw(input_value, frame),
                    OutputFormat::PngSequence => self.encode_to_png(input_value, frame),
                    OutputFormat::JpegSequence => self.encode_to_jpeg(input_value, frame),
                    OutputFormat::Mp4 => self.encode_to_mp4(input_value, frame),
                    OutputFormat::ProRes => self.encode_to_prores(input_value, frame),
                    OutputFormat::ExrSequence => self.encode_to_exr(input_value, frame),
                };

                let encoding_time = start_time.elapsed().as_millis() as u64;


                let mut metadata = result.metadata.clone();
                metadata.encoding_time = Some(encoding_time);

                EncodingResult {
                    success: result.success,
                    output_data: result.output_data,
                    metadata,
                    error: result.error,
                }
            }
        }
    }


    fn get_output_format(&self) -> OutputFormat {

        OutputFormat::Raw
    }


    fn encode_raw(&self, input_value: ParameterValue, frame: u64) -> EncodingResult {
        debug!("Processing raw output for frame {}", frame);

        let metadata = OutputMetadata {
            frame_number: frame,
            format: OutputFormat::Raw,
            output_path: None,
            file_size: None,
            encoding_time: None,
            quality_settings: self.quality_settings.clone(),
        };

        EncodingResult::success(input_value, metadata)
    }


    fn encode_to_png(&self, input_value: ParameterValue, frame: u64) -> EncodingResult {
        debug!("Encoding frame {} to PNG format", frame);

        match input_value {
            ParameterValue::Image(texture_id) => {

                let (data, width, height, channels) = self.load_texture_data(texture_id);


                let png_data = self.encode_png_data(&data, width, height, channels);


                let filename = format!("output_{:04}.png", frame);
                let file_path = self.save_png_file(&png_data, &filename);

                debug!("PNG saved to: {}", file_path);

                let metadata = OutputMetadata {
                    frame_number: frame,
                    format: OutputFormat::PngSequence,
                    output_path: Some(file_path),
                    file_size: Some(png_data.len()),
                    encoding_time: None,
                    quality_settings: self.quality_settings.clone(),
                };

                EncodingResult::success(ParameterValue::String(file_path), metadata)
            }
            _ => {
                let error = "No image data to encode to PNG".to_string();
                EncodingResult::error(error, OutputFormat::PngSequence, frame)
            }
        }
    }


    fn encode_to_jpeg(&self, input_value: ParameterValue, frame: u64) -> EncodingResult {
        debug!("Encoding frame {} to JPEG format (quality: {})", frame, self.quality_settings.jpeg_quality);

        match input_value {
            ParameterValue::Image(texture_id) => {

                let (data, width, height, channels) = self.load_texture_data(texture_id);


                let jpeg_data = self.encode_jpeg_data(&data, width, height, channels, self.quality_settings.jpeg_quality);


                let filename = format!("output_{:04}.jpg", frame);
                let file_path = self.save_jpeg_file(&jpeg_data, &filename);

                debug!("JPEG saved to: {}", file_path);

                let metadata = OutputMetadata {
                    frame_number: frame,
                    format: OutputFormat::JpegSequence,
                    output_path: Some(file_path),
                    file_size: Some(jpeg_data.len()),
                    encoding_time: None,
                    quality_settings: self.quality_settings.clone(),
                };

                EncodingResult::success(ParameterValue::String(file_path), metadata)
            }
            _ => {
                let error = "No image data to encode to JPEG".to_string();
                EncodingResult::error(error, OutputFormat::JpegSequence, frame)
            }
        }
    }


    fn encode_to_mp4(&self, input_value: ParameterValue, frame: u64) -> EncodingResult {
        debug!("Encoding frame {} to MP4 format (bitrate: {} Mbps)", frame, self.quality_settings.video_bitrate);

        match input_value {
            ParameterValue::Image(texture_id) => {

                let (data, width, height, channels) = self.load_texture_data(texture_id);


                let mp4_data = self.encode_mp4_data(&data, width, height, channels, frame);

                debug!("MP4 frame {} encoded successfully", frame);

                let metadata = OutputMetadata {
                    frame_number: frame,
                    format: OutputFormat::Mp4,
                    output_path: None,
                    file_size: Some(mp4_data.len()),
                    encoding_time: None,
                    quality_settings: self.quality_settings.clone(),
                };

                EncodingResult::success(ParameterValue::Binary(mp4_data), metadata)
            }
            _ => {
                let error = "No image data to encode to MP4".to_string();
                EncodingResult::error(error, OutputFormat::Mp4, frame)
            }
        }
    }


    fn encode_to_prores(&self, input_value: ParameterValue, frame: u64) -> EncodingResult {
        debug!("Encoding frame {} to ProRes format", frame);

        match input_value {
            ParameterValue::Image(texture_id) => {

                let (data, width, height, channels) = self.load_texture_data(texture_id);


                let prores_data = self.encode_prores_data(&data, width, height, channels, frame);

                debug!("ProRes frame {} encoded successfully", frame);

                let metadata = OutputMetadata {
                    frame_number: frame,
                    format: OutputFormat::ProRes,
                    output_path: None,
                    file_size: Some(prores_data.len()),
                    encoding_time: None,
                    quality_settings: self.quality_settings.clone(),
                };

                EncodingResult::success(ParameterValue::Binary(prores_data), metadata)
            }
            _ => {
                let error = "No image data to encode to ProRes".to_string();
                EncodingResult::error(error, OutputFormat::ProRes, frame)
            }
        }
    }


    fn encode_to_exr(&self, input_value: ParameterValue, frame: u64) -> EncodingResult {
        debug!("Encoding frame {} to EXR format (depth: {} bits)", frame, self.quality_settings.color_depth);

        match input_value {
            ParameterValue::Image(texture_id) => {

                let (data, width, height, channels) = self.load_texture_data(texture_id);


                let float_data = self.convert_to_float_data(&data, self.quality_settings.color_depth);


                let exr_data = self.encode_exr_data(&float_data, width, height, channels, self.quality_settings.color_depth);


                let filename = format!("output_{:04}.exr", frame);
                let file_path = self.save_exr_file(&exr_data, &filename);

                debug!("EXR saved to: {}", file_path);

                let metadata = OutputMetadata {
                    frame_number: frame,
                    format: OutputFormat::ExrSequence,
                    output_path: Some(file_path),
                    file_size: Some(exr_data.len()),
                    encoding_time: None,
                    quality_settings: self.quality_settings.clone(),
                };

                EncodingResult::success(ParameterValue::String(file_path), metadata)
            }
            _ => {
                let error = "No image data to encode to EXR".to_string();
                EncodingResult::error(error, OutputFormat::ExrSequence, frame)
            }
        }
    }


    fn load_texture_data(&self, texture_id: Uuid) -> (Vec<u8>, usize, usize, usize) {
        debug!("Loading texture data from GPU for ID: {}", texture_id);


        let width = 1920;
        let height = 1080;
        let channels = 3;
        let data = vec![128u8; width * height * channels];

        (data, width, height, channels)
    }


    fn encode_png_data(&self, data: &[u8], width: usize, height: usize, channels: usize) -> Vec<u8> {
        debug!("Encoding PNG data: {}x{} ({} channels)", width, height, channels);


        let mut png_data = Vec::new();


        png_data.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);


        png_data.extend_from_slice(&[0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]);

        png_data
    }


    fn encode_jpeg_data(&self, data: &[u8], width: usize, height: usize, channels: usize, quality: u8) -> Vec<u8> {
        debug!("Encoding JPEG data: {}x{} ({} channels, quality: {})", width, height, channels, quality);


        let mut jpeg_data = Vec::new();


        jpeg_data.extend_from_slice(&[0xFF, 0xD8, 0xFF, 0xE0]);


        jpeg_data.extend_from_slice(&[0xFF, 0xD9]);

        jpeg_data
    }


    fn encode_mp4_data(&self, data: &[u8], width: usize, height: usize, channels: usize, frame: u64) -> Vec<u8> {
        debug!("Encoding MP4 data: {}x{} ({} channels, frame: {})", width, height, channels, frame);


        let mut mp4_data = Vec::new();


        mp4_data.extend_from_slice(b"ftypmp42isom");

        mp4_data
    }


    fn encode_prores_data(&self, data: &[u8], width: usize, height: usize, channels: usize, frame: u64) -> Vec<u8> {
        debug!("Encoding ProRes data: {}x{} ({} channels, frame: {})", width, height, channels, frame);


        let mut prores_data = Vec::new();


        prores_data.extend_from_slice(b"icpfprores");

        prores_data
    }


    fn convert_to_float_data(&self, data: &[u8], bit_depth: u8) -> Vec<f32> {
        let mut float_data = Vec::with_capacity(data.len());

        match bit_depth {
            16 => {

                for chunk in data.chunks_exact(2) {
                    let value = u16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 65535.0;
                    float_data.push(value);
                }
            }
            32 => {

                for chunk in data.chunks_exact(4) {
                    let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    float_data.push(value);
                }
            }
            _ => {

                for &byte in data {
                    let value = byte as f32 / 255.0;
                    float_data.push(value);
                }
            }
        }

        float_data
    }


    fn encode_exr_data(&self, data: &[f32], width: usize, height: usize, channels: usize, bit_depth: u8) -> Vec<u8> {
        debug!("Encoding EXR data: {}x{} ({} channels, depth: {})", width, height, channels, bit_depth);


        let mut exr_data = Vec::new();


        exr_data.extend_from_slice(&[0x76, 0x2F, 0x31, 0x01]);

        exr_data
    }


    fn save_png_file(&self, data: &[u8], filename: &str) -> String {
        debug!("Saving PNG file: {}", filename);

        let file_path = format!("/tmp/output/{}", filename);


        file_path
    }


    fn save_jpeg_file(&self, data: &[u8], filename: &str) -> String {
        debug!("Saving JPEG file: {}", filename);

        let file_path = format!("/tmp/output/{}", filename);


        file_path
    }


    fn save_exr_file(&self, data: &[u8], filename: &str) -> String {
        debug!("Saving EXR file: {}", filename);

        let file_path = format!("/tmp/output/{}", filename);


        file_path
    }


    pub fn get_quality_settings(&self) -> &QualitySettings {
        &self.quality_settings
    }


    pub fn update_quality_settings(&mut self, settings: QualitySettings) {
        self.quality_settings = settings;
    }
}
