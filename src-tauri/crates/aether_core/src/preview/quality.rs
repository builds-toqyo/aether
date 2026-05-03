use std::time::{Duration, Instant};
use anyhow::Result;

/// Preview quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreviewQuality {
    Low,     // 1/4 resolution
    Medium,  // 1/2 resolution
    High,    // Full resolution
    Ultra,   // 2x resolution (for future use)
}

impl PreviewQuality {
    /// Get all quality levels
    pub fn all() -> Vec<Self> {
        vec![Self::Low, Self::Medium, Self::High, Self::Ultra]
    }
    
    /// Scale dimensions based on quality
    pub fn scale_dimensions(&self, width: u32, height: u32) -> (u32, u32) {
        match self {
            Self::Low => (width / 4, height / 4),
            Self::Medium => (width / 2, height / 2),
            Self::High => (width, height),
            Self::Ultra => (width * 2, height * 2),
        }
    }
    
    /// Convert to float for shader parameters
    pub fn to_f32(&self) -> f32 {
        match self {
            Self::Low => 0.25,
            Self::Medium => 0.5,
            Self::High => 1.0,
            Self::Ultra => 2.0,
        }
    }
    
    /// Get quality level as number (0-3)
    pub fn level(&self) -> u8 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Ultra => 3,
        }
    }
    
    /// Get next higher quality
    pub fn next_higher(&self) -> Option<Self> {
        match self {
            Self::Low => Some(Self::Medium),
            Self::Medium => Some(Self::High),
            Self::High => Some(Self::Ultra),
            Self::Ultra => None,
        }
    }
    
    /// Get next lower quality
    pub fn next_lower(&self) -> Option<Self> {
        match self {
            Self::Low => None,
            Self::Medium => Some(Self::Low),
            Self::High => Some(Self::Medium),
            Self::Ultra => Some(Self::High),
        }
    }
}

/// Adaptive quality controller
#[derive(Debug, Clone)]
pub struct AdaptiveQualityController {
    config: AdaptiveQualityConfig,
    current_quality: PreviewQuality,
    performance_history: Vec<Duration>,
    last_adjustment: Instant,
}

impl AdaptiveQualityController {
    /// Create a new adaptive quality controller
    pub fn new(config: AdaptiveQualityConfig) -> Self {
        Self {
            config,
            current_quality: PreviewQuality::Medium,
            performance_history: Vec::new(),
            last_adjustment: Instant::now(),
        }
    }
    
    /// Recommend quality based on performance
    pub fn recommend_quality(&mut self, performance: super::SessionPerformance) -> PreviewQuality {
        // Add recent performance to history
        self.performance_history.push(performance.recent_average_render_time());
        
        // Keep only recent history
        if self.performance_history.len() > self.config.history_size {
            self.performance_history.remove(0);
        }
        
        // Check if we should adjust quality
        if self.last_adjustment.elapsed() < self.config.adjustment_interval {
            return self.current_quality;
        }
        
        let avg_render_time = if self.performance_history.is_empty() {
            Duration::from_millis(16) // Default to 60 FPS target
        } else {
            let total: Duration = self.performance_history.iter().sum();
            total / self.performance_history.len() as u32
        };
        
        let target_frame_time = Duration::from_millis(1000 / self.config.target_fps);
        let quality = if avg_render_time > target_frame_time * 2 {
            // Too slow, decrease quality
            self.current_quality.next_lower().unwrap_or(self.current_quality)
        } else if avg_render_time < target_frame_time / 2 {
            // Very fast, can increase quality
            self.current_quality.next_higher().unwrap_or(self.current_quality)
        } else {
            // Good performance, keep current quality
            self.current_quality
        };
        
        if quality != self.current_quality {
            self.current_quality = quality;
            self.last_adjustment = Instant::now();
            log::debug!("Adjusted preview quality to: {:?}", quality);
        }
        
        self.current_quality
    }
    
    /// Get current quality
    pub fn current_quality(&self) -> PreviewQuality {
        self.current_quality
    }
    
    /// Force set quality
    pub fn set_quality(&mut self, quality: PreviewQuality) {
        self.current_quality = quality;
        self.last_adjustment = Instant::now();
    }
    
    /// Reset performance history
    pub fn reset_history(&mut self) {
        self.performance_history.clear();
    }
    
    /// Get performance history size
    pub fn history_size(&self) -> usize {
        self.performance_history.len()
    }
}

/// Adaptive quality configuration
#[derive(Debug, Clone)]
pub struct AdaptiveQualityConfig {
    pub target_fps: u32,
    pub adjustment_interval: Duration,
    pub history_size: usize,
    pub quality_threshold: f64, // Percentage of target FPS to trigger adjustment
    pub min_quality: Option<PreviewQuality>,
    pub max_quality: Option<PreviewQuality>,
}

impl Default for AdaptiveQualityConfig {
    fn default() -> Self {
        Self {
            target_fps: 30,
            adjustment_interval: Duration::from_secs(2),
            history_size: 10,
            quality_threshold: 0.8,
            min_quality: None,
            max_quality: None,
        }
    }
}

impl AdaptiveQualityConfig {
    /// Create a new config with custom parameters
    pub fn new(target_fps: u32) -> Self {
        Self {
            target_fps,
            adjustment_interval: Duration::from_secs(2),
            history_size: 10,
            quality_threshold: 0.8,
            min_quality: None,
            max_quality: None,
        }
    }
    
    /// Set adjustment interval
    pub fn with_adjustment_interval(mut self, interval: Duration) -> Self {
        self.adjustment_interval = interval;
        self
    }
    
    /// Set history size
    pub fn with_history_size(mut self, size: usize) -> Self {
        self.history_size = size;
        self
    }
    
    /// Set quality threshold
    pub fn with_quality_threshold(mut self, threshold: f64) -> Self {
        self.quality_threshold = threshold.clamp(0.1, 1.0);
        self
    }
    
    /// Set minimum quality
    pub fn with_min_quality(mut self, quality: PreviewQuality) -> Self {
        self.min_quality = Some(quality);
        self
    }
    
    /// Set maximum quality
    pub fn with_max_quality(mut self, quality: PreviewQuality) -> Self {
        self.max_quality = Some(quality);
        self
    }
    
    /// Clamp quality to configured bounds
    pub fn clamp_quality(&self, quality: PreviewQuality) -> PreviewQuality {
        let mut clamped = quality;
        
        if let Some(min) = self.min_quality {
            if clamped.level() < min.level() {
                clamped = min;
            }
        }
        
        if let Some(max) = self.max_quality {
            if clamped.level() > max.level() {
                clamped = max;
            }
        }
        
        clamped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preview_quality_scaling() {
        assert_eq!(PreviewQuality::Low.scale_dimensions(1920, 1080), (480, 270));
        assert_eq!(PreviewQuality::Medium.scale_dimensions(1920, 1080), (960, 540));
        assert_eq!(PreviewQuality::High.scale_dimensions(1920, 1080), (1920, 1080));
        assert_eq!(PreviewQuality::Ultra.scale_dimensions(1920, 1080), (3840, 2160));
    }
    
    #[test]
    fn test_quality_navigation() {
        assert_eq!(PreviewQuality::Low.next_higher(), Some(PreviewQuality::Medium));
        assert_eq!(PreviewQuality::Medium.next_lower(), Some(PreviewQuality::Low));
        assert_eq!(PreviewQuality::Ultra.next_higher(), None);
        assert_eq!(PreviewQuality::Low.next_lower(), None);
    }
    
    #[test]
    fn test_adaptive_quality() {
        let config = AdaptiveQualityConfig::new(30);
        let mut controller = AdaptiveQualityController::new(config);
        
        let performance = super::SessionPerformance::new();
        let quality = controller.recommend_quality(performance);
        
        // Should default to Medium
        assert_eq!(quality, PreviewQuality::Medium);
    }
    
    #[test]
    fn test_adaptive_config() {
        let config = AdaptiveQualityConfig::new(60)
            .with_adjustment_interval(Duration::from_secs(1))
            .with_history_size(5)
            .with_quality_threshold(0.9)
            .with_min_quality(PreviewQuality::Medium)
            .with_max_quality(PreviewQuality::High);
        
        assert_eq!(config.target_fps, 60);
        assert_eq!(config.adjustment_interval, Duration::from_secs(1));
        assert_eq!(config.history_size, 5);
        assert_eq!(config.quality_threshold, 0.9);
        assert_eq!(config.min_quality, Some(PreviewQuality::Medium));
        assert_eq!(config.max_quality, Some(PreviewQuality::High));
        
        // Test quality clamping
        assert_eq!(config.clamp_quality(PreviewQuality::Low), PreviewQuality::Medium);
        assert_eq!(config.clamp_quality(PreviewQuality::Ultra), PreviewQuality::High);
        assert_eq!(config.clamp_quality(PreviewQuality::High), PreviewQuality::High);
    }
}
