use std::sync::{Arc, Mutex};
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

const COMPUTE_SHADER: &str = r#"
struct Params {
    width: u32,
    height: u32,
    gamma: f32,
    saturation: f32,
    contrast: f32,
    brightness: f32,
    temp_r: f32,
    temp_g: f32,
    temp_b: f32,
    vignette: f32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read> input_buffer: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<u32>;
@group(0) @binding(2) var<uniform> params: Params;

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> vec3<f32> {
    let c = (1.0 - abs(2.0 * l - 1.0)) * s;
    let h6 = h * 6.0;
    let x = c * (1.0 - abs(h6 % 2.0 - 1.0));
    let m = l - c / 2.0;
    var rgb = vec3<f32>(0.0);
    if h6 < 1.0 { rgb = vec3<f32>(c, x, 0.0); }
    else if h6 < 2.0 { rgb = vec3<f32>(x, c, 0.0); }
    else if h6 < 3.0 { rgb = vec3<f32>(0.0, c, x); }
    else if h6 < 4.0 { rgb = vec3<f32>(0.0, x, c); }
    else if h6 < 5.0 { rgb = vec3<f32>(x, 0.0, c); }
    else { rgb = vec3<f32>(c, 0.0, x); }
    return rgb + vec3<f32>(m);
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> vec3<f32> {
    let maxc = max(max(r, g), b);
    let minc = min(min(r, g), b);
    let delta = maxc - minc;
    let l = (maxc + minc) / 2.0;
    var s = 0.0;
    if delta != 0.0 {
        s = delta / (1.0 - abs(2.0 * l - 1.0));
    }
    var h = 0.0;
    if delta != 0.0 {
        if maxc == r { h = ((g - b) / delta) % 6.0; }
        else if maxc == g { h = ((b - r) / delta) + 2.0; }
        else { h = ((r - g) / delta) + 4.0; }
    }
    if h < 0.0 { h = h + 6.0; }
    return vec3<f32>(h / 6.0, s, l);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if x >= params.width || y >= params.height { return; }

    let idx = (y * params.width + x) * 4u;

    var r = f32(input_buffer[idx]) / 255.0;
    var g = f32(input_buffer[idx + 1u]) / 255.0;
    var b = f32(input_buffer[idx + 2u]) / 255.0;
    let a = f32(input_buffer[idx + 3u]) / 255.0;

    // Gamma correction
    let gamma_inv = 1.0 / params.gamma;
    r = pow(r, gamma_inv);
    g = pow(g, gamma_inv);
    b = pow(b, gamma_inv);

    // Contrast
    r = ((r - 0.5) * params.contrast + 0.5);
    g = ((g - 0.5) * params.contrast + 0.5);
    b = ((b - 0.5) * params.contrast + 0.5);

    // Brightness
    r = r * params.brightness;
    g = g * params.brightness;
    b = b * params.brightness;

    // Saturation
    let hsl = rgb_to_hsl(r, g, b);
    let rgb = hsl_to_rgb(hsl.x, hsl.y * params.saturation, hsl.z);
    r = rgb.x; g = rgb.y; b = rgb.z;

    // Color temperature
    r = r * params.temp_r;
    g = g * params.temp_g;
    b = b * params.temp_b;

    // Vignette
    let cx = f32(params.width) * 0.5;
    let cy = f32(params.height) * 0.5;
    let dx = (f32(x) - cx) / cx;
    let dy = (f32(y) - cy) / cy;
    let dist = sqrt(dx * dx + dy * dy);
    let vignette_factor = 1.0 - dist * dist * params.vignette;
    r = r * vignette_factor;
    g = g * vignette_factor;
    b = b * vignette_factor;

    // Clamp and write
    output_buffer[idx] = u32(clamp(r * 255.0, 0.0, 255.0));
    output_buffer[idx + 1u] = u32(clamp(g * 255.0, 0.0, 255.0));
    output_buffer[idx + 2u] = u32(clamp(b * 255.0, 0.0, 255.0));
    output_buffer[idx + 3u] = u32(clamp(a * 255.0, 0.0, 255.0));
}
"#;

pub struct Frame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub timestamp: f64,
}

/// Hardware acceleration context types
#[derive(Debug)]
enum HardwareContext {
    #[cfg(feature = "cuda")]
    Cuda { context: *mut std::ffi::c_void },

    #[cfg(all(feature = "vaapi", target_os = "linux"))]
    Vaapi { display: *mut std::ffi::c_void },

    #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
    VideoToolbox { session: *mut std::ffi::c_void },

    #[cfg(feature = "amf")]
    Amf { factory: *mut std::ffi::c_void, context: *mut std::ffi::c_void },

    Software,
}

/// Shader programs for different hardware backends
enum Shaders {
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
enum GpuBuffers {
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
struct CpuBuffers {
    input: Vec<u8>,
    output: Vec<u8>,
}

/// Lookup tables for various effects
#[derive(Debug)]
struct LookupTables {
    gamma: Vec<u8>,
    vignette: Vec<u8>,
    // color_3d: Vec<u8>,
}

/// Post-processing pipeline stages
#[derive(Debug, Clone, Copy)]
enum PostProcessStage {
    ColorCorrection,
    ColorGrading,
    Vignette,
    Blur,
    Sharpen,
    Denoise,
    Custom(usize), // Index into custom effects
}

/// Post-processing pipeline
#[derive(Debug)]
struct PostProcessPipeline {
    stages: Vec<PostProcessStage>,
}

#[cfg(all(feature = "vaapi", target_os = "linux"))]
#[derive(Debug, Clone)]
struct VaapiConfig {
    // VAAPI specific configuration
    brightness: f32,
    contrast: f32,
    saturation: f32,
    hue: f32,
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
struct VideoToolboxConfig {
    // VideoToolbox specific configuration
    color_space: u32,
    pixel_format: u32,
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
struct PostProcessUniforms {
    width: u32,
    height: u32,
    gamma: f32,
    saturation: f32,
    contrast: f32,
    brightness: f32,
    temp_r: f32,
    temp_g: f32,
    temp_b: f32,
    vignette: f32,
    _pad: [u32; 2],
}

pub struct Renderer {
    config: RendererConfig,
    is_initialized: bool,
    hw_context: Option<HardwareContext>,
    shaders: Option<Shaders>,
    gpu_buffers: Option<GpuBuffers>,
    cpu_buffers: Option<CpuBuffers>,
    lookup_tables: Option<LookupTables>,
    post_process_pipeline: Option<PostProcessPipeline>,
    current_frame: Option<Frame>,
    frame_count: u64,
    state: Arc<Mutex<RendererState>>,
    // wgpu cross-platform GPU compute
    wgpu_device: Option<wgpu::Device>,
    wgpu_queue: Option<wgpu::Queue>,
    wgpu_compute_pipeline: Option<wgpu::ComputePipeline>,
    wgpu_bind_group_layout: Option<wgpu::BindGroupLayout>,
    wgpu_input_buffer: Option<wgpu::Buffer>,
    wgpu_output_buffer: Option<wgpu::Buffer>,
    wgpu_uniform_buffer: Option<wgpu::Buffer>,
    wgpu_staging_buffer: Option<wgpu::Buffer>,
}

struct RendererState {
    is_rendering: bool,
    last_render_time: std::time::Instant,
}

impl Renderer {
    pub fn new(config: RendererConfig) -> Self {
        let state = RendererState {
            is_rendering: false,
            last_render_time: std::time::Instant::now(),
        };

        Self {
            config,
            is_initialized: false,
            hw_context: None,
            shaders: None,
            gpu_buffers: None,
            cpu_buffers: None,
            lookup_tables: None,
            post_process_pipeline: None,
            current_frame: None,
            frame_count: 0,
            state: Arc::new(Mutex::new(state)),
            wgpu_device: None,
            wgpu_queue: None,
            wgpu_compute_pipeline: None,
            wgpu_bind_group_layout: None,
            wgpu_input_buffer: None,
            wgpu_output_buffer: None,
            wgpu_uniform_buffer: None,
            wgpu_staging_buffer: None,
        }
    }

    pub fn initialize(&mut self) -> Result<(), RendererError> {
        if self.is_initialized {
            return Ok(());
        }

        // Log initialization start
        log::debug!("Initializing renderer with resolution: {}x{}", self.config.width, self.config.height);

        // Initialize hardware acceleration if enabled
        if self.config.use_hardware_acceleration {
            self.initialize_hardware_acceleration()?;
        } else {
            log::debug!("Using software rendering");
        }

        // Allocate frame buffers
        self.allocate_frame_buffers()?;

        // Initialize any other resources needed for rendering
        self.initialize_resources()?;

        self.is_initialized = true;
        log::debug!("Renderer initialized successfully");
        Ok(())
    }

    /// Initialize hardware acceleration
    fn initialize_hardware_acceleration(&mut self) -> Result<(), RendererError> {
        let device = self.config.hw_device.as_deref().unwrap_or("auto");
        log::info!("Initializing hardware acceleration with device: {}", device);

        match device {
            "cuda" => {
                log::debug!("Initializing CUDA acceleration");
                self.initialize_cuda_acceleration()
            },
            "vaapi" => {
                log::debug!("Initializing VAAPI acceleration");
                self.initialize_vaapi_acceleration()
            },
            "videotoolbox" => {
                log::debug!("Initializing VideoToolbox acceleration");
                self.initialize_videotoolbox_acceleration()
            },
            "amf" => {
                log::debug!("Initializing AMF acceleration");
                self.initialize_amf_acceleration()
            },
            _ => {
                // Try to auto-detect the best hardware acceleration
                log::debug!("Auto-detecting hardware acceleration");
                self.auto_detect_acceleration()
            }
        }
    }

    /// Initialize wgpu compute for cross-platform GPU post-processing
    fn initialize_wgpu_compute(&mut self) -> Result<(), RendererError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .block_on()
            .ok_or_else(|| RendererError::HardwareAccelerationError(
                "No suitable GPU adapter found".to_string()
            ))?;

        let info = adapter.get_info();
        log::info!("Selected GPU: {} ({:?})", info.name, info.backend);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Aether Renderer"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .block_on()
            .map_err(|e| RendererError::HardwareAccelerationError(
                format!("Failed to create wgpu device: {}", e)
            ))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post_process"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(COMPUTE_SHADER)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post_process_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post_process_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("post_process_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            ..Default::default()
        });

        let width = self.config.width as u64;
        let height = self.config.height as u64;
        let buffer_size = width * height * 4;

        let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("input_buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output_buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            size: std::mem::size_of::<PostProcessUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        self.wgpu_device = Some(device);
        self.wgpu_queue = Some(queue);
        self.wgpu_compute_pipeline = Some(compute_pipeline);
        self.wgpu_bind_group_layout = Some(bind_group_layout);
        self.wgpu_input_buffer = Some(input_buffer);
        self.wgpu_output_buffer = Some(output_buffer);
        self.wgpu_uniform_buffer = Some(uniform_buffer);
        self.wgpu_staging_buffer = Some(staging_buffer);

        log::info!("wgpu compute pipeline initialized ({}x{})", width, height);
        Ok(())
    }

    /// Initialize CUDA acceleration for NVIDIA GPUs (stub — requires cuda feature)
    fn initialize_cuda_acceleration(&mut self) -> Result<(), RendererError> {
        #[cfg(feature = "cuda")]
        {
            if !self.has_nvidia_gpu() {
                return Err(RendererError::HardwareAccelerationError(
                    "No NVIDIA GPU found".to_string()
                ));
            }
            Err(RendererError::HardwareAccelerationError(
                "CUDA acceleration not yet implemented — use wgpu compute instead".to_string()
            ))
        }
        #[cfg(not(feature = "cuda"))]
        {
            Err(RendererError::HardwareAccelerationError(
                "CUDA feature not enabled".to_string()
            ))
        }
    }

    /// Initialize VAAPI acceleration for Intel GPUs on Linux (stub — requires vaapi feature)
    fn initialize_vaapi_acceleration(&mut self) -> Result<(), RendererError> {
        #[cfg(all(feature = "vaapi", target_os = "linux"))]
        {
            if !self.has_vaapi_support() {
                return Err(RendererError::HardwareAccelerationError(
                    "No VAAPI support found".to_string()
                ));
            }
            Err(RendererError::HardwareAccelerationError(
                "VAAPI acceleration not yet implemented — use wgpu compute instead".to_string()
            ))
        }
        #[cfg(not(all(feature = "vaapi", target_os = "linux")))]
        {
            Err(RendererError::HardwareAccelerationError(
                "VAAPI feature not enabled or not on Linux".to_string()
            ))
        }
    }

    /// Initialize VideoToolbox acceleration for macOS (stub — requires videotoolbox feature)
    fn initialize_videotoolbox_acceleration(&mut self) -> Result<(), RendererError> {
        #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
        {
            Err(RendererError::HardwareAccelerationError(
                "VideoToolbox acceleration not yet implemented — use wgpu compute instead".to_string()
            ))
        }
        #[cfg(not(all(feature = "videotoolbox", target_os = "macos")))]
        {
            Err(RendererError::HardwareAccelerationError(
                "VideoToolbox feature not enabled or not on macOS".to_string()
            ))
        }
    }

    /// Initialize AMD AMF acceleration (stub — requires amf feature)
    fn initialize_amf_acceleration(&mut self) -> Result<(), RendererError> {
        #[cfg(feature = "amf")]
        {
            if !self.has_amd_gpu() {
                return Err(RendererError::HardwareAccelerationError(
                    "No AMD GPU found".to_string()
                ));
            }
            Err(RendererError::HardwareAccelerationError(
                "AMF acceleration not yet implemented — use wgpu compute instead".to_string()
            ))
        }
        #[cfg(not(feature = "amf"))]
        {
            Err(RendererError::HardwareAccelerationError(
                "AMF feature not enabled".to_string()
            ))
        }
    }

    /// Auto-detect the best hardware acceleration method
    fn auto_detect_acceleration(&mut self) -> Result<(), RendererError> {
        // Try wgpu compute first (cross-platform GPU post-processing)
        match self.initialize_wgpu_compute() {
            Ok(_) => {
                self.hw_context = Some(HardwareContext::Software);
                return Ok(());
            }
            Err(e) => {
                log::warn!("wgpu compute initialization failed: {}", e);
            }
        }

        // Fallback to software rendering
        log::info!("Falling back to software rendering");
        self.config.use_hardware_acceleration = false;
        Ok(())
    }

    /// Detect GPU vendor using wgpu adapter enumeration
    fn detect_gpu_vendor(&self) -> Option<&'static str> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapters = instance.enumerate_adapters(wgpu::Backends::all());
        for adapter in adapters {
            let info = adapter.get_info();
            let vendor = match info.vendor {
                0x10DE => "nvidia",
                0x1002 => "amd",
                0x1022 => "amd",
                0x8086 => "intel",
                0x106B => "apple", // Apple Silicon
                _ => continue,
            };
            log::debug!("Detected GPU: {} ({:?})", info.name, info.backend);
            return Some(vendor);
        }
        None
    }

    fn has_nvidia_gpu(&self) -> bool {
        self.detect_gpu_vendor() == Some("nvidia")
    }

    fn has_amd_gpu(&self) -> bool {
        self.detect_gpu_vendor() == Some("amd")
    }

    fn has_vaapi_support(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/dev/dri/renderD128").exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    fn allocate_frame_buffers(&mut self) -> Result<(), RendererError> {
        let width = self.config.width as usize;
        let height = self.config.height as usize;


        let buffer_size = width * height * 4;
        log::debug!("Allocating frame buffer of {} bytes", buffer_size);


        Ok(())
    }

    fn initialize_resources(&mut self) -> Result<(), RendererError> {
        log::debug!("Initializing rendering resources");

        if self.config.use_hardware_acceleration {
            self.initialize_shader_programs()?;
        }

        self.initialize_lookup_tables()?;

        self.allocate_gpu_resources()?;

        self.initialize_post_processing()?;

        log::info!("Rendering resources initialized successfully");
        Ok(())
    }

    fn initialize_shader_programs(&mut self) -> Result<(), RendererError> {
        log::debug!("Initializing shader programs");

        if let Some(hw_context) = &self.hw_context {
            match hw_context {
                #[cfg(feature = "cuda")]
                HardwareContext::Cuda { .. } => {

                    let vertex_shader = include_str!("../shaders/cuda/vertex.cu");
                    let fragment_shader = include_str!("../shaders/cuda/fragment.cu");

                },

                #[cfg(all(feature = "vaapi", target_os = "linux"))]
                HardwareContext::Vaapi { .. } => {

                },

                #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
                HardwareContext::VideoToolbox { .. } => {

                },

                #[cfg(feature = "amf")]
                HardwareContext::Amf { .. } => {

                },

                _ => {

                }
            }
        } else {

            log::info!("No hardware context available, using software shaders");

        }

        log::debug!("Shader programs initialized");
        Ok(())
    }

    fn initialize_lookup_tables(&mut self) -> Result<(), RendererError> {
        log::debug!("Initializing lookup tables");


        let gamma = self.config.gamma as f32;
        let gamma_lut = (0..256).map(|i| {
            let normalized = i as f32 / 255.0;
            let corrected = normalized.powf(1.0 / gamma);
            (corrected * 255.0).round() as u8
        }).collect::<Vec<u8>>();


        let width = self.config.width as usize;
        let height = self.config.height as usize;
        let mut vignette_lut = vec![0u8; width * height];

        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let max_dist = (center_x.powi(2) + center_y.powi(2)).sqrt();

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx.powi(2) + dy.powi(2)).sqrt();
                let factor = 1.0 - (distance / max_dist).powi(2);
                vignette_lut[y * width + x] = (factor * 255.0).round() as u8;
            }
        }


        self.lookup_tables = Some(LookupTables {
            gamma: gamma_lut,
            vignette: vignette_lut,

        });

        log::debug!("Lookup tables initialized");
        Ok(())
    }


    fn allocate_gpu_resources(&mut self) -> Result<(), RendererError> {
        log::debug!("Allocating GPU resources");

        let width = self.config.width as usize;
        let height = self.config.height as usize;

        if self.config.use_hardware_acceleration {
            if let Some(hw_context) = &self.hw_context {
                match hw_context {
                    #[cfg(feature = "cuda")]
                    HardwareContext::Cuda { .. } => {

                    },

                    #[cfg(all(feature = "vaapi", target_os = "linux"))]
                    HardwareContext::Vaapi { .. } => {

                    },

                    #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
                    HardwareContext::VideoToolbox { .. } => {


                    },

                    #[cfg(feature = "amf")]
                    HardwareContext::Amf { .. } => {


                    },

                    _ => {

                        log::warn!("Unknown hardware context type, falling back to CPU buffers");
                        self.allocate_cpu_buffers(width, height)?;
                    }
                }
            } else {

                log::warn!("No hardware context available, falling back to CPU buffers");
                self.allocate_cpu_buffers(width, height)?;
            }
        } else {

            self.allocate_cpu_buffers(width, height)?;
        }

        log::debug!("GPU resources allocated");
        Ok(())
    }


    fn allocate_cpu_buffers(&mut self, width: usize, height: usize) -> Result<(), RendererError> {

        let buffer_size = width * height * 4;


        let input_buffer = vec![0u8; buffer_size];
        let output_buffer = vec![0u8; buffer_size];


        self.cpu_buffers = Some(CpuBuffers {
            input: input_buffer,
            output: output_buffer,
        });

        log::debug!("CPU buffers allocated: {} bytes each", buffer_size);
        Ok(())
    }


    fn initialize_post_processing(&mut self) -> Result<(), RendererError> {
        log::debug!("Initializing post-processing pipeline");


        let mut stages = Vec::new();

        if self.config.enable_color_correction {
            stages.push(PostProcessStage::ColorCorrection);
        }

        if self.config.enable_color_grading {
            stages.push(PostProcessStage::ColorGrading);
        }

        if self.config.enable_vignette {
            stages.push(PostProcessStage::Vignette);
        }


        let stage_count = stages.len();
        self.post_process_pipeline = Some(PostProcessPipeline { stages });

        log::debug!("Post-processing pipeline initialized with {} stages", stage_count);
        Ok(())
    }


    fn cleanup_hardware_acceleration(&mut self) {
        if let Some(device) = &self.config.hw_device {
            log::debug!("Cleaning up hardware acceleration resources for device: {}", device);


            match device.as_str() {
                "cuda" => {

                    log::debug!("Releasing CUDA resources");
                },
                "vaapi" => {

                    log::debug!("Releasing VAAPI resources");
                },
                "videotoolbox" => {

                    log::debug!("Releasing VideoToolbox resources");
                },
                "amf" => {

                    log::debug!("Releasing AMD AMF resources");
                },
                _ => {
                    log::debug!("Releasing auto-detected hardware acceleration resources");
                }
            }
        }
    }


    fn cleanup_frame_buffers(&mut self) {
        log::debug!("Cleaning up frame buffer resources");


    }


    fn cleanup_resources(&mut self) {
        log::debug!("Cleaning up additional rendering resources");


    }


    pub fn render(&mut self, input_data: &[u8], timestamp: f64) -> Result<&Frame, RendererError> {
        if !self.is_initialized {
            return Err(RendererError::InitializationError("Renderer not initialized".to_string()));
        }


        let mut state = self.state.lock().unwrap();
        state.is_rendering = true;
        state.last_render_time = std::time::Instant::now();


        let mut frame_data = input_data.to_vec();


        self.apply_post_processing(&mut frame_data)?;


        let frame = Frame {
            data: frame_data,
            width: self.config.width,
            height: self.config.height,
            timestamp,
        };

        self.current_frame = Some(frame);
        self.frame_count += 1;
        state.is_rendering = false;


        self.current_frame.as_ref().ok_or(RendererError::RenderError("Failed to create frame".to_string()))
    }

    pub fn current_frame(&self) -> Option<&Frame> {
        self.current_frame.as_ref()
    }


    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }


    fn apply_post_processing(&self, frame_data: &mut [u8]) -> Result<(), RendererError> {
        if frame_data.is_empty() {
            return Ok(());
        }

        let width = self.config.width as usize;
        let height = self.config.height as usize;

        if frame_data.len() < width * height * 4 {
            return Err(RendererError::RenderError(
                format!("Frame data too small: {} bytes for {}x{} RGBA frame",
                        frame_data.len(), width, height)
            ));
        }

        // Use wgpu compute if available, otherwise fallback to CPU
        if self.wgpu_compute_pipeline.is_some() {
            self.apply_post_processing_gpu(frame_data, width, height)?;
        } else {
            self.apply_post_processing_cpu(frame_data, width, height);
        }

        Ok(())
    }

    fn apply_post_processing_gpu(&self, frame_data: &mut [u8], width: usize, height: usize) -> Result<(), RendererError> {
        let device = self.wgpu_device.as_ref().ok_or(RendererError::RenderError("GPU device not available".to_string()))?;
        let queue = self.wgpu_queue.as_ref().ok_or(RendererError::RenderError("GPU queue not available".to_string()))?;
        let pipeline = self.wgpu_compute_pipeline.as_ref().ok_or(RendererError::RenderError("GPU pipeline not available".to_string()))?;
        let bind_group_layout = self.wgpu_bind_group_layout.as_ref().ok_or(RendererError::RenderError("GPU bind group layout not available".to_string()))?;
        let input_buffer = self.wgpu_input_buffer.as_ref().ok_or(RendererError::RenderError("GPU input buffer not available".to_string()))?;
        let output_buffer = self.wgpu_output_buffer.as_ref().ok_or(RendererError::RenderError("GPU output buffer not available".to_string()))?;
        let uniform_buffer = self.wgpu_uniform_buffer.as_ref().ok_or(RendererError::RenderError("GPU uniform buffer not available".to_string()))?;
        let staging_buffer = self.wgpu_staging_buffer.as_ref().ok_or(RendererError::RenderError("GPU staging buffer not available".to_string()))?;

        let uniforms = PostProcessUniforms {
            width: width as u32,
            height: height as u32,
            gamma: 1.1,
            saturation: 1.1,
            contrast: 1.05,
            brightness: 1.0,
            temp_r: 1.05,
            temp_g: 1.0,
            temp_b: 0.95,
            vignette: 0.3,
            _pad: [0; 2],
        };

        queue.write_buffer(input_buffer, 0, frame_data);
        queue.write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post_process_bind_group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: input_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: output_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: uniform_buffer.as_entire_binding() },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("post_process_encoder") });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("post_process_pass"),
                ..Default::default()
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                ((width as u32 + 7) / 8).max(1),
                ((height as u32 + 7) / 8).max(1),
                1,
            );
        }

        encoder.copy_buffer_to_buffer(output_buffer, 0, staging_buffer, 0, frame_data.len() as u64);
        queue.submit(std::iter::once(encoder.finish()));

        let slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        device.poll(wgpu::Maintain::Wait);
        rx.recv().map_err(|_| RendererError::RenderError("GPU map channel closed".to_string()))?
            .map_err(|e| RendererError::RenderError(format!("GPU map failed: {}", e)))?;

        let data = slice.get_mapped_range();
        frame_data.copy_from_slice(&data);
        drop(data);
        staging_buffer.unmap();

        Ok(())
    }

    fn apply_post_processing_cpu(&self, frame_data: &mut [u8], width: usize, height: usize) {
        self.apply_gamma_correction(frame_data, width, height);
        self.apply_color_grading(frame_data, width, height);
        self.apply_vignette(frame_data, width, height);
    }

    fn apply_gamma_correction(&self, frame_data: &mut [u8], width: usize, height: usize) {
        let gamma = 1.1;
        let gamma_inv = 1.0 / gamma;
        let mut gamma_table = [0u8; 256];
        for i in 0..256 {
            let normalized = i as f32 / 255.0;
            let corrected = normalized.powf(gamma_inv);
            gamma_table[i] = (corrected * 255.0).clamp(0.0, 255.0) as u8;
        }
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                frame_data[idx] = gamma_table[frame_data[idx] as usize];
                frame_data[idx + 1] = gamma_table[frame_data[idx + 1] as usize];
                frame_data[idx + 2] = gamma_table[frame_data[idx + 2] as usize];
            }
        }
    }

    fn apply_color_grading(&self, frame_data: &mut [u8], width: usize, height: usize) {
        let saturation = 1.1;
        let contrast = 1.05;
        let brightness = 1.0;
        let temp_r = 1.05;
        let temp_g = 1.0;
        let temp_b = 0.95;
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let mut r = frame_data[idx] as f32 / 255.0;
                let mut g = frame_data[idx + 1] as f32 / 255.0;
                let mut b = frame_data[idx + 2] as f32 / 255.0;
                r = ((r - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
                g = ((g - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
                b = ((b - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
                r = (r * brightness).clamp(0.0, 1.0);
                g = (g * brightness).clamp(0.0, 1.0);
                b = (b * brightness).clamp(0.0, 1.0);
                let (h, s, l) = self.rgb_to_hsl(r, g, b);
                let (r_new, g_new, b_new) = self.hsl_to_rgb(h, (s * saturation).clamp(0.0, 1.0), l);
                r = r_new; g = g_new; b = b_new;
                r = (r * temp_r).clamp(0.0, 1.0);
                g = (g * temp_g).clamp(0.0, 1.0);
                b = (b * temp_b).clamp(0.0, 1.0);
                frame_data[idx] = (r * 255.0) as u8;
                frame_data[idx + 1] = (g * 255.0) as u8;
                frame_data[idx + 2] = (b * 255.0) as u8;
            }
        }
    }

    fn apply_vignette(&self, frame_data: &mut [u8], width: usize, height: usize) {
        let vignette_strength = 0.3;
        let vignette_radius = 0.75;
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let max_dist = (center_x.powi(2) + center_y.powi(2)).sqrt() * vignette_radius;
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx.powi(2) + dy.powi(2)).sqrt();
                let factor = if distance > max_dist {
                    1.0 - vignette_strength
                } else {
                    1.0 - vignette_strength * (distance / max_dist).powi(2)
                };
                frame_data[idx] = (frame_data[idx] as f32 * factor) as u8;
                frame_data[idx + 1] = (frame_data[idx + 1] as f32 * factor) as u8;
                frame_data[idx + 2] = (frame_data[idx + 2] as f32 * factor) as u8;
            }
        }
    }

    fn rgb_to_hsl(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let l = (max + min) / 2.0;
        let s = if delta == 0.0 { 0.0 } else { delta / (1.0 - (2.0 * l - 1.0).abs()) };
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };
        let h = if h < 0.0 { h + 360.0 } else { h };
        (h / 360.0, s, l)
    }

    fn hsl_to_rgb(&self, h: f32, s: f32, l: f32) -> (f32, f32, f32) {
        if s == 0.0 { return (l, l, l); }
        let h = h * 360.0;
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        let (r1, g1, b1) = if h < 60.0 { (c, x, 0.0) }
        else if h < 120.0 { (x, c, 0.0) }
        else if h < 180.0 { (0.0, c, x) }
        else if h < 240.0 { (0.0, x, c) }
        else if h < 300.0 { (x, 0.0, c) }
        else { (c, 0.0, x) };
        (r1 + m, g1 + m, b1 + m)
    }


    pub fn update_config(&mut self, config: RendererConfig) -> Result<(), RendererError> {
        self.config = config;
        Ok(())
    }


    pub fn cleanup(&mut self) -> Result<(), RendererError> {
        if !self.is_initialized {
            return Ok(());
        }


        self.current_frame = None;


        self.frame_count = 0;


        {
            let mut state = self.state.lock().unwrap();
            state.is_rendering = false;
            state.last_render_time = std::time::Instant::now();
        }

        if self.config.use_hardware_acceleration {
            self.cleanup_hardware_acceleration();
        }

        self.cleanup_frame_buffers();

        self.cleanup_resources();


        log::debug!("Renderer cleanup completed");

        self.is_initialized = false;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
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

pub fn create_default_renderer() -> Result<Renderer, RendererError> {
    let mut renderer = Renderer::new(RendererConfig::default());
    renderer.initialize()?;
    Ok(renderer)
}
