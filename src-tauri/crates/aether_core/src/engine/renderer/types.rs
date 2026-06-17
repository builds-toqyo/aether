use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum RendererError {
    InitializationError(String),
    RenderError(String),
    ResourceError(String),
    HardwareAccelerationError(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RendererError::InitializationError(msg) => write!(f, "InitializationError: {}", msg),
            RendererError::RenderError(msg) => write!(f, "RenderError: {}", msg),
            RendererError::ResourceError(msg) => write!(f, "ResourceError: {}", msg),
            RendererError::HardwareAccelerationError(msg) => write!(f, "HardwareAccelerationError: {}", msg),
        }
    }
}

impl Error for RendererError {}

pub struct Frame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub timestamp: f64,
}

/// Hardware acceleration context types
#[derive(Debug)]
pub(super) enum HardwareContext {
    #[cfg(feature = "cuda")]
    Cuda,

    #[cfg(all(feature = "vaapi", target_os = "linux"))]
    Vaapi { display: *mut std::ffi::c_void },

    #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
    VideoToolbox { session: *mut std::ffi::c_void },

    #[cfg(feature = "amf")]
    Amf { factory: *mut std::ffi::c_void, context: *mut std::ffi::c_void },
}

/// Shader programs for different hardware backends
pub(super) enum Shaders {
    #[cfg(feature = "cuda")]
    Cuda { module: *mut std::ffi::c_void, kernel: *mut std::ffi::c_void },

    #[cfg(all(feature = "vaapi", target_os = "linux"))]
    Vaapi { config: VaapiConfig },

    #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
    VideoToolbox { config: VideoToolboxConfig },

    #[cfg(feature = "amf")]
    Amf { components: Vec<*mut std::ffi::c_void> },

    Software { functions: Vec<Box<dyn Fn(&[u8], &mut [u8], usize, usize) + Send>> },
}

impl std::fmt::Debug for Shaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "cuda")]
            Shaders::Cuda { module, kernel } => write!(f, "Cuda {{ module: {:?}, kernel: {:?} }}", module, kernel),
            #[cfg(all(feature = "vaapi", target_os = "linux"))]
            Shaders::Vaapi { config } => write!(f, "Vaapi {{ config: {:?} }}", config),
            #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
            Shaders::VideoToolbox { config } => write!(f, "VideoToolbox {{ config: {:?} }}", config),
            #[cfg(feature = "amf")]
            Shaders::Amf { components } => write!(f, "Amf {{ components: {:?} }}", components.len()),
            Shaders::Software { functions } => write!(f, "Software {{ functions: {:?} }}", functions.len()),
        }
    }
}

/// GPU buffer types for different hardware backends
#[derive(Debug)]
pub(super) enum GpuBuffers {
    #[cfg(feature = "cuda")]
    Cuda { input: *mut std::ffi::c_void, output: *mut std::ffi::c_void, size: usize },

    #[cfg(all(feature = "vaapi", target_os = "linux"))]
    Vaapi { surfaces: Vec<*mut std::ffi::c_void> },

    #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
    VideoToolbox { pixel_buffers: Vec<*mut std::ffi::c_void> },

    #[cfg(feature = "amf")]
    Amf { surfaces: Vec<*mut std::ffi::c_void> },
}

/// CPU buffers for software rendering
#[derive(Debug)]
pub(super) struct CpuBuffers {
    pub _input: Vec<u8>,
    pub _output: Vec<u8>,
}

/// Lookup tables for various effects
#[derive(Debug)]
pub(super) struct LookupTables {
    pub _gamma: Vec<u8>,
    pub _vignette: Vec<u8>,
}

/// Post-processing pipeline stages
#[derive(Debug, Clone, Copy)]
pub(super) enum PostProcessStage {
    ColorCorrection,
    ColorGrading,
    Vignette,
}

/// Post-processing pipeline
#[derive(Debug)]
pub(super) struct PostProcessPipeline {
    pub _stages: Vec<PostProcessStage>,
}

#[cfg(all(feature = "vaapi", target_os = "linux"))]
#[derive(Debug, Clone)]
pub(super) struct VaapiConfig {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub hue: f32,
}

#[cfg(all(feature = "vaapi", target_os = "linux"))]
impl Default for VaapiConfig {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue: 0.0,
        }
    }
}

#[cfg(all(feature = "videotoolbox", target_os = "macos"))]
#[derive(Debug, Clone)]
pub(super) struct VideoToolboxConfig {
    pub color_space: u32,
    pub pixel_format: u32,
}

#[cfg(all(feature = "videotoolbox", target_os = "macos"))]
impl Default for VideoToolboxConfig {
    fn default() -> Self {
        Self {
            color_space: 1, // kCVImageBufferColorSpace_ITU_R_709_2
            pixel_format: 32, // kCVPixelFormatType_32BGRA
        }
    }
}

/// Uniform buffer for compute post-processing
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct PostProcessUniforms {
    pub width: u32,
    pub height: u32,
    pub gamma: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub temp_r: f32,
    pub temp_g: f32,
    pub temp_b: f32,
    pub vignette: f32,
    pub _pad: [u32; 2],
}

pub(super) struct RendererState {
    pub is_rendering: bool,
    pub last_render_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub background_color: [u8; 4],
    pub use_hardware_acceleration: bool,
    pub hw_device: Option<String>,
    pub gamma: f64,
    pub enable_color_correction: bool,
    pub enable_color_grading: bool,
    pub enable_vignette: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            frame_rate: 30.0,
            background_color: [0, 0, 0, 255],
            use_hardware_acceleration: false,
            hw_device: None,
            gamma: 1.0,
            enable_color_correction: false,
            enable_color_grading: false,
            enable_vignette: false,
        }
    }
}
