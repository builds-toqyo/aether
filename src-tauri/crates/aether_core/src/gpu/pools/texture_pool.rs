use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wgpu::{Device, Texture, TextureView, TextureDescriptor, TextureViewDescriptor, Extent3d, TextureDimension, TextureFormat, TextureUsages};
use anyhow::{Result, anyhow};
use log::{debug, info};


pub struct TexturePool {
    device: Arc<Device>,
    textures: HashMap<Uuid, TextureEntry>,
    views: HashMap<Uuid, Arc<TextureView>>,
    available_textures: Vec<AvailableTexture>,
}

impl TexturePool {

    pub fn new(device: Arc<Device>) -> Self {
        info!("Creating texture pool");

        Self {
            device,
            textures: HashMap::new(),
            views: HashMap::new(),
            available_textures: Vec::new(),
        }
    }


    pub fn allocate(&mut self, width: u32, height: u32, format: TextureFormat) -> Result<Uuid> {
        debug!("Allocating texture: {}x{} format={:?}", width, height, format);


        if let Some(available) = self.find_available_texture(width, height, format) {
            let texture_id = available.id;
            debug!("Reusing available texture: {:?}", texture_id);
            return Ok(texture_id);
        }


        let texture_id = Uuid::new_v4();
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some(&format!("Texture {:?}", texture_id)),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let texture_entry = TextureEntry {
            texture: Arc::new(texture),
            width,
            height,
            format,
            ref_count: 1,
            last_used: std::time::Instant::now(),
        };

        self.textures.insert(texture_id, texture_entry);

        debug!("Created new texture: {:?}", texture_id);

        Ok(texture_id)
    }


    pub fn get_texture(&self, texture_id: &Uuid) -> Result<Arc<Texture>> {
        self.textures.get(texture_id)
            .map(|entry| entry.texture.clone())
            .ok_or_else(|| anyhow!("Texture not found: {:?}", texture_id))
    }


    pub fn get_view(&mut self, texture_id: &Uuid) -> Result<Arc<TextureView>> {
        if let Some(view) = self.views.get(texture_id) {
            return Ok(view.clone());
        }

        let texture = self.get_texture(texture_id)?;
        let view = texture.create_view(&TextureViewDescriptor::default());
        let view = Arc::new(view);

        self.views.insert(*texture_id, view.clone());

        Ok(view)
    }


    pub fn release_texture(&mut self, texture_id: &Uuid) -> Result<()> {
        debug!("Releasing texture: {:?}", texture_id);

        if let Some(entry) = self.textures.get_mut(texture_id) {
            entry.ref_count -= 1;
            entry.last_used = std::time::Instant::now();

            if entry.ref_count == 0 {

                let available = AvailableTexture {
                    id: *texture_id,
                    width: entry.width,
                    height: entry.height,
                    format: entry.format,
                };
                self.available_textures.push(available);
                debug!("Texture added to available pool: {:?}", texture_id);
            }

            Ok(())
        } else {
            Err(anyhow!("Texture not found: {:?}", texture_id))
        }
    }


    pub fn cleanup_unused(&mut self) -> Result<usize> {
        debug!("Cleaning up unused textures");

        let mut cleaned_count = 0;
        let now = std::time::Instant::now();
        const CLEANUP_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(30);


        let to_cleanup: Vec<Uuid> = self.textures.iter()
            .filter(|(_, entry)| entry.ref_count == 0 && now.duration_since(entry.last_used) > CLEANUP_THRESHOLD)
            .map(|(id, _)| *id)
            .collect();

        for texture_id in to_cleanup {
            self.textures.remove(&texture_id);
            self.views.remove(&texture_id);
            cleaned_count += 1;
        }

        info!("Cleaned up {} unused textures", cleaned_count);

        Ok(cleaned_count)
    }


    pub fn force_cleanup(&mut self) -> Result<()> {
        warn!("Force cleaning up all textures");

        self.textures.clear();
        self.views.clear();
        self.available_textures.clear();

        Ok(())
    }


    pub fn get_stats(&self) -> TexturePoolStats {
        let active_count = self.textures.len();
        let available_count = self.available_textures.len();
        let total_memory = self.textures.values()
            .map(|entry| entry.width as u64 * entry.height as u64 * self.bytes_per_pixel(entry.format) as u64)
            .sum();

        TexturePoolStats {
            active_textures: active_count,
            available_textures: available_count,
            total_memory_bytes: total_memory,
        }
    }


    fn find_available_texture(&mut self, width: u32, height: u32, format: TextureFormat) -> Option<AvailableTexture> {
        let index = self.available_textures.iter().position(|t| {
            t.width == width && t.height == height && t.format == format
        });

        if let Some(index) = index {
            let available = self.available_textures.swap_remove(index);


            if let Some(entry) = self.textures.get_mut(&available.id) {
                entry.ref_count += 1;
                entry.last_used = std::time::Instant::now();
            }

            Some(available)
        } else {
            None
        }
    }


    fn bytes_per_pixel(&self, format: TextureFormat) -> u32 {
        match format {
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::R32Float => 4,
            TextureFormat::Rg32Float => 8,
            TextureFormat::Rgba32Float => 16,
            _ => 4,
        }
    }
}


#[derive(Debug)]
pub struct TextureEntry {
    pub texture: Arc<Texture>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub ref_count: u32,
    pub last_used: std::time::Instant,
}


#[derive(Debug, Clone)]
pub struct AvailableTexture {
    pub id: Uuid,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}


#[derive(Debug, Clone)]
pub struct TexturePoolStats {
    pub active_textures: usize,
    pub available_textures: usize,
    pub total_memory_bytes: u64,
}

impl TexturePoolStats {

    pub fn format_memory(&self) -> String {
        let mb = self.total_memory_bytes as f64 / (1024.0 * 1024.0);
        format!("{:.1}MB ({} active, {} available)", mb, self.active_textures, self.available_textures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::{DeviceDescriptor, RequestAdapterOptions, Instance};

    #[test]
    fn test_texture_pool_stats() {
        let stats = TexturePoolStats {
            active_textures: 5,
            available_textures: 3,
            total_memory_bytes: 1024 * 1024,
        };

        let formatted = stats.format_memory();
        assert!(formatted.contains("1.0MB"));
        assert!(formatted.contains("5 active"));
        assert!(formatted.contains("3 available"));
    }

    #[test]
    fn test_bytes_per_pixel() {
        let pool = TexturePool::new(Arc::new(create_mock_device()));

        assert_eq!(pool.bytes_per_pixel(TextureFormat::R8Unorm), 1);
        assert_eq!(pool.bytes_per_pixel(TextureFormat::Rgba8UnormSrgb), 4);
        assert_eq!(pool.bytes_per_pixel(TextureFormat::R32Float), 4);
        assert_eq!(pool.bytes_per_pixel(TextureFormat::Rgba32Float), 16);
    }


    fn create_mock_device() -> Device {


        panic!("Mock device implementation needed for tests")
    }
}
