use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wgpu::{Device, Buffer, BufferDescriptor, BufferUsages};
use anyhow::{Result, anyhow};
use log::{debug, info, warn};


pub struct BufferPool {
    device: Arc<Device>,
    buffers: HashMap<Uuid, BufferEntry>,
    available_buffers: Vec<AvailableBuffer>,
}

impl BufferPool {

    pub fn new(device: Arc<Device>) -> Self {
        info!("Creating buffer pool");

        Self {
            device,
            buffers: HashMap::new(),
            available_buffers: Vec::new(),
        }
    }


    pub fn allocate(&mut self, size: u64, usage: BufferUsages) -> Result<Uuid> {
        debug!("Allocating buffer: {} bytes usage={:?}", size, usage);


        if let Some(available) = self.find_available_buffer(size, usage) {
            let buffer_id = available.id;
            debug!("Reusing available buffer: {:?}", buffer_id);
            return Ok(buffer_id);
        }


        let buffer_id = Uuid::new_v4();
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some(&format!("Buffer {:?}", buffer_id)),
            size,
            usage,
            mapped_at_creation: false,
        });

        let buffer_entry = BufferEntry {
            buffer: Arc::new(buffer),
            size,
            usage,
            ref_count: 1,
            last_used: std::time::Instant::now(),
        };

        self.buffers.insert(buffer_id, buffer_entry);

        debug!("Created new buffer: {:?}", buffer_id);

        Ok(buffer_id)
    }


    pub fn get_buffer(&self, buffer_id: &Uuid) -> Result<Arc<Buffer>> {
        self.buffers.get(buffer_id)
            .map(|entry| entry.buffer.clone())
            .ok_or_else(|| anyhow!("Buffer not found: {:?}", buffer_id))
    }


    pub fn release_buffer(&mut self, buffer_id: &Uuid) -> Result<()> {
        debug!("Releasing buffer: {:?}", buffer_id);

        if let Some(entry) = self.buffers.get_mut(buffer_id) {
            entry.ref_count -= 1;
            entry.last_used = std::time::Instant::now();

            if entry.ref_count == 0 {

                let available = AvailableBuffer {
                    id: *buffer_id,
                    size: entry.size,
                    usage: entry.usage,
                };
                self.available_buffers.push(available);
                debug!("Buffer added to available pool: {:?}", buffer_id);
            }

            Ok(())
        } else {
            Err(anyhow!("Buffer not found: {:?}", buffer_id))
        }
    }


    pub fn cleanup_unused(&mut self) -> Result<usize> {
        debug!("Cleaning up unused buffers");

        let mut cleaned_count = 0;
        let now = std::time::Instant::now();
        const CLEANUP_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(30);


        let to_cleanup: Vec<Uuid> = self.buffers.iter()
            .filter(|(_, entry)| entry.ref_count == 0 && now.duration_since(entry.last_used) > CLEANUP_THRESHOLD)
            .map(|(id, _)| *id)
            .collect();

        for buffer_id in to_cleanup {
            self.buffers.remove(&buffer_id);
            cleaned_count += 1;
        }

        info!("Cleaned up {} unused buffers", cleaned_count);

        Ok(cleaned_count)
    }


    pub fn force_cleanup(&mut self) -> Result<()> {
        warn!("Force cleaning up all buffers");

        self.buffers.clear();
        self.available_buffers.clear();

        Ok(())
    }


    pub fn get_stats(&self) -> BufferPoolStats {
        let active_count = self.buffers.len();
        let available_count = self.available_buffers.len();
        let total_memory = self.buffers.values()
            .map(|entry| entry.size)
            .sum();

        BufferPoolStats {
            active_buffers: active_count,
            available_buffers: available_count,
            total_memory_bytes: total_memory,
        }
    }


    fn find_available_buffer(&mut self, size: u64, usage: BufferUsages) -> Option<AvailableBuffer> {
        let index = self.available_buffers.iter().position(|b| {
            b.size >= size && (b.usage & usage) == usage
        });

        if let Some(index) = index {
            let available = self.available_buffers.swap_remove(index);


            if let Some(entry) = self.buffers.get_mut(&available.id) {
                entry.ref_count += 1;
                entry.last_used = std::time::Instant::now();
            }

            Some(available)
        } else {
            None
        }
    }
}


#[derive(Debug)]
pub struct BufferEntry {
    pub buffer: Arc<Buffer>,
    pub size: u64,
    pub usage: BufferUsages,
    pub ref_count: u32,
    pub last_used: std::time::Instant,
}


#[derive(Debug, Clone)]
pub struct AvailableBuffer {
    pub id: Uuid,
    pub size: u64,
    pub usage: BufferUsages,
}


#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    pub active_buffers: usize,
    pub available_buffers: usize,
    pub total_memory_bytes: u64,
}

impl BufferPoolStats {

    pub fn format_memory(&self) -> String {
        let mb = self.total_memory_bytes as f64 / (1024.0 * 1024.0);
        format!("{:.1}MB ({} active, {} available)", mb, self.active_buffers, self.available_buffers)
    }


    pub fn average_buffer_size(&self) -> f64 {
        if self.active_buffers > 0 {
            self.total_memory_bytes as f64 / self.active_buffers as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::{DeviceDescriptor, RequestAdapterOptions, Instance};

    #[test]
    fn test_buffer_pool_stats() {
        let stats = BufferPoolStats {
            active_buffers: 3,
            available_buffers: 2,
            total_memory_bytes: 512 * 1024,
        };

        let formatted = stats.format_memory();
        assert!(formatted.contains("0.5MB"));
        assert!(formatted.contains("3 active"));
        assert!(formatted.contains("2 available"));

        let avg_size = stats.average_buffer_size();
        assert_eq!(avg_size, (512 * 1024) as f64 / 3.0);
    }

    #[test]
    fn test_average_buffer_size() {
        let stats = BufferPoolStats {
            active_buffers: 0,
            available_buffers: 0,
            total_memory_bytes: 0,
        };

        let avg_size = stats.average_buffer_size();
        assert_eq!(avg_size, 0.0);

        let stats = BufferPoolStats {
            active_buffers: 2,
            available_buffers: 0,
            total_memory_bytes: 2048,
        };

        let avg_size = stats.average_buffer_size();
        assert_eq!(avg_size, 1024.0);
    }


    fn create_mock_device() -> Device {


        panic!("Mock device implementation needed for tests")
    }
}
