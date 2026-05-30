//! Worker Thread Utilities
//! 
//! Provides worker thread management and parallel processing utilities.

use std::sync::{Arc, Mutex, Condvar, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Worker thread state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkerState {
    Idle,
    Working,
    Paused,
    Stopped,
}

/// Worker thread with controllable lifecycle
pub struct Worker {
    handle: Option<JoinHandle<()>>,
    state: Arc<Mutex<WorkerState>>,
    should_stop: Arc<AtomicBool>,
    should_pause: Arc<AtomicBool>,
    pause_condvar: Arc<Condvar>,
    name: String,
}

impl Worker {
    /// Create and start a new worker
    pub fn spawn<F>(name: &str, work: F) -> Self
    where
        F: FnMut() -> bool + Send + 'static,
    {
        let state = Arc::new(Mutex::new(WorkerState::Idle));
        let should_stop = Arc::new(AtomicBool::new(false));
        let should_pause = Arc::new(AtomicBool::new(false));
        let pause_condvar = Arc::new(Condvar::new());

        let state_clone = Arc::clone(&state);
        let should_stop_clone = Arc::clone(&should_stop);
        let should_pause_clone = Arc::clone(&should_pause);
        let pause_condvar_clone = Arc::clone(&pause_condvar);

        let handle = thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let mut work = work;
                
                loop {
                    // Check for stop
                    if should_stop_clone.load(Ordering::Acquire) {
                        *state_clone.lock().unwrap() = WorkerState::Stopped;
                        break;
                    }

                    // Check for pause
                    {
                        let mut state = state_clone.lock().unwrap();
                        while should_pause_clone.load(Ordering::Acquire) {
                            *state = WorkerState::Paused;
                            state = pause_condvar_clone.wait(state).unwrap();
                        }
                    }

                    // Do work
                    {
                        *state_clone.lock().unwrap() = WorkerState::Working;
                    }

                    let should_continue = work();

                    {
                        *state_clone.lock().unwrap() = WorkerState::Idle;
                    }

                    if !should_continue {
                        *state_clone.lock().unwrap() = WorkerState::Stopped;
                        break;
                    }
                }
            })
            .expect("Failed to spawn worker thread");

        Self {
            handle: Some(handle),
            state,
            should_stop,
            should_pause,
            pause_condvar,
            name: name.to_string(),
        }
    }

    /// Get current worker state
    pub fn state(&self) -> WorkerState {
        *self.state.lock().unwrap()
    }

    /// Pause the worker
    pub fn pause(&self) {
        self.should_pause.store(true, Ordering::Release);
    }

    /// Resume the worker
    pub fn resume(&self) {
        self.should_pause.store(false, Ordering::Release);
        self.pause_condvar.notify_one();
    }

    /// Stop the worker
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::Release);
        self.resume(); // Wake up if paused
    }

    /// Wait for the worker to finish
    pub fn join(mut self) -> thread::Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.join()
        } else {
            Ok(())
        }
    }

    /// Check if worker is running
    pub fn is_running(&self) -> bool {
        matches!(self.state(), WorkerState::Idle | WorkerState::Working | WorkerState::Paused)
    }

    /// Get worker name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Worker pool with dynamic sizing
pub struct WorkerPool {
    workers: Mutex<Vec<Worker>>,
    min_workers: usize,
    max_workers: usize,
    active_count: AtomicUsize,
    total_tasks: AtomicUsize,
}

impl WorkerPool {
    pub fn new(min_workers: usize, max_workers: usize) -> Self {
        Self {
            workers: Mutex::new(Vec::new()),
            min_workers,
            max_workers,
            active_count: AtomicUsize::new(0),
            total_tasks: AtomicUsize::new(0),
        }
    }

    /// Add a worker to the pool
    pub fn add_worker<F>(&self, name: &str, work: F)
    where
        F: FnMut() -> bool + Send + 'static,
    {
        let mut workers = self.workers.lock().unwrap();
        if workers.len() < self.max_workers {
            workers.push(Worker::spawn(name, work));
        }
    }

    /// Get number of workers
    pub fn worker_count(&self) -> usize {
        self.workers.lock().unwrap().len()
    }

    /// Get number of active workers
    pub fn active_count(&self) -> usize {
        self.workers.lock().unwrap()
            .iter()
            .filter(|w| w.state() == WorkerState::Working)
            .count()
    }

    /// Pause all workers
    pub fn pause_all(&self) {
        let workers = self.workers.lock().unwrap();
        for worker in workers.iter() {
            worker.pause();
        }
    }

    /// Resume all workers
    pub fn resume_all(&self) {
        let workers = self.workers.lock().unwrap();
        for worker in workers.iter() {
            worker.resume();
        }
    }

    /// Stop all workers
    pub fn stop_all(&self) {
        let workers = self.workers.lock().unwrap();
        for worker in workers.iter() {
            worker.stop();
        }
    }

    /// Remove stopped workers
    pub fn cleanup(&self) {
        let mut workers = self.workers.lock().unwrap();
        workers.retain(|w| w.is_running());
    }
}

/// Parallel work distributor
pub struct ParallelWork<T, R> {
    items: Vec<T>,
    results: Arc<Mutex<Vec<Option<R>>>>,
    next_index: AtomicUsize,
}

impl<T: Send + 'static, R: Send + 'static> ParallelWork<T, R> {
    pub fn new(items: Vec<T>) -> Self {
        let len = items.len();
        Self {
            items,
            results: Arc::new(Mutex::new(vec![None; len])),
            next_index: AtomicUsize::new(0),
        }
    }

    /// Process items in parallel using the provided function
    pub fn process<F>(self, worker_count: usize, f: F) -> Vec<R>
    where
        F: Fn(T) -> R + Send + Sync + Clone + 'static,
    {
        let items = Arc::new(Mutex::new(self.items));
        let results = self.results;
        let next_index = Arc::new(self.next_index);
        let f = Arc::new(f);

        let mut handles = Vec::new();

        for _ in 0..worker_count {
            let items = Arc::clone(&items);
            let results = Arc::clone(&results);
            let next_index = Arc::clone(&next_index);
            let f = Arc::clone(&f);

            let handle = thread::spawn(move || {
                loop {
                    let index = next_index.fetch_add(1, Ordering::AcqRel);
                    
                    let item = {
                        let mut items = items.lock().unwrap();
                        if index >= items.len() {
                            break;
                        }
                        // Take item by swapping with a placeholder
                        let mut placeholder = unsafe { std::mem::zeroed() };
                        std::mem::swap(&mut items[index], &mut placeholder);
                        placeholder
                    };

                    let result = f(item);

                    {
                        let mut results = results.lock().unwrap();
                        results[index] = Some(result);
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        Arc::try_unwrap(results)
            .expect("Results still referenced")
            .into_inner()
            .unwrap()
            .into_iter()
            .map(|r| r.expect("Missing result"))
            .collect()
    }
}

/// Rate limiter for controlling work rate
pub struct RateLimiter {
    interval: Duration,
    last_tick: Mutex<Instant>,
    count: AtomicUsize,
    max_per_interval: usize,
}

impl RateLimiter {
    pub fn new(max_per_interval: usize, interval: Duration) -> Self {
        Self {
            interval,
            last_tick: Mutex::new(Instant::now()),
            count: AtomicUsize::new(0),
            max_per_interval,
        }
    }

    /// Try to acquire a permit
    pub fn try_acquire(&self) -> bool {
        let mut last_tick = self.last_tick.lock().unwrap();
        let now = Instant::now();

        if now.duration_since(*last_tick) >= self.interval {
            *last_tick = now;
            self.count.store(0, Ordering::Release);
        }

        let current = self.count.fetch_add(1, Ordering::AcqRel);
        current < self.max_per_interval
    }

    /// Wait until a permit is available
    pub fn acquire(&self) {
        while !self.try_acquire() {
            thread::sleep(Duration::from_millis(1));
        }
    }

    /// Get remaining permits
    pub fn remaining(&self) -> usize {
        let current = self.count.load(Ordering::Acquire);
        self.max_per_interval.saturating_sub(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_lifecycle() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let worker = Worker::spawn("test", move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(10));
            counter_clone.load(Ordering::Relaxed) < 5
        });

        thread::sleep(Duration::from_millis(100));
        
        assert!(counter.load(Ordering::Relaxed) >= 1);
        
        worker.stop();
    }

    #[test]
    fn test_worker_pause_resume() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let worker = Worker::spawn("test", move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(10));
            true
        });

        thread::sleep(Duration::from_millis(50));
        let count_before_pause = counter.load(Ordering::Relaxed);
        
        worker.pause();
        thread::sleep(Duration::from_millis(50));
        let count_during_pause = counter.load(Ordering::Relaxed);
        
        // Count should not increase much during pause
        assert!(count_during_pause <= count_before_pause + 1);
        
        worker.resume();
        thread::sleep(Duration::from_millis(50));
        let count_after_resume = counter.load(Ordering::Relaxed);
        
        assert!(count_after_resume > count_during_pause);
        
        worker.stop();
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, Duration::from_millis(100));

        // Should allow 5 acquires
        for _ in 0..5 {
            assert!(limiter.try_acquire());
        }

        // 6th should fail
        assert!(!limiter.try_acquire());

        // Wait for interval to reset
        thread::sleep(Duration::from_millis(110));
        
        assert!(limiter.try_acquire());
    }

    #[test]
    fn test_worker_pool() {
        let pool = WorkerPool::new(2, 4);

        let counter = Arc::new(AtomicUsize::new(0));
        
        for i in 0..3 {
            let counter = Arc::clone(&counter);
            pool.add_worker(&format!("worker-{}", i), move || {
                counter.fetch_add(1, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(10));
                counter.load(Ordering::Relaxed) < 10
            });
        }

        assert_eq!(pool.worker_count(), 3);
        
        thread::sleep(Duration::from_millis(50));
        pool.stop_all();
    }
}
