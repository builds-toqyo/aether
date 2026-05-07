use uuid::Uuid;


#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {

    Raw,

    PngSequence,

    JpegSequence,

    Mp4,

    ProRes,

    ExrSequence,
}


#[derive(Debug, Clone)]
pub struct QualitySettings {

    pub jpeg_quality: u8,

    pub png_compression: u8,

    pub video_bitrate: u32,

    pub color_depth: u8,
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            jpeg_quality: 95,
            png_compression: 6,
            video_bitrate: 50,
            color_depth: 8,
        }
    }
}

impl QualitySettings {

    pub fn new() -> Self {
        Self::default()
    }


    pub fn validate(&mut self) {
        self.jpeg_quality = self.jpeg_quality.clamp(1, 100);
        self.png_compression = self.png_compression.clamp(0, 9);
        self.video_bitrate = self.video_bitrate.clamp(1, 1000);
        self.color_depth = match self.color_depth {
            8 | 16 | 32 => self.color_depth,
            _ => 8,
        };
    }


    pub fn set_jpeg_quality(&mut self, quality: u8) {
        self.jpeg_quality = quality.clamp(1, 100);
    }


    pub fn set_png_compression(&mut self, compression: u8) {
        self.png_compression = compression.clamp(0, 9);
    }


    pub fn set_video_bitrate(&mut self, bitrate: u32) {
        self.video_bitrate = bitrate.clamp(1, 1000);
    }


    pub fn set_color_depth(&mut self, depth: u8) {
        self.color_depth = match depth {
            8 | 16 | 32 => depth,
            _ => 8,
        };
    }
}


#[derive(Debug, Clone)]
pub struct OutputMetadata {

    pub frame_number: u64,

    pub format: OutputFormat,

    pub output_path: Option<String>,

    pub file_size: Option<usize>,

    pub encoding_time: Option<u64>,

    pub quality_settings: QualitySettings,
}


#[derive(Debug, Clone)]
pub struct EncodingResult {

    pub success: bool,

    pub output_data: ParameterValue,

    pub metadata: OutputMetadata,

    pub error: Option<String>,
}

impl EncodingResult {

    pub fn success(output_data: ParameterValue, metadata: OutputMetadata) -> Self {
        Self {
            success: true,
            output_data,
            metadata,
            error: None,
        }
    }


    pub fn error(error: String, format: OutputFormat, frame: u64) -> Self {
        Self {
            success: false,
            output_data: ParameterValue::None,
            metadata: OutputMetadata {
                frame_number: frame,
                format,
                output_path: None,
                file_size: None,
                encoding_time: None,
                quality_settings: QualitySettings::default(),
            },
            error: Some(error),
        }
    }
}


#[derive(Debug, Clone)]
pub struct OutputParams {

    pub format: OutputFormat,

    pub quality: QualitySettings,

    pub output_dir: Option<String>,

    pub filename_pattern: Option<String>,

    pub overwrite: bool,
}

impl Default for OutputParams {
    fn default() -> Self {
        Self {
            format: OutputFormat::Raw,
            quality: QualitySettings::default(),
            output_dir: None,
            filename_pattern: None,
            overwrite: false,
        }
    }
}

impl OutputParams {

    pub fn new() -> Self {
        Self::default()
    }


    pub fn validate(&mut self) {
        self.quality.validate();


        if self.filename_pattern.is_none() {
            self.filename_pattern = Some("output_{:04}".to_string());
        }


        if self.output_dir.is_none() {
            self.output_dir = Some("/tmp/output".to_string());
        }
    }


    pub fn is_active(&self) -> bool {
        self.format != OutputFormat::Raw
    }
}
