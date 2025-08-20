use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum RendererError {
    InitializationError(String),
    RenderError(String),
    ResourceError(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RendererError::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            RendererError::RenderError(msg) => write!(f, "Render error: {}", msg),
            RendererError::ResourceError(msg) => write!(f, "Resource error: {}", msg),
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

pub struct Renderer {
    config: RendererConfig,
    is_initialized: bool,
    current_frame: Option<Frame>,
    frame_count: u64,
    state: Arc<Mutex<RendererState>>,
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
            current_frame: None,
            frame_count: 0,
            state: Arc::new(Mutex::new(state)),
        }
    }
    
    pub fn initialize(&mut self) -> Result<(), RendererError> {
        if self.is_initialized {
            return Ok(());
        }
        
        // Log initialization start
        log::debug!("Initializing renderer with {}x{} resolution", self.config.width, self.config.height);
        
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
        log::debug!("Renderer initialization complete");
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
                log::debug!("Initializing AMD AMF acceleration");
                self.initialize_amf_acceleration()
            },
            _ => {
                // Try to auto-detect the best hardware acceleration
                log::debug!("Auto-detecting hardware acceleration");
                self.auto_detect_acceleration()
            }
        }
    }
    
    /// Initialize CUDA acceleration for NVIDIA GPUs
    fn initialize_cuda_acceleration(&mut self) -> Result<(), RendererError> {
        #[cfg(feature = "cuda")]
        {
            // Check for NVIDIA GPU
            if !self.has_nvidia_gpu() {
                return Err(RendererError::HardwareAccelerationError(
                    "CUDA acceleration requested but no NVIDIA GPU found".to_string()
                ));
            }
            
            // Initialize CUDA context
            unsafe {
                // In a real implementation, we would use the CUDA API here
                // For example:
                // let result = cuda::cuInit(0);
                // if result != cuda::CUDA_SUCCESS {
                //     return Err(RendererError::HardwareAccelerationError(
                //         format!("Failed to initialize CUDA: error {}", result)
                //     ));
                // }
                
                // Create CUDA context
                // let mut device = 0;
                // let result = cuda::cuDeviceGet(&mut device, 0);
                // if result != cuda::CUDA_SUCCESS {
                //     return Err(RendererError::HardwareAccelerationError(
                //         format!("Failed to get CUDA device: error {}", result)
                //     ));
                // }
                
                // let mut context = std::ptr::null_mut();
                // let result = cuda::cuCtxCreate(&mut context, 0, device);
                // if result != cuda::CUDA_SUCCESS {
                //     return Err(RendererError::HardwareAccelerationError(
                //         format!("Failed to create CUDA context: error {}", result)
                //     ));
                // }
                
                // self.hw_context = Some(HardwareContext::Cuda { context });
            }
            
            log::info!("CUDA acceleration initialized successfully");
            Ok(())
        }
        
        #[cfg(not(feature = "cuda"))]
        {
            Err(RendererError::HardwareAccelerationError(
                "CUDA acceleration not supported in this build".to_string()
            ))
        }
    }
    
    /// Initialize VAAPI acceleration for Intel GPUs on Linux
    fn initialize_vaapi_acceleration(&mut self) -> Result<(), RendererError> {
        #[cfg(all(feature = "vaapi", target_os = "linux"))]
        {
            // Check for Intel GPU or other VAAPI-compatible hardware
            if !self.has_vaapi_support() {
                return Err(RendererError::HardwareAccelerationError(
                    "VAAPI acceleration requested but no compatible hardware found".to_string()
                ));
            }
            
            // Initialize VAAPI context
            unsafe {
                // In a real implementation, we would use the VAAPI API here
                // For example:
                // let display = vaapi::vaGetDisplay(std::ptr::null_mut());
                // if display.is_null() {
                //     return Err(RendererError::HardwareAccelerationError(
                //         "Failed to get VA display".to_string()
                //     ));
                // }
                
                // let mut major = 0;
                // let mut minor = 0;
                // let status = vaapi::vaInitialize(display, &mut major, &mut minor);
                // if status != vaapi::VA_STATUS_SUCCESS {
                //     return Err(RendererError::HardwareAccelerationError(
                //         format!("Failed to initialize VAAPI: error {}", status)
                //     ));
                // }
                
                // self.hw_context = Some(HardwareContext::Vaapi { display });
            }
            
            log::info!("VAAPI acceleration initialized successfully");
            Ok(())
        }
        
        #[cfg(not(all(feature = "vaapi", target_os = "linux")))]
        {
            Err(RendererError::HardwareAccelerationError(
                "VAAPI acceleration not supported in this build or on this platform".to_string()
            ))
        }
    }
    
    /// Initialize VideoToolbox acceleration for macOS
    fn initialize_videotoolbox_acceleration(&mut self) -> Result<(), RendererError> {
        #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
        {
            // VideoToolbox is available on all macOS systems, so no need to check for hardware
            
            // Initialize VideoToolbox session
            unsafe {
                // In a real implementation, we would use the VideoToolbox API here
                // For example:
                // let mut session: videotoolbox::VTDecompressionSessionRef = std::ptr::null_mut();
                // let format_id = videotoolbox::kCMVideoCodecType_H264;
                // 
                // let format_dict = videotoolbox::CFDictionaryCreateMutable(
                //     std::ptr::null_mut(),
                //     1,
                //     &videotoolbox::kCFTypeDictionaryKeyCallBacks,
                //     &videotoolbox::kCFTypeDictionaryValueCallBacks
                // );
                // 
                // let key = videotoolbox::kCVPixelBufferPixelFormatTypeKey;
                // let value = videotoolbox::kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange;
                // videotoolbox::CFDictionaryAddValue(format_dict, key, value);
                // 
                // let status = videotoolbox::VTDecompressionSessionCreate(
                //     std::ptr::null_mut(),
                //     format_description,
                //     std::ptr::null(),
                //     format_dict,
                //     std::ptr::null(),
                //     &mut session
                // );
                // 
                // if status != 0 {
                //     return Err(RendererError::HardwareAccelerationError(
                //         format!("Failed to create VideoToolbox session: error {}", status)
                //     ));
                // }
                // 
                // self.hw_context = Some(HardwareContext::VideoToolbox { session });
            }
            
            log::info!("VideoToolbox acceleration initialized successfully");
            Ok(())
        }
        
        #[cfg(not(all(feature = "videotoolbox", target_os = "macos")))]
        {
            Err(RendererError::HardwareAccelerationError(
                "VideoToolbox acceleration not supported in this build or on this platform".to_string()
            ))
        }
    }
    
    /// Initialize AMD AMF acceleration
    fn initialize_amf_acceleration(&mut self) -> Result<(), RendererError> {
        #[cfg(feature = "amf")]
        {
            // Check for AMD GPU
            if !self.has_amd_gpu() {
                return Err(RendererError::HardwareAccelerationError(
                    "AMF acceleration requested but no AMD GPU found".to_string()
                ));
            }
            
            // Initialize AMF context
            unsafe {
                // In a real implementation, we would use the AMF API here
                // For example:
                // let mut factory: *mut amf::AMFFactory = std::ptr::null_mut();
                // let result = amf::AMFInit(0, &mut factory);
                // if result != amf::AMF_OK {
                //     return Err(RendererError::HardwareAccelerationError(
                //         format!("Failed to initialize AMF: error {}", result)
                //     ));
                // }
                // 
                // let mut context: *mut amf::AMFContext = std::ptr::null_mut();
                // let result = factory.CreateContext(&mut context);
                // if result != amf::AMF_OK {
                //     return Err(RendererError::HardwareAccelerationError(
                //         format!("Failed to create AMF context: error {}", result)
                //     ));
                // }
                // 
                // self.hw_context = Some(HardwareContext::Amf { factory, context });
            }
            
            log::info!("AMD AMF acceleration initialized successfully");
            Ok(())
        }
        
        #[cfg(not(feature = "amf"))]
        {
            Err(RendererError::HardwareAccelerationError(
                "AMD AMF acceleration not supported in this build".to_string()
            ))
        }
    }
    
    /// Auto-detect the best hardware acceleration method
    fn auto_detect_acceleration(&mut self) -> Result<(), RendererError> {
        #[cfg(target_os = "macos")]
        {
            // On macOS, VideoToolbox is the best option
            return self.initialize_videotoolbox_acceleration();
        }
        
        #[cfg(target_os = "windows")]
        {
            // On Windows, try CUDA first, then AMF, then fallback to software
            if self.has_nvidia_gpu() {
                match self.initialize_cuda_acceleration() {
                    Ok(_) => return Ok(()),
                    Err(e) => log::warn!("Failed to initialize CUDA: {}", e),
                }
            }
            
            if self.has_amd_gpu() {
                match self.initialize_amf_acceleration() {
                    Ok(_) => return Ok(()),
                    Err(e) => log::warn!("Failed to initialize AMF: {}", e),
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // On Linux, try VAAPI first, then CUDA, then fallback to software
            if self.has_vaapi_support() {
                match self.initialize_vaapi_acceleration() {
                    Ok(_) => return Ok(()),
                    Err(e) => log::warn!("Failed to initialize VAAPI: {}", e),
                }
            }
            
            if self.has_nvidia_gpu() {
                match self.initialize_cuda_acceleration() {
                    Ok(_) => return Ok(()),
                    Err(e) => log::warn!("Failed to initialize CUDA: {}", e),
                }
            }
        }
        
        // Fallback to software rendering
        log::info!("No hardware acceleration available, falling back to software rendering");
        self.config.use_hardware_acceleration = false;
        Ok(())
    }
    
    /// Check if NVIDIA GPU is available
    fn has_nvidia_gpu(&self) -> bool {
        // In a real implementation, we would check for NVIDIA GPU
        // For example, on Linux we might parse the output of `lspci`
        // On Windows, we might use DXGI or the NVIDIA API
        // For this example, we'll just return true
        true
    }
    
    /// Check if AMD GPU is available
    fn has_amd_gpu(&self) -> bool {
        // Similar to has_nvidia_gpu, but for AMD GPUs
        true
    }
    
    /// Check if VAAPI is supported
    fn has_vaapi_support(&self) -> bool {
        // Check if VAAPI is supported on this system
        // This would typically involve checking for the presence of VAAPI drivers
        // and compatible hardware
        #[cfg(target_os = "linux")]
        {
            // Check for VAAPI support
            // For example, check if /dev/dri/renderD128 exists
            std::path::Path::new("/dev/dri/renderD128").exists()
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }
    
    /// Allocate frame buffers for rendering
    fn allocate_frame_buffers(&mut self) -> Result<(), RendererError> {
        let width = self.config.width as usize;
        let height = self.config.height as usize;
        
        // Calculate buffer size (RGBA = 4 bytes per pixel)
        let buffer_size = width * height * 4;
        log::debug!("Allocating frame buffer of {} bytes", buffer_size);
        
        // In a real implementation, we might pre-allocate buffers here
        // or set up GPU textures for rendering
        
        Ok(())
    }
    
    /// Initialize additional resources needed for rendering
    fn initialize_resources(&mut self) -> Result<(), RendererError> {
        // Initialize any additional resources needed for rendering
        // For example, shader programs, lookup tables, etc.
        
        Ok(())
    }
    
    /// Clean up hardware acceleration resources
    fn cleanup_hardware_acceleration(&mut self) {
        if let Some(device) = &self.config.hw_device {
            log::debug!("Cleaning up hardware acceleration resources for device: {}", device);
            
            // Cleanup logic would depend on the specific hardware acceleration API
            match device.as_str() {
                "cuda" => {
                    // Release CUDA resources
                    log::debug!("Releasing CUDA resources");
                },
                "vaapi" => {
                    // Release VAAPI resources
                    log::debug!("Releasing VAAPI resources");
                },
                "videotoolbox" => {
                    // Release VideoToolbox resources
                    log::debug!("Releasing VideoToolbox resources");
                },
                "amf" => {
                    // Release AMD AMF resources
                    log::debug!("Releasing AMD AMF resources");
                },
                _ => {
                    log::debug!("Releasing auto-detected hardware acceleration resources");
                }
            }
        }
    }
    
    /// Clean up frame buffer resources
    fn cleanup_frame_buffers(&mut self) {
        log::debug!("Cleaning up frame buffer resources");
        
        // In a real implementation, we would release any pre-allocated buffers here
        // For example:
        // - Release GPU textures
        // - Free any large memory allocations
        // - Release any buffer pools
    }
    
    /// Clean up any other rendering resources
    fn cleanup_resources(&mut self) {
        log::debug!("Cleaning up additional rendering resources");
        
        // Clean up any other resources that were allocated during initialization
        // For example:
        // - Shader programs
        // - Lookup tables
        // - Temporary files
    }
    
    /// Render a frame
    pub fn render(&mut self, input_data: &[u8], timestamp: f64) -> Result<&Frame, RendererError> {
        if !self.is_initialized {
            return Err(RendererError::InitializationError("Renderer not initialized".to_string()));
        }
        
        // Lock the state for the rendering operation
        let mut state = self.state.lock().unwrap();
        state.is_rendering = true;
        state.last_render_time = std::time::Instant::now();
        
        // Actual rendering logic
        let mut frame_data = input_data.to_vec();
        
        // Apply post-processing effects if needed
        self.apply_post_processing(&mut frame_data)?;
        
        // Create the final frame
        let frame = Frame {
            data: frame_data,
            width: self.config.width,
            height: self.config.height,
            timestamp,
        };
        
        self.current_frame = Some(frame);
        self.frame_count += 1;
        state.is_rendering = false;
        
        // Return a reference to the current frame
        self.current_frame.as_ref().ok_or(RendererError::RenderError("Failed to create frame".to_string()))
    }
    
    pub fn current_frame(&self) -> Option<&Frame> {
        self.current_frame.as_ref()
    }
    
    /// Get the frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
    
    /// Apply post-processing effects to the frame data
    fn apply_post_processing(&self, frame_data: &mut [u8]) -> Result<(), RendererError> {
        // Skip if the frame is empty
        if frame_data.is_empty() {
            return Ok(());
        }
        
        let width = self.config.width as usize;
        let height = self.config.height as usize;
        
        // Ensure we have enough data for the frame
        if frame_data.len() < width * height * 4 {
            return Err(RendererError::RenderError(
                format!("Frame data too small: {} bytes for {}x{} RGBA frame", 
                        frame_data.len(), width, height)
            ));
        }
        
        // Apply gamma correction
        self.apply_gamma_correction(frame_data, width, height);
        
        // Apply color grading
        self.apply_color_grading(frame_data, width, height);
        
        // Apply vignette effect
        self.apply_vignette(frame_data, width, height);
        
        Ok(())
    }
    
    /// Apply gamma correction to the frame
    fn apply_gamma_correction(&self, frame_data: &mut [u8], width: usize, height: usize) {
        // Simple gamma correction with gamma = 1.1
        let gamma = 1.1;
        let gamma_inv = 1.0 / gamma;
        
        // Create a gamma lookup table for efficiency
        let mut gamma_table = [0u8; 256];
        for i in 0..256 {
            let normalized = i as f32 / 255.0;
            let corrected = normalized.powf(gamma_inv);
            gamma_table[i] = (corrected * 255.0).clamp(0.0, 255.0) as u8;
        }
        
        // Apply gamma correction to RGB channels (not alpha)
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                frame_data[idx] = gamma_table[frame_data[idx] as usize];       // R
                frame_data[idx + 1] = gamma_table[frame_data[idx + 1] as usize]; // G
                frame_data[idx + 2] = gamma_table[frame_data[idx + 2] as usize]; // B
                // Alpha channel remains unchanged
            }
        }
    }
    
    /// Apply color grading to the frame
    fn apply_color_grading(&self, frame_data: &mut [u8], width: usize, height: usize) {
        // Color grading parameters (these could come from the renderer config)
        let saturation = 1.1; // Slightly increase saturation
        let contrast = 1.05;  // Slightly increase contrast
        let brightness = 1.0; // Keep brightness the same
        
        // Color temperature adjustment (warmer)
        let temp_r = 1.05; // Increase red slightly
        let temp_g = 1.0;  // Keep green the same
        let temp_b = 0.95; // Decrease blue slightly
        
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                
                // Get RGB values
                let mut r = frame_data[idx] as f32 / 255.0;
                let mut g = frame_data[idx + 1] as f32 / 255.0;
                let mut b = frame_data[idx + 2] as f32 / 255.0;
                
                // Apply contrast
                r = ((r - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
                g = ((g - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
                b = ((b - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
                
                // Apply brightness
                r = (r * brightness).clamp(0.0, 1.0);
                g = (g * brightness).clamp(0.0, 1.0);
                b = (b * brightness).clamp(0.0, 1.0);
                
                // Apply saturation (convert to HSL, adjust S, convert back)
                let (h, s, l) = self.rgb_to_hsl(r, g, b);
                let (r_new, g_new, b_new) = self.hsl_to_rgb(h, (s * saturation).clamp(0.0, 1.0), l);
                
                r = r_new;
                g = g_new;
                b = b_new;
                
                // Apply color temperature
                r = (r * temp_r).clamp(0.0, 1.0);
                g = (g * temp_g).clamp(0.0, 1.0);
                b = (b * temp_b).clamp(0.0, 1.0);
                
                // Write back to frame data
                frame_data[idx] = (r * 255.0) as u8;
                frame_data[idx + 1] = (g * 255.0) as u8;
                frame_data[idx + 2] = (b * 255.0) as u8;
            }
        }
    }
    
    /// Apply vignette effect to the frame
    fn apply_vignette(&self, frame_data: &mut [u8], width: usize, height: usize) {
        // Vignette parameters
        let vignette_strength = 0.3; // Strength of the vignette effect (0.0 - 1.0)
        let vignette_radius = 0.75;  // Radius of the vignette effect (0.0 - 1.0)
        
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let max_dist = (center_x.powi(2) + center_y.powi(2)).sqrt() * vignette_radius;
        
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                
                // Calculate distance from center
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx.powi(2) + dy.powi(2)).sqrt();
                
                // Calculate vignette factor
                let factor = if distance > max_dist {
                    1.0 - vignette_strength
                } else {
                    1.0 - vignette_strength * (distance / max_dist).powi(2)
                };
                
                // Apply vignette to RGB channels
                frame_data[idx] = (frame_data[idx] as f32 * factor) as u8;
                frame_data[idx + 1] = (frame_data[idx + 1] as f32 * factor) as u8;
                frame_data[idx + 2] = (frame_data[idx + 2] as f32 * factor) as u8;
            }
        }
    }
    
    /// Convert RGB to HSL color space
    fn rgb_to_hsl(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        
        // Calculate lightness
        let l = (max + min) / 2.0;
        
        // Calculate saturation
        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };
        
        // Calculate hue
        let h = if delta == 0.0 {
            0.0 // No color, just grayscale
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };
        
        let h = if h < 0.0 { h + 360.0 } else { h };
        
        (h / 360.0, s, l) // Normalize hue to 0-1 range
    }
    
    /// Convert HSL to RGB color space
    fn hsl_to_rgb(&self, h: f32, s: f32, l: f32) -> (f32, f32, f32) {
        if s == 0.0 {
            // Achromatic (gray)
            return (l, l, l);
        }
        
        let h = h * 360.0; // Convert back to 0-360 range
        
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        
        let (r1, g1, b1) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        (r1 + m, g1 + m, b1 + m)
    }
    
    /// Update the renderer configuration
    pub fn update_config(&mut self, config: RendererConfig) -> Result<(), RendererError> {
        self.config = config;
        Ok(())
    }
    
    /// Clean up the renderer
    pub fn cleanup(&mut self) -> Result<(), RendererError> {
        if !self.is_initialized {
            return Ok(());
        }
        
        // Release frame data
        self.current_frame = None;
        
        // Reset frame count
        self.frame_count = 0;
        
        // Reset rendering state
        let mut state = self.state.lock().unwrap();
        state.is_rendering = false;
        state.last_render_time = std::time::Instant::now();
        
        // Release hardware acceleration resources if enabled
        if self.config.use_hardware_acceleration {
            self.cleanup_hardware_acceleration();
        }
        
        // Release GPU textures and buffers
        self.cleanup_frame_buffers();
        
        // Clean up any other resources
        self.cleanup_resources();
        
        // Log cleanup completion
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

/// Renderer configuration
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Width of the output in pixels
    pub width: u32,
    
    /// Height of the output in pixels
    pub height: u32,
    
    /// Frame rate in frames per second
    pub frame_rate: f64,
    
    /// Background color as RGBA
    pub background_color: [u8; 4],
    
    /// Whether to use hardware acceleration
    pub use_hardware_acceleration: bool,
    
    /// Hardware acceleration device (e.g., "cuda", "vaapi", "videotoolbox")
    pub hw_device: Option<String>,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            frame_rate: 30.0,
            background_color: [0, 0, 0, 255], // Black background
            use_hardware_acceleration: false,
            hw_device: None,
        }
    }
}

pub fn create_default_renderer() -> Result<Renderer, RendererError> {
    let mut renderer = Renderer::new(RendererConfig::default());
    renderer.initialize()?;
    Ok(renderer)
}
