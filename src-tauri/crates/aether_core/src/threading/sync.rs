use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock, Mutex, Condvar};
use std::collections::VecDeque;
use std::time::Duration;

/// A thread-safe double buffer for producer-consumer patterns
pub struct DoubleBuffer<T> {
    buffers: [RwLock<T>; 2],
    active: AtomicUsize,
}

impl<T: Default> DoubleBuffer<T> {
    pub fn new() -> Self {
        Self {
            buffers: [RwLock::new(T::default()), RwLock::new(T::default())],
            active: AtomicUsize::new(0),
        }
    }
}

impl<T> DoubleBuffer<T> {
    pub fn with_values(front: T, back: T) -> Self {
        Self {
            buffers: [RwLock::new(front), RwLock::new(back)],
            active: AtomicUsize::new(0),
        }
    }

    /// Get read access to the front buffer
    pub fn read_front(&self) -> std::sync::RwLockReadGuard<T> {
        let active = self.active.load(Ordering::Acquire);
        self.buffers[active].read().unwrap()
    }

    /// Get write access to the back buffer
    pub fn write_back(&self) -> std::sync::RwLockWriteGuard<T> {
        let active = self.active.load(Ordering::Acquire);
        let back = 1 - active;
        self.buffers[back].write().unwrap()
    }

    /// Swap front and back buffers
    pub fn swap(&self) {
        let current = self.active.load(Ordering::Acquire);
        self.active.store(1 - current, Ordering::Release);
    }
}

impl<T: Default> Default for DoubleBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A thread-safe ring buffer
pub struct RingBuffer<T> {
    buffer: Vec<Mutex<Option<T>>>,
    capacity: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
    count: AtomicUsize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let buffer = (0..capacity)
            .map(|_| Mutex::new(None))
            .collect();

        Self {
            buffer,
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        if self.count.load(Ordering::Acquire) >= self.capacity {
            return Err(value);
        }

        let tail = self.tail.fetch_add(1, Ordering::AcqRel) % self.capacity;
        let mut slot = self.buffer[tail].lock().unwrap();
        *slot = Some(value);
        self.count.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        if self.count.load(Ordering::Acquire) == 0 {
            return None;
        }

        let head = self.head.fetch_add(1, Ordering::AcqRel) % self.capacity;
        let mut slot = self.buffer[head].lock().unwrap();
        let value = slot.take();
        if value.is_some() {
            self.count.fetch_sub(1, Ordering::Release);
        }
        value
    }

    pub fn len(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// A blocking queue with timeout support
pub struct BlockingQueue<T> {
    queue: Mutex<VecDeque<T>>,
    not_empty: Condvar,
    not_full: Condvar,
    capacity: usize,
    closed: AtomicBool,
}

impl<T> BlockingQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
            closed: AtomicBool::new(false),
        }
    }

    pub fn unbounded() -> Self {
        Self::new(usize::MAX)
    }

    /// Push an item, blocking if full
    pub fn push(&self, value: T) -> Result<(), T> {
        if self.closed.load(Ordering::Acquire) {
            return Err(value);
        }

        let mut queue = self.queue.lock().unwrap();
        
        while queue.len() >= self.capacity && !self.closed.load(Ordering::Acquire) {
            queue = self.not_full.wait(queue).unwrap();
        }

        if self.closed.load(Ordering::Acquire) {
            return Err(value);
        }

        queue.push_back(value);
        self.not_empty.notify_one();
        Ok(())
    }

    /// Push with timeout
    pub fn push_timeout(&self, value: T, timeout: Duration) -> Result<(), T> {
        if self.closed.load(Ordering::Acquire) {
            return Err(value);
        }

        let mut queue = self.queue.lock().unwrap();
        
        while queue.len() >= self.capacity && !self.closed.load(Ordering::Acquire) {
            let (new_queue, result) = self.not_full.wait_timeout(queue, timeout).unwrap();
            queue = new_queue;
            if result.timed_out() {
                return Err(value);
            }
        }

        if self.closed.load(Ordering::Acquire) {
            return Err(value);
        }

        queue.push_back(value);
        self.not_empty.notify_one();
        Ok(())
    }

    /// Pop an item, blocking if empty
    pub fn pop(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        
        while queue.is_empty() && !self.closed.load(Ordering::Acquire) {
            queue = self.not_empty.wait(queue).unwrap();
        }

        let value = queue.pop_front();
        if value.is_some() {
            self.not_full.notify_one();
        }
        value
    }

    /// Pop with timeout
    pub fn pop_timeout(&self, timeout: Duration) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        
        while queue.is_empty() && !self.closed.load(Ordering::Acquire) {
            let (new_queue, result) = self.not_empty.wait_timeout(queue, timeout).unwrap();
            queue = new_queue;
            if result.timed_out() {
                return None;
            }
        }

        let value = queue.pop_front();
        if value.is_some() {
            self.not_full.notify_one();
        }
        value
    }

    /// Try to pop without blocking
    pub fn try_pop(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        let value = queue.pop_front();
        if value.is_some() {
            self.not_full.notify_one();
        }
        value
    }

    /// Close the queue
    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }
}

/// A read-write lock with upgrade capability
pub struct UpgradableRwLock<T> {
    inner: RwLock<T>,
    upgrade_lock: Mutex<()>,
}

impl<T> UpgradableRwLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: RwLock::new(value),
            upgrade_lock: Mutex::new(()),
        }
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<T> {
        self.inner.read().unwrap()
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<T> {
        self.inner.write().unwrap()
    }

    /// Acquire a read lock that can be upgraded to write
    pub fn upgradable_read(&self) -> UpgradableReadGuard<T> {
        let _upgrade = self.upgrade_lock.lock().unwrap();
        UpgradableReadGuard {
            lock: self,
            _upgrade,
        }
    }
}

pub struct UpgradableReadGuard<'a, T> {
    lock: &'a UpgradableRwLock<T>,
    _upgrade: std::sync::MutexGuard<'a, ()>,
}

impl<'a, T> UpgradableReadGuard<'a, T> {
    pub fn read(&self) -> std::sync::RwLockReadGuard<T> {
        self.lock.inner.read().unwrap()
    }

    pub fn upgrade(self) -> std::sync::RwLockWriteGuard<'a, T> {
        self.lock.inner.write().unwrap()
    }
}

/// Atomic counter with statistics
pub struct AtomicCounter {
    value: AtomicU64,
    min: AtomicU64,
    max: AtomicU64,
    sum: AtomicU64,
    count: AtomicU64,
}

impl AtomicCounter {
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Release);
        
        // Update min
        let mut current_min = self.min.load(Ordering::Acquire);
        while value < current_min {
            match self.min.compare_exchange_weak(
                current_min, value, Ordering::AcqRel, Ordering::Acquire
            ) {
                Ok(_) => break,
                Err(m) => current_min = m,
            }
        }

        // Update max
        let mut current_max = self.max.load(Ordering::Acquire);
        while value > current_max {
            match self.max.compare_exchange_weak(
                current_max, value, Ordering::AcqRel, Ordering::Acquire
            ) {
                Ok(_) => break,
                Err(m) => current_max = m,
            }
        }

        self.sum.fetch_add(value, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment(&self) -> u64 {
        let new_value = self.value.fetch_add(1, Ordering::AcqRel) + 1;
        self.set(new_value);
        new_value
    }

    pub fn decrement(&self) -> u64 {
        let new_value = self.value.fetch_sub(1, Ordering::AcqRel) - 1;
        self.set(new_value);
        new_value
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Acquire)
    }

    pub fn min(&self) -> u64 {
        let min = self.min.load(Ordering::Acquire);
        if min == u64::MAX { 0 } else { min }
    }

    pub fn max(&self) -> u64 {
        self.max.load(Ordering::Acquire)
    }

    pub fn average(&self) -> f64 {
        let count = self.count.load(Ordering::Acquire);
        if count == 0 {
            return 0.0;
        }
        self.sum.load(Ordering::Acquire) as f64 / count as f64
    }

    pub fn reset(&self) {
        self.value.store(0, Ordering::Release);
        self.min.store(u64::MAX, Ordering::Release);
        self.max.store(0, Ordering::Release);
        self.sum.store(0, Ordering::Release);
        self.count.store(0, Ordering::Release);
    }
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_double_buffer() {
        let buffer: DoubleBuffer<i32> = DoubleBuffer::with_values(1, 2);

        assert_eq!(*buffer.read_front(), 1);
        
        *buffer.write_back() = 3;
        buffer.swap();
        
        assert_eq!(*buffer.read_front(), 3);
    }

    #[test]
    fn test_ring_buffer() {
        let buffer = RingBuffer::new(3);

        assert!(buffer.push(1).is_ok());
        assert!(buffer.push(2).is_ok());
        assert!(buffer.push(3).is_ok());
        assert!(buffer.push(4).is_err()); // Full

        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.pop(), Some(2));
        assert!(buffer.push(4).is_ok());
    }

    #[test]
    fn test_blocking_queue() {
        let queue = Arc::new(BlockingQueue::new(10));
        let queue2 = Arc::clone(&queue);

        let producer = thread::spawn(move || {
            for i in 0..5 {
                queue2.push(i).unwrap();
            }
        });

        producer.join().unwrap();

        for i in 0..5 {
            assert_eq!(queue.try_pop(), Some(i));
        }
    }

    #[test]
    fn test_blocking_queue_close() {
        let queue = Arc::new(BlockingQueue::<i32>::new(10));
        let queue2 = Arc::clone(&queue);

        let consumer = thread::spawn(move || {
            queue2.pop()
        });

        thread::sleep(Duration::from_millis(10));
        queue.close();

        let result = consumer.join().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new();

        counter.set(10);
        counter.set(5);
        counter.set(15);

        assert_eq!(counter.min(), 5);
        assert_eq!(counter.max(), 15);
        assert_eq!(counter.average(), 10.0);
    }
}
