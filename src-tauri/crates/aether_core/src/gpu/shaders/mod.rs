use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Device, ShaderModule, ComputePipeline, BindGroupLayout, PipelineLayout};
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};

pub mod pipeline;
pub mod compiler;
pub mod binder;
pub mod cache;

pub use pipeline::{ComputePipelineManager, PipelineHandle};
pub use compiler::{ShaderCompiler, CompiledShader, ShaderSource};
pub use binder::{ParameterBinder, BindGroupManager, BindingConfig};
pub use cache::{ShaderCache, CacheKey, CacheEntry};

/// Main shader system that coordinates all shader operations
pub struct ShaderSystem {
    device: Arc<Device>,
    pipeline_manager: Arc<ComputePipelineManager>,
    shader_compiler: Arc<ShaderCompiler>,
    parameter_binder: Arc<ParameterBinder>,
    shader_cache: Arc<ShaderCache>,
}

impl ShaderSystem {
    /// Create a new shader system
    pub fn new(device: Arc<Device>) -> Result<Self> {
        info!("Initializing GPU shader system");
        
        let pipeline_manager = Arc::new(ComputePipelineManager::new(device.clone())?);
        let shader_compiler = Arc::new(ShaderCompiler::new(device.clone())?);
        let parameter_binder = Arc::new(ParameterBinder::new(device.clone())?);
        let shader_cache = Arc::new(ShaderCache::new());
        
        Ok(Self {
            device,
            pipeline_manager,
            shader_compiler,
            parameter_binder,
            shader_cache,
        })
    }
    
    /// Create a compute pipeline from shader source
    pub fn create_compute_pipeline(
        &self,
        name: &str,
        shader_source: &str,
        binding_config: &BindingConfig,
    ) -> Result<PipelineHandle> {
        debug!("Creating compute pipeline: {}", name);
        
        // Compile shader
        let compiled_shader = self.shader_compiler.compile_shader(name, shader_source)?;
        
        // Create pipeline
        let pipeline = self.pipeline_manager.create_pipeline(
            name,
            compiled_shader,
            binding_config,
        )?;
        
        // Cache the pipeline
        let cache_key = CacheKey::new(name, shader_source);
        self.shader_cache.cache_pipeline(cache_key, pipeline.clone());
        
        info!("Created compute pipeline: {}", name);
        
        Ok(pipeline)
    }
    
    /// Get a cached pipeline or create if not exists
    pub fn get_or_create_pipeline(
        &self,
        name: &str,
        shader_source: &str,
        binding_config: &BindingConfig,
    ) -> Result<PipelineHandle> {
        let cache_key = CacheKey::new(name, shader_source);
        
        // Try to get from cache first
        if let Some(cached_pipeline) = self.shader_cache.get_pipeline(&cache_key) {
            debug!("Using cached pipeline: {}", name);
            return Ok(cached_pipeline);
        }
        
        // Create new pipeline if not cached
        self.create_compute_pipeline(name, shader_source, binding_config)
    }
    
    /// Create bind group for pipeline parameters
    pub fn create_bind_group(
        &self,
        pipeline: &PipelineHandle,
        bindings: &HashMap<String, wgpu::BindingResource>,
    ) -> Result<wgpu::BindGroup> {
        debug!("Creating bind group for pipeline: {}", pipeline.name());
        
        self.parameter_binder.create_bind_group(pipeline, bindings)
    }
    
    /// Execute a compute pipeline
    pub fn execute_pipeline(
        &self,
        pipeline: &PipelineHandle,
        bind_group: &wgpu::BindGroup,
        workgroup_count: (u32, u32, u32),
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<()> {
        debug!("Executing pipeline: {} with workgroups: {:?}", 
            pipeline.name(), workgroup_count);
        
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(&format!("Compute pass: {}", pipeline.name())),
        });
        
        compute_pass.set_pipeline(&pipeline.pipeline());
        compute_pass.set_bind_group(0, bind_group, &[]);
        compute_pass.dispatch_workgroups(
            workgroup_count.0,
            workgroup_count.1,
            workgroup_count.2,
        );
        
        Ok(())
    }
    
    /// Get pipeline manager
    pub fn pipeline_manager(&self) -> &ComputePipelineManager {
        &self.pipeline_manager
    }
    
    /// Get shader compiler
    pub fn shader_compiler(&self) -> &ShaderCompiler {
        &self.shader_compiler
    }
    
    /// Get parameter binder
    pub fn parameter_binder(&self) -> &ParameterBinder {
        &self.parameter_binder
    }
    
    /// Get shader cache
    pub fn shader_cache(&self) -> &ShaderCache {
        &self.shader_cache
    }
    
    /// Clear shader cache
    pub fn clear_cache(&self) -> Result<()> {
        info!("Clearing shader cache");
        self.shader_cache.clear();
        Ok(())
    }
    
    /// Get shader system statistics
    pub fn get_stats(&self) -> ShaderSystemStats {
        ShaderSystemStats {
            cached_pipelines: self.shader_cache.pipeline_count(),
            compiled_shaders: self.shader_compiler.compiled_count(),
            active_pipelines: self.pipeline_manager.pipeline_count(),
            cache_hit_rate: self.shader_cache.hit_rate(),
        }
    }
}

/// Shader system statistics
#[derive(Debug, Clone)]
pub struct ShaderSystemStats {
    pub cached_pipelines: usize,
    pub compiled_shaders: usize,
    pub active_pipelines: usize,
    pub cache_hit_rate: f64,
}

impl ShaderSystemStats {
    /// Format statistics for display
    pub fn format(&self) -> String {
        format!(
            "Pipelines: {} cached, {} active | Shaders: {} compiled | Cache hit rate: {:.1}%",
            self.cached_pipelines,
            self.active_pipelines,
            self.compiled_shaders,
            self.cache_hit_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shader_system_stats() {
        let stats = ShaderSystemStats {
            cached_pipelines: 5,
            compiled_shaders: 3,
            active_pipelines: 2,
            cache_hit_rate: 0.75,
        };
        
        let formatted = stats.format();
        assert!(formatted.contains("5 cached"));
        assert!(formatted.contains("2 active"));
        assert!(formatted.contains("3 compiled"));
        assert!(formatted.contains("75.0%"));
    }
}
