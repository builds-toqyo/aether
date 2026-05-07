use uuid::Uuid;


#[derive(Debug, Clone)]
pub struct CorrectedImage {

    pub original_id: Uuid,

    pub corrected_id: Uuid,

    pub brightness: f32,

    pub contrast: f32,

    pub saturation: f32,

    pub gamma: f32,

    pub temperature: f32,

    pub tint: f32,

    pub hue: f32,

    pub lift: f32,

    pub gamma_gain: f32,

    pub gain: f32,
}


#[derive(Debug, Clone)]
pub struct TextureInfo {

    pub id: Uuid,

    pub width: usize,

    pub height: usize,

    pub format: TextureFormat,

    pub internal_format: TextureInternalFormat,

    pub pixel_type: PixelType,

    pub mipmapped: bool,

    pub mipmap_count: usize,
}


#[derive(Debug, Clone, PartialEq)]
pub enum TextureFormat {

    RGBA8,

    RGB8,

    R8,

    RGBA16,

    RGB16,

    R16,

    RGBA32F,

    RGB32F,

    R32F,
}


#[derive(Debug, Clone, PartialEq)]
pub enum TextureInternalFormat {

    RGBA8,

    RGB8,

    R8,

    RGBA16,

    RGB16,

    R16,

    RGBA32F,

    RGB32F,

    R32F,
}


#[derive(Debug, Clone, PartialEq)]
pub enum PixelType {

    UnsignedByte,

    UnsignedShort,

    Float,
}


#[derive(Debug, Clone)]
pub struct ColorCorrectionParams {

    pub brightness: f32,

    pub contrast: f32,

    pub saturation: f32,

    pub gamma: f32,

    pub temperature: f32,

    pub tint: f32,

    pub hue: f32,

    pub lift: f32,

    pub gamma_gain: f32,

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

    pub fn new() -> Self {
        Self::default()
    }


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
