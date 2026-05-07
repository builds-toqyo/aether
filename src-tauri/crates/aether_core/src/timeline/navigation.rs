use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use log::{debug, info, warn};

use crate::preview::{PreviewFrame, PreviewQuality};

use super::{TimelinePosition, TimelineDirection, TimelineRange, FrameCache, PreRollBuffer};


pub struct TimelineNavigator {
    config: NavigationConfig,
    frame_cache: Arc<FrameCache>,
    pre_roll_buffer: Arc<PreRollBuffer>,


    current_position: Arc<RwLock<TimelinePosition>>,
    navigation_direction: Arc<RwLock<TimelineDirection>>,
    navigation_speed: Arc<RwLock<f64>>,
    is_navigating: Arc<RwLock<bool>>,


    navigation_history: Arc<RwLock<Vec<NavigationEntry>>>,
    bookmarks: Arc<RwLock<HashMap<String, TimelinePosition>>>,


    nav_stats: Arc<RwLock<NavigationStats>>,


    timeline_range: Arc<RwLock<Option<TimelineRange>>>,
}

impl TimelineNavigator {

    pub fn new(
        config: NavigationConfig,
        frame_cache: Arc<FrameCache>,
        pre_roll_buffer: Arc<PreRollBuffer>,
    ) -> Result<Self> {
        info!("Creating timeline navigator with config: {:?}", config);

        let navigator = Self {
            config,
            frame_cache,
            pre_roll_buffer,
            current_position: Arc::new(RwLock::new(TimelinePosition::from_frame(0))),
            navigation_direction: Arc::new(RwLock::new(TimelineDirection::Forward)),
            navigation_speed: Arc::new(RwLock::new(30.0)),
            is_navigating: Arc::new(RwLock::new(false)),
            navigation_history: Arc::new(RwLock::new(Vec::new())),
            bookmarks: Arc::new(RwLock::new(HashMap::new())),
            nav_stats: Arc::new(RwLock::new(NavigationStats::new())),
            timeline_range: Arc::new(RwLock::new(None)),
        };

        info!("Timeline navigator created successfully");

        Ok(navigator)
    }


    pub fn set_timeline_range(&self, range: TimelineRange) -> Result<()> {
        debug!("Setting timeline range: {:?}", range);

        {
            let mut timeline_range = self.timeline_range.write().map_err(|e| anyhow!("Timeline range lock error: {}", e))?;
            *timeline_range = Some(range.clone());
        }


        self.frame_cache.initialize_range(range.clone())?;


        self.pre_roll_buffer.initialize(range.start, self.config.pre_roll_size)?;


        {
            let mut current_pos = self.current_position.write().map_err(|e| anyhow!("Current position lock error: {}", e))?;
            *current_pos = range.start;
        }

        info!("Timeline range set: {:?}", range);

        Ok(())
    }


    pub async fn navigate_to(&self, position: TimelinePosition) -> Result<PreviewFrame> {
        debug!("Navigating to position: {:?}", position);


        if let Some(range) = self.timeline_range.read().unwrap().as_ref() {
            if !range.contains(&position) {
                return Err(anyhow!("Position {:?} is outside timeline range {:?}", position, range));
            }
        }


        self.add_navigation_entry(position, NavigationAction::Jump).await?;


        if let Some(cached_frame) = self.frame_cache.get_frame(position).await? {
            debug!("Found cached frame for position: {:?}", position);


            {
                let mut current_pos = self.current_position.write().map_err(|e| anyhow!("Current position lock error: {}", e))?;
                *current_pos = position;
            }


            {
                let mut stats = self.nav_stats.write().map_err(|e| anyhow!("Navigation stats lock error: {}", e))?;
                stats.cache_hits += 1;
                stats.total_navigations += 1;
            }

            return Ok(cached_frame);
        }


        if let Some(buffered_frame) = self.pre_roll_buffer.get_frame(position).await? {
            debug!("Found buffered frame for position: {:?}", position);


            {
                let mut current_pos = self.current_position.write().map_err(|e| anyhow!("Current position lock error: {}", e))?;
                *current_pos = position;
            }


            {
                let mut stats = self.nav_stats.write().map_err(|e| anyhow!("Navigation stats lock error: {}", e))?;
                stats.buffer_hits += 1;
                stats.total_navigations += 1;
            }

            return Ok(buffered_frame);
        }


        let frame = self.generate_frame_at_position(position).await?;


        self.frame_cache.cache_frame(position, frame.clone()).await?;


        {
            let mut current_pos = self.current_position.write().map_err(|e| anyhow!("Current position lock error: {}", e))?;
            *current_pos = position;
        }


        {
            let mut stats = self.nav_stats.write().map_err(|e| anyhow!("Navigation stats lock error: {}", e))?;
            stats.cache_misses += 1;
            stats.total_navigations += 1;
        }

        info!("Navigated to position: {:?}", position);

        Ok(frame)
    }


    pub fn start_navigation(&self, direction: TimelineDirection, speed: f64) -> Result<()> {
        debug!("Starting navigation: {:?} at {:.1} FPS", direction, speed);

        {
            let mut is_navigating = self.is_navigating.write().map_err(|e| anyhow!("Navigation lock error: {}", e))?;
            let mut nav_direction = self.navigation_direction.write().map_err(|e| anyhow!("Direction lock error: {}", e))?;
            let mut nav_speed = self.navigation_speed.write().map_err(|e| anyhow!("Speed lock error: {}", e))?;

            *is_navigating = true;
            *nav_direction = direction;
            *nav_speed = speed.clamp(1.0, 120.0);
        }


        self.start_navigation_task()?;

        info!("Started navigation: {:?} at {:.1} FPS", direction, speed);

        Ok(())
    }


    pub fn stop_navigation(&self) -> Result<()> {
        debug!("Stopping navigation");

        {
            let mut is_navigating = self.is_navigating.write().map_err(|e| anyhow!("Navigation lock error: {}", e))?;
            *is_navigating = false;
        }

        info!("Stopped navigation");

        Ok(())
    }


    pub async fn navigate_forward(&self, frames: u64) -> Result<PreviewFrame> {
        let current = self.current_position().map_err(|e| anyhow!("Failed to get current position: {}", e))?;
        let target = current.add_frames(frames as i64);

        self.add_navigation_entry(target, NavigationAction::StepForward).await?;

        self.navigate_to(target).await
    }


    pub async fn navigate_backward(&self, frames: u64) -> Result<PreviewFrame> {
        let current = self.current_position().map_err(|e| anyhow!("Failed to get current position: {}", e))?;
        let target = current.add_frames(-(frames as i64));

        self.add_navigation_entry(target, NavigationAction::StepBackward).await?;

        self.navigate_to(target).await
    }


    pub async fn jump_to_next_keyframe(&self) -> Result<Option<PreviewFrame>> {
        debug!("Jumping to next keyframe");


        let keyframe_interval = 30;
        let result = self.navigate_forward(keyframe_interval).await;

        match result {
            Ok(frame) => {
                self.add_navigation_entry(self.current_position().unwrap(), NavigationAction::JumpToKeyframe).await?;
                Ok(Some(frame))
            }
            Err(e) => {
                warn!("Failed to jump to next keyframe: {}", e);
                Ok(None)
            }
        }
    }


    pub async fn jump_to_previous_keyframe(&self) -> Result<Option<PreviewFrame>> {
        debug!("Jumping to previous keyframe");

        let keyframe_interval = 30;
        let result = self.navigate_backward(keyframe_interval).await;

        match result {
            Ok(frame) => {
                self.add_navigation_entry(self.current_position().unwrap(), NavigationAction::JumpToKeyframe).await?;
                Ok(Some(frame))
            }
            Err(e) => {
                warn!("Failed to jump to previous keyframe: {}", e);
                Ok(None)
            }
        }
    }


    pub fn add_bookmark(&self, name: String, position: Option<TimelinePosition>) -> Result<()> {
        let pos = position.unwrap_or_else(|| self.current_position().unwrap());

        debug!("Adding bookmark '{}' at position: {:?}", name, pos);

        {
            let mut bookmarks = self.bookmarks.write().map_err(|e| anyhow!("Bookmarks lock error: {}", e))?;
            bookmarks.insert(name.clone(), pos);
        }


        {
            let mut stats = self.nav_stats.write().map_err(|e| anyhow!("Navigation stats lock error: {}", e))?;
            stats.bookmarks_added += 1;
        }

        info!("Added bookmark '{}' at position: {:?}", name, pos);

        Ok(())
    }


    pub fn remove_bookmark(&self, name: &str) -> Result<bool> {
        debug!("Removing bookmark: {}", name);

        let removed = {
            let mut bookmarks = self.bookmarks.write().map_err(|e| anyhow!("Bookmarks lock error: {}", e))?;
            bookmarks.remove(name).is_some()
        };

        if removed {

            let mut stats = self.nav_stats.write().map_err(|e| anyhow!("Navigation stats lock error: {}", e))?;
            stats.bookmarks_removed += 1;

            info!("Removed bookmark: {}", name);
        }

        Ok(removed)
    }


    pub async fn navigate_to_bookmark(&self, name: &str) -> Result<Option<PreviewFrame>> {
        debug!("Navigating to bookmark: {}", name);

        let position = {
            let bookmarks = self.bookmarks.read().map_err(|e| anyhow!("Bookmarks lock error: {}", e))?;
            bookmarks.get(name).copied()
        };

        match position {
            Some(pos) => {
                self.add_navigation_entry(pos, NavigationAction::JumpToBookmark).await?;
                Ok(Some(self.navigate_to(pos).await?))
            }
            None => {
                warn!("Bookmark not found: {}", name);
                Ok(None)
            }
        }
    }


    pub fn get_bookmarks(&self) -> Result<HashMap<String, TimelinePosition>> {
        let bookmarks = self.bookmarks.read().map_err(|e| anyhow!("Bookmarks lock error: {}", e))?;
        Ok(bookmarks.clone())
    }


    pub fn current_position(&self) -> Result<TimelinePosition> {
        let position = self.current_position.read().map_err(|e| anyhow!("Current position lock error: {}", e))?;
        Ok(*position)
    }


    pub fn get_stats(&self) -> Result<NavigationStats> {
        let stats = self.nav_stats.read().map_err(|e| anyhow!("Navigation stats lock error: {}", e))?;
        let cache_stats = self.frame_cache.get_stats()?;
        let buffer_stats = self.pre_roll_buffer.get_stats()?;

        let mut current_stats = stats.clone();
        current_stats.cache_stats = Some(cache_stats);
        current_stats.buffer_stats = Some(buffer_stats);

        Ok(current_stats)
    }


    pub fn get_navigation_history(&self) -> Result<Vec<NavigationEntry>> {
        let history = self.navigation_history.read().map_err(|e| anyhow!("Navigation history lock error: {}", e))?;
        Ok(history.clone())
    }


    pub fn clear_history(&self) -> Result<()> {
        debug!("Clearing navigation history");

        {
            let mut history = self.navigation_history.write().map_err(|e| anyhow!("Navigation history lock error: {}", e))?;
            history.clear();
        }


        {
            let mut stats = self.nav_stats.write().map_err(|e| anyhow!("Navigation stats lock error: {}", e))?;
            stats.history_cleared += 1;
        }

        info!("Navigation history cleared");

        Ok(())
    }


    async fn add_navigation_entry(&self, position: TimelinePosition, action: NavigationAction) -> Result<()> {
        let entry = NavigationEntry {
            position,
            action,
            timestamp: Instant::now(),
            previous_position: self.current_position().ok(),
        };

        {
            let mut history = self.navigation_history.write().map_err(|e| anyhow!("Navigation history lock error: {}", e))?;

            history.push(entry);


            if history.len() > self.config.max_history_size {
                history.remove(0);
            }
        }

        Ok(())
    }


    fn start_navigation_task(&self) -> Result<()> {
        let is_navigating = self.is_navigating.clone();
        let current_position = self.current_position.clone();
        let navigation_direction = self.navigation_direction.clone();
        let navigation_speed = self.navigation_speed.clone();
        let frame_cache = self.frame_cache.clone();
        let pre_roll_buffer = self.pre_roll_buffer.clone();
        let timeline_range = self.timeline_range.clone();
        let nav_stats = self.nav_stats.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut last_frame_time = Instant::now();
            let target_frame_interval = Duration::from_millis(33);

            loop {

                {
                    let navigating = is_navigating.read().unwrap();
                    if !*navigating {
                        break;
                    }
                }


                let now = Instant::now();
                let elapsed = now - last_frame_time;

                if elapsed >= target_frame_interval {

                    let (mut position, direction, speed) = {
                        let pos = current_position.read().unwrap();
                        let dir = navigation_direction.read().unwrap();
                        let spd = navigation_speed.read().unwrap();
                        (*pos, *dir, *spd)
                    };


                    let frame_delta = (speed * elapsed.as_secs_f64()) as i64;
                    let next_position = if direction.is_forward() {
                        position.add_frames(frame_delta)
                    } else {
                        position.add_frames(-frame_delta)
                    };


                    let within_range = {
                        let range = timeline_range.read().unwrap();
                        range.as_ref().map_or(true, |r| r.contains(&next_position))
                    };

                    if within_range {

                        let frame = if let Some(cached_frame) = frame_cache.get_frame(next_position).await.unwrap() {
                            cached_frame
                        } else if let Some(buffered_frame) = pre_roll_buffer.get_frame(next_position).await.unwrap() {
                            buffered_frame
                        } else {

                            match generate_frame(&next_position).await {
                                Ok(frame) => {

                                    let _ = frame_cache.cache_frame(next_position, frame.clone()).await;
                                    frame
                                }
                                Err(e) => {
                                    error!("Failed to generate frame during navigation: {}", e);
                                    continue;
                                }
                            }
                        };


                        {
                            let mut pos = current_position.write().unwrap();
                            *pos = next_position;
                        }


                        {
                            let mut stats = nav_stats.write().unwrap();
                            stats.continuous_frames += 1;
                            stats.total_navigation_time += elapsed;

                            if frame.render_time > config.max_frame_time {
                                stats.slow_frames += 1;
                            }
                        }
                    } else {

                        let mut navigating = is_navigating.write().unwrap();
                        *navigating = false;
                        break;
                    }

                    last_frame_time = now;
                }


                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        Ok(())
    }


    async fn generate_frame_at_position(&self, position: TimelinePosition) -> Result<PreviewFrame> {
        generate_frame(&position).await
    }
}


async fn generate_frame(position: &TimelinePosition) -> Result<PreviewFrame> {


    Ok(PreviewFrame::new(
        uuid::Uuid::new_v4(),
        position.time(),
        PreviewQuality::Medium,
        crate::gpu::FrameBufferHandle {
            id: uuid::Uuid::new_v4(),
            frame_buffer: std::sync::Arc::new(crate::gpu::FrameBuffer {
                id: uuid::Uuid::new_v4(),
                width: 1920,
                height: 1080,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                texture: std::sync::Arc::new(crate::gpu::TextureHandle { id: uuid::Uuid::new_v4() }),
                view: std::sync::Arc::new(crate::gpu::TextureViewHandle { id: uuid::Uuid::new_v4() }),
                created_at: std::time::Instant::now(),
                last_used: std::sync::Mutex::new(std::time::Instant::now()),
                access_count: std::sync::atomic::AtomicU64::new(0),
            }),
        },
        std::time::Duration::from_millis(16),
    ))
}


#[derive(Debug, Clone)]
pub struct NavigationConfig {
    pub pre_roll_size: usize,
    pub max_history_size: usize,
    pub max_frame_time: Duration,
    pub auto_pre_roll: bool,
    pub smooth_navigation: bool,
}

impl Default for NavigationConfig {
    fn default() -> Self {
        Self {
            pre_roll_size: 30,
            max_history_size: 1000,
            max_frame_time: Duration::from_millis(50),
            auto_pre_roll: true,
            smooth_navigation: true,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    Jump,
    StepForward,
    StepBackward,
    JumpToKeyframe,
    JumpToBookmark,
    Continuous,
}


#[derive(Debug, Clone)]
pub struct NavigationEntry {
    pub position: TimelinePosition,
    pub action: NavigationAction,
    pub timestamp: Instant,
    pub previous_position: Option<TimelinePosition>,
}


#[derive(Debug, Clone)]
pub struct NavigationStats {
    pub total_navigations: u64,
    pub cache_hits: u64,
    pub buffer_hits: u64,
    pub cache_misses: u64,
    pub continuous_frames: u64,
    pub total_navigation_time: Duration,
    pub slow_frames: u64,
    pub bookmarks_added: u64,
    pub bookmarks_removed: u64,
    pub history_cleared: u64,
    pub cache_stats: Option<super::CacheStats>,
    pub buffer_stats: Option<super::BufferStats>,
}

impl NavigationStats {

    pub fn new() -> Self {
        Self {
            total_navigations: 0,
            cache_hits: 0,
            buffer_hits: 0,
            cache_misses: 0,
            continuous_frames: 0,
            total_navigation_time: Duration::ZERO,
            slow_frames: 0,
            bookmarks_added: 0,
            bookmarks_removed: 0,
            history_cleared: 0,
            cache_stats: None,
            buffer_stats: None,
        }
    }


    pub fn average_fps(&self) -> f64 {
        if self.total_navigation_time.as_secs_f64() > 0.0 {
            self.continuous_frames as f64 / self.total_navigation_time.as_secs_f64()
        } else {
            0.0
        }
    }


    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests > 0 {
            (self.cache_hits + self.buffer_hits) as f64 / total_requests as f64
        } else {
            0.0
        }
    }


    pub fn slow_frame_rate(&self) -> f64 {
        if self.continuous_frames > 0 {
            self.slow_frames as f64 / self.continuous_frames as f64
        } else {
            0.0
        }
    }


    pub fn format(&self) -> String {
        format!(
            "Navigations: {} | FPS: {:.1} | Cache Hit Rate: {:.1}% | Slow Frames: {:.1}% | Bookmarks: {}",
            self.total_navigations,
            self.average_fps(),
            self.cache_hit_rate() * 100.0,
            self.slow_frame_rate() * 100.0,
            self.bookmarks_added.saturating_sub(self.bookmarks_removed)
        )
    }
}

impl Default for NavigationStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeline_navigator_basic() {
        let config = NavigationConfig::default();
        let cache_config = super::cache::CacheConfig::default();
        let buffer_config = super::buffer::BufferConfig::default();

        let frame_cache = Arc::new(super::cache::FrameCache::new(cache_config));
        let pre_roll_buffer = Arc::new(super::buffer::PreRollBuffer::new(buffer_config));

        let navigator = TimelineNavigator::new(config, frame_cache, pre_roll_buffer).unwrap();

        let range = TimelineRange::new(
            TimelinePosition::from_frame(0),
            TimelinePosition::from_frame(100),
        );

        navigator.set_timeline_range(range).unwrap();

        let current = navigator.current_position().unwrap();
        assert_eq!(current.frame(), 0);

        let frame = navigator.navigate_forward(10).await.unwrap();
        assert!(frame.session_id != uuid::Uuid::nil());

        let current = navigator.current_position().unwrap();
        assert_eq!(current.frame(), 10);
    }

    #[test]
    fn test_navigation_config() {
        let config = NavigationConfig::default();
        assert_eq!(config.pre_roll_size, 30);
        assert_eq!(config.max_history_size, 1000);
        assert!(config.auto_pre_roll);
        assert!(config.smooth_navigation);
    }

    #[test]
    fn test_navigation_entry() {
        let position = TimelinePosition::from_frame(100);
        let entry = NavigationEntry {
            position,
            action: NavigationAction::Jump,
            timestamp: std::time::Instant::now(),
            previous_position: Some(TimelinePosition::from_frame(90)),
        };

        assert_eq!(entry.position.frame(), 100);
        assert_eq!(entry.action, NavigationAction::Jump);
        assert_eq!(entry.previous_position.unwrap().frame(), 90);
    }

    #[test]
    fn test_navigation_stats() {
        let mut stats = NavigationStats::new();
        stats.total_navigations = 100;
        stats.cache_hits = 70;
        stats.buffer_hits = 20;
        stats.cache_misses = 10;
        stats.continuous_frames = 300;
        stats.total_navigation_time = Duration::from_secs(10);

        assert_eq!(stats.average_fps(), 30.0);
        assert_eq!(stats.cache_hit_rate(), 0.9);
        assert_eq!(stats.slow_frame_rate(), 0.0);

        let formatted = stats.format();
        assert!(formatted.contains("Navigations: 100"));
        assert!(formatted.contains("FPS: 30.0"));
        assert!(formatted.contains("Cache Hit Rate: 90.0%"));
    }
}
