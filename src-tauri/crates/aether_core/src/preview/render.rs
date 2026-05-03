use std::time::Instant;
use uuid::Uuid;
use tokio::sync::oneshot;
use anyhow::{Result, anyhow};
use log::{debug, error};

use crate::gpu::{
    FrameBufferManager, ShaderSystem,
    TexturePool, BufferPool, GpuCpuSynchronization
};
use crate::nodes::{Graph, NodeExecutor, ExecutionContext};

use super::{PreviewQuality, PreviewFrame, SessionPerformance, PreviewStats};

/// Render task for background processing
#[derive(Debug)]
pub struct RenderTask {
    pub session_id: Uuid,
    pub frame_time: f64,
    pub quality: PreviewQuality,
    pub response_tx: oneshot::Sender<Result<PreviewFrame>>,
    pub timestamp: Instant,
}

impl RenderTask {
    /// Create a new render task
    pub fn new(
        session_id: Uuid,
        frame_time: f64,
        quality: PreviewQuality,
    ) -> Result<Self> {
        let (response_tx, _) = oneshot::channel();
        
        Ok(Self {
            session_id,
            frame_time,
            quality,
            response_tx,
            timestamp: Instant::now(),
        })
    }
    
    /// Create a render task with custom response channel
    pub fn with_response_channel(
        session_id: Uuid,
        frame_time: f64,
        quality: PreviewQuality,
        response_tx: oneshot::Sender<Result<PreviewFrame>>,
    ) -> Self {
        Self {
            session_id,
            frame_time,
            quality,
            response_tx,
            timestamp: Instant::now(),
        }
    }
    
    /// Get task age
    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
    
    /// Check if task is expired
    pub fn is_expired(&self, timeout: std::time::Duration) -> bool {
        self.age() > timeout
    }
}

/// Preview frame result
#[derive(Debug, Clone)]
pub struct PreviewFrame {
    pub session_id: Uuid,
    pub frame_time: f64,
    pub quality: PreviewQuality,
    pub frame_buffer: crate::gpu::FrameBufferHandle,
    pub render_time: std::time::Duration,
    pub timestamp: Instant,
}

impl PreviewFrame {
    /// Create a new preview frame
    pub fn new(
        session_id: Uuid,
        frame_time: f64,
        quality: PreviewQuality,
        frame_buffer: crate::gpu::FrameBufferHandle,
        render_time: std::time::Duration,
    ) -> Self {
        Self {
            session_id,
            frame_time,
            quality,
            frame_buffer,
            render_time,
            timestamp: Instant::now(),
        }
    }
    
    /// Get frame age
    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
    
    /// Check if frame is stale (too old)
    pub fn is_stale(&self, max_age: std::time::Duration) -> bool {
        self.age() > max_age
    }
    
    /// Get frame dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        self.frame_buffer.dimensions()
    }
    
    /// Get frame format
    pub fn format(&self) -> wgpu::TextureFormat {
        self.frame_buffer.format()
    }
    
    /// Get texture view
    pub fn texture_view(&self) -> &wgpu::TextureView {
        self.frame_buffer.view()
    }
}

/// Render worker for processing background tasks
#[derive(Debug)]
pub struct RenderWorker {
    pub worker_id: usize,
    node_executor: NodeExecutor,
}

impl RenderWorker {
    /// Create a new render worker
    pub fn new(worker_id: usize) -> Self {
        Self {
            worker_id,
            node_executor: NodeExecutor::new(
                std::sync::Arc::new(std::sync::Mutex::new(crate::gpu::TexturePool::new())),
                std::sync::Arc::new(std::sync::Mutex::new(crate::gpu::BufferPool::new())),
                std::sync::Arc::new(crate::gpu::GpuCpuSynchronization::new()),
            ),
        }
    }
    
    /// Process a single render task
    pub fn process_task(
        &self,
        task: RenderTask,
        frame_buffer_manager: &std::sync::Arc<FrameBufferManager>,
        shader_system: &std::sync::Arc<ShaderSystem>,
        texture_pool: &std::sync::Arc<std::sync::Mutex<TexturePool>>,
        buffer_pool: &std::sync::Arc<std::sync::Mutex<BufferPool>>,
        synchronization: &std::sync::Arc<GpuCpuSynchronization>,
        preview_sessions: &std::sync::Arc<std::sync::RwLock<std::collections::HashMap<Uuid, super::PreviewSession>>>,
        performance_monitor: &std::sync::Arc<std::sync::Mutex<super::PerformanceMonitor>>,
        stats: &std::sync::Arc<std::sync::Mutex<PreviewStats>>,
    ) -> Result<()> {
        let start_time = Instant::now();
        
        // Check if task is expired
        if task.is_expired(std::time::Duration::from_millis(100)) {
            debug!("Render worker {} skipping expired task", self.worker_id);
            let _ = task.response_tx.send(Err(anyhow!("Task expired")));
            return Ok(());
        }
        
        // Get session
        let sessions = preview_sessions.read().map_err(|e| anyhow!("Sessions read lock error: {}", e))?;
        let session = sessions.get(&task.session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", task.session_id))?;
        
        // Create execution context
        let context = ExecutionContext {
            frame: (task.frame_time * 30.0) as u32, // Assuming 30 FPS base
            time: task.frame_time,
            quality: task.quality.to_f32(),
        };
        
        // Get appropriate frame buffer
        let frame_buffer = session.get_frame_buffer_for_quality(task.quality)?;
        
        // Execute node graph (this would be the actual rendering logic)
        let result = self.execute_node_graph(&session.graph, &context, &frame_buffer)?;
        
        // Create preview frame
        let render_time = start_time.elapsed();
        let frame = PreviewFrame::new(
            task.session_id,
            task.frame_time,
            task.quality,
            frame_buffer,
            render_time,
        );
        
        // Send response
        if let Err(e) = task.response_tx.send(Ok(frame)) {
            debug!("Failed to send render response: {}", e);
        }
        
        // Update stats
        {
            let mut stats = stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.update_frame_stats(render_time, false);
        }
        
        // Update performance monitor
        {
            let mut monitor = performance_monitor.lock().map_err(|e| anyhow!("Performance monitor lock error: {}", e))?;
            monitor.record_frame_render(task.session_id, render_time, task.quality);
        }
        
        debug!("Render worker {} completed frame in {:?}", self.worker_id, render_time);
        
        Ok(())
    }
    
    /// Execute node graph (placeholder implementation)
    fn execute_node_graph(
        &self,
        graph: &std::sync::Arc<Graph>,
        context: &ExecutionContext,
        frame_buffer: &crate::gpu::FrameBufferHandle,
    ) -> Result<()> {
        // This would contain the actual node graph execution logic
        // For now, we'll simulate the work
        
        // Mark frame buffer as used
        frame_buffer.mark_used();
        
        // Simulate processing time based on quality
        let processing_time = match context.quality {
            q if q <= 0.25 => std::time::Duration::from_millis(5),  // Low quality
            q if q <= 0.5 => std::time::Duration::from_millis(10), // Medium quality
            q if q <= 1.0 => std::time::Duration::from_millis(16), // High quality
            _ => std::time::Duration::from_millis(25),             // Ultra quality
        };
        
        std::thread::sleep(processing_time);
        
        debug!("Executed node graph for frame {} (quality: {:.2})", context.frame, context.quality);
        
        Ok(())
    }
}

/// Render queue for managing tasks
#[derive(Debug)]
pub struct RenderQueue {
    tasks: std::collections::VecDeque<RenderTask>,
    max_size: usize,
}

impl RenderQueue {
    /// Create a new render queue
    pub fn new(max_size: usize) -> Self {
        Self {
            tasks: std::collections::VecDeque::new(),
            max_size,
        }
    }
    
    /// Add a task to the queue
    pub fn push(&mut self, task: RenderTask) -> Result<()> {
        if self.tasks.len() >= self.max_size {
            // Remove oldest task
            if let Some(old_task) = self.tasks.pop_front() {
                let _ = old_task.response_tx.send(Err(anyhow!("Queue overflow")));
            }
        }
        
        self.tasks.push_back(task);
        Ok(())
    }
    
    /// Get the next task
    pub fn pop(&mut self) -> Option<RenderTask> {
        self.tasks.pop_front()
    }
    
    /// Get queue size
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
    
    /// Clear the queue
    pub fn clear(&mut self) {
        // Notify all waiting tasks that they're being cancelled
        while let Some(task) = self.tasks.pop_front() {
            let _ = task.response_tx.send(Err(anyhow!("Queue cleared")));
        }
    }
    
    /// Remove expired tasks
    pub fn remove_expired(&mut self, timeout: std::time::Duration) -> usize {
        let initial_len = self.tasks.len();
        
        self.tasks.retain(|task| {
            if task.is_expired(timeout) {
                let _ = task.response_tx.send(Err(anyhow!("Task expired")));
                false
            } else {
                true
            }
        });
        
        initial_len - self.tasks.len()
    }
    
    /// Get queue statistics
    pub fn stats(&self) -> RenderQueueStats {
        let mut age_stats = Vec::new();
        let mut quality_counts = std::collections::HashMap::new();
        
        for task in &self.tasks {
            age_stats.push(task.age());
            *quality_counts.entry(task.quality).or_insert(0) += 1;
        }
        
        let avg_age = if age_stats.is_empty() {
            std::time::Duration::ZERO
        } else {
            let total: std::time::Duration = age_stats.iter().sum();
            total / age_stats.len() as u32
        };
        
        RenderQueueStats {
            size: self.tasks.len(),
            max_size: self.max_size,
            average_age: avg_age,
            quality_distribution: quality_counts,
        }
    }
}

/// Render queue statistics
#[derive(Debug, Clone)]
pub struct RenderQueueStats {
    pub size: usize,
    pub max_size: usize,
    pub average_age: std::time::Duration,
    pub quality_distribution: std::collections::HashMap<PreviewQuality, usize>,
}

impl RenderQueueStats {
    /// Format statistics for display
    pub fn format(&self) -> String {
        format!(
            "Queue: {}/{} | Avg Age: {:.1}ms | Quality: {:?}",
            self.size,
            self.max_size,
            self.average_age.as_secs_f64() * 1000.0,
            self.quality_distribution
        )
    }
}

/// Render pool for managing multiple workers
#[derive(Debug)]
pub struct RenderPool {
    workers: Vec<RenderWorker>,
    queue: std::sync::Mutex<RenderQueue>,
    active_tasks: std::sync::atomic::AtomicUsize,
}

impl RenderPool {
    /// Create a new render pool
    pub fn new(worker_count: usize, queue_size: usize) -> Self {
        let workers: Vec<RenderWorker> = (0..worker_count)
            .map(RenderWorker::new)
            .collect();
        
        Self {
            workers,
            queue: std::sync::Mutex::new(RenderQueue::new(queue_size)),
            active_tasks: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    
    /// Submit a render task
    pub fn submit_task(&self, task: RenderTask) -> Result<()> {
        let mut queue = self.queue.lock().map_err(|e| anyhow!("Queue lock error: {}", e))?;
        queue.push(task)?;
        Ok(())
    }
    
    /// Get active task count
    pub fn active_task_count(&self) -> usize {
        self.active_tasks.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Get queue statistics
    pub fn queue_stats(&self) -> Result<RenderQueueStats> {
        let queue = self.queue.lock().map_err(|e| anyhow!("Queue lock error: {}", e))?;
        Ok(queue.stats())
    }
    
    /// Clear all tasks
    pub fn clear(&self) -> Result<()> {
        let mut queue = self.queue.lock().map_err(|e| anyhow!("Queue lock error: {}", e))?;
        queue.clear();
        Ok(())
    }
    
    /// Get worker count
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_task() {
        let task = RenderTask::new(
            Uuid::new_v4(),
            1.5,
            PreviewQuality::High,
        ).unwrap();
        
        assert_eq!(task.quality, PreviewQuality::High);
        assert_eq!(task.frame_time, 1.5);
        assert!(!task.is_expired(std::time::Duration::from_secs(1)));
    }
    
    #[test]
    fn test_preview_frame() {
        let frame_buffer = crate::gpu::FrameBufferHandle {
            id: Uuid::new_v4(),
            frame_buffer: std::sync::Arc::new(crate::gpu::FrameBuffer {
                id: Uuid::new_v4(),
                width: 1920,
                height: 1080,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                texture: std::sync::Arc::new(crate::gpu::TextureHandle { id: Uuid::new_v4() }),
                view: std::sync::Arc::new(crate::gpu::TextureViewHandle { id: Uuid::new_v4() }),
                created_at: std::time::Instant::now(),
                last_used: std::sync::Mutex::new(std::time::Instant::now()),
                access_count: std::sync::atomic::AtomicU64::new(0),
            }),
        };
        
        let frame = PreviewFrame::new(
            Uuid::new_v4(),
            1.5,
            PreviewQuality::High,
            frame_buffer,
            std::time::Duration::from_millis(16),
        );
        
        assert_eq!(frame.quality, PreviewQuality::High);
        assert_eq!(frame.frame_time, 1.5);
        assert_eq!(frame.render_time, std::time::Duration::from_millis(16));
    }
    
    #[test]
    fn test_render_queue() {
        let mut queue = RenderQueue::new(3);
        
        let task = RenderTask::new(
            Uuid::new_v4(),
            1.0,
            PreviewQuality::Medium,
        ).unwrap();
        
        queue.push(task).unwrap();
        assert_eq!(queue.len(), 1);
        
        let popped = queue.pop();
        assert!(popped.is_some());
        assert_eq!(queue.len(), 0);
    }
    
    #[test]
    fn test_render_pool() {
        let pool = RenderPool::new(2, 10);
        
        assert_eq!(pool.worker_count(), 2);
        assert_eq!(pool.active_task_count(), 0);
        
        let stats = pool.queue_stats().unwrap();
        assert_eq!(stats.size, 0);
        assert_eq!(stats.max_size, 10);
    }
}
