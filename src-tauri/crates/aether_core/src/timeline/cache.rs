use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use log::{debug, info, warn};

use crate::preview::PreviewFrame;

use super::TimelinePosition;


pub struct FrameCache {
    config: CacheConfig,
    cache: Arc<RwLock<HashMap<TimelinePosition, CachedFrame>>>,
    access_order: Arc<RwLock<VecDeque<TimelinePosition>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl FrameCache {

    pub fn new(config: CacheConfig) -> Self {
        info!("Creating frame cache with config: {:?}", config);

        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(RwLock::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(CacheStats::new())),
        }
    }


    pub fn initialize_range(&self, range: super::TimelineRange) -> Result<()> {
        debug!("Initializing frame cache with range: {:?}", range);


        let estimated_size = (range.duration_frames() as usize).min(self.config.max_size);

        {
            let mut cache = self.cache.write().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            cache.reserve(estimated_size);
        }

        info!("Frame cache initialized for range with estimated size: {}", estimated_size);

        Ok(())
    }


    pub async fn get_frame(&self, position: TimelinePosition) -> Result<Option<PreviewFrame>> {
        let cache = self.cache.read().map_err(|e| anyhow!("Cache lock error: {}", e))?;

        if let Some(cached_frame) = cache.get(&position) {

            {
                let mut access_order = self.access_order.write().map_err(|e| anyhow!("Access order lock error: {}", e))?;


                access_order.retain(|&pos| pos != position);


                access_order.push_front(position);
            }


            {
                let mut stats = self.stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
                stats.hits += 1;
                stats.last_access_time = Instant::now();
            }

            debug!("Cache hit for position: {:?}", position);

            return Ok(Some(cached_frame.frame.clone()));
        }


        {
            let mut stats = self.stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.misses += 1;
        }

        debug!("Cache miss for position: {:?}", position);

        Ok(None)
    }


    pub async fn cache_frame(&self, position: TimelinePosition, frame: PreviewFrame) -> Result<()> {
        let cache_size = {
            let cache = self.cache.read().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            cache.len()
        };


        if cache_size >= self.config.max_size {
            self.evict_oldest_frames(1).await?;
        }

        let cached_frame = CachedFrame {
            frame,
            cached_at: Instant::now(),
            access_count: 1,
            last_accessed: Instant::now(),
        };


        {
            let mut cache = self.cache.write().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            cache.insert(position, cached_frame);
        }


        {
            let mut access_order = self.access_order.write().map_err(|e| anyhow!("Access order lock error: {}", e))?;
            access_order.push_front(position);
        }


        {
            let mut stats = self.stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.frames_cached += 1;
            stats.total_size = cache_size + 1;
        }

        debug!("Cached frame for position: {:?}", position);

        Ok(())
    }


    pub async fn preload_range(&self, range: super::TimelineRange, frame_generator: impl Fn(TimelinePosition) -> Result<PreviewFrame>) -> Result<usize> {
        debug!("Preloading frames for range: {:?}", range);

        let mut preloaded = 0;
        let step = (range.duration_frames() / self.config.preload_batch_size as u64).max(1);

        let mut current_pos = range.start;
        while current_pos.frame <= range.end.frame && preloaded < self.config.preload_batch_size {

            if self.get_frame(current_pos).await?.is_none() {

                match frame_generator(current_pos) {
                    Ok(frame) => {
                        self.cache_frame(current_pos, frame).await?;
                        preloaded += 1;
                    }
                    Err(e) => {
                        warn!("Failed to generate frame for preloading: {}", e);
                        break;
                    }
                }
            }

            current_pos = current_pos.add_frames(step);
        }

        info!("Preloaded {} frames for range", preloaded);

        Ok(preloaded)
    }


    async fn evict_oldest_frames(&self, count: usize) -> Result<usize> {
        debug!("Evicting {} oldest frames from cache", count);

        let mut evicted = 0;
        let positions_to_evict: Vec<TimelinePosition> = {
            let mut access_order = self.access_order.write().map_err(|e| anyhow!("Access order lock error: {}", e))?;


            let positions: Vec<TimelinePosition> = access_order.iter().rev().take(count).copied().collect();


            access_order.retain(|pos| !positions.contains(pos));

            positions
        };


        {
            let mut cache = self.cache.write().map_err(|e| anyhow!("Cache lock error: {}", e))?;

            for position in &positions_to_evict {
                if cache.remove(position).is_some() {
                    evicted += 1;
                }
            }
        }


        {
            let mut stats = self.stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.evictions += evicted as u64;
            stats.total_size = stats.total_size.saturating_sub(evicted);
        }

        debug!("Evicted {} frames from cache", evicted);

        Ok(evicted)
    }


    pub async fn clear(&self) -> Result<()> {
        debug!("Clearing frame cache");

        let cache_size = {
            let cache = self.cache.read().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            cache.len()
        };

        {
            let mut cache = self.cache.write().map_err(|e| anyhow!("Cache lock error: {}", e))?;
            cache.clear();
        }

        {
            let mut access_order = self.access_order.write().map_err(|e| anyhow!("Access order lock error: {}", e))?;
            access_order.clear();
        }


        {
            let mut stats = self.stats.write().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            *stats = CacheStats::new();
        }

        info!("Cleared frame cache (removed {} frames)", cache_size);

        Ok(())
    }


    pub fn get_stats(&self) -> Result<CacheStats> {
        let stats = self.stats.read().map_err(|e| anyhow!("Stats lock error: {}", e))?;
        let cache = self.cache.read().map_err(|e| anyhow!("Cache lock error: {}", e))?;

        let mut current_stats = stats.clone();
        current_stats.total_size = cache.len();
        current_stats.hit_rate = if stats.hits + stats.misses > 0 {
            stats.hits as f64 / (stats.hits + stats.misses) as f64
        } else {
            0.0
        };

        Ok(current_stats)
    }


    pub fn size(&self) -> Result<usize> {
        let cache = self.cache.read().map_err(|e| anyhow!("Cache lock error: {}", e))?;
        Ok(cache.len())
    }


    pub fn is_full(&self) -> Result<bool> {
        let cache = self.cache.read().map_err(|e| anyhow!("Cache lock error: {}", e))?;
        Ok(cache.len() >= self.config.max_size)
    }


    pub fn get_access_order(&self) -> Result<Vec<TimelinePosition>> {
        let access_order = self.access_order.read().map_err(|e| anyhow!("Access order lock error: {}", e))?;
        Ok(access_order.iter().copied().collect())
    }
}


#[derive(Debug, Clone)]
struct CachedFrame {
    frame: PreviewFrame,
    cached_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}


#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size: usize,
    pub preload_batch_size: usize,
    pub eviction_policy: EvictionPolicy,
    pub ttl: Option<Duration>,
    pub compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            preload_batch_size: 50,
            eviction_policy: EvictionPolicy::LRU,
            ttl: Some(Duration::from_secs(300)),
            compression: false,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    FIFO,
    TTL,
}


#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub frames_cached: u64,
    pub evictions: u64,
    pub total_size: usize,
    pub hit_rate: f64,
    pub last_access_time: Instant,
    pub average_access_time: Duration,
}

impl CacheStats {

    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            frames_cached: 0,
            evictions: 0,
            total_size: 0,
            hit_rate: 0.0,
            last_access_time: Instant::now(),
            average_access_time: Duration::ZERO,
        }
    }


    pub fn format(&self) -> String {
        format!(
            "Size: {}/{} | Hits: {} | Misses: {} | Hit Rate: {:.1}% | Evictions: {}",
            self.total_size,
            1000,
            self.hits,
            self.misses,
            self.hit_rate * 100.0,
            self.evictions
        )
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}


pub struct CacheWarmer {
    cache: Arc<FrameCache>,
}

impl CacheWarmer {

    pub fn new(cache: Arc<FrameCache>) -> Self {
        Self { cache }
    }


    pub async fn warm_frequent_positions(
        &self,
        positions: Vec<TimelinePosition>,
        frame_generator: impl Fn(TimelinePosition) -> Result<PreviewFrame>,
    ) -> Result<usize> {
        debug!("Warming cache with {} frequent positions", positions.len());

        let mut warmed = 0;

        for position in positions {

            if self.cache.get_frame(position).await?.is_none() {

                match frame_generator(position) {
                    Ok(frame) => {
                        self.cache.cache_frame(position, frame).await?;
                        warmed += 1;
                    }
                    Err(e) => {
                        warn!("Failed to generate frame for warming: {}", e);
                    }
                }
            }
        }

        info!("Warmed cache with {} frames", warmed);

        Ok(warmed)
    }


    pub async fn warm_around_position(
        &self,
        center: TimelinePosition,
        radius: u64,
        frame_generator: impl Fn(TimelinePosition) -> Result<PreviewFrame>,
    ) -> Result<usize> {
        debug!("Warming cache around position: {:?} with radius: {}", center, radius);

        let mut positions = Vec::new();


        for i in 1..=radius {
            positions.push(center.add_frames(-(i as i64)));
        }


        positions.push(center);


        for i in 1..=radius {
            positions.push(center.add_frames(i as i64));
        }

        self.warm_frequent_positions(positions, frame_generator).await
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
    async fn test_frame_cache_basic() {
        let config = CacheConfig::default();
        let cache = FrameCache::new(config);

        let position = TimelinePosition::from_frame(100);
        let frame = create_test_frame(position);


        cache.cache_frame(position, frame.clone()).await.unwrap();


        let cached = cache.get_frame(position).await.unwrap();
        assert!(cached.is_some());


        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.frames_cached, 1);
        assert_eq!(stats.total_size, 1);
    }

    #[tokio::test]
    async fn test_frame_cache_miss() {
        let config = CacheConfig::default();
        let cache = FrameCache::new(config);

        let position = TimelinePosition::from_frame(100);


        let cached = cache.get_frame(position).await.unwrap();
        assert!(cached.is_none());


        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_size, 0);
    }

    #[tokio::test]
    async fn test_frame_cache_eviction() {
        let config = CacheConfig {
            max_size: 2,
            ..Default::default()
        };
        let cache = FrameCache::new(config);


        let pos1 = TimelinePosition::from_frame(100);
        let pos2 = TimelinePosition::from_frame(101);
        let pos3 = TimelinePosition::from_frame(102);

        let frame1 = create_test_frame(pos1);
        let frame2 = create_test_frame(pos2);
        let frame3 = create_test_frame(pos3);

        cache.cache_frame(pos1, frame1).await.unwrap();
        cache.cache_frame(pos2, frame2).await.unwrap();
        cache.cache_frame(pos3, frame3).await.unwrap();


        let cached = cache.get_frame(pos1).await.unwrap();
        assert!(cached.is_none());

        let cached = cache.get_frame(pos2).await.unwrap();
        assert!(cached.is_some());

        let cached = cache.get_frame(pos3).await.unwrap();
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_cache_warming() {
        let config = CacheConfig::default();
        let cache = Arc::new(FrameCache::new(config));
        let warmer = CacheWarmer::new(cache.clone());

        let positions = vec![
            TimelinePosition::from_frame(100),
            TimelinePosition::from_frame(101),
            TimelinePosition::from_frame(102),
        ];

        let warmed = warmer.warm_frequent_positions(positions.clone(), |pos| {
            Ok(create_test_frame(pos))
        }).await.unwrap();

        assert_eq!(warmed, 3);


        for position in positions {
            let cached = cache.get_frame(position).await.unwrap();
            assert!(cached.is_some());
        }
    }

    #[test]
    fn test_cache_config() {
        let config = CacheConfig::default();
        assert_eq!(config.max_size, 1000);
        assert_eq!(config.preload_batch_size, 50);
        assert_eq!(config.eviction_policy, EvictionPolicy::LRU);
        assert!(config.ttl.is_some());
        assert!(!config.compression);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();
        stats.hits = 80;
        stats.misses = 20;
        stats.frames_cached = 50;
        stats.total_size = 50;

        assert_eq!(stats.hit_rate, 0.8);

        let formatted = stats.format();
        assert!(formatted.contains("Size: 50/1000"));
        assert!(formatted.contains("Hits: 80"));
        assert!(formatted.contains("Hit Rate: 80.0%"));
    }
}
