

#[cfg(test)]
mod tests {
    use std::collections::HashMap;


    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum MemoryType {
        DeviceLocal,
        HostVisible,
        HostCoherent,
    }


    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum BufferUsage {
        Vertex,
        Index,
        Uniform,
        Storage,
        Transfer,
    }


    #[derive(Debug, Clone)]
    pub struct MemoryAllocation {
        pub id: u64,
        pub size: usize,
        pub memory_type: MemoryType,
        pub usage: BufferUsage,
        pub mapped: bool,
    }


    pub struct MockMemoryAllocator {
        allocations: HashMap<u64, MemoryAllocation>,
        next_id: u64,
        total_allocated: usize,
        max_memory: usize,
        device_local_used: usize,
        host_visible_used: usize,
    }

    impl MockMemoryAllocator {
        pub fn new(max_memory: usize) -> Self {
            Self {
                allocations: HashMap::new(),
                next_id: 1,
                total_allocated: 0,
                max_memory,
                device_local_used: 0,
                host_visible_used: 0,
            }
        }

        pub fn allocate(&mut self, size: usize, memory_type: MemoryType, usage: BufferUsage) -> Result<u64, String> {
            if size == 0 {
                return Err("Cannot allocate zero bytes".to_string());
            }


            let aligned_size = (size + 255) & !255;

            if self.total_allocated + aligned_size > self.max_memory {
                return Err(format!(
                    "Out of memory: requested {} bytes, {} available",
                    aligned_size,
                    self.max_memory - self.total_allocated
                ));
            }

            let id = self.next_id;
            self.next_id += 1;

            let allocation = MemoryAllocation {
                id,
                size: aligned_size,
                memory_type,
                usage,
                mapped: false,
            };

            self.total_allocated += aligned_size;
            match memory_type {
                MemoryType::DeviceLocal => self.device_local_used += aligned_size,
                MemoryType::HostVisible | MemoryType::HostCoherent => {
                    self.host_visible_used += aligned_size
                }
            }

            self.allocations.insert(id, allocation);
            Ok(id)
        }

        pub fn free(&mut self, id: u64) -> Result<(), String> {
            let allocation = self.allocations.remove(&id)
                .ok_or_else(|| format!("Allocation {} not found", id))?;

            if allocation.mapped {
                return Err("Cannot free mapped memory".to_string());
            }

            self.total_allocated -= allocation.size;
            match allocation.memory_type {
                MemoryType::DeviceLocal => self.device_local_used -= allocation.size,
                MemoryType::HostVisible | MemoryType::HostCoherent => {
                    self.host_visible_used -= allocation.size
                }
            }

            Ok(())
        }

        pub fn map(&mut self, id: u64) -> Result<*mut u8, String> {
            let allocation = self.allocations.get_mut(&id)
                .ok_or_else(|| format!("Allocation {} not found", id))?;

            if allocation.memory_type == MemoryType::DeviceLocal {
                return Err("Cannot map device-local memory".to_string());
            }

            if allocation.mapped {
                return Err("Memory already mapped".to_string());
            }

            allocation.mapped = true;

            Ok(allocation.size as *mut u8)
        }

        pub fn unmap(&mut self, id: u64) -> Result<(), String> {
            let allocation = self.allocations.get_mut(&id)
                .ok_or_else(|| format!("Allocation {} not found", id))?;

            if !allocation.mapped {
                return Err("Memory not mapped".to_string());
            }

            allocation.mapped = false;
            Ok(())
        }

        pub fn get_allocation(&self, id: u64) -> Option<&MemoryAllocation> {
            self.allocations.get(&id)
        }

        pub fn total_allocated(&self) -> usize {
            self.total_allocated
        }

        pub fn available_memory(&self) -> usize {
            self.max_memory - self.total_allocated
        }

        pub fn allocation_count(&self) -> usize {
            self.allocations.len()
        }

        pub fn device_local_used(&self) -> usize {
            self.device_local_used
        }

        pub fn host_visible_used(&self) -> usize {
            self.host_visible_used
        }
    }


    pub struct MemoryPool {
        allocator: MockMemoryAllocator,
        pool_allocations: Vec<u64>,
        block_size: usize,
        blocks_per_pool: usize,
    }

    impl MemoryPool {
        pub fn new(max_memory: usize, block_size: usize, blocks_per_pool: usize) -> Self {
            Self {
                allocator: MockMemoryAllocator::new(max_memory),
                pool_allocations: Vec::new(),
                block_size,
                blocks_per_pool,
            }
        }

        pub fn allocate_pool(&mut self, memory_type: MemoryType) -> Result<(), String> {
            let pool_size = self.block_size * self.blocks_per_pool;
            let id = self.allocator.allocate(pool_size, memory_type, BufferUsage::Storage)?;
            self.pool_allocations.push(id);
            Ok(())
        }

        pub fn pool_count(&self) -> usize {
            self.pool_allocations.len()
        }

        pub fn total_pool_memory(&self) -> usize {
            self.pool_allocations.len() * self.block_size * self.blocks_per_pool
        }
    }


    #[derive(Debug, Clone)]
    pub struct TextureAllocation {
        pub id: u64,
        pub width: u32,
        pub height: u32,
        pub format: TextureFormat,
        pub mip_levels: u32,
        pub memory_size: usize,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum TextureFormat {
        RGBA8,
        RGBA16F,
        RGBA32F,
        R8,
        RG8,
        Depth24Stencil8,
    }

    impl TextureFormat {
        pub fn bytes_per_pixel(&self) -> usize {
            match self {
                TextureFormat::RGBA8 => 4,
                TextureFormat::RGBA16F => 8,
                TextureFormat::RGBA32F => 16,
                TextureFormat::R8 => 1,
                TextureFormat::RG8 => 2,
                TextureFormat::Depth24Stencil8 => 4,
            }
        }
    }

    pub fn calculate_texture_size(width: u32, height: u32, format: TextureFormat, mip_levels: u32) -> usize {
        let bpp = format.bytes_per_pixel();
        let mut total = 0;
        let mut w = width;
        let mut h = height;

        for _ in 0..mip_levels {
            total += (w as usize) * (h as usize) * bpp;
            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }

        total
    }

    #[test]
    fn test_basic_allocation() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let id = allocator.allocate(1024, MemoryType::DeviceLocal, BufferUsage::Vertex).unwrap();
        assert!(id > 0);
        assert!(allocator.total_allocated() >= 1024);
    }

    #[test]
    fn test_allocation_alignment() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);


        let id = allocator.allocate(100, MemoryType::DeviceLocal, BufferUsage::Vertex).unwrap();
        let allocation = allocator.get_allocation(id).unwrap();
        assert_eq!(allocation.size, 256);
    }

    #[test]
    fn test_zero_allocation_fails() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let result = allocator.allocate(0, MemoryType::DeviceLocal, BufferUsage::Vertex);
        assert!(result.is_err());
    }

    #[test]
    fn test_out_of_memory() {
        let mut allocator = MockMemoryAllocator::new(1024);


        let result = allocator.allocate(2048, MemoryType::DeviceLocal, BufferUsage::Vertex);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Out of memory"));
    }

    #[test]
    fn test_free_allocation() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let id = allocator.allocate(1024, MemoryType::DeviceLocal, BufferUsage::Vertex).unwrap();
        let allocated_before = allocator.total_allocated();

        allocator.free(id).unwrap();
        assert!(allocator.total_allocated() < allocated_before);
        assert!(allocator.get_allocation(id).is_none());
    }

    #[test]
    fn test_double_free_fails() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let id = allocator.allocate(1024, MemoryType::DeviceLocal, BufferUsage::Vertex).unwrap();
        allocator.free(id).unwrap();

        let result = allocator.free(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_host_visible_memory() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let id = allocator.allocate(1024, MemoryType::HostVisible, BufferUsage::Uniform).unwrap();
        let ptr = allocator.map(id).unwrap();
        assert!(!ptr.is_null());

        allocator.unmap(id).unwrap();
    }

    #[test]
    fn test_map_device_local_fails() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let id = allocator.allocate(1024, MemoryType::DeviceLocal, BufferUsage::Vertex).unwrap();
        let result = allocator.map(id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("device-local"));
    }

    #[test]
    fn test_double_map_fails() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let id = allocator.allocate(1024, MemoryType::HostVisible, BufferUsage::Uniform).unwrap();
        allocator.map(id).unwrap();

        let result = allocator.map(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_free_mapped_memory_fails() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let id = allocator.allocate(1024, MemoryType::HostVisible, BufferUsage::Uniform).unwrap();
        allocator.map(id).unwrap();

        let result = allocator.free(id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mapped"));
    }

    #[test]
    fn test_memory_type_tracking() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        allocator.allocate(1024, MemoryType::DeviceLocal, BufferUsage::Vertex).unwrap();
        allocator.allocate(2048, MemoryType::HostVisible, BufferUsage::Uniform).unwrap();

        assert!(allocator.device_local_used() >= 1024);
        assert!(allocator.host_visible_used() >= 2048);
    }

    #[test]
    fn test_memory_pool() {
        let mut pool = MemoryPool::new(1024 * 1024, 4096, 16);

        pool.allocate_pool(MemoryType::DeviceLocal).unwrap();
        assert_eq!(pool.pool_count(), 1);
        assert_eq!(pool.total_pool_memory(), 4096 * 16);
    }

    #[test]
    fn test_texture_size_calculation() {

        let size = calculate_texture_size(1024, 1024, TextureFormat::RGBA8, 1);
        assert_eq!(size, 1024 * 1024 * 4);


        let size_mips = calculate_texture_size(1024, 1024, TextureFormat::RGBA8, 11);
        assert!(size_mips > size);
    }

    #[test]
    fn test_texture_format_sizes() {
        assert_eq!(TextureFormat::RGBA8.bytes_per_pixel(), 4);
        assert_eq!(TextureFormat::RGBA16F.bytes_per_pixel(), 8);
        assert_eq!(TextureFormat::RGBA32F.bytes_per_pixel(), 16);
        assert_eq!(TextureFormat::R8.bytes_per_pixel(), 1);
    }

    #[test]
    fn test_multiple_allocations() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let mut ids = Vec::new();
        for _ in 0..10 {
            let id = allocator.allocate(1024, MemoryType::DeviceLocal, BufferUsage::Vertex).unwrap();
            ids.push(id);
        }

        assert_eq!(allocator.allocation_count(), 10);


        for id in ids.iter().take(5) {
            allocator.free(*id).unwrap();
        }

        assert_eq!(allocator.allocation_count(), 5);
    }

    #[test]
    fn test_available_memory() {
        let mut allocator = MockMemoryAllocator::new(1024 * 1024);

        let initial = allocator.available_memory();
        allocator.allocate(1024, MemoryType::DeviceLocal, BufferUsage::Vertex).unwrap();

        assert!(allocator.available_memory() < initial);
    }
}
