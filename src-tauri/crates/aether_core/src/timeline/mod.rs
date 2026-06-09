

pub mod scrubbing;
pub mod cache;
pub mod buffer;
pub mod navigation;


pub use scrubbing::{TimelineScrubber, ScrubbingConfig, ScrubbingDirection};
pub use cache::{FrameCache, CacheConfig, CacheStats};
pub use buffer::{PreRollBuffer, BufferConfig};
pub use navigation::{TimelineNavigator, NavigationConfig};


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


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimelinePosition {
    pub frame: u64,
    pub time: f64,
}

impl TimelinePosition {

    pub fn new(frame: u64, time: f64) -> Self {
        Self { frame, time }
    }


    pub fn from_frame(frame: u64) -> Self {
        Self {
            frame,
            time: frame as f64 / 30.0,
        }
    }


    pub fn from_time(time: f64) -> Self {
        Self {
            frame: (time * 30.0) as u64,
            time,
        }
    }


    pub fn frame(&self) -> u64 {
        self.frame
    }


    pub fn time(&self) -> f64 {
        self.time
    }


    pub fn add_frames(&self, frames: i64) -> TimelinePosition {
        let new_frame = (self.frame as i64 + frames).max(0) as u64;
        TimelinePosition::from_frame(new_frame)
    }


    pub fn add_time(&self, time: f64) -> TimelinePosition {
        TimelinePosition::from_time(self.time + time)
    }


    pub fn distance_to(&self, other: &TimelinePosition) -> i64 {
        other.frame as i64 - self.frame as i64
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineDirection {
    Forward,
    Backward,
}

impl TimelineDirection {

    pub fn opposite(&self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }


    pub fn is_forward(&self) -> bool {
        matches!(self, Self::Forward)
    }


    pub fn is_backward(&self) -> bool {
        matches!(self, Self::Backward)
    }
}


#[derive(Debug, Clone)]
pub struct TimelineRange {
    pub start: TimelinePosition,
    pub end: TimelinePosition,
}

impl TimelineRange {

    pub fn new(start: TimelinePosition, end: TimelinePosition) -> Self {
        Self { start, end }
    }


    pub fn duration_frames(&self) -> u64 {
        self.end.frame.saturating_sub(self.start.frame)
    }


    pub fn duration_time(&self) -> f64 {
        self.end.time - self.start.time
    }


    pub fn contains(&self, position: &TimelinePosition) -> bool {
        position.frame >= self.start.frame && position.frame <= self.end.frame
    }


    pub fn position_at_offset(&self, offset: f64) -> TimelinePosition {
        let time = self.start.time + (self.duration_time() * offset.clamp(0.0, 1.0));
        TimelinePosition::from_time(time)
    }


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
