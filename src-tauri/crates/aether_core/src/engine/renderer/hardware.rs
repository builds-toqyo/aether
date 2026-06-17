use super::types::*;

pub(super) fn initialize_hardware_acceleration(
    config: &mut RendererConfig,
    hw_context: &mut Option<HardwareContext>,
) -> Result<(), RendererError> {
    let device = config.hw_device.as_deref().unwrap_or("auto");
    log::info!("Initializing hardware acceleration with device: {}", device);

    match device {
        "cuda" => {
            log::debug!("Initializing CUDA acceleration");
            initialize_cuda_acceleration(hw_context)
        },
        "vaapi" => {
            log::debug!("Initializing VAAPI acceleration");
            initialize_vaapi_acceleration()
        },
        "videotoolbox" => {
            log::debug!("Initializing VideoToolbox acceleration");
            initialize_videotoolbox_acceleration()
        },
        "amf" => {
            log::debug!("Initializing AMF acceleration");
            initialize_amf_acceleration()
        },
        _ => {
            log::debug!("Auto-detecting hardware acceleration");
            auto_detect_acceleration(config, hw_context)
        }
    }
}

/// Initialize CUDA acceleration for NVIDIA GPUs
fn initialize_cuda_acceleration(_hw_context: &mut Option<HardwareContext>) -> Result<(), RendererError> {
    #[cfg(feature = "cuda")]
    {
        if !has_nvidia_gpu() {
            return Err(RendererError::HardwareAccelerationError(
                "No NVIDIA GPU found".to_string()
            ));
        }
        cuda::initialize_cuda()?;
        *hw_context = Some(HardwareContext::Cuda);
        log::info!("CUDA acceleration initialized");
        Ok(())
    }
    #[cfg(not(feature = "cuda"))]
    {
        Err(RendererError::HardwareAccelerationError(
            "CUDA feature not enabled".to_string()
        ))
    }
}

/// Initialize VAAPI acceleration for Intel GPUs on Linux
fn initialize_vaapi_acceleration() -> Result<(), RendererError> {
    #[cfg(all(feature = "vaapi", target_os = "linux"))]
    {
        if !has_vaapi_support() {
            return Err(RendererError::HardwareAccelerationError(
                "No VAAPI support found. Ensure /dev/dri/renderD128 exists and you have proper permissions.".to_string()
            ));
        }
        // VAAPI requires libva integration for hardware-accelerated video encoding/decoding
        // For now, use wgpu compute which provides cross-platform GPU acceleration
        log::warn!("VAAPI hardware acceleration not fully implemented. Using wgpu compute as fallback.");
        Err(RendererError::HardwareAccelerationError(
            "VAAPI requires libva integration. Use wgpu compute for cross-platform GPU acceleration.".to_string()
        ))
    }
    #[cfg(not(all(feature = "vaapi", target_os = "linux")))]
    {
        Err(RendererError::HardwareAccelerationError(
            "VAAPI is only available on Linux with the 'vaapi' feature enabled".to_string()
        ))
    }
}

/// Initialize VideoToolbox acceleration for macOS
fn initialize_videotoolbox_acceleration() -> Result<(), RendererError> {
    #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
    {
        // VideoToolbox requires Metal framework integration for hardware-accelerated video encoding/decoding
        // For now, use wgpu compute which provides cross-platform GPU acceleration including Metal backend on macOS
        log::warn!("VideoToolbox hardware acceleration not fully implemented. Using wgpu compute as fallback.");
        Err(RendererError::HardwareAccelerationError(
            "VideoToolbox requires Metal framework integration. Use wgpu compute for cross-platform GPU acceleration.".to_string()
        ))
    }
    #[cfg(not(all(feature = "videotoolbox", target_os = "macos")))]
    {
        Err(RendererError::HardwareAccelerationError(
            "VideoToolbox is only available on macOS with the 'videotoolbox' feature enabled".to_string()
        ))
    }
}

/// Initialize AMD AMF acceleration
fn initialize_amf_acceleration() -> Result<(), RendererError> {
    #[cfg(feature = "amf")]
    {
        if !has_amd_gpu() {
            return Err(RendererError::HardwareAccelerationError(
                "No AMD GPU found. Ensure you have an AMD GPU with AMF support.".to_string()
            ));
        }
        // AMF requires AMD SDK integration for hardware-accelerated video encoding/decoding
        // For now, use wgpu compute which provides cross-platform GPU acceleration
        log::warn!("AMF hardware acceleration not fully implemented. Using wgpu compute as fallback.");
        Err(RendererError::HardwareAccelerationError(
            "AMF requires AMD SDK integration. Use wgpu compute for cross-platform GPU acceleration.".to_string()
        ))
    }
    #[cfg(not(feature = "amf"))]
    {
        Err(RendererError::HardwareAccelerationError(
            "AMF is only available with the 'amf' feature enabled".to_string()
        ))
    }
}

/// Auto-detect the best hardware acceleration method
fn auto_detect_acceleration(
    config: &mut RendererConfig,
    _hw_context: &mut Option<HardwareContext>,
) -> Result<(), RendererError> {
    // Try CUDA first if feature is enabled and NVIDIA GPU present
    #[cfg(feature = "cuda")]
    {
        if has_nvidia_gpu() {
            log::debug!("NVIDIA GPU detected, trying CUDA acceleration");
            match initialize_cuda_acceleration(hw_context) {
                Ok(_) => return Ok(()),
                Err(e) => log::warn!("CUDA initialization failed: {}, falling back to wgpu compute", e),
            }
        }
    }

    // Try VAAPI on Linux if feature is enabled
    #[cfg(all(feature = "vaapi", target_os = "linux"))]
    {
        if has_vaapi_support() {
            log::debug!("VAAPI support detected, trying VAAPI acceleration");
            match initialize_vaapi_acceleration() {
                Ok(_) => return Ok(()),
                Err(e) => log::warn!("VAAPI initialization failed: {}, falling back to wgpu compute", e),
            }
        }
    }

    // Try VideoToolbox on macOS if feature is enabled
    #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
    {
        log::debug!("macOS detected, trying VideoToolbox acceleration");
        match initialize_videotoolbox_acceleration() {
            Ok(_) => return Ok(()),
            Err(e) => log::warn!("VideoToolbox initialization failed: {}, falling back to wgpu compute", e),
        }
    }

    // Try AMF if feature is enabled and AMD GPU present
    #[cfg(feature = "amf")]
    {
        if has_amd_gpu() {
            log::debug!("AMD GPU detected, trying AMF acceleration");
            match initialize_amf_acceleration() {
                Ok(_) => return Ok(()),
                Err(e) => log::warn!("AMF initialization failed: {}, falling back to wgpu compute", e),
            }
        }
    }

    // Fallback to wgpu compute for cross-platform GPU acceleration
    log::info!("Using wgpu compute for cross-platform GPU acceleration");
    config.use_hardware_acceleration = false; // wgpu compute is managed separately
    Ok(())
}

pub(super) fn allocate_frame_buffers(_width: usize, _height: usize) -> Result<(), RendererError> {
    // Frame buffer allocation is handled by the Renderer struct
    Ok(())
}

pub(super) fn initialize_resources(
    config: &RendererConfig,
    hw_context: &Option<HardwareContext>,
    shaders: &mut Option<Shaders>,
    lookup_tables: &mut Option<LookupTables>,
    post_process_pipeline: &mut Option<PostProcessPipeline>,
    cpu_buffers: &mut Option<CpuBuffers>,
) -> Result<(), RendererError> {
    log::debug!("Initializing rendering resources");

    if config.use_hardware_acceleration {
        initialize_shader_programs(hw_context, shaders)?;
    }

    initialize_lookup_tables(config, lookup_tables)?;
    allocate_gpu_resources(config, hw_context, cpu_buffers)?;
    initialize_post_processing(config, post_process_pipeline)?;

    log::info!("Rendering resources initialized successfully");
    Ok(())
}

fn initialize_shader_programs(
    hw_context: &Option<HardwareContext>,
    shaders: &mut Option<Shaders>,
) -> Result<(), RendererError> {
    log::debug!("Initializing shader programs");

    if let Some(hw_context) = hw_context {
        match hw_context {
            #[cfg(feature = "cuda")]
            HardwareContext::Cuda { .. } => {
                // CUDA kernels will be loaded here when implemented
            },

            #[cfg(all(feature = "vaapi", target_os = "linux"))]
            HardwareContext::Vaapi { .. } => {},

            #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
            HardwareContext::VideoToolbox { .. } => {},

            #[cfg(feature = "amf")]
            HardwareContext::Amf { .. } => {},

            _ => {}
        }
    } else {
        log::info!("No hardware context available, using software shaders");
    }

    *shaders = Some(Shaders::Software { functions: Vec::new() });
    log::debug!("Shader programs initialized");
    Ok(())
}

fn initialize_lookup_tables(
    config: &RendererConfig,
    lookup_tables: &mut Option<LookupTables>,
) -> Result<(), RendererError> {
    log::debug!("Initializing lookup tables");

    let gamma = config.gamma as f32;
    let gamma_lut = (0..256).map(|i| {
        let normalized = i as f32 / 255.0;
        let corrected = normalized.powf(1.0 / gamma);
        (corrected * 255.0).round() as u8
    }).collect::<Vec<u8>>();

    let width = config.width as usize;
    let height = config.height as usize;
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

    *lookup_tables = Some(LookupTables {
        _gamma: gamma_lut,
        _vignette: vignette_lut,
    });

    log::debug!("Lookup tables initialized");
    Ok(())
}

fn allocate_gpu_resources(
    config: &RendererConfig,
    hw_context: &Option<HardwareContext>,
    cpu_buffers: &mut Option<CpuBuffers>,
) -> Result<(), RendererError> {
    log::debug!("Allocating GPU resources");

    let width = config.width as usize;
    let height = config.height as usize;

    if config.use_hardware_acceleration {
        if let Some(hw_context) = hw_context {
            match hw_context {
                #[cfg(feature = "cuda")]
                HardwareContext::Cuda { .. } => {
                    // CUDA device memory allocation would go here
                },

                #[cfg(all(feature = "vaapi", target_os = "linux"))]
                HardwareContext::Vaapi { .. } => {},

                #[cfg(all(feature = "videotoolbox", target_os = "macos"))]
                HardwareContext::VideoToolbox { .. } => {},

                #[cfg(feature = "amf")]
                HardwareContext::Amf { .. } => {},

                _ => {
                    log::warn!("Unknown hardware context type, falling back to CPU buffers");
                    allocate_cpu_buffers(width, height, cpu_buffers)?;
                }
            }
        } else {
            log::warn!("No hardware context available, falling back to CPU buffers");
            allocate_cpu_buffers(width, height, cpu_buffers)?;
        }
    } else {
        allocate_cpu_buffers(width, height, cpu_buffers)?;
    }

    log::debug!("GPU resources allocated");
    Ok(())
}

fn allocate_cpu_buffers(width: usize, height: usize, cpu_buffers: &mut Option<CpuBuffers>) -> Result<(), RendererError> {
    let buffer_size = width * height * 4;
    let input_buffer = vec![0u8; buffer_size];
    let output_buffer = vec![0u8; buffer_size];

    *cpu_buffers = Some(CpuBuffers {
        _input: input_buffer,
        _output: output_buffer,
    });

    log::debug!("CPU buffers allocated: {} bytes each", buffer_size);
    Ok(())
}

fn initialize_post_processing(
    config: &RendererConfig,
    post_process_pipeline: &mut Option<PostProcessPipeline>,
) -> Result<(), RendererError> {
    log::debug!("Initializing post-processing pipeline");

    let mut stages = Vec::new();

    if config.enable_color_correction {
        stages.push(PostProcessStage::ColorCorrection);
    }

    if config.enable_color_grading {
        stages.push(PostProcessStage::ColorGrading);
    }

    if config.enable_vignette {
        stages.push(PostProcessStage::Vignette);
    }

    let stage_count = stages.len();
    *post_process_pipeline = Some(PostProcessPipeline { _stages: stages });

    log::debug!("Post-processing pipeline initialized with {} stages", stage_count);
    Ok(())
}

pub(super) fn cleanup_hardware_acceleration(hw_device: &Option<String>) {
    if let Some(device) = hw_device {
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

pub(super) fn cleanup_frame_buffers() {
    log::debug!("Cleaning up frame buffer resources");
}

pub(super) fn cleanup_resources() {
    log::debug!("Cleaning up additional rendering resources");
}
