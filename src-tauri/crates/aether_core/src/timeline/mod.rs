//! Timeline integration system for smooth video scrubbing and playback
//! 
//! This module provides comprehensive timeline capabilities including frame caching,
//! smooth scrubbing, pre-roll buffering, and bidirectional navigation for the Aether
//! video editing platform.

pub mod scrubbing;
pub mod cache;
pub mod buffer;
pub mod navigation;

// Re-export main timeline types
pub use scrubbing::{TimelineScrubber, ScrubbingConfig, ScrubbingDirection};
pub use cache::{FrameCache, CacheConfig, CacheStats};
pub use buffer::{PreRollBuffer, BufferConfig};
pub use navigation::{TimelineNavigator, NavigationConfig};

/// Timeline configuration
#[derive(Debug, Clone)]
pub struct TimelineConfig {
    pub scrubbing: ScrubbingConfig,
    pub cache: CacheConfig,
    pub buffer: BufferConfig,
    pub navigation: NavigationConfig,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            scrubbing: ScrubbingConfig::default(),
            cache: CacheConfig::default(),
            buffer: BufferConfig::default(),
            navigation: NavigationConfig::default(),
        }
    }
}

/// Timeline position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimelinePosition {
    pub frame: u64,
    pub time: f64,
}

impl TimelinePosition {
    /// Create a new timeline position
    pub fn new(frame: u64, time: f64) -> Self {
        Self { frame, time }
    }
    
    /// Create position from frame number (assuming 30 FPS)
    pub fn from_frame(frame: u64) -> Self {
        Self {
            frame,
            time: frame as f64 / 30.0,
        }
    }
    
    /// Create position from time in seconds
    pub fn from_time(time: f64) -> Self {
        Self {
            frame: (time * 30.0) as u64,
            time,
        }
    }
    
    /// Get frame number
    pub fn frame(&self) -> u64 {
        self.frame
    }
    
    /// Get time in seconds
    pub fn time(&self) -> f64 {
        self.time
    }
    
    /// Add frames
    pub fn add_frames(&self, frames: i64) -> TimelinePosition {
        let new_frame = (self.frame as i64 + frames).max(0) as u64;
        TimelinePosition::from_frame(new_frame)
    }
    
    /// Add time in seconds
    pub fn add_time(&self, time: f64) -> TimelinePosition {
        TimelinePosition::from_time(self.time + time)
    }
    
    /// Distance to another position
    pub fn distance_to(&self, other: &TimelinePosition) -> i64 {
        other.frame as i64 - self.frame as i64
    }
}

/// Timeline direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineDirection {
    Forward,
    Backward,
}

impl TimelineDirection {
    /// Get the opposite direction
    pub fn opposite(&self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
    
    /// Check if this is forward direction
    pub fn is_forward(&self) -> bool {
        matches!(self, Self::Forward)
    }
    
    /// Check if this is backward direction
    pub fn is_backward(&self) -> bool {
        matches!(self, Self::Backward)
    }
}

/// Timeline range
#[derive(Debug, Clone)]
pub struct TimelineRange {
    pub start: TimelinePosition,
    pub end: TimelinePosition,
}

impl TimelineRange {
    /// Create a new timeline range
    pub fn new(start: TimelinePosition, end: TimelinePosition) -> Self {
        Self { start, end }
    }
    
    /// Get range duration in frames
    pub fn duration_frames(&self) -> u64 {
        self.end.frame.saturating_sub(self.start.frame)
    }
    
    /// Get range duration in seconds
    pub fn duration_time(&self) -> f64 {
        self.end.time - self.start.time
    }
    
    /// Check if a position is within the range
    pub fn contains(&self, position: &TimelinePosition) -> bool {
        position.frame >= self.start.frame && position.frame <= self.end.frame
    }
    
    /// Get a position at a specific offset within the range
    pub fn position_at_offset(&self, offset: f64) -> TimelinePosition {
        let time = self.start.time + (self.duration_time() * offset.clamp(0.0, 1.0));
        TimelinePosition::from_time(time)
    }
    
    /// Clamp a position to the range
    pub fn clamp(&self, position: TimelinePosition) -> TimelinePosition {
        if position.frame < self.start.frame {
            self.start
        } else if position.frame > self.end.frame {
            self.end
        } else {
            position
        }
    }
}
