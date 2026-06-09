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


#[derive(Debug)]
pub struct RenderTask {
    pub session_id: Uuid,
    pub frame_time: f64,
    pub quality: PreviewQuality,
    pub response_tx: oneshot::Sender<Result<PreviewFrame>>,
    pub timestamp: Instant,
}

impl RenderTask {

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


    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }


    pub fn is_expired(&self, timeout: std::time::Duration) -> bool {
        self.age() > timeout
    }
}


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


    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }


    pub fn is_stale(&self, max_age: std::time::Duration) -> bool {
        self.age() > max_age
    }


    pub fn dimensions(&self) -> (u32, u32) {
        self.frame_buffer.dimensions()
    }


    pub fn format(&self) -> wgpu::TextureFormat {
        self.frame_buffer.format()
    }


    pub fn texture_view(&self) -> &wgpu::TextureView {
        self.frame_buffer.view()
    }
}


#[derive(Debug)]
pub struct RenderWorker {
    pub worker_id: usize,
    node_executor: NodeExecutor,
}

impl RenderWorker {

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


        if task.is_expired(std::time::Duration::from_millis(100)) {
            debug!("Render worker {} skipping expired task", self.worker_id);
            let _ = task.response_tx.send(Err(anyhow!("Task expired")));
            return Ok(());
        }


        let sessions = preview_sessions.read().map_err(|e| anyhow!("Sessions read lock error: {}", e))?;
        let session = sessions.get(&task.session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", task.session_id))?;


        let context = ExecutionContext {
            frame: (task.frame_time * 30.0) as u32,
            time: task.frame_time,
            quality: task.quality.to_f32(),
        };


        let frame_buffer = session.get_frame_buffer_for_quality(task.quality)?;


        let result = self.execute_node_graph(&session.graph, &context, &frame_buffer)?;


        let render_time = start_time.elapsed();
        let frame = PreviewFrame::new(
            task.session_id,
            task.frame_time,
            task.quality,
            frame_buffer,
            render_time,
        );


        if let Err(e) = task.response_tx.send(Ok(frame)) {
            debug!("Failed to send render response: {}", e);
        }


        {
            let mut stats = stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.update_frame_stats(render_time, false);
        }


        {
            let mut monitor = performance_monitor.lock().map_err(|e| anyhow!("Performance monitor lock error: {}", e))?;
            monitor.record_frame_render(task.session_id, render_time, task.quality);
        }

        debug!("Render worker {} completed frame in {:?}", self.worker_id, render_time);

        Ok(())
    }


    fn execute_node_graph(
        &self,
        graph: &std::sync::Arc<Graph>,
        context: &ExecutionContext,
        frame_buffer: &crate::gpu::FrameBufferHandle,
    ) -> Result<()> {


        while let Some(task) = self.tasks.pop_front() {
            let _ = task.response_tx.send(Err(anyhow!("Queue cleared")));
        }
    }


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


#[derive(Debug, Clone)]
pub struct RenderQueueStats {
    pub size: usize,
    pub max_size: usize,
    pub average_age: std::time::Duration,
    pub quality_distribution: std::collections::HashMap<PreviewQuality, usize>,
}

impl RenderQueueStats {

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


#[derive(Debug)]
pub struct RenderPool {
    workers: Vec<RenderWorker>,
    queue: std::sync::Mutex<RenderQueue>,
    active_tasks: std::sync::atomic::AtomicUsize,
}

impl RenderPool {

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


    pub fn submit_task(&self, task: RenderTask) -> Result<()> {
        let mut queue = self.queue.lock().map_err(|e| anyhow!("Queue lock error: {}", e))?;
        queue.push(task)?;
        Ok(())
    }


    pub fn active_task_count(&self) -> usize {
        self.active_tasks.load(std::sync::atomic::Ordering::Relaxed)
    }


    pub fn queue_stats(&self) -> Result<RenderQueueStats> {
        let queue = self.queue.lock().map_err(|e| anyhow!("Queue lock error: {}", e))?;
        Ok(queue.stats())
    }


    pub fn clear(&self) -> Result<()> {
        let mut queue = self.queue.lock().map_err(|e| anyhow!("Queue lock error: {}", e))?;
        queue.clear();
        Ok(())
    }


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
