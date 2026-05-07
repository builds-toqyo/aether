use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wgpu::{Device, Texture, TextureView, TextureDescriptor, TextureViewDescriptor, Extent3d, TextureDimension, TextureFormat, TextureUsages, Buffer};
use anyhow::{Result, anyhow};
use log::{debug, info};

pub mod texture_pool;
pub mod buffer_pool;

pub use texture_pool::{TexturePool, TextureEntry, AvailableTexture};
pub use buffer_pool::{BufferPool, BufferEntry, AvailableBuffer};


#[derive(Debug, Clone)]
pub struct TextureHandle {
    pub id: Uuid,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    manager: Arc<Mutex<TexturePool>>,
}

impl TextureHandle {

    pub fn id(&self) -> Uuid {
        self.id
    }


    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }


    pub fn format(&self) -> TextureFormat {
        self.format
    }


    pub fn get_texture(&self) -> Result<Arc<Texture>> {
        let pool = self.manager.lock().map_err(|e| anyhow!("Texture pool lock error: {}", e))?;
        pool.get_texture(&self.id)
    }


    pub fn get_view(&mut self) -> Result<Arc<TextureView>> {
        let mut pool = self.manager.lock().map_err(|e| anyhow!("Texture pool lock error: {}", e))?;
        pool.get_view(&self.id)
    }
}

impl Drop for TextureHandle {
    fn drop(&mut self) {
        debug!("Dropping texture handle: {:?}", self.id);
    }
}


#[derive(Debug, Clone)]
pub struct BufferHandle {
    pub id: Uuid,
    pub size: u64,
    pub usage: wgpu::BufferUsages,
    manager: Arc<Mutex<BufferPool>>,
}

impl BufferHandle {

    pub fn id(&self) -> Uuid {
        self.id
    }


    pub fn size(&self) -> u64 {
        self.size
    }


    pub fn usage(&self) -> wgpu::BufferUsages {
        self.usage
    }


    pub fn get_buffer(&self) -> Result<Arc<Buffer>> {
        let pool = self.manager.lock().map_err(|e| anyhow!("Buffer pool lock error: {}", e))?;
        pool.get_buffer(&self.id)
    }
}

impl Drop for BufferHandle {
    fn drop(&mut self) {
        debug!("Dropping buffer handle: {:?}", self.id);
    }
}
