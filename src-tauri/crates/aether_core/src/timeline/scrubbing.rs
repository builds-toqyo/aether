use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};

use crate::preview::{PreviewSystem, PreviewFrame, PreviewQuality};
use crate::nodes::Graph;

use super::{
    TimelinePosition, TimelineDirection, TimelineRange,
    FrameCache, PreRollBuffer, CacheConfig, BufferConfig
};


pub struct TimelineScrubber {
    config: ScrubbingConfig,
    preview_system: Arc<PreviewSystem>,
    graph: Arc<Graph>,
    frame_cache: Arc<FrameCache>,
    pre_roll_buffer: Arc<PreRollBuffer>,


    current_position: Arc<RwLock<TimelinePosition>>,
    scrubbing_direction: Arc<RwLock<TimelineDirection>>,
    scrubbing_speed: Arc<RwLock<f64>>,
    is_scrubbing: Arc<RwLock<bool>>,


    scrub_stats: Arc<RwLock<ScrubbingStats>>,


    preview_session_id: Arc<RwLock<Option<Uuid>>>,
}

impl TimelineScrubber {

    pub fn new(
        config: ScrubbingConfig,
        preview_system: Arc<PreviewSystem>,
        graph: Arc<Graph>,
        cache_config: CacheConfig,
        buffer_config: BufferConfig,
    ) -> Result<Self> {
        info!("Creating timeline scrubber with config: {:?}", config);

        let frame_cache = Arc::new(FrameCache::new(cache_config));
        let pre_roll_buffer = Arc::new(PreRollBuffer::new(buffer_config));

        let scrubber = Self {
            config,
            preview_system,
            graph,
            frame_cache,
            pre_roll_buffer,
            current_position: Arc::new(RwLock::new(TimelinePosition::from_frame(0))),
            scrubbing_direction: Arc::new(RwLock::new(TimelineDirection::Forward)),
            scrubbing_speed: Arc::new(RwLock::new(30.0)),
            is_scrubbing: Arc::new(RwLock::new(false)),
            scrub_stats: Arc::new(RwLock::new(ScrubbingStats::new())),
            preview_session_id: Arc::new(RwLock::new(None)),
        };

        info!("Timeline scrubber created successfully");

        Ok(scrubber)
    }


    pub fn initialize(&self, range: TimelineRange) -> Result<()> {
        debug!("Initializing timeline scrubber with range: {:?}", range);


        let session = self.preview_system.create_session(
            self.graph.clone(),
            1920, 1080,
            PreviewQuality::Medium,
        )?;


        {
            let mut session_id = self.preview_session_id.write().map_err(|e| anyhow!("Session ID lock error: {}", e))?;
            *session_id = Some(session.id());
        }


        self.preview_system.start_preview(&session.id())?;


        self.frame_cache.initialize_range(range.clone())?;


        self.pre_roll_buffer.initialize(range.start, self.config.pre_roll_size)?;


        {
            let mut position = self.current_position.write().map_err(|e| anyhow!("Position lock error: {}", e))?;
            *position = range.start;
        }

        info!("Timeline scrubber initialized with range: {:?}", range);

        Ok(())
    }


    pub fn start_scrubbing(&self, direction: TimelineDirection, speed: f64) -> Result<()> {
        debug!("Starting scrubbing: {:?} at {:.1} FPS", direction, speed);

        {
            let mut is_scrubbing = self.is_scrubbing.write().map_err(|e| anyhow!("Scrubbing lock error: {}", e))?;
            let mut scrub_direction = self.scrubbing_direction.write().map_err(|e| anyhow!("Direction lock error: {}", e))?;
            let mut scrub_speed = self.scrubbing_speed.write().map_err(|e| anyhow!("Speed lock error: {}", e))?;

            *is_scrubbing = true;
            *scrub_direction = direction;
            *scrub_speed = speed.clamp(1.0, 120.0);
        }


        self.start_scrubbing_task()?;

        info!("Started scrubbing: {:?} at {:.1} FPS", direction, speed);

        Ok(())
    }


    pub fn stop_scrubbing(&self) -> Result<()> {
        debug!("Stopping scrubbing");

        {
            let mut is_scrubbing = self.is_scrubbing.write().map_err(|e| anyhow!("Scrubbing lock error: {}", e))?;
            *is_scrubbing = false;
        }

        info!("Stopped scrubbing");

        Ok(())
    }


    pub async fn seek_to(&self, position: TimelinePosition) -> Result<PreviewFrame> {
        debug!("Seeking to position: {:?}", position);


        if let Some(cached_frame) = self.frame_cache.get_frame(position).await? {
            debug!("Found cached frame for position: {:?}", position);
            return Ok(cached_frame);
        }


        let frame = self.generate_frame_at_position(position).await?;


        self.frame_cache.cache_frame(position, frame.clone()).await?;


        {
            let mut current_pos = self.current_position.write().map_err(|e| anyhow!("Position lock error: {}", e))?;
            *current_pos = position;
        }


        {
            let mut stats = self.scrub_stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.seek_operations += 1;
            stats.cache_misses += 1;
        }

        info!("Seeked to position: {:?}", position);

        Ok(frame)
    }


    pub fn current_position(&self) -> Result<TimelinePosition> {
        let position = self.current_position.read().map_err(|e| anyhow!("Position lock error: {}", e))?;
        Ok(*position)
    }


    pub fn get_stats(&self) -> Result<ScrubbingStats> {
        let stats = self.scrub_stats.read().map_err(|e| anyhow!("Stats lock error: {}", e))?;
        let cache_stats = self.frame_cache.get_stats()?;
        let buffer_stats = self.pre_roll_buffer.get_stats()?;

        let mut current_stats = stats.clone();
        current_stats.cache_stats = Some(cache_stats);
        current_stats.buffer_stats = Some(buffer_stats);

        Ok(current_stats)
    }


    pub fn clear_cache(&self) -> Result<()> {
        debug!("Clearing timeline cache and buffers");

        self.frame_cache.clear().await?;
        self.pre_roll_buffer.clear().await?;


        {
            let mut stats = self.scrub_stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            *stats = ScrubbingStats::new();
        }

        info!("Cleared timeline cache and buffers");

        Ok(())
    }


    fn start_scrubbing_task(&self) -> Result<()> {
        let is_scrubbing = self.is_scrubbing.clone();
        let current_position = self.current_position.clone();
        let scrubbing_direction = self.scrubbing_direction.clone();
        let scrubbing_speed = self.scrubbing_speed.clone();
        let frame_cache = self.frame_cache.clone();
        let pre_roll_buffer = self.pre_roll_buffer.clone();
        let preview_system = self.preview_system.clone();
        let preview_session_id = self.preview_session_id.clone();
        let scrub_stats = self.scrub_stats.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut last_frame_time = Instant::now();
            let target_frame_interval = Duration::from_millis(33);

            loop {

                {
                    let scrubbing = is_scrubbing.read().unwrap();
                    if !*scrubbing {
                        break;
                    }
                }


                let now = Instant::now();
                let elapsed = now - last_frame_time;

                if elapsed >= target_frame_interval {

                    let (mut position, direction, speed) = {
                        let pos = current_position.read().unwrap();
                        let dir = scrubbing_direction.read().unwrap();
                        let spd = scrubbing_speed.read().unwrap();
                        (*pos, *dir, *spd)
                    };


                    let frame_delta = (speed * elapsed.as_secs_f64()) as i64;
                    let next_position = if direction.is_forward() {
                        position.add_frames(frame_delta)
                    } else {
                        position.add_frames(-frame_delta)
                    };


                    let frame = if let Some(cached_frame) = frame_cache.get_frame(next_position).await.unwrap() {
                        cached_frame
                    } else if let Some(buffered_frame) = pre_roll_buffer.get_frame(next_position).await.unwrap() {
                        buffered_frame
                    } else {

                        match generate_frame(&preview_system, &preview_session_id, next_position).await {
                            Ok(frame) => {

                                let _ = frame_cache.cache_frame(next_position, frame.clone()).await;
                                frame
                            }
                            Err(e) => {
                                error!("Failed to generate frame: {}", e);
                                continue;
                            }
                        }
                    };


                    {
                        let mut pos = current_position.write().unwrap();
                        *pos = next_position;
                    }


                    {
                        let mut stats = scrub_stats.write().unwrap();
                        stats.frames_scrubbed += 1;
                        stats.total_scrub_time += elapsed;

                        if frame.render_time > config.max_frame_time {
                            stats.slow_frames += 1;
                        }
                    }

                    last_frame_time = now;
                }


                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        Ok(())
    }


    async fn generate_frame_at_position(&self, position: TimelinePosition) -> Result<PreviewFrame> {
        let session_id = {
            let id = self.preview_session_id.read().map_err(|e| anyhow!("Session ID lock error: {}", e))?;
            id.ok_or_else(|| anyhow!("No preview session available"))?
        };

        generate_frame(&self.preview_system, &self.preview_session_id, position).await
    }
}


async fn generate_frame(
    preview_system: &PreviewSystem,
    preview_session_id: &Arc<RwLock<Option<Uuid>>>,
    position: TimelinePosition,
) -> Result<PreviewFrame> {
    let session_id = {
        let id = preview_session_id.read().unwrap();
        id.ok_or_else(|| anyhow!("No preview session available"))?
    };

    preview_system.request_frame(&session_id, position.time(), None).await
}


#[derive(Debug, Clone)]
pub struct ScrubbingConfig {
    pub pre_roll_size: usize,
    pub max_frame_time: Duration,
    pub cache_size: usize,
    pub buffer_size: usize,
    pub adaptive_quality: bool,
    pub bidirectional: bool,
}

impl Default for ScrubbingConfig {
    fn default() -> Self {
        Self {
            pre_roll_size: 30,
            max_frame_time: Duration::from_millis(50),
            cache_size: 1000,
            buffer_size: 60,
            adaptive_quality: true,
            bidirectional: true,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubbingDirection {
    Forward,
    Backward,
}

impl From<TimelineDirection> for ScrubbingDirection {
    fn from(direction: TimelineDirection) -> Self {
        match direction {
            TimelineDirection::Forward => Self::Forward,
            TimelineDirection::Backward => Self::Backward,
        }
    }
}


#[derive(Debug, Clone)]
pub struct ScrubbingStats {
    pub frames_scrubbed: u64,
    pub seek_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_scrub_time: Duration,
    pub slow_frames: u64,
    pub cache_stats: Option<super::CacheStats>,
    pub buffer_stats: Option<super::BufferStats>,
}

impl ScrubbingStats {

    pub fn new() -> Self {
        Self {
            frames_scrubbed: 0,
            seek_operations: 0,
            cache_hits: 0,
            cache_misses: 0,
            total_scrub_time: Duration::ZERO,
            slow_frames: 0,
            cache_stats: None,
            buffer_stats: None,
        }
    }


    pub fn average_fps(&self) -> f64 {
        if self.total_scrub_time.as_secs_f64() > 0.0 {
            self.frames_scrubbed as f64 / self.total_scrub_time.as_secs_f64()
        } else {
            0.0
        }
    }


    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests > 0 {
            self.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }


    pub fn slow_frame_rate(&self) -> f64 {
        if self.frames_scrubbed > 0 {
            self.slow_frames as f64 / self.frames_scrubbed as f64
        } else {
            0.0
        }
    }


    pub fn format(&self) -> String {
        format!(
            "Frames: {} | FPS: {:.1} | Seeks: {} | Cache Hit Rate: {:.1}% | Slow Frames: {:.1}%",
            self.frames_scrubbed,
            self.average_fps(),
            self.seek_operations,
            self.cache_hit_rate() * 100.0,
            self.slow_frame_rate() * 100.0
        )
    }
}

impl Default for ScrubbingStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_position() {
        let pos = TimelinePosition::new(100, 3.33);
        assert_eq!(pos.frame(), 100);
        assert_eq!(pos.time(), 3.33);

        let from_frame = TimelinePosition::from_frame(60);
        assert_eq!(from_frame.frame(), 60);
        assert_eq!(from_frame.time(), 2.0);

        let from_time = TimelinePosition::from_time(2.5);
        assert_eq!(from_time.frame(), 75);
        assert_eq!(from_time.time(), 2.5);

        let added = pos.add_frames(10);
        assert_eq!(added.frame(), 110);

        let distance = pos.distance_to(&added);
        assert_eq!(distance, 10);
    }

    #[test]
    fn test_timeline_range() {
        let start = TimelinePosition::from_frame(0);
        let end = TimelinePosition::from_frame(100);
        let range = TimelineRange::new(start, end);

        assert_eq!(range.duration_frames(), 100);
        assert_eq!(range.duration_time(), 3.33);

        let middle = TimelinePosition::from_frame(50);
        assert!(range.contains(&middle));

        let outside = TimelinePosition::from_frame(150);
        assert!(!range.contains(&outside));

        let clamped = range.clamp(outside);
        assert_eq!(clamped.frame(), 100);
    }

    #[test]
    fn test_scrubbing_config() {
        let config = ScrubbingConfig::default();
        assert_eq!(config.pre_roll_size, 30);
        assert_eq!(config.max_frame_time, Duration::from_millis(50));
        assert!(config.adaptive_quality);
        assert!(config.bidirectional);
    }

    #[test]
    fn test_scrubbing_stats() {
        let mut stats = ScrubbingStats::new();
        stats.frames_scrubbed = 300;
        stats.cache_hits = 250;
        stats.cache_misses = 50;
        stats.total_scrub_time = Duration::from_secs(10);

        assert_eq!(stats.average_fps(), 30.0);
        assert_eq!(stats.cache_hit_rate(), 0.833);
        assert_eq!(stats.slow_frame_rate(), 0.0);

        let formatted = stats.format();
        assert!(formatted.contains("Frames: 300"));
        assert!(formatted.contains("FPS: 30.0"));
        assert!(formatted.contains("Cache Hit Rate: 83.3%"));
    }
}
