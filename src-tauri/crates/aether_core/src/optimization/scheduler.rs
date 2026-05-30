use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}


pub struct Task {
    pub id: u64,
    pub priority: TaskPriority,
    pub work: Box<dyn FnOnce() + Send + 'static>,
    pub created_at: Instant,
}

impl Task {
    pub fn new<F>(id: u64, priority: TaskPriority, work: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            id,
            priority,
            work: Box::new(work),
            created_at: Instant::now(),
        }
    }
}


#[derive(Debug, Clone)]
pub struct ThreadPoolStats {
    pub worker_count: usize,
    pub active_workers: usize,
    pub pending_tasks: usize,
    pub completed_tasks: u64,
    pub total_wait_time: Duration,
    pub total_execution_time: Duration,
}


struct TaskQueue {
    queues: [VecDeque<Task>; 4],
    total_count: usize,
}

impl TaskQueue {
    fn new() -> Self {
        Self {
            queues: [
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
            total_count: 0,
        }
    }

    fn push(&mut self, task: Task) {
        let priority = task.priority as usize;
        self.queues[priority].push_back(task);
        self.total_count += 1;
    }

    fn pop(&mut self) -> Option<Task> {

        for queue in self.queues.iter_mut().rev() {
            if let Some(task) = queue.pop_front() {
                self.total_count -= 1;
                return Some(task);
            }
        }
        None
    }

    fn len(&self) -> usize {
        self.total_count
    }

    fn is_empty(&self) -> bool {
        self.total_count == 0
    }
}


pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    queue: Arc<Mutex<TaskQueue>>,
    condvar: Arc<Condvar>,
    shutdown: Arc<AtomicBool>,
    active_count: Arc<AtomicUsize>,
    completed_count: Arc<AtomicUsize>,
    next_task_id: AtomicUsize,
}

impl ThreadPool {

    pub fn new(worker_count: usize) -> Self {
        let queue = Arc::new(Mutex::new(TaskQueue::new()));
        let condvar = Arc::new(Condvar::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        let active_count = Arc::new(AtomicUsize::new(0));
        let completed_count = Arc::new(AtomicUsize::new(0));

        let mut workers = Vec::with_capacity(worker_count);

        for id in 0..worker_count {
            let queue = Arc::clone(&queue);
            let condvar = Arc::clone(&condvar);
            let shutdown = Arc::clone(&shutdown);
            let active_count = Arc::clone(&active_count);
            let completed_count = Arc::clone(&completed_count);

            let handle = thread::Builder::new()
                .name(format!("worker-{}", id))
                .spawn(move || {
                    loop {
                        let task = {
                            let mut queue = queue.lock().unwrap();

                            while queue.is_empty() && !shutdown.load(Ordering::Relaxed) {
                                queue = condvar.wait(queue).unwrap();
                            }

                            if shutdown.load(Ordering::Relaxed) && queue.is_empty() {
                                break;
                            }

                            queue.pop()
                        };

                        if let Some(task) = task {
                            active_count.fetch_add(1, Ordering::Relaxed);
                            (task.work)();
                            active_count.fetch_sub(1, Ordering::Relaxed);
                            completed_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                })
                .expect("Failed to spawn worker thread");

            workers.push(handle);
        }

        Self {
            workers,
            queue,
            condvar,
            shutdown,
            active_count,
            completed_count,
            next_task_id: AtomicUsize::new(0),
        }
    }


    pub fn with_cpu_count() -> Self {
        let count = thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        Self::new(count)
    }


    pub fn submit<F>(&self, work: F) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        self.submit_with_priority(work, TaskPriority::Normal)
    }

    /// Submit a task with specified priority
    pub fn submit_with_priority<F>(&self, work: F, priority: TaskPriority) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let id = self.next_task_id.fetch_add(1, Ordering::Relaxed) as u64;
        let task = Task::new(id, priority, work);

        {
            let mut queue = self.queue.lock().unwrap();
            queue.push(task);
        }

        self.condvar.notify_one();
        id
    }


    pub fn stats(&self) -> ThreadPoolStats {
        let pending = self.queue.lock().unwrap().len();

        ThreadPoolStats {
            worker_count: self.workers.len(),
            active_workers: self.active_count.load(Ordering::Relaxed),
            pending_tasks: pending,
            completed_tasks: self.completed_count.load(Ordering::Relaxed) as u64,
            total_wait_time: Duration::ZERO,
            total_execution_time: Duration::ZERO,
        }
    }


    pub fn pending_count(&self) -> usize {
        self.queue.lock().unwrap().len()
    }


    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }


    pub fn wait_all(&self) {
        loop {
            let (pending, active) = {
                let queue = self.queue.lock().unwrap();
                (queue.len(), self.active_count.load(Ordering::Relaxed))
            };

            if pending == 0 && active == 0 {
                break;
            }

            thread::sleep(Duration::from_millis(1));
        }
    }


    pub fn shutdown(self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.condvar.notify_all();

        for worker in self.workers {
            let _ = worker.join();
        }
    }
}


pub struct WorkStealingScheduler {
    local_queues: Vec<Arc<Mutex<VecDeque<Task>>>>,
    global_queue: Arc<Mutex<VecDeque<Task>>>,
    worker_count: usize,
}

impl WorkStealingScheduler {
    pub fn new(worker_count: usize) -> Self {
        let local_queues = (0..worker_count)
            .map(|_| Arc::new(Mutex::new(VecDeque::new())))
            .collect();

        Self {
            local_queues,
            global_queue: Arc::new(Mutex::new(VecDeque::new())),
            worker_count,
        }
    }


        F: Fn(T) + Send + Sync + Clone + 'static,
    {
        let f = Arc::new(f);

        for item in items {
            let f = Arc::clone(&f);
            pool.submit(move || {
                f(item);
            });
        }

        pool.wait_all();
    }

    /// Map items in parallel and collect results
    pub fn map<T, U, F>(items: Vec<T>, pool: &ThreadPool, f: F) -> Vec<U>
    where
        T: Send + 'static,
        U: Send + 'static,
        F: Fn(T) -> U + Send + Sync + Clone + 'static,
    {
        let results = Arc::new(Mutex::new(Vec::with_capacity(items.len())));
        let f = Arc::new(f);
        let len = items.len();

        for (i, item) in items.into_iter().enumerate() {
            let f = Arc::clone(&f);
            let results = Arc::clone(&results);

            pool.submit(move || {
                let result = f(item);
                let mut results = results.lock().unwrap();

                while results.len() <= i {
                    results.push(None);
                }
                results[i] = Some(result);
            });
        }

        pool.wait_all();

        let results = Arc::try_unwrap(results)
            .expect("Results still referenced")
            .into_inner()
            .unwrap();

        results.into_iter()
            .take(len)
            .map(|r| r.expect("Missing result"))
            .collect()
    }
}


pub struct BatchProcessor {
    batch_size: usize,
    pool: ThreadPool,
}

impl BatchProcessor {
    pub fn new(batch_size: usize, worker_count: usize) -> Self {
        Self {
            batch_size,
            pool: ThreadPool::new(worker_count),
        }
    }


    pub fn process<T, F>(&self, items: Vec<T>, f: F)
    where
        T: Send + 'static,
        F: Fn(Vec<T>) + Send + Sync + Clone + 'static,
    {
        let f = Arc::new(f);
        let chunks: Vec<Vec<T>> = items
            .into_iter()
            .collect::<Vec<_>>()
            .chunks(self.batch_size)
            .map(|c| c.to_vec())
            .collect();

        for chunk in chunks {
            let f = Arc::clone(&f);
            self.pool.submit(move || {
                f(chunk);
            });
        }

        self.pool.wait_all();
    }

    pub fn shutdown(self) {
        self.pool.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn test_thread_pool_basic() {
        let pool = ThreadPool::new(4);
        let counter = Arc::new(AtomicU64::new(0));

        for _ in 0..100 {
            let counter = Arc::clone(&counter);
            pool.submit(move || {
                counter.fetch_add(1, Ordering::Relaxed);
            });
        }

        pool.wait_all();
        assert_eq!(counter.load(Ordering::Relaxed), 100);
        pool.shutdown();
    }

    #[test]
    fn test_thread_pool_priority() {
        let pool = ThreadPool::new(1);
        let results = Arc::new(Mutex::new(Vec::new()));


        {
            let results = Arc::clone(&results);
            pool.submit_with_priority(move || {
                results.lock().unwrap().push("low");
            }, TaskPriority::Low);
        }


        {
            let results = Arc::clone(&results);
            pool.submit_with_priority(move || {
                results.lock().unwrap().push("high");
            }, TaskPriority::High);
        }

        pool.wait_all();
        pool.shutdown();


        let results = results.lock().unwrap();
        assert_eq!(results[0], "high");
    }

    #[test]
    fn test_work_stealing_scheduler() {
        let scheduler = WorkStealingScheduler::new(4);


        for i in 0..4 {
            scheduler.push_local(i, Task::new(i as u64, TaskPriority::Normal, || {}));
        }


        for i in 0..4 {
            assert!(scheduler.pop(i).is_some());
        }


        assert!(scheduler.pop(0).is_none());
    }

    #[test]
    fn test_parallel_for_each() {
        let pool = ThreadPool::new(4);
        let sum = Arc::new(AtomicU64::new(0));

        let items: Vec<u64> = (1..=100).collect();
        let sum_clone = Arc::clone(&sum);

        ParallelIterator::for_each(items, &pool, move |x| {
            sum_clone.fetch_add(x, Ordering::Relaxed);
        });

        assert_eq!(sum.load(Ordering::Relaxed), 5050);
        pool.shutdown();
    }

    #[test]
    fn test_batch_processor() {
        let processor = BatchProcessor::new(10, 4);
        let count = Arc::new(AtomicU64::new(0));

        let items: Vec<u64> = (0..100).collect();
        let count_clone = Arc::clone(&count);

        processor.process(items, move |batch| {
            count_clone.fetch_add(batch.len() as u64, Ordering::Relaxed);
        });

        assert_eq!(count.load(Ordering::Relaxed), 100);
        processor.shutdown();
    }
}
