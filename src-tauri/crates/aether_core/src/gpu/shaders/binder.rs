use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wgpu::{Device, BindGroup, BindGroupLayout, BindingResource, BindGroupEntry, BufferBinding};
use anyhow::{Result, anyhow};
use log::{debug, info, warn};

use super::PipelineHandle;


pub struct ParameterBinder {
    device: Arc<Device>,
    bind_groups: Arc<Mutex<HashMap<Uuid, BindGroupEntry>>>,
    binding_configs: Arc<Mutex<HashMap<String, BindingConfig>>>,
}

impl ParameterBinder {

    pub fn new(device: Arc<Device>) -> Result<Self> {
        info!("Creating parameter binder");

        Ok(Self {
            device,
            bind_groups: Arc::new(Mutex::new(HashMap::new())),
            binding_configs: Arc::new(Mutex::new(HashMap::new())),
        })
    }


    pub fn create_bind_group(
        &self,
        pipeline: &PipelineHandle,
        bindings: &HashMap<String, BindingResource>,
    ) -> Result<BindGroup> {
        debug!("Creating bind group for pipeline: {}", pipeline.name());


        self.validate_bindings(pipeline, bindings)?;


        let mut bind_group_entries = Vec::new();
        let binding_layout = pipeline.bind_group_layout();

        for (index, (name, resource)) in bindings.iter().enumerate() {
            debug!("Binding {}: {} -> {:?}", index, name, resource);

            let entry = BindGroupEntry {
                binding: index as u32,
                resource: resource.clone(),
            };
            bind_group_entries.push(entry);
        }


        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Bind Group: {}", pipeline.name())),
            layout: binding_layout,
            entries: &bind_group_entries,
        });


        let bind_group_id = Uuid::new_v4();
        let entry = BindGroupEntry {
            id: bind_group_id,
            pipeline_id: pipeline.id(),
            name: format!("{}-bind-group", pipeline.name()),
            bind_group: bind_group.clone(),
            created_at: std::time::Instant::now(),
        };

        let mut bind_groups = self.bind_groups.lock().map_err(|e| anyhow!("Bind group lock error: {}", e))?;
        bind_groups.insert(bind_group_id, entry);

        info!("Created bind group for pipeline: {}", pipeline.name());

        Ok(bind_group)
    }


    fn validate_bindings(
        &self,
        pipeline: &PipelineHandle,
        bindings: &HashMap<String, BindingResource>,
    ) -> Result<()> {
        debug!("Validating bindings for pipeline: {}", pipeline.name());


        if bindings.is_empty() {
            return Err(anyhow!("No bindings provided for pipeline: {}", pipeline.name()));
        }

        debug!("Validated {} bindings for pipeline: {}", bindings.len(), pipeline.name());

        Ok(())
    }


    pub fn create_binding_config(&self, name: &str, bindings: Vec<BindingDescriptor>) -> BindingConfig {
        debug!("Creating binding config: {}", name);

        let config = BindingConfig {
            name: name.to_string(),
            bindings,
            created_at: std::time::Instant::now(),
        };


        let mut configs = self.binding_configs.lock().map_err(|e| anyhow!("Config lock error: {}", e)).unwrap();
        configs.insert(name.to_string(), config.clone());

        info!("Created binding config: {}", name);

        config
    }


    pub fn get_binding_config(&self, name: &str) -> Result<BindingConfig> {
        let configs = self.binding_configs.lock().map_err(|e| anyhow!("Config lock error: {}", e))?;

        configs.get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Binding config not found: {}", name))
    }


    pub fn create_buffer_binding(
        &self,
        buffer: &wgpu::Buffer,
        offset: u64,
        size: Option<u64>,
    ) -> BindingResource {
        BindingResource::Buffer(BufferBinding {
            buffer,
            offset,
            size,
        })
    }


    pub fn create_texture_binding(&self, view: &wgpu::TextureView) -> BindingResource {
        BindingResource::TextureView(view.clone())
    }


    pub fn create_sampler_binding(&self, sampler: &wgpu::Sampler) -> BindingResource {
        BindingResource::Sampler(sampler.clone())
    }


    pub fn remove_bind_group(&self, bind_group_id: &Uuid) -> Result<()> {
        debug!("Removing bind group: {}", bind_group_id);

        let mut bind_groups = self.bind_groups.lock().map_err(|e| anyhow!("Bind group lock error: {}", e))?;
        if bind_groups.remove(bind_group_id).is_some() {
            info!("Removed bind group: {}", bind_group_id);
        }

        Ok(())
    }


    pub fn clear_bind_groups(&self) -> Result<()> {
        warn!("Clearing all bind groups");

        let mut bind_groups = self.bind_groups.lock().map_err(|e| anyhow!("Bind group lock error: {}", e))?;
        bind_groups.clear();

        info!("Cleared all bind groups");

        Ok(())
    }


    pub fn bind_group_count(&self) -> usize {
        self.bind_groups.lock().map(|bg| bg.len()).unwrap_or(0)
    }


    pub fn config_count(&self) -> usize {
        self.binding_configs.lock().map(|c| c.len()).unwrap_or(0)
    }


    pub fn get_stats(&self) -> BinderStats {
        BinderStats {
            active_bind_groups: self.bind_group_count(),
            cached_configs: self.config_count(),
        }
    }
}


#[derive(Debug, Clone)]
pub struct BindGroupEntry {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub name: String,
    pub bind_group: BindGroup,
    pub created_at: std::time::Instant,
}


#[derive(Debug, Clone)]
pub struct BindingConfig {
    pub name: String,
    pub bindings: Vec<BindingDescriptor>,
    pub created_at: std::time::Instant,
}


#[derive(Debug, Clone)]
pub struct BindingDescriptor {
    pub name: String,
    pub binding_type: wgpu::BindingType,
    pub min_size: Option<wgpu::BufferSize>,
}

impl BindingDescriptor {

    pub fn storage_buffer(name: &str, min_size: Option<u64>) -> Self {
        Self {
            name: name.to_string(),
            binding_type: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: min_size.map(|size| size.into()),
            },
            min_size: min_size.map(|size| size.into()),
        }
    }


    pub fn uniform_buffer(name: &str, min_size: Option<u64>) -> Self {
        Self {
            name: name.to_string(),
            binding_type: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: min_size.map(|size| size.into()),
            },
            min_size: min_size.map(|size| size.into()),
        }
    }


    pub fn texture(name: &str) -> Self {
        Self {
            name: name.to_string(),
            binding_type: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            min_size: None,
        }
    }


    pub fn sampler(name: &str) -> Self {
        Self {
            name: name.to_string(),
            binding_type: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            min_size: None,
        }
    }
}


#[derive(Debug, Clone)]
pub struct BinderStats {
    pub active_bind_groups: usize,
    pub cached_configs: usize,
}

impl BinderStats {

    pub fn format(&self) -> String {
        format!(
            "Active bind groups: {}, Cached configs: {}",
            self.active_bind_groups, self.cached_configs
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binding_descriptor() {
        let storage_desc = BindingDescriptor::storage_buffer("storage", Some(1024));
        assert_eq!(storage_desc.name, "storage");
        assert!(storage_desc.min_size.is_some());

        let uniform_desc = BindingDescriptor::uniform_buffer("uniform", Some(256));
        assert_eq!(uniform_desc.name, "uniform");

        let texture_desc = BindingDescriptor::texture("texture");
        assert_eq!(texture_desc.name, "texture");

        let sampler_desc = BindingDescriptor::sampler("sampler");
        assert_eq!(sampler_desc.name, "sampler");
    }

    #[test]
    fn test_binding_config() {
        let bindings = vec![
            BindingDescriptor::storage_buffer("input", Some(1024)),
            BindingDescriptor::storage_buffer("output", Some(1024)),
        ];

        let config = BindingConfig {
            name: "test_config".to_string(),
            bindings,
            created_at: std::time::Instant::now(),
        };

        assert_eq!(config.name, "test_config");
        assert_eq!(config.bindings.len(), 2);
    }

    #[test]
    fn test_binder_stats() {
        let stats = BinderStats {
            active_bind_groups: 5,
            cached_configs: 3,
        };

        let formatted = stats.format();
        assert!(formatted.contains("Active bind groups: 5"));
        assert!(formatted.contains("Cached configs: 3"));
    }
}
