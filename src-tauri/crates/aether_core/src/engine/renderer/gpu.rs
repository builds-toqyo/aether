use super::types::{RendererConfig, RendererError, PostProcessUniforms};
use super::shader::COMPUTE_SHADER;

/// wgpu GPU compute state managed separately for clarity
pub(super) struct WgpuComputeState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub input_buffer: wgpu::Buffer,
    pub output_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub staging_buffer: wgpu::Buffer,
}

impl WgpuComputeState {
    pub fn new(config: &RendererConfig) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });

        let adapter = pollster::block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
        )
        .map_err(|e| RendererError::HardwareAccelerationError(
            format!("No suitable GPU adapter found: {}", e)
        ))?;

        let info = adapter.get_info();
        log::info!("Selected GPU: {} ({:?})", info.name, info.backend);

        let (device, queue) = pollster::block_on(
            adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Aether Renderer"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                }
            )
        )
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
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
            ..Default::default()
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("post_process_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let width = config.width as u64;
        let height = config.height as u64;
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

        log::info!("wgpu compute pipeline initialized ({}x{})", width, height);

        Ok(Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
            input_buffer,
            output_buffer,
            uniform_buffer,
            staging_buffer,
        })
    }
}

/// Detect GPU vendor using wgpu adapter enumeration
pub(super) fn detect_gpu_vendor() -> Option<&'static str> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
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

pub(super) fn has_nvidia_gpu() -> bool {
    detect_gpu_vendor() == Some("nvidia")
}

pub(super) fn has_amd_gpu() -> bool {
    detect_gpu_vendor() == Some("amd")
}

pub(super) fn has_vaapi_support() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new("/dev/dri/renderD128").exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}
