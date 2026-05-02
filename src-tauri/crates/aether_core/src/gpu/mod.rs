//! GPU memory management and processing modules for the Aether node system
//! 
//! This module provides comprehensive GPU memory management, texture handling,
//! buffer management, and GPU-CPU synchronization for efficient node processing.

pub mod pools;
pub mod sync;

// Re-export main GPU memory management types
pub use pools::{TexturePool, BufferPool, TextureHandle, BufferHandle};
pub use sync::{GpuCpuSynchronization, MemoryTracker, MemoryStats, SyncStatus, SamplerHandle};

/// GPU memory manager for handling textures, buffers, and synchronization
pub struct GpuMemoryManager {
    device: std::sync::Arc<wgpu::Device>,
    queue: std::sync::Arc<wgpu::Queue>,
    texture_pool: std::sync::Arc<std::sync::Mutex<TexturePool>>,
    buffer_pool: std::sync::Arc<std::sync::Mutex<BufferPool>>,
    memory_tracker: std::sync::Arc<std::sync::Mutex<MemoryTracker>>,
    synchronization: std::sync::Arc<std::sync::Mutex<GpuCpuSynchronization>>,
}

impl GpuMemoryManager {
    /// Create a new GPU memory manager
    pub fn new(device: std::sync::Arc<wgpu::Device>, queue: std::sync::Arc<wgpu::Queue>) -> Self {
        log::info!("Initializing GPU memory manager");
        
        Self {
            device: device.clone(),
            queue: queue.clone(),
            texture_pool: std::sync::Arc::new(std::sync::Mutex::new(TexturePool::new(device.clone()))),
            buffer_pool: std::sync::Arc::new(std::sync::Mutex::new(BufferPool::new(device.clone()))),
            memory_tracker: std::sync::Arc::new(std::sync::Mutex::new(MemoryTracker::new())),
            synchronization: std::sync::Arc::new(std::sync::Mutex::new(GpuCpuSynchronization::new())),
        }
    }
    
    /// Allocate a texture for frame buffer
    pub fn allocate_texture(&self, width: u32, height: u32, format: wgpu::TextureFormat) -> anyhow::Result<TextureHandle> {
        log::debug!("Allocating texture: {}x{} format={:?}", width, height, format);
        
        let mut pool = self.texture_pool.lock().map_err(|e| anyhow::anyhow!("Texture pool lock error: {}", e))?;
        let texture_id = pool.allocate(width, height, format)?;
        
        // Track memory usage
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow::anyhow!("Memory tracker lock error: {}", e))?;
        tracker.track_texture_allocation(&texture_id, width, height, format);
        
        Ok(TextureHandle {
            id: texture_id,
            width,
            height,
            format,
            manager: self.texture_pool.clone(),
        })
    }
    
    /// Allocate a buffer for data transfer
    pub fn allocate_buffer(&self, size: u64, usage: wgpu::BufferUsages) -> anyhow::Result<BufferHandle> {
        log::debug!("Allocating buffer: {} bytes usage={:?}", size, usage);
        
        let mut pool = self.buffer_pool.lock().map_err(|e| anyhow::anyhow!("Buffer pool lock error: {}", e))?;
        let buffer_id = pool.allocate(size, usage)?;
        
        // Track memory usage
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow::anyhow!("Memory tracker lock error: {}", e))?;
        tracker.track_buffer_allocation(&buffer_id, size, usage);
        
        Ok(BufferHandle {
            id: buffer_id,
            size,
            usage,
            manager: self.buffer_pool.clone(),
        })
    }
    
    /// Create a sampler for texture sampling
    pub fn create_sampler(&self, descriptor: &wgpu::SamplerDescriptor<'_>) -> anyhow::Result<SamplerHandle> {
        log::debug!("Creating sampler");
        
        let sampler = self.device.create_sampler(descriptor);
        let sampler_id = uuid::Uuid::new_v4();
        
        // Track sampler allocation
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow::anyhow!("Memory tracker lock error: {}", e))?;
        tracker.track_sampler_allocation(&sampler_id);
        
        Ok(SamplerHandle {
            id: sampler_id,
            sampler: std::sync::Arc::new(sampler),
            manager: self.memory_tracker.clone(),
        })
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> anyhow::Result<MemoryStats> {
        let tracker = self.memory_tracker.lock().map_err(|e| anyhow::anyhow!("Memory tracker lock error: {}", e))?;
        Ok(tracker.get_stats())
    }
    
    /// Cleanup unused resources
    pub fn cleanup_unused_resources(&self) -> anyhow::Result<usize> {
        log::debug!("Cleaning up unused GPU resources");
        
        let mut pool = self.texture_pool.lock().map_err(|e| anyhow::anyhow!("Texture pool lock error: {}", e))?;
        let texture_cleanup_count = pool.cleanup_unused()?;
        
        let mut pool = self.buffer_pool.lock().map_err(|e| anyhow::anyhow!("Buffer pool lock error: {}", e))?;
        let buffer_cleanup_count = pool.cleanup_unused()?;
        
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow::anyhow!("Memory tracker lock error: {}", e))?;
        tracker.cleanup_unused_resources();
        
        let total_cleaned = texture_cleanup_count + buffer_cleanup_count;
        log::info!("Cleaned up {} unused GPU resources", total_cleaned);
        
        Ok(total_cleaned)
    }
    
    /// Force cleanup all resources
    pub fn force_cleanup(&self) -> anyhow::Result<()> {
        log::warn!("Force cleaning up all GPU resources");
        
        let mut pool = self.texture_pool.lock().map_err(|e| anyhow::anyhow!("Texture pool lock error: {}", e))?;
        pool.force_cleanup()?;
        
        let mut pool = self.buffer_pool.lock().map_err(|e| anyhow::anyhow!("Buffer pool lock error: {}", e))?;
        pool.force_cleanup()?;
        
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow::anyhow!("Memory tracker lock error: {}", e))?;
        tracker.force_cleanup();
        
        log::info!("Force cleanup completed");
        
        Ok(())
    }
    
    /// Get GPU-CPU synchronization status
    pub fn get_sync_status(&self) -> anyhow::Result<SyncStatus> {
        let sync = self.synchronization.lock().map_err(|e| anyhow::anyhow!("Synchronization lock error: {}", e))?;
        Ok(sync.get_status())
    }
    
    /// Wait for GPU operations to complete
    pub fn wait_for_gpu(&self) -> anyhow::Result<()> {
        log::debug!("Waiting for GPU operations to complete");
        
        let mut sync = self.synchronization.lock().map_err(|e| anyhow::anyhow!("Synchronization lock error: {}", e))?;
        sync.wait_for_gpu(&self.device)?;
        
        Ok(())
    }
    
    /// Wait for GPU operations with timeout
    pub fn wait_for_gpu_with_timeout(&self, timeout: std::time::Duration) -> anyhow::Result<bool> {
        log::debug!("Waiting for GPU operations with timeout: {:?}", timeout);
        
        let mut sync = self.synchronization.lock().map_err(|e| anyhow::anyhow!("Synchronization lock error: {}", e))?;
        sync.wait_for_gpu_with_timeout(&self.device, timeout)
    }
    
    /// Track a GPU operation for synchronization
    pub fn track_gpu_operation(&self, operation_type: String) -> anyhow::Result<u64> {
        let mut sync = self.synchronization.lock().map_err(|e| anyhow::anyhow!("Synchronization lock error: {}", e))?;
        Ok(sync.track_operation(operation_type))
    }
    
    /// Mark a specific GPU operation as completed
    pub fn complete_gpu_operation(&self, operation_id: u64) -> anyhow::Result<()> {
        let mut sync = self.synchronization.lock().map_err(|e| anyhow::anyhow!("Synchronization lock error: {}", e))?;
        sync.complete_operation(operation_id)
    }
    
    /// Get pending GPU operations
    pub fn get_pending_operations(&self) -> anyhow::Result<Vec<String>> {
        let sync = self.synchronization.lock().map_err(|e| anyhow::anyhow!("Synchronization lock error: {}", e))?;
        let operations = sync.get_pending_operations()
            .into_iter()
            .map(|op| format!("{}: {} (submitted {:?})", op.operation_id, op.operation_type, op.submitted_at))
            .collect();
        Ok(operations)
    }
    
    /// Force clear all pending operations (emergency cleanup)
    pub fn force_clear_gpu_operations(&self) -> anyhow::Result<()> {
        log::warn!("Force clearing all pending GPU operations");
        
        let mut sync = self.synchronization.lock().map_err(|e| anyhow::anyhow!("Synchronization lock error: {}", e))?;
        sync.force_clear();
        
        Ok(())
    }
}

/// GPU-related error types
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("Memory allocation failed: {0}")]
    AllocationFailed(String),
    
    #[error("Texture creation failed: {0}")]
    TextureCreationFailed(String),
    
    #[error("Buffer creation failed: {0}")]
    BufferCreationFailed(String),
    
    #[error("GPU synchronization failed: {0}")]
    SynchronizationFailed(String),
    
    #[error("Memory leak detected: {0}")]
    MemoryLeak(String),
    
    #[error("Invalid texture dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    
    #[error("Unsupported texture format: {0:?}")]
    UnsupportedFormat(wgpu::TextureFormat),
    
    #[error("GPU device lost")]
    DeviceLost,
    
    #[error("GPU out of memory")]
    OutOfMemory,
}

/// GPU configuration options
#[derive(Debug, Clone)]
pub struct GpuConfig {
    /// Maximum texture size in pixels
    pub max_texture_size: u32,
    
    /// Maximum buffer size in bytes
    pub max_buffer_size: u64,
    
    /// Memory cleanup threshold in seconds
    pub cleanup_threshold_seconds: u64,
    
    /// Enable memory pooling
    pub enable_pooling: bool,
    
    /// Maximum number of pooled textures
    pub max_pooled_textures: usize,
    
    /// Maximum number of pooled buffers
    pub max_pooled_buffers: usize,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            max_texture_size: 8192,
            max_buffer_size: 256 * 1024 * 1024, // 256MB
            cleanup_threshold_seconds: 30,
            enable_pooling: true,
            max_pooled_textures: 100,
            max_pooled_buffers: 50,
        }
    }
}

/// GPU performance metrics
#[derive(Debug, Clone)]
pub struct GpuMetrics {
    pub memory_used: u64,
    pub memory_available: u64,
    pub texture_allocations: u64,
    pub buffer_allocations: u64,
    pub gpu_utilization: f32,
    pub average_frame_time: f32,
    pub frames_processed: u64,
}

impl Default for GpuMetrics {
    fn default() -> Self {
        Self {
            memory_used: 0,
            memory_available: 0,
            texture_allocations: 0,
            buffer_allocations: 0,
            gpu_utilization: 0.0,
            average_frame_time: 0.0,
            frames_processed: 0,
        }
    }
}

/// GPU initialization result
#[derive(Debug)]
pub struct GpuInitResult {
    pub memory_manager: GpuMemoryManager,
    pub config: GpuConfig,
    pub initial_metrics: GpuMetrics,
}

/// Initialize GPU subsystem
pub fn initialize_gpu(
    device: std::sync::Arc<wgpu::Device>,
    queue: std::sync::Arc<wgpu::Queue>,
    config: Option<GpuConfig>,
) -> Result<GpuInitResult, GpuError> {
    let config = config.unwrap_or_default();
    
    log::info!("Initializing GPU subsystem with config: {:?}", config);
    
    // Create memory manager
    let memory_manager = GpuMemoryManager::new(device, queue);
    
    // Get initial metrics
    let initial_stats = memory_manager.get_memory_stats()
        .map_err(|e| GpuError::AllocationFailed(e.to_string()))?;
    
    let initial_metrics = GpuMetrics {
        memory_used: initial_stats.total_memory,
        memory_available: 0, // Would need to query GPU for this
        texture_allocations: initial_stats.texture_count as u64,
        buffer_allocations: initial_stats.buffer_count as u64,
        gpu_utilization: 0.0,
        average_frame_time: 0.0,
        frames_processed: 0,
    };
    
    log::info!("GPU subsystem initialized successfully");
    log::debug!("Initial memory usage: {}", initial_stats.format_memory());
    
    Ok(GpuInitResult {
        memory_manager,
        config,
        initial_metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_config_default() {
        let config = GpuConfig::default();
        assert_eq!(config.max_texture_size, 8192);
        assert_eq!(config.max_buffer_size, 256 * 1024 * 1024);
        assert_eq!(config.cleanup_threshold_seconds, 30);
        assert!(config.enable_pooling);
        assert_eq!(config.max_pooled_textures, 100);
        assert_eq!(config.max_pooled_buffers, 50);
    }
    
    #[test]
    fn test_gpu_metrics_default() {
        let metrics = GpuMetrics::default();
        assert_eq!(metrics.memory_used, 0);
        assert_eq!(metrics.memory_available, 0);
        assert_eq!(metrics.texture_allocations, 0);
        assert_eq!(metrics.buffer_allocations, 0);
        assert_eq!(metrics.gpu_utilization, 0.0);
        assert_eq!(metrics.average_frame_time, 0.0);
        assert_eq!(metrics.frames_processed, 0);
    }
}
