use uuid::Uuid;

/// Metadata for a corrected image
#[derive(Debug, Clone)]
pub struct CorrectedImage {
    /// Original image ID
    pub original_id: Uuid,
    /// Corrected image ID
    pub corrected_id: Uuid,
    /// Brightness adjustment
    pub brightness: f32,
    /// Contrast adjustment
    pub contrast: f32,
    /// Saturation adjustment
    pub saturation: f32,
    /// Gamma correction
    pub gamma: f32,
    /// Color temperature
    pub temperature: f32,
    /// Color tint
    pub tint: f32,
    /// Hue shift
    pub hue: f32,
    /// Shadow lift
    pub lift: f32,
    /// Midtone gamma gain
    pub gamma_gain: f32,
    /// Highlight gain
    pub gain: f32,
}

/// Texture format information
#[derive(Debug, Clone)]
pub struct TextureInfo {
    /// Texture ID
    pub id: Uuid,
    /// Texture width
    pub width: usize,
    /// Texture height
    pub height: usize,
    /// Texture format
    pub format: TextureFormat,
    /// Internal format
    pub internal_format: TextureInternalFormat,
    /// Pixel data type
    pub pixel_type: PixelType,
    /// Whether texture has mipmaps
    pub mipmapped: bool,
    /// Number of mipmap levels
    pub mipmap_count: usize,
}

/// Texture format
#[derive(Debug, Clone, PartialEq)]
pub enum TextureFormat {
    /// 8-bit RGBA
    RGBA8,
    /// 8-bit RGB
    RGB8,
    /// 8-bit Red (luminance)
    R8,
    /// 16-bit RGBA
    RGBA16,
    /// 16-bit RGB
    RGB16,
    /// 16-bit Red
    R16,
    /// 32-bit RGBA (float)
    RGBA32F,
    /// 32-bit RGB (float)
    RGB32F,
    /// 32-bit Red (float)
    R32F,
}

/// Internal texture format
#[derive(Debug, Clone, PartialEq)]
pub enum TextureInternalFormat {
    /// 8-bit RGBA
    RGBA8,
    /// 8-bit RGB
    RGB8,
    /// 8-bit Red
    R8,
    /// 16-bit RGBA
    RGBA16,
    /// 16-bit RGB
    RGB16,
    /// 16-bit Red
    R16,
    /// 32-bit RGBA (float)
    RGBA32F,
    /// 32-bit RGB (float)
    RGB32F,
    /// 32-bit Red (float)
    R32F,
}

/// Pixel data type
#[derive(Debug, Clone, PartialEq)]
pub enum PixelType {
    /// Unsigned byte (8-bit)
    UnsignedByte,
    /// Unsigned short (16-bit)
    UnsignedShort,
    /// Float (32-bit)
    Float,
}

/// Color correction parameters
#[derive(Debug, Clone)]
pub struct ColorCorrectionParams {
    /// Brightness (-1.0 to 1.0)
    pub brightness: f32,
    /// Contrast (0.0 to 2.0)
    pub contrast: f32,
    /// Saturation (0.0 to 2.0)
    pub saturation: f32,
    /// Gamma (0.1 to 3.0)
    pub gamma: f32,
    /// Temperature in Kelvin (2000 to 12000)
    pub temperature: f32,
    /// Tint (-100 to 100)
    pub tint: f32,
    /// Hue (-180 to 180 degrees)
    pub hue: f32,
    /// Shadow lift (-1.0 to 1.0)
    pub lift: f32,
    /// Midtone gamma gain (0.1 to 3.0)
    pub gamma_gain: f32,
    /// Highlight gain (0.0 to 2.0)
    pub gain: f32,
}

impl Default for ColorCorrectionParams {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            temperature: 6500.0,
            tint: 0.0,
            hue: 0.0,
            lift: 0.0,
            gamma_gain: 1.0,
            gain: 1.0,
        }
    }
}

impl ColorCorrectionParams {
    /// Create new parameters with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate and clamp parameters to valid ranges
    pub fn validate(&mut self) {
        self.brightness = self.brightness.clamp(-1.0, 1.0);
        self.contrast = self.contrast.clamp(0.0, 2.0);
        self.saturation = self.saturation.clamp(0.0, 2.0);
        self.gamma = self.gamma.clamp(0.1, 3.0);
        self.temperature = self.temperature.clamp(2000.0, 12000.0);
        self.tint = self.tint.clamp(-100.0, 100.0);
        self.hue = self.hue.clamp(-180.0, 180.0);
        self.lift = self.lift.clamp(-1.0, 1.0);
        self.gamma_gain = self.gamma_gain.clamp(0.1, 3.0);
        self.gain = self.gain.clamp(0.0, 2.0);
    }
    
    /// Check if any parameter is non-default
    pub fn is_active(&self) -> bool {
        self.brightness != 0.0 ||
        self.contrast != 1.0 ||
        self.saturation != 1.0 ||
        self.gamma != 1.0 ||
        self.temperature != 6500.0 ||
        self.tint != 0.0 ||
        self.hue != 0.0 ||
        self.lift != 0.0 ||
        self.gamma_gain != 1.0 ||
        self.gain != 1.0
    }
}
