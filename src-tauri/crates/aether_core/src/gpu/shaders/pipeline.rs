use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wgpu::{Device, ComputePipeline, PipelineLayout, BindGroupLayout, ShaderModule};
use anyhow::{Result, anyhow};
use log::{debug, info, warn};

use super::{CompiledShader, BindingConfig};

/// Manager for compute pipelines
pub struct ComputePipelineManager {
    device: Arc<Device>,
    pipelines: Arc<Mutex<HashMap<Uuid, PipelineEntry>>>,
    layouts: Arc<Mutex<HashMap<String, Arc<BindGroupLayout>>>>,
}

impl ComputePipelineManager {
    /// Create a new pipeline manager
    pub fn new(device: Arc<Device>) -> Result<Self> {
        info!("Creating compute pipeline manager");
        
        Ok(Self {
            device,
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            layouts: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Create a compute pipeline
    pub fn create_pipeline(
        &self,
        name: &str,
        compiled_shader: CompiledShader,
        binding_config: &BindingConfig,
    ) -> Result<PipelineHandle> {
        debug!("Creating compute pipeline: {}", name);
        
        // Create bind group layout
        let bind_group_layout = self.create_bind_group_layout(name, binding_config)?;
        
        // Create pipeline layout
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("Pipeline Layout: {}", name)),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create compute pipeline
        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("Compute Pipeline: {}", name)),
            layout: Some(&pipeline_layout),
            module: &compiled_shader.module,
            entry_point: compiled_shader.entry_point,
        });
        
        let pipeline_id = Uuid::new_v4();
        let pipeline_entry = PipelineEntry {
            id: pipeline_id,
            name: name.to_string(),
            pipeline: Arc::new(pipeline),
            layout: Arc::new(pipeline_layout),
            bind_group_layout: Arc::new(bind_group_layout),
            created_at: std::time::Instant::now(),
        };
        
        // Store pipeline
        let mut pipelines = self.pipelines.lock().map_err(|e| anyhow!("Pipeline lock error: {}", e))?;
        pipelines.insert(pipeline_id, pipeline_entry.clone());
        
        info!("Created compute pipeline: {} ({})", name, pipeline_id);
        
        Ok(PipelineHandle {
            id: pipeline_id,
            entry: Arc::new(pipeline_entry),
        })
    }
    
    /// Get a pipeline by ID
    pub fn get_pipeline(&self, pipeline_id: &Uuid) -> Result<PipelineHandle> {
        let pipelines = self.pipelines.lock().map_err(|e| anyhow!("Pipeline lock error: {}", e))?;
        
        pipelines.get(pipeline_id)
            .map(|entry| PipelineHandle {
                id: *pipeline_id,
                entry: Arc::new(entry.clone()),
            })
            .ok_or_else(|| anyhow!("Pipeline not found: {}", pipeline_id))
    }
    
    /// Create bind group layout for pipeline
    fn create_bind_group_layout(&self, name: &str, config: &BindingConfig) -> Result<BindGroupLayout> {
        debug!("Creating bind group layout: {}", name);
        
        let layout_key = format!("{}-{:?}", name, config.bindings.len());
        
        // Check if layout already exists
        {
            let layouts = self.layouts.lock().map_err(|e| anyhow!("Layout lock error: {}", e))?;
            if let Some(existing_layout) = layouts.get(&layout_key) {
                return Ok((**existing_layout).clone());
            }
        }
        
        // Create new layout
        let mut entries = Vec::new();
        for (index, binding) in config.bindings.iter().enumerate() {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: index as u32,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: binding.binding_type.clone(),
                has_dynamic_offset: false,
                min_binding_size: binding.min_size,
            });
        }
        
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("Bind Group Layout: {}", name)),
            entries: &entries,
        });
        
        // Cache layout
        let mut layouts = self.layouts.lock().map_err(|e| anyhow!("Layout lock error: {}", e))?;
        layouts.insert(layout_key, Arc::new(layout.clone()));
        
        Ok(layout)
    }
    
    /// Remove a pipeline
    pub fn remove_pipeline(&self, pipeline_id: &Uuid) -> Result<()> {
        debug!("Removing pipeline: {}", pipeline_id);
        
        let mut pipelines = self.pipelines.lock().map_err(|e| anyhow!("Pipeline lock error: {}", e))?;
        if pipelines.remove(pipeline_id).is_some() {
            info!("Removed pipeline: {}", pipeline_id);
        }
        
        Ok(())
    }
    
    /// Clear all pipelines
    pub fn clear_pipelines(&self) -> Result<()> {
        warn!("Clearing all compute pipelines");
        
        let mut pipelines = self.pipelines.lock().map_err(|e| anyhow!("Pipeline lock error: {}", e))?;
        pipelines.clear();
        
        let mut layouts = self.layouts.lock().map_err(|e| anyhow!("Layout lock error: {}", e))?;
        layouts.clear();
        
        info!("Cleared all pipelines and layouts");
        
        Ok(())
    }
    
    /// Get pipeline count
    pub fn pipeline_count(&self) -> usize {
        self.pipelines.lock().map(|p| p.len()).unwrap_or(0)
    }
    
    /// Get all pipeline names
    pub fn pipeline_names(&self) -> Vec<String> {
        self.pipelines.lock()
            .map(|pipelines| pipelines.values().map(|p| p.name.clone()).collect())
            .unwrap_or_default()
    }
    
    /// Get pipeline statistics
    pub fn get_stats(&self) -> PipelineStats {
        let pipelines = self.pipelines.lock().map(|p| p.len()).unwrap_or(0);
        let layouts = self.layouts.lock().map(|l| l.len()).unwrap_or(0);
        
        PipelineStats {
            active_pipelines: pipelines,
            cached_layouts: layouts,
        }
    }
}

/// Pipeline entry in the manager
#[derive(Debug, Clone)]
pub struct PipelineEntry {
    pub id: Uuid,
    pub name: String,
    pub pipeline: Arc<ComputePipeline>,
    pub layout: Arc<PipelineLayout>,
    pub bind_group_layout: Arc<BindGroupLayout>,
    pub created_at: std::time::Instant,
}

/// Handle for a compute pipeline
#[derive(Debug, Clone)]
pub struct PipelineHandle {
    pub id: Uuid,
    entry: Arc<PipelineEntry>,
}

impl PipelineHandle {
    /// Get pipeline ID
    pub fn id(&self) -> Uuid {
        self.id
    }
    
    /// Get pipeline name
    pub fn name(&self) -> &str {
        &self.entry.name
    }
    
    /// Get the compute pipeline
    pub fn pipeline(&self) -> &ComputePipeline {
        &self.entry.pipeline
    }
    
    /// Get pipeline layout
    pub fn layout(&self) -> &PipelineLayout {
        &self.entry.layout
    }
    
    /// Get bind group layout
    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.entry.bind_group_layout
    }
    
    /// Get creation time
    pub fn created_at(&self) -> std::time::Instant {
        self.entry.created_at
    }
}

/// Pipeline statistics
#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub active_pipelines: usize,
    pub cached_layouts: usize,
}

impl PipelineStats {
    /// Format statistics for display
    pub fn format(&self) -> String {
        format!("Active pipelines: {}, Cached layouts: {}", 
            self.active_pipelines, self.cached_layouts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pipeline_stats() {
        let stats = PipelineStats {
            active_pipelines: 5,
            cached_layouts: 3,
        };
        
        let formatted = stats.format();
        assert!(formatted.contains("Active pipelines: 5"));
        assert!(formatted.contains("Cached layouts: 3"));
    }
    
    #[test]
    fn test_pipeline_handle() {
        // This would need a mock device for testing
        // For now, just test the structure
        let pipeline_id = Uuid::new_v4();
        
        // PipelineHandle requires actual wgpu objects for real testing
        // This is just a placeholder test
        assert_ne!(pipeline_id, Uuid::new_v4());
    }
}
