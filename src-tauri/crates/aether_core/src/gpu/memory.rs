use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wgpu::{Device, Queue, Buffer, Texture, TextureView, Sampler, BufferDescriptor, TextureDescriptor, TextureViewDescriptor, SamplerDescriptor, Extent3d, TextureDimension, TextureFormat, TextureUsages, BufferUsages};
use log::{debug, info, warn, error};
use anyhow::{Result, anyhow};

/// GPU memory manager for handling textures, buffers, and synchronization
pub struct GpuMemoryManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    texture_pool: Arc<Mutex<TexturePool>>,
    buffer_pool: Arc<Mutex<BufferPool>>,
    memory_tracker: Arc<Mutex<MemoryTracker>>,
    synchronization: Arc<Mutex<GpuCpuSynchronization>>,
}

impl GpuMemoryManager {
    /// Create a new GPU memory manager
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        info!("Initializing GPU memory manager");
        
        Self {
            device: device.clone(),
            queue: queue.clone(),
            texture_pool: Arc::new(Mutex::new(TexturePool::new(device.clone()))),
            buffer_pool: Arc::new(Mutex::new(BufferPool::new(device.clone()))),
            memory_tracker: Arc::new(Mutex::new(MemoryTracker::new())),
            synchronization: Arc::new(Mutex::new(GpuCpuSynchronization::new())),
        }
    }
    
    /// Allocate a texture for frame buffer
    pub fn allocate_texture(&self, width: u32, height: u32, format: TextureFormat) -> Result<TextureHandle> {
        debug!("Allocating texture: {}x{} format={:?}", width, height, format);
        
        let mut pool = self.texture_pool.lock().map_err(|e| anyhow!("Texture pool lock error: {}", e))?;
        let texture_id = pool.allocate(width, height, format)?;
        
        // Track memory usage
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow!("Memory tracker lock error: {}", e))?;
        tracker.track_texture_allocation(&texture_id, width, height, format);
        
        Ok(TextureHandle {
            id: texture_id,
            width,
            height,
            format,
            manager: self.texture_pool.clone(),
        })
    }
    
    /// Allocate a buffer for data transfer
    pub fn allocate_buffer(&self, size: u64, usage: BufferUsages) -> Result<BufferHandle> {
        debug!("Allocating buffer: {} bytes usage={:?}", size, usage);
        
        let mut pool = self.buffer_pool.lock().map_err(|e| anyhow!("Buffer pool lock error: {}", e))?;
        let buffer_id = pool.allocate(size, usage)?;
        
        // Track memory usage
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow!("Memory tracker lock error: {}", e))?;
        tracker.track_buffer_allocation(&buffer_id, size, usage);
        
        Ok(BufferHandle {
            id: buffer_id,
            size,
            usage,
            manager: self.buffer_pool.clone(),
        })
    }
    
    /// Create a sampler for texture sampling
    pub fn create_sampler(&self, descriptor: &SamplerDescriptor<'_>) -> Result<SamplerHandle> {
        debug!("Creating sampler");
        
        let sampler = self.device.create_sampler(descriptor);
        let sampler_id = Uuid::new_v4();
        
        // Track sampler allocation
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow!("Memory tracker lock error: {}", e))?;
        tracker.track_sampler_allocation(&sampler_id);
        
        Ok(SamplerHandle {
            id: sampler_id,
            sampler: Arc::new(sampler),
            manager: self.memory_tracker.clone(),
        })
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> Result<MemoryStats> {
        let tracker = self.memory_tracker.lock().map_err(|e| anyhow!("Memory tracker lock error: {}", e))?;
        Ok(tracker.get_stats())
    }
    
    /// Cleanup unused resources
    pub fn cleanup_unused_resources(&self) -> Result<usize> {
        debug!("Cleaning up unused GPU resources");
        
        let mut pool = self.texture_pool.lock().map_err(|e| anyhow!("Texture pool lock error: {}", e))?;
        let texture_cleanup_count = pool.cleanup_unused()?;
        
        let mut pool = self.buffer_pool.lock().map_err(|e| anyhow!("Buffer pool lock error: {}", e))?;
        let buffer_cleanup_count = pool.cleanup_unused()?;
        
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow!("Memory tracker lock error: {}", e))?;
        tracker.cleanup_unused_resources();
        
        let total_cleaned = texture_cleanup_count + buffer_cleanup_count;
        info!("Cleaned up {} unused GPU resources", total_cleaned);
        
        Ok(total_cleaned)
    }
    
    /// Force cleanup all resources
    pub fn force_cleanup(&self) -> Result<()> {
        warn!("Force cleaning up all GPU resources");
        
        let mut pool = self.texture_pool.lock().map_err(|e| anyhow!("Texture pool lock error: {}", e))?;
        pool.force_cleanup()?;
        
        let mut pool = self.buffer_pool.lock().map_err(|e| anyhow!("Buffer pool lock error: {}", e))?;
        pool.force_cleanup()?;
        
        let mut tracker = self.memory_tracker.lock().map_err(|e| anyhow!("Memory tracker lock error: {}", e))?;
        tracker.force_cleanup();
        
        info!("Force cleanup completed");
        
        Ok(())
    }
    
    /// Get GPU-CPU synchronization status
    pub fn get_sync_status(&self) -> Result<SyncStatus> {
        let sync = self.synchronization.lock().map_err(|e| anyhow!("Synchronization lock error: {}", e))?;
        Ok(sync.get_status())
    }
    
    /// Wait for GPU operations to complete
    pub fn wait_for_gpu(&self) -> Result<()> {
        debug!("Waiting for GPU operations to complete");
        
        let mut sync = self.synchronization.lock().map_err(|e| anyhow!("Synchronization lock error: {}", e))?;
        sync.wait_for_gpu(&self.device)?;
        
        Ok(())
    }
    
    /// Wait for GPU operations with timeout
    pub fn wait_for_gpu_with_timeout(&self, timeout: std::time::Duration) -> Result<bool> {
        debug!("Waiting for GPU operations with timeout: {:?}", timeout);
        
        let mut sync = self.synchronization.lock().map_err(|e| anyhow!("Synchronization lock error: {}", e))?;
        sync.wait_for_gpu_with_timeout(&self.device, timeout)
    }
    
    /// Track a GPU operation for synchronization
    pub fn track_gpu_operation(&self, operation_type: String) -> Result<u64> {
        let mut sync = self.synchronization.lock().map_err(|e| anyhow!("Synchronization lock error: {}", e))?;
        Ok(sync.track_operation(operation_type))
    }
    
    /// Mark a specific GPU operation as completed
    pub fn complete_gpu_operation(&self, operation_id: u64) -> Result<()> {
        let mut sync = self.synchronization.lock().map_err(|e| anyhow!("Synchronization lock error: {}", e))?;
        sync.complete_operation(operation_id)
    }
    
    /// Get pending GPU operations
    pub fn get_pending_operations(&self) -> Result<Vec<String>> {
        let sync = self.synchronization.lock().map_err(|e| anyhow!("Synchronization lock error: {}", e))?;
        let operations = sync.get_pending_operations()
            .into_iter()
            .map(|op| format!("{}: {} (submitted {:?})", op.operation_id, op.operation_type, op.submitted_at))
            .collect();
        Ok(operations)
    }
    
    /// Force clear all pending operations (emergency cleanup)
    pub fn force_clear_gpu_operations(&self) -> Result<()> {
        warn!("Force clearing all pending GPU operations");
        
        let mut sync = self.synchronization.lock().map_err(|e| anyhow!("Synchronization lock error: {}", e))?;
        sync.force_clear();
        
        Ok(())
    }
}

/// Handle for allocated textures
#[derive(Debug, Clone)]
pub struct TextureHandle {
    id: Uuid,
    width: u32,
    height: u32,
    format: TextureFormat,
    manager: Arc<Mutex<TexturePool>>,
}

impl TextureHandle {
    /// Get texture ID
    pub fn id(&self) -> Uuid {
        self.id
    }
    
    /// Get texture dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Get texture format
    pub fn format(&self) -> TextureFormat {
        self.format
    }
    
    /// Get the underlying texture
    pub fn get_texture(&self) -> Result<Arc<Texture>> {
        let pool = self.manager.lock().map_err(|e| anyhow!("Texture pool lock error: {}", e))?;
        pool.get_texture(&self.id)
    }
    
    /// Get texture view
    pub fn get_view(&self) -> Result<Arc<TextureView>> {
        let pool = self.manager.lock().map_err(|e| anyhow!("Texture pool lock error: {}", e))?;
        pool.get_view(&self.id)
    }
}

impl Drop for TextureHandle {
    fn drop(&mut self) {
        debug!("Dropping texture handle: {:?}", self.id);
    }
}

/// Handle for allocated buffers
#[derive(Debug, Clone)]
pub struct BufferHandle {
    id: Uuid,
    size: u64,
    usage: BufferUsages,
    manager: Arc<Mutex<BufferPool>>,
}

impl BufferHandle {
    /// Get buffer ID
    pub fn id(&self) -> Uuid {
        self.id
    }
    
    /// Get buffer size
    pub fn size(&self) -> u64 {
        self.size
    }
    
    /// Get buffer usage flags
    pub fn usage(&self) -> BufferUsages {
        self.usage
    }
    
    /// Get the underlying buffer
    pub fn get_buffer(&self) -> Result<Arc<Buffer>> {
        let pool = self.manager.lock().map_err(|e| anyhow!("Buffer pool lock error: {}", e))?;
        pool.get_buffer(&self.id)
    }
}

impl Drop for BufferHandle {
    fn drop(&mut self) {
        debug!("Dropping buffer handle: {:?}", self.id);
    }
}

/// Handle for samplers
#[derive(Debug, Clone)]
pub struct SamplerHandle {
    id: Uuid,
    sampler: Arc<Sampler>,
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

/// Texture pool for efficient texture allocation and reuse
struct TexturePool {
    device: Arc<Device>,
    textures: HashMap<Uuid, TextureEntry>,
    views: HashMap<Uuid, Arc<TextureView>>,
    available_textures: Vec<AvailableTexture>,
}

impl TexturePool {
    fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            textures: HashMap::new(),
            views: HashMap::new(),
            available_textures: Vec::new(),
        }
    }
    
    fn allocate(&mut self, width: u32, height: u32, format: TextureFormat) -> Result<Uuid> {
        // Try to reuse an available texture
        if let Some(available) = self.find_available_texture(width, height, format) {
            let texture_id = available.id;
            debug!("Reusing available texture: {:?}", texture_id);
            return Ok(texture_id);
        }
        
        // Create new texture
        let texture_id = Uuid::new_v4();
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some(&format!("Texture {:?}", texture_id)),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        
        let texture_entry = TextureEntry {
            texture: Arc::new(texture),
            width,
            height,
            format,
            ref_count: 1,
            last_used: std::time::Instant::now(),
        };
        
        self.textures.insert(texture_id, texture_entry);
        
        debug!("Created new texture: {:?}", texture_id);
        
        Ok(texture_id)
    }
    
    fn find_available_texture(&mut self, width: u32, height: u32, format: TextureFormat) -> Option<AvailableTexture> {
        let index = self.available_textures.iter().position(|t| {
            t.width == width && t.height == height && t.format == format
        });
        
        if let Some(index) = index {
            let available = self.available_textures.swap_remove(index);
            
            // Update reference count
            if let Some(entry) = self.textures.get_mut(&available.id) {
                entry.ref_count += 1;
                entry.last_used = std::time::Instant::now();
            }
            
            Some(available)
        } else {
            None
        }
    }
    
    fn get_texture(&self, texture_id: &Uuid) -> Result<Arc<Texture>> {
        self.textures.get(texture_id)
            .map(|entry| entry.texture.clone())
            .ok_or_else(|| anyhow!("Texture not found: {:?}", texture_id))
    }
    
    fn get_view(&mut self, texture_id: &Uuid) -> Result<Arc<TextureView>> {
        if let Some(view) = self.views.get(texture_id) {
            return Ok(view.clone());
        }
        
        let texture = self.get_texture(texture_id)?;
        let view = texture.create_view(&TextureViewDescriptor::default());
        let view = Arc::new(view);
        
        self.views.insert(*texture_id, view.clone());
        
        Ok(view)
    }
    
    fn cleanup_unused(&mut self) -> Result<usize> {
        let mut cleaned_count = 0;
        let now = std::time::Instant::now();
        const CLEANUP_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(30);
        
        // Find textures that haven't been used recently
        let to_cleanup: Vec<Uuid> = self.textures.iter()
            .filter(|(_, entry)| entry.ref_count == 0 && now.duration_since(entry.last_used) > CLEANUP_THRESHOLD)
            .map(|(id, _)| *id)
            .collect();
        
        for texture_id in to_cleanup {
            self.textures.remove(&texture_id);
            self.views.remove(&texture_id);
            cleaned_count += 1;
        }
        
        Ok(cleaned_count)
    }
    
    fn force_cleanup(&mut self) -> Result<()> {
        self.textures.clear();
        self.views.clear();
        self.available_textures.clear();
        Ok(())
    }
}

/// Buffer pool for efficient buffer allocation and reuse
struct BufferPool {
    device: Arc<Device>,
    buffers: HashMap<Uuid, BufferEntry>,
    available_buffers: Vec<AvailableBuffer>,
}

impl BufferPool {
    fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            buffers: HashMap::new(),
            available_buffers: Vec::new(),
        }
    }
    
    fn allocate(&mut self, size: u64, usage: BufferUsages) -> Result<Uuid> {
        // Try to reuse an available buffer
        if let Some(available) = self.find_available_buffer(size, usage) {
            let buffer_id = available.id;
            debug!("Reusing available buffer: {:?}", buffer_id);
            return Ok(buffer_id);
        }
        
        // Create new buffer
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
    
    fn find_available_buffer(&mut self, size: u64, usage: BufferUsages) -> Option<AvailableBuffer> {
        let index = self.available_buffers.iter().position(|b| {
            b.size >= size && (b.usage & usage) == usage
        });
        
        if let Some(index) = index {
            let available = self.available_buffers.swap_remove(index);
            
            // Update reference count
            if let Some(entry) = self.buffers.get_mut(&available.id) {
                entry.ref_count += 1;
                entry.last_used = std::time::Instant::now();
            }
            
            Some(available)
        } else {
            None
        }
    }
    
    fn get_buffer(&self, buffer_id: &Uuid) -> Result<Arc<Buffer>> {
        self.buffers.get(buffer_id)
            .map(|entry| entry.buffer.clone())
            .ok_or_else(|| anyhow!("Buffer not found: {:?}", buffer_id))
    }
    
    fn cleanup_unused(&mut self) -> Result<usize> {
        let mut cleaned_count = 0;
        let now = std::time::Instant::now();
        const CLEANUP_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(30);
        
        // Find buffers that haven't been used recently
        let to_cleanup: Vec<Uuid> = self.buffers.iter()
            .filter(|(_, entry)| entry.ref_count == 0 && now.duration_since(entry.last_used) > CLEANUP_THRESHOLD)
            .map(|(id, _)| *id)
            .collect();
        
        for buffer_id in to_cleanup {
            self.buffers.remove(&buffer_id);
            cleaned_count += 1;
        }
        
        Ok(cleaned_count)
    }
    
    fn force_cleanup(&mut self) -> Result<()> {
        self.buffers.clear();
        self.available_buffers.clear();
        Ok(())
    }
}

/// Memory usage tracker
struct MemoryTracker {
    texture_allocations: HashMap<Uuid, TextureAllocation>,
    buffer_allocations: HashMap<Uuid, BufferAllocation>,
    sampler_allocations: HashMap<Uuid, SamplerAllocation>,
    total_texture_memory: u64,
    total_buffer_memory: u64,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            texture_allocations: HashMap::new(),
            buffer_allocations: HashMap::new(),
            sampler_allocations: HashMap::new(),
            total_texture_memory: 0,
            total_buffer_memory: 0,
        }
    }
    
    fn track_texture_allocation(&mut self, id: &Uuid, width: u32, height: u32, format: TextureFormat) {
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
    }
    
    fn track_buffer_allocation(&mut self, id: &Uuid, size: u64, usage: BufferUsages) {
        let allocation = BufferAllocation {
            id: *id,
            size,
            usage,
            created_at: std::time::Instant::now(),
        };
        
        self.buffer_allocations.insert(*id, allocation);
        self.total_buffer_memory += size;
    }
    
    fn track_sampler_allocation(&mut self, id: &Uuid) {
        let allocation = SamplerAllocation {
            id: *id,
            created_at: std::time::Instant::now(),
        };
        
        self.sampler_allocations.insert(*id, allocation);
    }
    
    fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            texture_count: self.texture_allocations.len(),
            buffer_count: self.buffer_allocations.len(),
            sampler_count: self.sampler_allocations.len(),
            total_texture_memory: self.total_texture_memory,
            total_buffer_memory: self.total_buffer_memory,
            total_memory: self.total_texture_memory + self.total_buffer_memory,
        }
    }
    
    fn cleanup_unused_resources(&mut self) {
        // Remove allocations that no longer exist in the pools
        self.texture_allocations.retain(|_, allocation| {
            std::time::Instant::now().duration_since(allocation.created_at) < std::time::Duration::from_secs(300)
        });
        
        self.buffer_allocations.retain(|_, allocation| {
            std::time::Instant::now().duration_since(allocation.created_at) < std::time::Duration::from_secs(300)
        });
        
        self.sampler_allocations.retain(|_, allocation| {
            std::time::Instant::now().duration_since(allocation.created_at) < std::time::Duration::from_secs(300)
        });
    }
    
    fn force_cleanup(&mut self) {
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
            _ => 4, // Default estimate
        };
        
        (width as u64) * (height as u64) * (bytes_per_pixel as u64)
    }
}

/// GPU-CPU synchronization manager
struct GpuCpuSynchronization {
    pending_operations: Vec<PendingOperation>,
    last_sync_time: std::time::Instant,
    operation_counter: u64,
}

impl GpuCpuSynchronization {
    fn new() -> Self {
        Self {
            pending_operations: Vec::new(),
            last_sync_time: std::time::Instant::now(),
            operation_counter: 0,
        }
    }
    
    /// Track a new GPU operation
    fn track_operation(&mut self, operation_type: String) -> u64 {
        let operation_id = self.operation_counter;
        self.operation_counter += 1;
        
        let operation = PendingOperation {
            id: Uuid::new_v4(),
            operation_id,
            operation_type,
            submitted_at: std::time::Instant::now(),
        };
        
        self.pending_operations.push(operation);
        debug!("Tracked GPU operation {}: {}", operation_id, operation.operation_type);
        
        operation_id
    }
    
    /// Wait for GPU to complete all operations
    fn wait_for_gpu(&mut self, device: &Device) -> Result<()> {
        debug!("Waiting for GPU to complete {} operations", self.pending_operations.len());
        
        // Use wgpu device polling to wait for GPU operations to complete
        // This ensures all submitted commands are processed before continuing
        device.poll(wgpu::Maintain::Wait);
        
        // Clear pending operations since we've waited for all to complete
        let completed_count = self.pending_operations.len();
        self.pending_operations.clear();
        self.last_sync_time = std::time::Instant::now();
        
        debug!("GPU operations completed: {} operations finished", completed_count);
        
        Ok(())
    }
    
    /// Wait for GPU with timeout
    fn wait_for_gpu_with_timeout(&mut self, device: &Device, timeout: std::time::Duration) -> Result<bool> {
        debug!("Waiting for GPU operations with timeout: {:?}", timeout);
        
        let start_time = std::time::Instant::now();
        
        // Poll the device and check if operations complete within timeout
        loop {
            device.poll(wgpu::Maintain::Poll);
            
            // Check if all operations are complete (in a real implementation, 
            // you'd track specific command buffers or fences)
            if self.pending_operations.is_empty() {
                self.last_sync_time = std::time::Instant::now();
                debug!("GPU operations completed within timeout");
                return Ok(true);
            }
            
            // Check timeout
            if start_time.elapsed() > timeout {
                warn!("GPU operations did not complete within timeout");
                return Ok(false);
            }
            
            // Small delay to prevent busy waiting
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    
    /// Mark specific operation as completed
    fn complete_operation(&mut self, operation_id: u64) -> Result<()> {
        let initial_count = self.pending_operations.len();
        
        self.pending_operations.retain(|op| op.operation_id != operation_id);
        
        if self.pending_operations.len() < initial_count {
            debug!("Completed GPU operation: {}", operation_id);
        } else {
            warn!("Operation {} not found in pending operations", operation_id);
        }
        
        Ok(())
    }
    
    /// Get synchronization status
    fn get_status(&self) -> SyncStatus {
        SyncStatus {
            pending_operations: self.pending_operations.len(),
            last_sync_time: self.last_sync_time,
            is_synced: self.pending_operations.is_empty(),
        }
    }
    
    /// Get pending operation details
    fn get_pending_operations(&self) -> Vec<&PendingOperation> {
        self.pending_operations.iter().collect()
    }
    
    /// Force clear all pending operations (emergency cleanup)
    fn force_clear(&mut self) {
        let count = self.pending_operations.len();
        self.pending_operations.clear();
        self.last_sync_time = std::time::Instant::now();
        warn!("Force cleared {} pending GPU operations", count);
    }
}

// Data structures for the pools and tracker
#[derive(Debug)]
struct TextureEntry {
    texture: Arc<Texture>,
    width: u32,
    height: u32,
    format: TextureFormat,
    ref_count: u32,
    last_used: std::time::Instant,
}

#[derive(Debug)]
struct BufferEntry {
    buffer: Arc<Buffer>,
    size: u64,
    usage: BufferUsages,
    ref_count: u32,
    last_used: std::time::Instant,
}

#[derive(Debug)]
struct AvailableTexture {
    id: Uuid,
    width: u32,
    height: u32,
    format: TextureFormat,
}

#[derive(Debug)]
struct AvailableBuffer {
    id: Uuid,
    size: u64,
    usage: BufferUsages,
}

#[derive(Debug)]
struct TextureAllocation {
    id: Uuid,
    width: u32,
    height: u32,
    format: TextureFormat,
    size: u64,
    created_at: std::time::Instant,
}

#[derive(Debug)]
struct BufferAllocation {
    id: Uuid,
    size: u64,
    usage: BufferUsages,
    created_at: std::time::Instant,
}

#[derive(Debug)]
struct SamplerAllocation {
    id: Uuid,
    created_at: std::time::Instant,
}

#[derive(Debug)]
struct PendingOperation {
    id: Uuid,
    operation_id: u64,
    operation_type: String,
    submitted_at: std::time::Instant,
}

/// Memory usage statistics
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
    /// Get memory usage in human readable format
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
}

/// GPU-CPU synchronization status
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub pending_operations: usize,
    pub last_sync_time: std::time::Instant,
    pub is_synced: bool,
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
            total_texture_memory: 1024 * 1024, // 1MB
            total_buffer_memory: 2 * 1024 * 1024, // 2MB
            total_memory: 3 * 1024 * 1024, // 3MB
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
        
        // Test different formats
        let rgba_size = tracker.estimate_texture_size(100, 100, TextureFormat::Rgba8UnormSrgb);
        assert_eq!(rgba_size, 100 * 100 * 4); // 4 bytes per pixel
        
        let r_size = tracker.estimate_texture_size(50, 50, TextureFormat::R8Unorm);
        assert_eq!(r_size, 50 * 50 * 1); // 1 byte per pixel
        
        let float_size = tracker.estimate_texture_size(32, 32, TextureFormat::R32Float);
        assert_eq!(float_size, 32 * 32 * 4); // 4 bytes per pixel
    }
    
    #[test]
    fn test_gpu_synchronization_initialization() {
        let sync = GpuCpuSynchronization::new();
        
        assert_eq!(sync.pending_operations.len(), 0);
        assert_eq!(sync.operation_counter, 0);
        assert!(!sync.last_sync_time.elapsed().is_zero());
    }
    
    #[test]
    fn test_gpu_operation_tracking() {
        let mut sync = GpuCpuSynchronization::new();
        
        // Track some operations
        let op1_id = sync.track_operation("texture_upload".to_string());
        let op2_id = sync.track_operation("buffer_copy".to_string());
        
        assert_eq!(op1_id, 0);
        assert_eq!(op2_id, 1);
        assert_eq!(sync.pending_operations.len(), 2);
        assert_eq!(sync.operation_counter, 2);
        
        // Check operation details
        let operations = sync.get_pending_operations();
        assert_eq!(operations.len(), 2);
        assert_eq!(operations[0].operation_type, "texture_upload");
        assert_eq!(operations[1].operation_type, "buffer_copy");
    }
    
    #[test]
    fn test_gpu_operation_completion() {
        let mut sync = GpuCpuSynchronization::new();
        
        // Track operations
        let op1_id = sync.track_operation("texture_upload".to_string());
        let op2_id = sync.track_operation("buffer_copy".to_string());
        
        assert_eq!(sync.pending_operations.len(), 2);
        
        // Complete one operation
        sync.complete_operation(op1_id).unwrap();
        assert_eq!(sync.pending_operations.len(), 1);
        
        // Complete the other
        sync.complete_operation(op2_id).unwrap();
        assert_eq!(sync.pending_operations.len(), 0);
        
        // Try to complete non-existent operation
        let result = sync.complete_operation(999);
        assert!(result.is_ok()); // Should not panic, just log warning
    }
    
    #[test]
    fn test_gpu_sync_status() {
        let mut sync = GpuCpuSynchronization::new();
        
        // Initial status
        let status = sync.get_status();
        assert_eq!(status.pending_operations, 0);
        assert!(status.is_synced);
        
        // Track an operation
        sync.track_operation("test_operation".to_string());
        
        let status = sync.get_status();
        assert_eq!(status.pending_operations, 1);
        assert!(!status.is_synced);
        
        // Complete the operation
        sync.complete_operation(0).unwrap();
        
        let status = sync.get_status();
        assert_eq!(status.pending_operations, 0);
        assert!(status.is_synced);
    }
    
    #[test]
    fn test_gpu_force_clear() {
        let mut sync = GpuCpuSynchronization::new();
        
        // Track multiple operations
        sync.track_operation("op1".to_string());
        sync.track_operation("op2".to_string());
        sync.track_operation("op3".to_string());
        
        assert_eq!(sync.pending_operations.len(), 3);
        
        // Force clear
        sync.force_clear();
        
        assert_eq!(sync.pending_operations.len(), 0);
        assert!(!sync.last_sync_time.elapsed().is_zero());
    }
    
    #[test]
    fn test_sync_status_formatting() {
        let status = SyncStatus {
            pending_operations: 5,
            last_sync_time: std::time::Instant::now(),
            is_synced: false,
        };
        
        assert_eq!(status.pending_operations, 5);
        assert!(!status.is_synced);
        assert!(!status.last_sync_time.elapsed().is_zero());
    }
}
