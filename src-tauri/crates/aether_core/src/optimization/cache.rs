use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};


#[derive(Debug)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    size_bytes: usize,
}

impl<T> CacheEntry<T> {
    fn new(value: T, size_bytes: usize) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            size_bytes,
        }
    }

    fn access(&mut self) -> &T {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.value
    }

    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed()
    }
}


pub struct LruCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    order: VecDeque<K>,
    max_entries: usize,
    max_size_bytes: usize,
    current_size_bytes: usize,
    hits: u64,
    misses: u64,
}

impl<K: Clone + Eq + std::hash::Hash, V> LruCache<K, V> {
    pub fn new(max_entries: usize, max_size_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            max_entries,
            max_size_bytes,
            current_size_bytes: 0,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.entries.contains_key(key) {
            self.hits += 1;

            self.order.retain(|k| k != key);
            self.order.push_front(key.clone());
            self.entries.get_mut(key).map(|e| e.access())
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V, size_bytes: usize) {

        if let Some(existing) = self.entries.remove(&key) {
            self.current_size_bytes -= existing.size_bytes;
            self.order.retain(|k| k != &key);
        }


        while self.entries.len() >= self.max_entries ||
              self.current_size_bytes + size_bytes > self.max_size_bytes {
            if let Some(old_key) = self.order.pop_back() {
                if let Some(entry) = self.entries.remove(&old_key) {
                    self.current_size_bytes -= entry.size_bytes;
                }
            } else {
                break;
            }
        }


        self.entries.insert(key.clone(), CacheEntry::new(value, size_bytes));
        self.order.push_front(key);
        self.current_size_bytes += size_bytes;
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.remove(key) {
            self.current_size_bytes -= entry.size_bytes;
            self.order.retain(|k| k != key);
            Some(entry.value)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.current_size_bytes = 0;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn size_bytes(&self) -> usize {
        self.current_size_bytes
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f64 / total as f64
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.entries.len(),
            size_bytes: self.current_size_bytes,
            max_entries: self.max_entries,
            max_size_bytes: self.max_size_bytes,
            hits: self.hits,
            misses: self.misses,
            hit_rate: self.hit_rate(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub size_bytes: usize,
    pub max_entries: usize,
    pub max_size_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}


pub struct BufferPool {
    pools: RwLock<HashMap<usize, Mutex<Vec<Vec<u8>>>>>,
    max_buffers_per_size: usize,
    total_allocated: Mutex<usize>,
    total_reused: Mutex<usize>,
}

impl BufferPool {
    pub fn new(max_buffers_per_size: usize) -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            max_buffers_per_size,
            total_allocated: Mutex::new(0),
            total_reused: Mutex::new(0),
        }
    }


    pub fn acquire(&self, size: usize) -> PooledBuffer {

        let bucket_size = size.next_power_of_two();


        {
            let pools = self.pools.read().unwrap();
            if let Some(pool) = pools.get(&bucket_size) {
                let mut pool = pool.lock().unwrap();
                if let Some(mut buffer) = pool.pop() {
                    buffer.resize(size, 0);
                    *self.total_reused.lock().unwrap() += 1;
                    return PooledBuffer {
                        buffer,
                        bucket_size,
                        pool: Some(self),
                    };
                }
            }
        }


        *self.total_allocated.lock().unwrap() += 1;
        PooledBuffer {
            buffer: vec![0u8; size],
            bucket_size,
            pool: Some(self),
        }
    }


    fn release(&self, buffer: Vec<u8>, bucket_size: usize) {
        let mut pools = self.pools.write().unwrap();
        let pool = pools.entry(bucket_size).or_insert_with(|| Mutex::new(Vec::new()));
        let mut pool = pool.lock().unwrap();

        if pool.len() < self.max_buffers_per_size {
            pool.push(buffer);
        }

    }

    pub fn stats(&self) -> BufferPoolStats {
        let pools = self.pools.read().unwrap();
        let pooled_buffers: usize = pools.values()
            .map(|p| p.lock().unwrap().len())
            .sum();
        let pooled_bytes: usize = pools.iter()
            .map(|(size, p)| size * p.lock().unwrap().len())
            .sum();

        BufferPoolStats {
            total_allocated: *self.total_allocated.lock().unwrap(),
            total_reused: *self.total_reused.lock().unwrap(),
            pooled_buffers,
            pooled_bytes,
            bucket_count: pools.len(),
        }
    }

    pub fn clear(&self) {
        let mut pools = self.pools.write().unwrap();
        pools.clear();
    }
}


pub struct PooledBuffer<'a> {
    buffer: Vec<u8>,
    bucket_size: usize,
    pool: Option<&'a BufferPool>,
}

impl<'a> PooledBuffer<'a> {
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }


    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    pub total_allocated: usize,
    pub total_reused: usize,
    pub pooled_buffers: usize,
    pub pooled_bytes: usize,
    pub bucket_count: usize,
}

impl BufferPoolStats {
    pub fn reuse_rate(&self) -> f64 {
        let total = self.total_allocated + self.total_reused;
        if total == 0 {
            return 0.0;
        }
        self.total_reused as f64 / total as f64
    }
}


pub struct FrameCache {
    cache: Mutex<LruCache<u64, Arc<Vec<u8>>>>,
    frame_size: usize,
}

impl FrameCache {
    pub fn new(max_frames: usize, frame_size: usize) -> Self {
        let max_bytes = max_frames * frame_size;
        Self {
            cache: Mutex::new(LruCache::new(max_frames, max_bytes)),
            frame_size,
        }
    }

    pub fn get(&self, frame_number: u64) -> Option<Arc<Vec<u8>>> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(&frame_number).cloned()
    }

    pub fn insert(&self, frame_number: u64, data: Vec<u8>) {
        let arc = Arc::new(data);
        let size = arc.len();
        let mut cache = self.cache.lock().unwrap();
        cache.insert(frame_number, arc, size);
    }

    pub fn contains(&self, frame_number: u64) -> bool {
        let mut cache = self.cache.lock().unwrap();
        cache.get(&frame_number).is_some()
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        cache.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LruCache::new(3, 1024);

        cache.insert("a", 1, 100);
        cache.insert("b", 2, 100);
        cache.insert("c", 3, 100);

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(2, 1024);

        cache.insert("a", 1, 100);
        cache.insert("b", 2, 100);
        cache.insert("c", 3, 100);

        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn test_lru_cache_access_updates_order() {
        let mut cache = LruCache::new(2, 1024);

        cache.insert("a", 1, 100);
        cache.insert("b", 2, 100);
        cache.get(&"a");
        cache.insert("c", 3, 100);

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn test_buffer_pool_basic() {
        let pool = BufferPool::new(10);

        let buffer1 = pool.acquire(1024);
        assert_eq!(buffer1.len(), 1024);

        drop(buffer1);

        let buffer2 = pool.acquire(1024);
        assert_eq!(buffer2.len(), 1024);

        let stats = pool.stats();
        assert!(stats.total_reused > 0 || stats.total_allocated > 0);
    }

    #[test]
    fn test_buffer_pool_reuse() {
        let pool = BufferPool::new(10);


        let buffer = pool.acquire(1024);
        drop(buffer);


        let _buffer = pool.acquire(1024);

        let stats = pool.stats();
        assert_eq!(stats.total_reused, 1);
    }

    #[test]
    fn test_frame_cache() {
        let cache = FrameCache::new(10, 1920 * 1080 * 4);

        let frame_data = vec![0u8; 1920 * 1080 * 4];
        cache.insert(0, frame_data);

        assert!(cache.contains(0));
        assert!(!cache.contains(1));

        let retrieved = cache.get(0);
        assert!(retrieved.is_some());
    }
}
