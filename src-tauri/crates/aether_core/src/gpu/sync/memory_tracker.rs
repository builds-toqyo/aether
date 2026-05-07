use std::collections::HashMap;
use uuid::Uuid;
use wgpu::{TextureFormat, BufferUsages};
use log::{debug, info, warn};


pub struct MemoryTracker {
    texture_allocations: HashMap<Uuid, TextureAllocation>,
    buffer_allocations: HashMap<Uuid, BufferAllocation>,
    sampler_allocations: HashMap<Uuid, SamplerAllocation>,
    total_texture_memory: u64,
    total_buffer_memory: u64,
}

impl MemoryTracker {

    pub fn new() -> Self {
        info!("Creating memory tracker");

        Self {
            texture_allocations: HashMap::new(),
            buffer_allocations: HashMap::new(),
            sampler_allocations: HashMap::new(),
            total_texture_memory: 0,
            total_buffer_memory: 0,
        }
    }


    pub fn track_texture_allocation(&mut self, id: &Uuid, width: u32, height: u32, format: TextureFormat) {
        let size = self.estimate_texture_size(width, height, format);

        let allocation = TextureAllocation {
            id: *id,
            width,
            height,
            format,
            size,
            created_at: std::time::Instant::now(),
        };

        self.texture_allocations.insert(*id, allocation);
        self.total_texture_memory += size;

        debug!("Tracked texture allocation: {} ({}x{} {:?} = {} bytes)",
            id, width, height, format, size);
    }


    pub fn track_buffer_allocation(&mut self, id: &Uuid, size: u64, usage: BufferUsages) {
        let allocation = BufferAllocation {
            id: *id,
            size,
            usage,
            created_at: std::time::Instant::now(),
        };

        self.buffer_allocations.insert(*id, allocation);
        self.total_buffer_memory += size;

        debug!("Tracked buffer allocation: {} ({} bytes)", id, size);
    }


    pub fn track_sampler_allocation(&mut self, id: &Uuid) {
        let allocation = SamplerAllocation {
            id: *id,
            created_at: std::time::Instant::now(),
        };

        self.sampler_allocations.insert(*id, allocation);

        debug!("Tracked sampler allocation: {}", id);
    }


    pub fn untrack_texture_allocation(&mut self, id: &Uuid) -> Result<u64> {
        if let Some(allocation) = self.texture_allocations.remove(id) {
            self.total_texture_memory -= allocation.size;
            debug!("Untracked texture allocation: {} (freed {} bytes)", id, allocation.size);
            Ok(allocation.size)
        } else {
            warn!("Attempted to untrack non-existent texture: {}", id);
            Ok(0)
        }
    }


    pub fn untrack_buffer_allocation(&mut self, id: &Uuid) -> Result<u64> {
        if let Some(allocation) = self.buffer_allocations.remove(id) {
            self.total_buffer_memory -= allocation.size;
            debug!("Untracked buffer allocation: {} (freed {} bytes)", id, allocation.size);
            Ok(allocation.size)
        } else {
            warn!("Attempted to untrack non-existent buffer: {}", id);
            Ok(0)
        }
    }


    pub fn untrack_sampler_allocation(&mut self, id: &Uuid) -> Result<()> {
        if self.sampler_allocations.remove(id).is_some() {
            debug!("Untracked sampler allocation: {}", id);
        } else {
            warn!("Attempted to untrack non-existent sampler: {}", id);
        }
        Ok(())
    }


    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            texture_count: self.texture_allocations.len(),
            buffer_count: self.buffer_allocations.len(),
            sampler_count: self.sampler_allocations.len(),
            total_texture_memory: self.total_texture_memory,
            total_buffer_memory: self.total_buffer_memory,
            total_memory: self.total_texture_memory + self.total_buffer_memory,
        }
    }


    pub fn get_allocation_report(&self) -> AllocationReport {
        let mut texture_formats = HashMap::new();
        let mut buffer_usages = HashMap::new();


        for allocation in self.texture_allocations.values() {
            *texture_formats.entry(allocation.format).or_insert(0) += 1;
        }


        for allocation in self.buffer_allocations.values() {
            let usage_str = format!("{:?}", allocation.usage);
            *buffer_usages.entry(usage_str).or_insert(0) += 1;
        }

        AllocationReport {
            texture_formats,
            buffer_usages,
            oldest_texture_age: self.get_oldest_allocation_age(&self.texture_allocations),
            oldest_buffer_age: self.get_oldest_allocation_age(&self.buffer_allocations),
        }
    }


    pub fn cleanup_unused_resources(&mut self) {
        debug!("Cleaning up old allocation records");

        let now = std::time::Instant::now();
        const CLEANUP_AGE: std::time::Duration = std::time::Duration::from_secs(300);


        self.texture_allocations.retain(|_, allocation| {
            now.duration_since(allocation.created_at) < CLEANUP_AGE
        });


        self.buffer_allocations.retain(|_, allocation| {
            now.duration_since(allocation.created_at) < CLEANUP_AGE
        });


        self.sampler_allocations.retain(|_, allocation| {
            now.duration_since(allocation.created_at) < CLEANUP_AGE
        });

        debug!("Cleanup completed");
    }


    pub fn force_cleanup(&mut self) {
        warn!("Force cleaning up all allocation records");

        self.texture_allocations.clear();
        self.buffer_allocations.clear();
        self.sampler_allocations.clear();
        self.total_texture_memory = 0;
        self.total_buffer_memory = 0;
    }


    fn estimate_texture_size(&self, width: u32, height: u32, format: TextureFormat) -> u64 {
        let bytes_per_pixel = match format {
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::R32Float => 4,
            TextureFormat::Rg32Float => 8,
            TextureFormat::Rgba32Float => 16,
            _ => 4,
        };

        (width as u64) * (height as u64) * (bytes_per_pixel as u64)
    }


    fn get_oldest_allocation_age<T>(&self, allocations: &HashMap<Uuid, T>) -> std::time::Duration
    where
        T: AllocationAge,
    {
        allocations.values()
            .map(|alloc| alloc.get_age())
            .min()
            .unwrap_or_default()
    }
}


trait AllocationAge {
    fn get_age(&self) -> std::time::Duration;
}

impl AllocationAge for TextureAllocation {
    fn get_age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
}

impl AllocationAge for BufferAllocation {
    fn get_age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
}

impl AllocationAge for SamplerAllocation {
    fn get_age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
}


#[derive(Debug, Clone)]
pub struct TextureAllocation {
    pub id: Uuid,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub size: u64,
    pub created_at: std::time::Instant,
}


#[derive(Debug, Clone)]
pub struct BufferAllocation {
    pub id: Uuid,
    pub size: u64,
    pub usage: BufferUsages,
    pub created_at: std::time::Instant,
}


#[derive(Debug, Clone)]
pub struct SamplerAllocation {
    pub id: Uuid,
    pub created_at: std::time::Instant,
}


#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub texture_count: usize,
    pub buffer_count: usize,
    pub sampler_count: usize,
    pub total_texture_memory: u64,
    pub total_buffer_memory: u64,
    pub total_memory: u64,
}

impl MemoryStats {

    pub fn format_memory(&self) -> String {
        let total_mb = self.total_memory as f64 / (1024.0 * 1024.0);
        let texture_mb = self.total_texture_memory as f64 / (1024.0 * 1024.0);
        let buffer_mb = self.total_buffer_memory as f64 / (1024.0 * 1024.0);

        format!(
            "Total: {:.1}MB (Textures: {:.1}MB, Buffers: {:.1}MB) - {} textures, {} buffers, {} samplers",
            total_mb, texture_mb, buffer_mb,
            self.texture_count, self.buffer_count, self.sampler_count
        )
    }


    pub fn get_efficiency_metrics(&self) -> EfficiencyMetrics {
        let avg_texture_size = if self.texture_count > 0 {
            self.total_texture_memory / self.texture_count as u64
        } else {
            0
        };

        let avg_buffer_size = if self.buffer_count > 0 {
            self.total_buffer_memory / self.buffer_count as u64
        } else {
            0
        };

        EfficiencyMetrics {
            average_texture_size_bytes: avg_texture_size,
            average_buffer_size_bytes: avg_buffer_size,
            texture_to_buffer_ratio: self.total_texture_memory as f64 / self.total_buffer_memory.max(1) as f64,
        }
    }
}


#[derive(Debug, Clone)]
pub struct AllocationReport {
    pub texture_formats: HashMap<TextureFormat, usize>,
    pub buffer_usages: HashMap<String, usize>,
    pub oldest_texture_age: std::time::Duration,
    pub oldest_buffer_age: std::time::Duration,
}


#[derive(Debug, Clone)]
pub struct EfficiencyMetrics {
    pub average_texture_size_bytes: u64,
    pub average_buffer_size_bytes: u64,
    pub texture_to_buffer_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_formatting() {
        let stats = MemoryStats {
            texture_count: 10,
            buffer_count: 5,
            sampler_count: 3,
            total_texture_memory: 1024 * 1024,
            total_buffer_memory: 2 * 1024 * 1024,
            total_memory: 3 * 1024 * 1024,
        };

        let formatted = stats.format_memory();
        assert!(formatted.contains("3.0MB"));
        assert!(formatted.contains("1.0MB"));
        assert!(formatted.contains("2.0MB"));
        assert!(formatted.contains("10 textures"));
        assert!(formatted.contains("5 buffers"));
        assert!(formatted.contains("3 samplers"));
    }

    #[test]
    fn test_texture_size_estimation() {
        let tracker = MemoryTracker::new();


        let rgba_size = tracker.estimate_texture_size(100, 100, TextureFormat::Rgba8UnormSrgb);
        assert_eq!(rgba_size, 100 * 100 * 4);

        let r_size = tracker.estimate_texture_size(50, 50, TextureFormat::R8Unorm);
        assert_eq!(r_size, 50 * 50 * 1);

        let float_size = tracker.estimate_texture_size(32, 32, TextureFormat::R32Float);
        assert_eq!(float_size, 32 * 32 * 4);
    }

    #[test]
    fn test_memory_tracking() {
        let mut tracker = MemoryTracker::new();


        let texture_id = Uuid::new_v4();
        tracker.track_texture_allocation(&texture_id, 100, 100, TextureFormat::Rgba8UnormSrgb);


        let buffer_id = Uuid::new_v4();
        tracker.track_buffer_allocation(&buffer_id, 1024, BufferUsages::STORAGE | BufferUsages::COPY_DST);


        let sampler_id = Uuid::new_v4();
        tracker.track_sampler_allocation(&sampler_id);

        let stats = tracker.get_stats();
        assert_eq!(stats.texture_count, 1);
        assert_eq!(stats.buffer_count, 1);
        assert_eq!(stats.sampler_count, 1);
        assert_eq!(stats.total_texture_memory, 100 * 100 * 4);
        assert_eq!(stats.total_buffer_memory, 1024);
    }

    #[test]
    fn test_efficiency_metrics() {
        let stats = MemoryStats {
            texture_count: 2,
            buffer_count: 4,
            sampler_count: 1,
            total_texture_memory: 2048,
            total_buffer_memory: 4096,
            total_memory: 6144,
        };

        let metrics = stats.get_efficiency_metrics();
        assert_eq!(metrics.average_texture_size_bytes, 1024);
        assert_eq!(metrics.average_buffer_size_bytes, 1024);
        assert_eq!(metrics.texture_to_buffer_ratio, 0.5);
    }
}
