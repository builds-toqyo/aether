//! GPU memory management and processing modules for the Aether node system
//! 
//! This module provides comprehensive GPU memory management, texture handling,
//! buffer management, and GPU-CPU synchronization for efficient node processing.

pub mod memory;

// Re-export main GPU memory management types
pub use memory::{
    GpuMemoryManager, TextureHandle, BufferHandle, SamplerHandle,
    MemoryStats, SyncStatus
};

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
