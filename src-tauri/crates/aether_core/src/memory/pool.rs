use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct ObjectPool<T> {
    objects: Mutex<VecDeque<T>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
    stats: Mutex<PoolStats>,
}

#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    pub allocations: u64,
    pub reuses: u64,
    pub returns: u64,
    pub current_size: usize,
}

impl PoolStats {
    pub fn reuse_rate(&self) -> f64 {
        let total = self.allocations + self.reuses;
        if total == 0 {
            return 0.0;
        }
        self.reuses as f64 / total as f64
    }
}

impl<T> ObjectPool<T> {

    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            objects: Mutex::new(VecDeque::with_capacity(max_size)),
            factory: Box::new(factory),
            max_size,
            stats: Mutex::new(PoolStats::default()),
        }
    }

    /// Pre-populate the pool with objects
    pub fn warm(&self, count: usize) {
        let count = count.min(self.max_size);
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        for _ in objects.len()..count {
            objects.push_back((self.factory)());
            stats.allocations += 1;
        }
        stats.current_size = objects.len();
    }

    /// Acquire an object from the pool
    pub fn acquire(&self) -> PooledObject<T> {
        let object = {
            let mut objects = self.objects.lock().unwrap();
            let mut stats = self.stats.lock().unwrap();

            if let Some(obj) = objects.pop_front() {
                stats.reuses += 1;
                stats.current_size = objects.len();
                obj
            } else {
                stats.allocations += 1;
                (self.factory)()
            }
        };

        PooledObject {
            object: Some(object),
            pool: self,
        }
    }

    /// Return an object to the pool
    fn release(&self, object: T) {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if objects.len() < self.max_size {
            objects.push_back(object);
            stats.returns += 1;
            stats.current_size = objects.len();
        }
        // Otherwise, object is dropped
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// Clear all pooled objects
    pub fn clear(&self) {
        let mut objects = self.objects.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        objects.clear();
        stats.current_size = 0;
    }

    /// Get current pool size
    pub fn size(&self) -> usize {
        self.objects.lock().unwrap().len()
    }
}

/// A pooled object that returns to the pool when dropped
pub struct PooledObject<'a, T> {
    object: Option<T>,
    pool: &'a ObjectPool<T>,
}

impl<'a, T> PooledObject<'a, T> {
    /// Take ownership of the object (won't return to pool)
    pub fn take(mut self) -> T {
        self.object.take().expect("Object already taken")
    }
}

impl<'a, T> std::ops::Deref for PooledObject<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().expect("Object already taken")
    }
}

impl<'a, T> std::ops::DerefMut for PooledObject<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().expect("Object already taken")
    }
}

impl<'a, T> Drop for PooledObject<'a, T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            self.pool.release(object);
        }
    }
}


pub struct TypedBufferPool {
    pools: Vec<(usize, Arc<ObjectPool<Vec<u8>>>)>,
}

impl TypedBufferPool {

    pub fn new(sizes: &[usize], max_per_size: usize) -> Self {
        let pools = sizes.iter()
            .map(|&size| {
                let pool = ObjectPool::new(move || vec![0u8; size], max_per_size);
                (size, Arc::new(pool))
            })
            .collect();

        Self { pools }
    }


    pub fn power_of_two(min_size: usize, max_size: usize, max_per_size: usize) -> Self {
        let mut sizes = Vec::new();
        let mut size = min_size.next_power_of_two();

        while size <= max_size {
            sizes.push(size);
            size *= 2;
        }

        Self::new(&sizes, max_per_size)
    }


    pub fn acquire(&self, min_size: usize) -> Option<TypedPooledBuffer> {
        for (size, pool) in &self.pools {
            if *size >= min_size {
                let buffer = pool.acquire();
                return Some(TypedPooledBuffer {
                    buffer: Some(buffer.take()),
                    pool: Arc::clone(pool),
                    requested_size: min_size,
                });
            }
        }
        None
    }


    pub fn total_stats(&self) -> PoolStats {
        let mut total = PoolStats::default();
        for (_, pool) in &self.pools {
            let stats = pool.stats();
            total.allocations += stats.allocations;
            total.reuses += stats.reuses;
            total.returns += stats.returns;
            total.current_size += stats.current_size;
        }
        total
    }
}


pub struct TypedPooledBuffer {
    buffer: Option<Vec<u8>>,
    pool: Arc<ObjectPool<Vec<u8>>>,
    requested_size: usize,
}

impl TypedPooledBuffer {
    pub fn len(&self) -> usize {
        self.requested_size
    }

    pub fn capacity(&self) -> usize {
        self.buffer.as_ref().map(|b| b.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.requested_size == 0
    }
}

impl std::ops::Deref for TypedPooledBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer.as_ref().expect("Buffer taken")[..self.requested_size]
    }
}

impl std::ops::DerefMut for TypedPooledBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let size = self.requested_size;
        &mut self.buffer.as_mut().expect("Buffer taken")[..size]
    }
}

impl Drop for TypedPooledBuffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            self.pool.release(buffer);
        }
    }
}


pub struct FrameBufferPool {
    width: u32,
    height: u32,
    channels: u32,
    pool: ObjectPool<Vec<u8>>,
}

impl FrameBufferPool {
    pub fn new(width: u32, height: u32, channels: u32, max_frames: usize) -> Self {
        let frame_size = (width * height * channels) as usize;
        let pool = ObjectPool::new(move || vec![0u8; frame_size], max_frames);

        Self {
            width,
            height,
            channels,
            pool,
        }
    }

    pub fn acquire(&self) -> PooledObject<Vec<u8>> {
        self.pool.acquire()
    }

    pub fn frame_size(&self) -> usize {
        (self.width * self.height * self.channels) as usize
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn stats(&self) -> PoolStats {
        self.pool.stats()
    }

    pub fn warm(&self, count: usize) {
        self.pool.warm(count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_basic() {
        let pool = ObjectPool::new(|| vec![0u8; 1024], 10);

        let obj1 = pool.acquire();
        assert_eq!(obj1.len(), 1024);

        drop(obj1);

        let obj2 = pool.acquire();
        assert_eq!(obj2.len(), 1024);

        let stats = pool.stats();
        assert_eq!(stats.allocations, 1);
        assert_eq!(stats.reuses, 1);
    }

    #[test]
    fn test_object_pool_warm() {
        let pool = ObjectPool::new(|| vec![0u8; 1024], 10);
        pool.warm(5);

        assert_eq!(pool.size(), 5);

        let stats = pool.stats();
        assert_eq!(stats.allocations, 5);
    }

    #[test]
    fn test_object_pool_max_size() {
        let pool = ObjectPool::new(|| vec![0u8; 1024], 2);

        let obj1 = pool.acquire();
        let obj2 = pool.acquire();
        let obj3 = pool.acquire();

        drop(obj1);
        drop(obj2);
        drop(obj3);


        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_typed_buffer_pool() {
        let pool = TypedBufferPool::power_of_two(256, 4096, 10);

        let buf = pool.acquire(500).unwrap();
        assert!(buf.capacity() >= 500);
        assert_eq!(buf.len(), 500);
    }

    #[test]
    fn test_frame_buffer_pool() {
        let pool = FrameBufferPool::new(1920, 1080, 4, 5);

        assert_eq!(pool.frame_size(), 1920 * 1080 * 4);
        assert_eq!(pool.dimensions(), (1920, 1080));

        let frame = pool.acquire();
        assert_eq!(frame.len(), 1920 * 1080 * 4);
    }

    #[test]
    fn test_pool_reuse_rate() {
        let pool = ObjectPool::new(|| vec![0u8; 1024], 10);


        let obj = pool.acquire();
        drop(obj);


        let _obj = pool.acquire();

        let stats = pool.stats();
        assert_eq!(stats.reuse_rate(), 0.5);
    }
}
