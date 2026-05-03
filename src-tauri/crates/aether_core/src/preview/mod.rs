//! Real-time preview system with adaptive quality and background processing
//! 
//! This module provides comprehensive preview capabilities for the Aether node system,
//! including adaptive quality control, multi-threaded rendering, and performance monitoring.

pub mod system;
pub mod session;
pub mod quality;
pub mod performance;
pub mod render;

// Re-export main preview types
pub use system::PreviewSystem;
pub use session::{PreviewSession, PreviewSessionHandle};
pub use quality::{PreviewQuality, AdaptiveQualityController, AdaptiveQualityConfig};
pub use performance::{PerformanceMonitor, SessionPerformance, PreviewStats};
pub use render::{RenderTask, PreviewFrame, RenderWorker};

/// Preview configuration
#[derive(Debug, Clone)]
pub struct PreviewConfig {
    pub render_workers: usize,
    pub max_concurrent_renders: usize,
    pub adaptive_config: AdaptiveQualityConfig,
    pub cleanup_interval: std::time::Duration,
    pub max_inactive_time: std::time::Duration,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            render_workers: num_cpus::get(),
            max_concurrent_renders: 4,
            adaptive_config: AdaptiveQualityConfig::default(),
            cleanup_interval: std::time::Duration::from_secs(30),
            max_inactive_time: std::time::Duration::from_secs(300), // 5 minutes
        }
    }
}
