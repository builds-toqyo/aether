use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wgpu::{Device, Sampler};
use anyhow::{Result, anyhow};
use log::{debug, info, warn};

pub mod gpu_sync;
pub mod memory_tracker;

pub use gpu_sync::{GpuCpuSynchronization, PendingOperation, SyncStatus};
pub use memory_tracker::{MemoryTracker, MemoryStats, TextureAllocation, BufferAllocation, SamplerAllocation};

/// Handle for samplers
#[derive(Debug, Clone)]
pub struct SamplerHandle {
    pub id: Uuid,
    pub sampler: Arc<Sampler>,
    manager: Arc<Mutex<MemoryTracker>>,
}

impl SamplerHandle {
    /// Get sampler ID
    pub fn id(&self) -> Uuid {
        self.id
    }
    
    /// Get the underlying sampler
    pub fn get_sampler(&self) -> Arc<Sampler> {
        self.sampler.clone()
    }
}

impl Drop for SamplerHandle {
    fn drop(&mut self) {
        debug!("Dropping sampler handle: {:?}", self.id);
    }
}
