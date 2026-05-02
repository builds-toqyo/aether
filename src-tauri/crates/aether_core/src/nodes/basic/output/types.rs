use uuid::Uuid;

/// Output format options
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    /// Raw frame buffer
    Raw,
    /// PNG image sequence
    PngSequence,
    /// JPEG image sequence
    JpegSequence,
    /// MP4 video
    Mp4,
    /// ProRes video
    ProRes,
    /// EXR image sequence (HDR)
    ExrSequence,
}

/// Quality settings for output
#[derive(Debug, Clone)]
pub struct QualitySettings {
    /// JPEG quality (1-100)
    pub jpeg_quality: u8,
    /// PNG compression level (0-9)
    pub png_compression: u8,
    /// Video bitrate in Mbps
    pub video_bitrate: u32,
    /// Color depth (8, 16, 32)
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
    /// Create new quality settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate and clamp settings to valid ranges
    pub fn validate(&mut self) {
        self.jpeg_quality = self.jpeg_quality.clamp(1, 100);
        self.png_compression = self.png_compression.clamp(0, 9);
        self.video_bitrate = self.video_bitrate.clamp(1, 1000);
        self.color_depth = match self.color_depth {
            8 | 16 | 32 => self.color_depth,
            _ => 8, // Default to 8-bit
        };
    }
    
    /// Set JPEG quality
    pub fn set_jpeg_quality(&mut self, quality: u8) {
        self.jpeg_quality = quality.clamp(1, 100);
    }
    
    /// Set PNG compression
    pub fn set_png_compression(&mut self, compression: u8) {
        self.png_compression = compression.clamp(0, 9);
    }
    
    /// Set video bitrate
    pub fn set_video_bitrate(&mut self, bitrate: u32) {
        self.video_bitrate = bitrate.clamp(1, 1000);
    }
    
    /// Set color depth
    pub fn set_color_depth(&mut self, depth: u8) {
        self.color_depth = match depth {
            8 | 16 | 32 => depth,
            _ => 8,
        };
    }
}

/// Output metadata
#[derive(Debug, Clone)]
pub struct OutputMetadata {
    /// Frame number
    pub frame_number: u64,
    /// Output format
    pub format: OutputFormat,
    /// File path or buffer identifier
    pub output_path: Option<String>,
    /// File size in bytes
    pub file_size: Option<usize>,
    /// Encoding time in milliseconds
    pub encoding_time: Option<u64>,
    /// Quality settings used
    pub quality_settings: QualitySettings,
}

/// Encoding result
#[derive(Debug, Clone)]
pub struct EncodingResult {
    /// Success status
    pub success: bool,
    /// Output data (file path or binary data)
    pub output_data: ParameterValue,
    /// Metadata
    pub metadata: OutputMetadata,
    /// Error message if failed
    pub error: Option<String>,
}

impl EncodingResult {
    /// Create successful result
    pub fn success(output_data: ParameterValue, metadata: OutputMetadata) -> Self {
        Self {
            success: true,
            output_data,
            metadata,
            error: None,
        }
    }
    
    /// Create failed result
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

/// Output parameters
#[derive(Debug, Clone)]
pub struct OutputParams {
    /// Output format
    pub format: OutputFormat,
    /// Quality settings
    pub quality: QualitySettings,
    /// Output directory
    pub output_dir: Option<String>,
    /// File name pattern
    pub filename_pattern: Option<String>,
    /// Whether to overwrite existing files
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
    /// Create new output parameters
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate parameters
    pub fn validate(&mut self) {
        self.quality.validate();
        
        // Set default filename pattern if not provided
        if self.filename_pattern.is_none() {
            self.filename_pattern = Some("output_{:04}".to_string());
        }
        
        // Set default output directory if not provided
        if self.output_dir.is_none() {
            self.output_dir = Some("/tmp/output".to_string());
        }
    }
    
    /// Check if output is active (not Raw)
    pub fn is_active(&self) -> bool {
        self.format != OutputFormat::Raw
    }
}
