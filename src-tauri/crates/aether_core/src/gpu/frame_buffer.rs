use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wgpu::{Device, Queue, Texture, TextureView, Buffer, CommandEncoder, TextureFormat};
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};

use crate::gpu::pools::{TexturePool, TextureHandle};

/// Frame buffer management system for efficient GPU rendering
pub struct FrameBufferManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    texture_pool: Arc<Mutex<TexturePool>>,
    frame_buffers: Arc<Mutex<HashMap<Uuid, FrameBuffer>>>,
    texture_cache: Arc<Mutex<TextureCache>>,
    config: FrameBufferConfig,
    stats: Arc<Mutex<FrameBufferStats>>,
}

impl FrameBufferManager {
    /// Create a new frame buffer manager
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        texture_pool: Arc<Mutex<TexturePool>>,
        config: FrameBufferConfig,
    ) -> Result<Self> {
        info!("Creating frame buffer manager with config: {:?}", config);
        
        Ok(Self {
            device,
            queue,
            texture_pool,
            frame_buffers: Arc::new(Mutex::new(HashMap::new())),
            texture_cache: Arc::new(Mutex::new(TextureCache::new(config.max_cache_size))),
            config,
            stats: Arc::new(Mutex::new(FrameBufferStats::new())),
        ))
    }
    
    /// Create a new frame buffer
    pub fn create_frame_buffer(
        &self,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Result<FrameBufferHandle> {
        debug!("Creating frame buffer: {}x{} {:?}", width, height, format);
        
        let frame_buffer_id = Uuid::new_v4();
        
        // Create texture from pool
        let texture = self.create_texture(width, height, format, usage)?;
        
        // Create texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("Frame Buffer View: {}", frame_buffer_id)),
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        
        // Create frame buffer
        let frame_buffer = FrameBuffer {
            id: frame_buffer_id,
            width,
            height,
            format,
            texture: Arc::new(texture),
            view: Arc::new(view),
            created_at: std::time::Instant::now(),
            last_used: std::sync::Mutex::new(std::time::Instant::now()),
            access_count: std::sync::atomic::AtomicU64::new(0),
        };
        
        // Store frame buffer
        let mut frame_buffers = self.frame_buffers.lock().map_err(|e| anyhow!("Frame buffer lock error: {}", e))?;
        frame_buffers.insert(frame_buffer_id, frame_buffer.clone());
        
        // Update stats
        {
            let mut stats = self.stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.active_frame_buffers += 1;
            stats.total_created += 1;
        }
        
        info!("Created frame buffer: {} ({}x{})", frame_buffer_id, width, height);
        
        Ok(FrameBufferHandle {
            id: frame_buffer_id,
            frame_buffer: Arc::new(frame_buffer),
        })
    }
    
    /// Get a frame buffer by ID
    pub fn get_frame_buffer(&self, frame_buffer_id: &Uuid) -> Result<FrameBufferHandle> {
        let frame_buffers = self.frame_buffers.lock().map_err(|e| anyhow!("Frame buffer lock error: {}", e))?;
        
        frame_buffers.get(frame_buffer_id)
            .map(|fb| FrameBufferHandle {
                id: *frame_buffer_id,
                frame_buffer: Arc::new(fb.clone()),
            })
            .ok_or_else(|| anyhow!("Frame buffer not found: {}", frame_buffer_id))
    }
    
    /// Create or get a cached frame buffer
    pub fn get_or_create_frame_buffer(
        &self,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Result<FrameBufferHandle> {
        let cache_key = FrameBufferKey {
            width,
            height,
            format,
            usage,
        };
        
        // Try to get from cache first
        if let Some(cached_id) = self.texture_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?.get(&cache_key) {
            if let Ok(frame_buffer) = self.get_frame_buffer(&cached_id) {
                debug!("Using cached frame buffer: {}x{} {:?}", width, height, format);
                return Ok(frame_buffer);
            }
        }
        
        // Create new frame buffer if not cached
        let frame_buffer = self.create_frame_buffer(width, height, format, usage)?;
        
        // Add to cache
        {
            let mut cache = self.texture_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            cache.insert(cache_key, frame_buffer.id);
        }
        
        Ok(frame_buffer)
    }
    
    /// Create a multi-buffer rendering pipeline
    pub fn create_multi_buffer_pipeline(
        &self,
        width: u32,
        height: u32,
        format: TextureFormat,
        buffer_count: usize,
    ) -> Result<MultiBufferPipeline> {
        debug!("Creating multi-buffer pipeline: {}x{} with {} buffers", width, height, buffer_count);
        
        if buffer_count < 2 {
            return Err(anyhow!("Multi-buffer pipeline requires at least 2 buffers"));
        }
        
        let mut buffers = Vec::new();
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        
        for i in 0..buffer_count {
            let frame_buffer = self.create_frame_buffer(width, height, format, usage)?;
            buffers.push(frame_buffer);
        }
        
        let pipeline = MultiBufferPipeline {
            buffers,
            current_buffer: 0,
            width,
            height,
            format,
            created_at: std::time::Instant::now(),
        };
        
        info!("Created multi-buffer pipeline with {} buffers", buffer_count);
        
        Ok(pipeline)
    }
    
    /// Recycle a frame buffer for reuse
    pub fn recycle_frame_buffer(&self, frame_buffer_id: &Uuid) -> Result<()> {
        debug!("Recycling frame buffer: {}", frame_buffer_id);
        
        let frame_buffers = self.frame_buffers.lock().map_err(|e| anyhow!("Frame buffer lock error: {}", e))?;
        
        if let Some(frame_buffer) = frame_buffers.get(frame_buffer_id) {
            // Update last used time
            if let Ok(mut last_used) = frame_buffer.last_used.lock() {
                *last_used = std::time::Instant::now();
            }
            
            // Increment access count
            let access_count = frame_buffer.access_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            debug!("Frame buffer recycled: {} (access count: {})", frame_buffer_id, access_count);
        }
        
        Ok(())
    }
    
    /// Remove a frame buffer
    pub fn remove_frame_buffer(&self, frame_buffer_id: &Uuid) -> Result<()> {
        debug!("Removing frame buffer: {}", frame_buffer_id);
        
        let mut frame_buffers = self.frame_buffers.lock().map_err(|e| anyhow!("Frame buffer lock error: {}", e))?;
        
        if frame_buffers.remove(frame_buffer_id).is_some() {
            // Remove from cache
            let mut cache = self.texture_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            cache.remove_by_frame_buffer_id(frame_buffer_id);
            
            // Update stats
            let mut stats = self.stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.active_frame_buffers = frame_buffers.len();
            stats.total_recycled += 1;
            
            info!("Removed frame buffer: {}", frame_buffer_id);
        }
        
        Ok(())
    }
    
    /// Clean up old frame buffers
    pub fn cleanup_old_buffers(&self, max_age: std::time::Duration) -> Result<usize> {
        debug!("Cleaning up old frame buffers (max age: {:?})", max_age);
        
        let mut frame_buffers = self.frame_buffers.lock().map_err(|e| anyhow!("Frame buffer lock error: {}", e))?;
        let mut to_remove = Vec::new();
        
        for (id, frame_buffer) in &frame_buffers {
            if let Ok(last_used) = frame_buffer.last_used.lock() {
                if last_used.elapsed() > max_age {
                    to_remove.push(*id);
                }
            }
        }
        
        let removed_count = to_remove.len();
        
        for id in to_remove {
            frame_buffers.remove(&id);
            
            // Remove from cache
            let mut cache = self.texture_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            cache.remove_by_frame_buffer_id(&id);
        }
        
        // Update stats
        {
            let mut stats = self.stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.active_frame_buffers = frame_buffers.len();
            stats.total_cleaned += removed_count;
        }
        
        info!("Cleaned up {} old frame buffers", removed_count);
        
        Ok(removed_count)
    }
    
    /// Get frame buffer statistics
    pub fn get_stats(&self) -> Result<FrameBufferStats> {
        let stats = self.stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
        let frame_buffers = self.frame_buffers.lock().map_err(|e| anyhow!("Frame buffer lock error: {}", e))?;
        let cache = self.texture_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?;
        
        let mut current_stats = stats.clone();
        current_stats.active_frame_buffers = frame_buffers.len();
        current_stats.cached_entries = cache.len();
        current_stats.cache_hit_rate = cache.hit_rate();
        
        Ok(current_stats)
    }
    
    /// Clear all frame buffers
    pub fn clear_all(&self) -> Result<()> {
        warn!("Clearing all frame buffers");
        
        let mut frame_buffers = self.frame_buffers.lock().map_err(|e| anyhow!("Frame buffer lock error: {}", e))?;
        let count = frame_buffers.len();
        frame_buffers.clear();
        
        let mut cache = self.texture_cache.lock().map_err(|e| anyhow!("Cache lock error: {}", e))?;
        cache.clear();
        
        let mut stats = self.stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
        stats.active_frame_buffers = 0;
        stats.cached_entries = 0;
        
        info!("Cleared {} frame buffers", count);
        
        Ok(())
    }
    
    /// Create texture from pool or new
    fn create_texture(
        &self,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Result<Texture> {
        let descriptor = wgpu::TextureDescriptor {
            label: Some(&format!("Frame Buffer Texture: {}x{}", width, height)),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[format],
        };
        
        Ok(self.device.create_texture(&descriptor))
    }
}

/// Frame buffer configuration
#[derive(Debug, Clone)]
pub struct FrameBufferConfig {
    pub max_cache_size: usize,
    pub cleanup_interval: std::time::Duration,
    pub max_buffer_age: std::time::Duration,
}

impl Default for FrameBufferConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 100,
            cleanup_interval: std::time::Duration::from_secs(30),
            max_buffer_age: std::time::Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Frame buffer entry
#[derive(Debug)]
pub struct FrameBuffer {
    pub id: Uuid,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub texture: Arc<Texture>,
    pub view: Arc<TextureView>,
    pub created_at: std::time::Instant,
    pub last_used: std::sync::Mutex<std::time::Instant>,
    pub access_count: std::sync::atomic::AtomicU64,
}

/// Handle for a frame buffer
#[derive(Debug, Clone)]
pub struct FrameBufferHandle {
    pub id: Uuid,
    frame_buffer: Arc<FrameBuffer>,
}

impl FrameBufferHandle {
    /// Get frame buffer ID
    pub fn id(&self) -> Uuid {
        self.id
    }
    
    /// Get texture
    pub fn texture(&self) -> &Texture {
        &self.frame_buffer.texture
    }
    
    /// Get texture view
    pub fn view(&self) -> &TextureView {
        &self.frame_buffer.view
    }
    
    /// Get dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.frame_buffer.width, self.frame_buffer.height)
    }
    
    /// Get format
    pub fn format(&self) -> TextureFormat {
        self.frame_buffer.format
    }
    
    /// Mark as used
    pub fn mark_used(&self) {
        // Update last used time
        if let Ok(mut last_used) = self.frame_buffer.last_used.lock() {
            *last_used = std::time::Instant::now();
        }
        
        // Increment access count
        self.frame_buffer.access_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Multi-buffer rendering pipeline
#[derive(Debug)]
pub struct MultiBufferPipeline {
    pub buffers: Vec<FrameBufferHandle>,
    pub current_buffer: usize,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub created_at: std::time::Instant,
}

impl MultiBufferPipeline {
    /// Get current buffer
    pub fn current_buffer(&self) -> &FrameBufferHandle {
        &self.buffers[self.current_buffer]
    }
    
    /// Get next buffer (for double buffering)
    pub fn next_buffer(&mut self) -> &FrameBufferHandle {
        self.current_buffer = (self.current_buffer + 1) % self.buffers.len();
        &self.buffers[self.current_buffer]
    }
    
    /// Get buffer by index
    pub fn get_buffer(&self, index: usize) -> Option<&FrameBufferHandle> {
        self.buffers.get(index)
    }
    
    /// Get buffer count
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
}

/// Texture cache for frame buffers
#[derive(Debug)]
struct TextureCache {
    cache: HashMap<FrameBufferKey, Uuid>,
    access_times: HashMap<Uuid, std::time::Instant>,
    hits: u64,
    misses: u64,
    max_size: usize,
}

impl TextureCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            access_times: HashMap::new(),
            hits: 0,
            misses: 0,
            max_size,
        }
    }
    
    fn get(&mut self, key: &FrameBufferKey) -> Option<Uuid> {
        if let Some(frame_buffer_id) = self.cache.get(key) {
            self.hits += 1;
            self.access_times.insert(*frame_buffer_id, std::time::Instant::now());
            Some(*frame_buffer_id)
        } else {
            self.misses += 1;
            None
        }
    }
    
    fn insert(&mut self, key: FrameBufferKey, frame_buffer_id: Uuid) {
        // Remove oldest if at capacity
        if self.cache.len() >= self.max_size {
            self.remove_oldest();
        }
        
        self.cache.insert(key, frame_buffer_id);
        self.access_times.insert(frame_buffer_id, std::time::Instant::now());
    }
    
    fn remove_by_frame_buffer_id(&mut self, frame_buffer_id: &Uuid) {
        self.cache.retain(|_, id| id != frame_buffer_id);
        self.access_times.remove(frame_buffer_id);
    }
    
    fn remove_oldest(&mut self) {
        if let Some((oldest_id, _)) = self.access_times.iter().min_by_key(|(_, time)| *time) {
            let oldest_id = *oldest_id;
            self.cache.retain(|_, id| id != &oldest_id);
            self.access_times.remove(&oldest_id);
        }
    }
    
    fn clear(&mut self) {
        self.cache.clear();
        self.access_times.clear();
    }
    
    fn len(&self) -> usize {
        self.cache.len()
    }
    
    fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Frame buffer cache key
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FrameBufferKey {
    width: u32,
    height: u32,
    format: TextureFormat,
    usage: wgpu::TextureUsages,
}

/// Frame buffer statistics
#[derive(Debug, Clone)]
pub struct FrameBufferStats {
    pub active_frame_buffers: usize,
    pub total_created: u64,
    pub total_recycled: u64,
    pub total_cleaned: u64,
    pub cached_entries: usize,
    pub cache_hit_rate: f64,
}

impl FrameBufferStats {
    fn new() -> Self {
        Self {
            active_frame_buffers: 0,
            total_created: 0,
            total_recycled: 0,
            total_cleaned: 0,
            cached_entries: 0,
            cache_hit_rate: 0.0,
        }
    }
    
    /// Format statistics for display
    pub fn format(&self) -> String {
        format!(
            "Active: {}, Created: {}, Recycled: {}, Cleaned: {}, Cached: {} (hit rate: {:.1}%)",
            self.active_frame_buffers,
            self.total_created,
            self.total_recycled,
            self.total_cleaned,
            self.cached_entries,
            self.cache_hit_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_buffer_config() {
        let config = FrameBufferConfig::default();
        assert_eq!(config.max_cache_size, 100);
        assert_eq!(config.cleanup_interval, std::time::Duration::from_secs(30));
    }
    
    #[test]
    fn test_frame_buffer_stats() {
        let stats = FrameBufferStats::new();
        assert_eq!(stats.active_frame_buffers, 0);
        assert_eq!(stats.total_created, 0);
        
        let formatted = stats.format();
        assert!(formatted.contains("Active: 0"));
        assert!(formatted.contains("Created: 0"));
    }
    
    #[test]
    fn test_multi_buffer_pipeline() {
        // This would need actual frame buffers for testing
        // For now, just test the structure
        let pipeline = MultiBufferPipeline {
            buffers: Vec::new(),
            current_buffer: 0,
            width: 1920,
            height: 1080,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            created_at: std::time::Instant::now(),
        };
        
        assert_eq!(pipeline.width, 1920);
        assert_eq!(pipeline.height, 1080);
        assert_eq!(pipeline.current_buffer, 0);
        assert_eq!(pipeline.buffer_count(), 0);
    }
}
