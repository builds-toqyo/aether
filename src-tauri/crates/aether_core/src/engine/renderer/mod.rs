mod types;
mod shader;
mod gpu;
mod post_process;
mod hardware;
mod cuda;

pub use types::{Frame, RendererConfig, RendererError};

use std::sync::{Arc, Mutex};
use types::*;
use gpu::WgpuComputeState;

/// Hardware-accelerated video renderer with wgpu compute fallback
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
    wgpu_compute: Option<WgpuComputeState>,
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
            wgpu_compute: None,
        }
    }

    pub fn initialize(&mut self) -> Result<(), RendererError> {
        if self.is_initialized {
            return Ok(());
        }

        log::debug!("Initializing renderer with resolution: {}x{}", self.config.width, self.config.height);

        if self.config.use_hardware_acceleration {
            hardware::initialize_hardware_acceleration(&mut self.config, &mut self.hw_context)?;
        } else {
            log::debug!("Using software rendering");
        }

        // Always try to initialize wgpu compute for cross-platform GPU post-processing
        if self.wgpu_compute.is_none() {
            match WgpuComputeState::new(&self.config) {
                Ok(state) => {
                    self.wgpu_compute = Some(state);
                    log::info!("wgpu compute available for post-processing");
                }
                Err(e) => {
                    log::warn!("wgpu compute not available: {}", e);
                }
            }
        }

        hardware::allocate_frame_buffers(self.config.width as usize, self.config.height as usize)?;
        hardware::initialize_resources(
            &self.config,
            &self.hw_context,
            &mut self.shaders,
            &mut self.lookup_tables,
            &mut self.post_process_pipeline,
            &mut self.cpu_buffers,
        )?;

        self.is_initialized = true;
        log::debug!("Renderer initialized successfully");
        Ok(())
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

        if let Some(ref wgpu) = self.wgpu_compute {
            post_process::apply_post_processing_gpu(wgpu, frame_data, width, height)?;
        } else {
            post_process::apply_post_processing_cpu(frame_data, width, height);
        }

        Ok(())
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
            hardware::cleanup_hardware_acceleration(&self.config.hw_device);
        }

        self.wgpu_compute = None;
        hardware::cleanup_frame_buffers();
        hardware::cleanup_resources();

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

pub fn create_default_renderer() -> Result<Renderer, RendererError> {
    let mut renderer = Renderer::new(RendererConfig::default());
    renderer.initialize()?;
    Ok(renderer)
}
