use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use log::{debug, info, warn};

use crate::preview::PreviewFrame;

use super::TimelinePosition;


pub struct PreRollBuffer {
    config: BufferConfig,
    buffer: Arc<RwLock<VecDeque<BufferedFrame>>>,
    target_position: Arc<RwLock<TimelinePosition>>,
    buffer_stats: Arc<RwLock<BufferStats>>,
}

impl PreRollBuffer {

    pub fn new(config: BufferConfig) -> Self {
        info!("Creating pre-roll buffer with config: {:?}", config);

        Self {
            config,
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            target_position: Arc::new(RwLock::new(TimelinePosition::from_frame(0))),
            buffer_stats: Arc::new(RwLock::new(BufferStats::new())),
        }
    }


    pub fn initialize(&self, start_position: TimelinePosition, buffer_size: usize) -> Result<()> {
        debug!("Initializing pre-roll buffer at position: {:?} with size: {}", start_position, buffer_size);


        {
            let mut buffer = self.buffer.write().map_err(|e| anyhow!("Buffer lock error: {}", e))?;
            buffer.clear();
        }


        {
            let mut target = self.target_position.write().map_err(|e| anyhow!("Target position lock error: {}", e))?;
            *target = start_position;
        }


        {
            let mut stats = self.buffer_stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.initializations += 1;
            stats.buffer_size = buffer_size;
        }

        info!("Pre-roll buffer initialized at position: {:?}", start_position);

        Ok(())
    }


    pub async fn add_frame(&self, position: TimelinePosition, frame: PreviewFrame) -> Result<()> {
        let buffered_frame = BufferedFrame {
            frame,
            position,
            added_at: Instant::now(),
            access_count: 0,
        };

        {
            let mut buffer = self.buffer.write().map_err(|e| anyhow!("Buffer lock error: {}", e))?;


            if buffer.len() >= self.config.max_size {
                buffer.pop_front();
            }

            buffer.push_back(buffered_frame);
        }


        {
            let mut stats = self.buffer_stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.frames_added += 1;
            stats.total_frames = stats.total_frames.saturating_add(1);
        }

        debug!("Added frame to pre-roll buffer at position: {:?}", position);

        Ok(())
    }


    pub async fn get_frame(&self, position: TimelinePosition) -> Result<Option<PreviewFrame>> {
        let buffer = self.buffer.read().map_err(|e| anyhow!("Buffer lock error: {}", e))?;


        for buffered_frame in buffer.iter() {
            let frame_diff = (buffered_frame.position.frame as i64 - position.frame as i64).abs();

            if frame_diff <= self.config.position_tolerance as i64 {

                {
                    let mut stats = self.buffer_stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
                    stats.frames_accessed += 1;
                    stats.cache_hits += 1;
                }

                debug!("Found frame in pre-roll buffer for position: {:?} (diff: {})", position, frame_diff);

                return Ok(Some(buffered_frame.frame.clone()));
            }
        }


        {
            let mut stats = self.buffer_stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.cache_misses += 1;
        }

        debug!("Frame not found in pre-roll buffer for position: {:?}", position);

        Ok(None)
    }


    pub async fn pre_fill(&self, frame_generator: impl Fn(TimelinePosition) -> Result<PreviewFrame>) -> Result<usize> {
        let target = {
            let target_pos = self.target_position.read().map_err(|e| anyhow!("Target position lock error: {}", e))?;
            *target_pos
        };

        debug!("Pre-filling pre-roll buffer around position: {:?}", target);

        let mut pre_filled = 0;
        let pre_roll_size = self.config.pre_roll_size.min(self.config.max_size);


        for i in 1..=pre_roll_size {
            let position = target.add_frames(-(i as i64));

            match frame_generator(position) {
                Ok(frame) => {
                    self.add_frame(position, frame).await?;
                    pre_filled += 1;
                }
                Err(e) => {
                    warn!("Failed to generate frame for pre-filling: {}", e);
                    break;
                }
            }
        }


        match frame_generator(target) {
            Ok(frame) => {
                self.add_frame(target, frame).await?;
                pre_filled += 1;
            }
            Err(e) => {
                warn!("Failed to generate target frame for pre-filling: {}", e);
            }
        }


        for i in 1..=pre_roll_size {
            let position = target.add_frames(i as i64);

            match frame_generator(position) {
                Ok(frame) => {
                    self.add_frame(position, frame).await?;
                    pre_filled += 1;
                }
                Err(e) => {
                    warn!("Failed to generate frame for pre-filling: {}", e);
                    break;
                }
            }
        }

        info!("Pre-filled pre-roll buffer with {} frames", pre_filled);

        Ok(pre_filled)
    }


    pub fn update_target(&self, new_target: TimelinePosition) -> Result<()> {
        debug!("Updating pre-roll buffer target to: {:?}", new_target);

        {
            let mut target = self.target_position.write().map_err(|e| anyhow!("Target position lock error: {}", e))?;
            *target = new_target;
        }


        {
            let mut stats = self.buffer_stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.target_updates += 1;
        }

        Ok(())
    }


    pub fn target_position(&self) -> Result<TimelinePosition> {
        let target = self.target_position.read().map_err(|e| anyhow!("Target position lock error: {}", e))?;
        Ok(*target)
    }


    pub async fn is_ready_for(&self, position: TimelinePosition) -> Result<bool> {
        let buffer = self.buffer.read().map_err(|e| anyhow!("Buffer lock error: {}", e))?;

        for buffered_frame in buffer.iter() {
            let frame_diff = (buffered_frame.position.frame as i64 - position.frame as i64).abs();

            if frame_diff <= self.config.position_tolerance as i64 {
                return Ok(true);
            }
        }

        Ok(false)
    }


    pub fn get_stats(&self) -> Result<BufferStats> {
        let stats = self.buffer_stats.read().map_err(|e| anyhow!("Stats lock error: {}", e))?;
        let buffer = self.buffer.read().map_err(|e| anyhow!("Buffer lock error: {}", e))?;

        let mut current_stats = stats.clone();
        current_stats.current_size = buffer.len();
        current_stats.hit_rate = if stats.cache_hits + stats.cache_misses > 0 {
            stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64
        } else {
            0.0
        };

        Ok(current_stats)
    }


    pub async fn clear(&self) -> Result<()> {
        debug!("Clearing pre-roll buffer");

        let buffer_size = {
            let buffer = self.buffer.read().map_err(|e| anyhow!("Buffer lock error: {}", e))?;
            buffer.len()
        };

        {
            let mut buffer = self.buffer.write().map_err(|e| anyhow!("Buffer lock error: {}", e))?;
            buffer.clear();
        }


        {
            let mut stats = self.buffer_stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.clears += 1;
        }

        info!("Cleared pre-roll buffer (removed {} frames)", buffer_size);

        Ok(())
    }


    pub fn size(&self) -> Result<usize> {
        let buffer = self.buffer.read().map_err(|e| anyhow!("Buffer lock error: {}", e))?;
        Ok(buffer.len())
    }


    pub fn is_empty(&self) -> Result<bool> {
        let buffer = self.buffer.read().map_err(|e| anyhow!("Buffer lock error: {}", e))?;
        Ok(buffer.is_empty())
    }


    pub fn get_buffered_positions(&self) -> Result<Vec<TimelinePosition>> {
        let buffer = self.buffer.read().map_err(|e| anyhow!("Buffer lock error: {}", e))?;
        Ok(buffer.iter().map(|bf| bf.position).collect())
    }
}


#[derive(Debug, Clone)]
struct BufferedFrame {
    frame: PreviewFrame,
    position: TimelinePosition,
    added_at: Instant,
    access_count: u64,
}


#[derive(Debug, Clone)]
pub struct BufferConfig {
    pub max_size: usize,
    pub pre_roll_size: usize,
    pub position_tolerance: u64,
    pub auto_refill: bool,
    pub refill_threshold: f64,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            max_size: 60,
            pre_roll_size: 30,
            position_tolerance: 2,
            auto_refill: true,
            refill_threshold: 0.5,
        }
    }
}


#[derive(Debug, Clone)]
pub struct BufferStats {
    pub frames_added: u64,
    pub frames_accessed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_frames: u64,
    pub current_size: usize,
    pub hit_rate: f64,
    pub initializations: u64,
    pub target_updates: u64,
    pub clears: u64,
    pub buffer_size: usize,
}

impl BufferStats {

    pub fn new() -> Self {
        Self {
            frames_added: 0,
            frames_accessed: 0,
            cache_hits: 0,
            cache_misses: 0,
            total_frames: 0,
            current_size: 0,
            hit_rate: 0.0,
            initializations: 0,
            target_updates: 0,
            clears: 0,
            buffer_size: 0,
        }
    }


    pub fn format(&self) -> String {
        format!(
            "Size: {}/{} | Hits: {} | Misses: {} | Hit Rate: {:.1}% | Total: {}",
            self.current_size,
            self.buffer_size,
            self.cache_hits,
            self.cache_misses,
            self.hit_rate * 100.0,
            self.total_frames
        )
    }
}

impl Default for BufferStats {
    fn default() -> Self {
        Self::new()
    }
}


pub struct BufferRefillManager {
    buffer: Arc<PreRollBuffer>,
}

impl BufferRefillManager {

    pub fn new(buffer: Arc<PreRollBuffer>) -> Self {
        Self { buffer }
    }


    pub async fn start_auto_refill(&self, frame_generator: impl Fn(TimelinePosition) -> Result<PreviewFrame> + Send + Sync + 'static) -> Result<()> {
        let buffer = self.buffer.clone();
        let config = buffer.config.clone();

        tokio::spawn(async move {
            let mut last_refill = Instant::now();
            let refill_interval = Duration::from_secs(1);

            loop {
                tokio::time::sleep(refill_interval).await;


                if !config.auto_refill {
                    continue;
                }

                let stats = buffer.get_stats().unwrap();
                let fill_ratio = stats.current_size as f64 / config.max_size as f64;

                if fill_ratio < config.refill_threshold {
                    debug!("Auto-refilling pre-roll buffer (fill ratio: {:.2})", fill_ratio);

                    match buffer.pre_fill(&frame_generator).await {
                        Ok(count) => {
                            debug!("Auto-refilled {} frames", count);
                        }
                        Err(e) => {
                            warn!("Auto-refill failed: {}", e);
                        }
                    }

                    last_refill = Instant::now();
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview::PreviewQuality;

    fn create_test_frame(position: TimelinePosition) -> PreviewFrame {
        PreviewFrame::new(
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
        )
    }

    #[tokio::test]
    async fn test_pre_roll_buffer_basic() {
        let config = BufferConfig::default();
        let buffer = PreRollBuffer::new(config);

        let position = TimelinePosition::from_frame(100);
        buffer.initialize(position, 30).unwrap();

        let frame = create_test_frame(position);
        buffer.add_frame(position, frame).await.unwrap();

        let cached = buffer.get_frame(position).await.unwrap();
        assert!(cached.is_some());

        let stats = buffer.get_stats().unwrap();
        assert_eq!(stats.frames_added, 1);
        assert_eq!(stats.cache_hits, 1);
    }

    #[tokio::test]
    async fn test_pre_roll_buffer_tolerance() {
        let config = BufferConfig {
            position_tolerance: 5,
            ..Default::default()
        };
        let buffer = PreRollBuffer::new(config);

        let position = TimelinePosition::from_frame(100);
        buffer.initialize(position, 30).unwrap();


        let frame = create_test_frame(position);
        buffer.add_frame(position, frame).await.unwrap();


        let nearby_pos = TimelinePosition::from_frame(102);
        let cached = buffer.get_frame(nearby_pos).await.unwrap();
        assert!(cached.is_some());


        let far_pos = TimelinePosition::from_frame(110);
        let cached = buffer.get_frame(far_pos).await.unwrap();
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_pre_roll_buffer_pre_fill() {
        let config = BufferConfig {
            max_size: 10,
            pre_roll_size: 3,
            ..Default::default()
        };
        let buffer = PreRollBuffer::new(config);

        let center = TimelinePosition::from_frame(100);
        buffer.initialize(center, 6).unwrap();

        let pre_filled = buffer.pre_fill(|pos| Ok(create_test_frame(pos))).await.unwrap();

        assert!(pre_filled > 0);
        assert!(pre_filled <= 7);


        let cached = buffer.get_frame(center).await.unwrap();
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_pre_roll_buffer_size_limit() {
        let config = BufferConfig {
            max_size: 3,
            ..Default::default()
        };
        let buffer = PreRollBuffer::new(config);

        let position = TimelinePosition::from_frame(100);
        buffer.initialize(position, 3).unwrap();


        for i in 0..5 {
            let frame_pos = TimelinePosition::from_frame(100 + i);
            let frame = create_test_frame(frame_pos);
            buffer.add_frame(frame_pos, frame).await.unwrap();
        }

        let stats = buffer.get_stats().unwrap();
        assert_eq!(stats.current_size, 3);
        assert_eq!(stats.frames_added, 5);
    }

    #[test]
    fn test_buffer_config() {
        let config = BufferConfig::default();
        assert_eq!(config.max_size, 60);
        assert_eq!(config.pre_roll_size, 30);
        assert_eq!(config.position_tolerance, 2);
        assert!(config.auto_refill);
        assert_eq!(config.refill_threshold, 0.5);
    }

    #[test]
    fn test_buffer_stats() {
        let mut stats = BufferStats::new();
        stats.frames_added = 50;
        stats.cache_hits = 40;
        stats.cache_misses = 10;
        stats.current_size = 25;
        stats.buffer_size = 60;

        assert_eq!(stats.hit_rate, 0.8);

        let formatted = stats.format();
        assert!(formatted.contains("Size: 25/60"));
        assert!(formatted.contains("Hits: 40"));
        assert!(formatted.contains("Hit Rate: 80.0%"));
    }
}
