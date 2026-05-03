use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;
use anyhow::{Result, anyhow};

use crate::gpu::FrameBufferHandle;
use crate::nodes::Graph;

use super::{PreviewQuality, AdaptiveQualityController, AdaptiveQualityConfig};

/// Preview session for managing a single preview instance
#[derive(Debug, Clone)]
pub struct PreviewSession {
    pub id: Uuid,
    pub graph: std::sync::Arc<Graph>,
    pub width: u32,
    pub height: u32,
    current_quality: PreviewQuality,
    target_quality: PreviewQuality,
    pub frame_buffers: HashMap<PreviewQuality, FrameBufferHandle>,
    pub created_at: Instant,
    pub last_frame_time: std::sync::Mutex<Instant>,
    pub frame_count: std::sync::atomic::AtomicU64,
    pub dropped_frames: std::sync::atomic::AtomicU64,
    pub adaptive_quality: AdaptiveQualityController,
}

impl PreviewSession {
    /// Create a new preview session
    pub fn new(
        id: Uuid,
        graph: std::sync::Arc<Graph>,
        width: u32,
        height: u32,
        quality: PreviewQuality,
        frame_buffers: HashMap<PreviewQuality, FrameBufferHandle>,
        adaptive_config: AdaptiveQualityConfig,
    ) -> Self {
        Self {
            id,
            graph,
            width,
            height,
            current_quality: quality,
            target_quality: quality,
            frame_buffers,
            created_at: Instant::now(),
            last_frame_time: std::sync::Mutex::new(Instant::now()),
            frame_count: std::sync::atomic::AtomicU64::new(0),
            dropped_frames: std::sync::atomic::AtomicU64::new(0),
            adaptive_quality: AdaptiveQualityController::new(adaptive_config),
        }
    }
    
    /// Get frame buffer for quality level
    pub fn get_frame_buffer_for_quality(&self, quality: PreviewQuality) -> Result<FrameBufferHandle> {
        self.frame_buffers.get(&quality)
            .cloned()
            .ok_or_else(|| anyhow!("Frame buffer not available for quality: {:?}", quality))
    }
    
    /// Get current quality
    pub fn get_current_quality(&self) -> PreviewQuality {
        self.current_quality
    }
    
    /// Set quality
    pub fn set_quality(&mut self, quality: PreviewQuality) {
        self.current_quality = quality;
        self.target_quality = quality;
    }
    
    /// Get frame rate
    pub fn frame_rate(&self) -> f64 {
        let elapsed = self.created_at.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.frame_count.load(std::sync::atomic::Ordering::Relaxed) as f64 / elapsed
        } else {
            0.0
        }
    }
    
    /// Get drop rate
    pub fn drop_rate(&self) -> f64 {
        let frame_count = self.frame_count.load(std::sync::atomic::Ordering::Relaxed);
        if frame_count > 0 {
            self.dropped_frames.load(std::sync::atomic::Ordering::Relaxed) as f64 / frame_count as f64
        } else {
            0.0
        }
    }
    
    /// Increment frame count
    pub fn increment_frame_count(&self) {
        self.frame_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Increment dropped frames
    pub fn increment_dropped_frames(&self) {
        self.dropped_frames.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Update last frame time
    pub fn update_last_frame_time(&self) {
        if let Ok(mut last_time) = self.last_frame_time.lock() {
            *last_time = Instant::now();
        }
    }
    
    /// Get adaptive quality recommendation
    pub fn recommend_quality(&mut self, performance: super::SessionPerformance) -> PreviewQuality {
        self.adaptive_quality.recommend_quality(performance)
    }
}

/// Handle for a preview session
#[derive(Debug, Clone)]
pub struct PreviewSessionHandle {
    pub id: Uuid,
    pub session: std::sync::Arc<PreviewSession>,
}

impl PreviewSessionHandle {
    /// Create a new session handle
    pub fn new(id: Uuid, session: std::sync::Arc<PreviewSession>) -> Self {
        Self { id, session }
    }
    
    /// Get session ID
    pub fn id(&self) -> Uuid {
        self.id
    }
    
    /// Get session reference
    pub fn session(&self) -> &std::sync::Arc<PreviewSession> {
        &self.session
    }
    
    /// Get frame rate
    pub fn frame_rate(&self) -> f64 {
        self.session.frame_rate()
    }
    
    /// Get drop rate
    pub fn drop_rate(&self) -> f64 {
        self.session.drop_rate()
    }
    
    /// Get current quality
    pub fn current_quality(&self) -> PreviewQuality {
        self.session.get_current_quality()
    }
    
    /// Get dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.session.width, self.session.height)
    }
}
