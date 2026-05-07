use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::{mpsc, Semaphore};
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};

use crate::gpu::{
    FrameBufferManager, ShaderSystem,
    TexturePool, BufferPool, GpuCpuSynchronization
};
use crate::nodes::{Graph, ExecutionContext};

use super::{
    PreviewConfig, PreviewSession, PreviewSessionHandle, PreviewFrame, PreviewQuality,
    PreviewStats, PerformanceMonitor, RenderTask, RenderWorker
};


pub struct PreviewSystem {
    config: PreviewConfig,
    frame_buffer_manager: Arc<FrameBufferManager>,
    shader_system: Arc<ShaderSystem>,
    texture_pool: Arc<Mutex<TexturePool>>,
    buffer_pool: Arc<Mutex<BufferPool>>,
    synchronization: Arc<GpuCpuSynchronization>,


    preview_sessions: Arc<RwLock<HashMap<Uuid, PreviewSession>>>,
    active_sessions: Arc<Mutex<HashMap<Uuid, SessionHandle>>>,


    performance_monitor: Arc<Mutex<PerformanceMonitor>>,


    render_queue: Arc<Mutex<mpsc::UnboundedSender<RenderTask>>>,
    queue_semaphore: Arc<Semaphore>,


    stats: Arc<Mutex<PreviewStats>>,
}

impl PreviewSystem {

    pub fn new(
        config: PreviewConfig,
        frame_buffer_manager: Arc<FrameBufferManager>,
        shader_system: Arc<ShaderSystem>,
        texture_pool: Arc<Mutex<TexturePool>>,
        buffer_pool: Arc<Mutex<BufferPool>>,
        synchronization: Arc<GpuCpuSynchronization>,
    ) -> Result<Self> {
        info!("Creating preview system with config: {:?}", config);

        let (render_tx, render_rx) = mpsc::unbounded_channel();
        let queue_semaphore = Arc::new(Semaphore::new(config.max_concurrent_renders));

        let system = Self {
            config,
            frame_buffer_manager,
            shader_system,
            texture_pool,
            buffer_pool,
            synchronization,
            preview_sessions: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            performance_monitor: Arc::new(Mutex::new(PerformanceMonitor::new())),
            render_queue: Arc::new(Mutex::new(render_tx)),
            queue_semaphore,
            stats: Arc::new(Mutex::new(PreviewStats::new())),
        };


        system.start_render_workers(render_rx)?;

        info!("Preview system created successfully");

        Ok(system)
    }


    pub fn create_session(
        &self,
        graph: Arc<Graph>,
        width: u32,
        height: u32,
        quality: PreviewQuality,
    ) -> Result<PreviewSessionHandle> {
        debug!("Creating preview session: {}x{} @ {:?}", width, height, quality);

        let session_id = Uuid::new_v4();


        let frame_buffers = self.create_session_frame_buffers(width, height)?;


        let session = PreviewSession::new(
            session_id,
            graph,
            width,
            height,
            quality,
            frame_buffers,
            self.config.adaptive_config.clone(),
        );


        {
            let mut sessions = self.preview_sessions.write().map_err(|e| anyhow!("Sessions write lock error: {}", e))?;
            sessions.insert(session_id, session.clone());
        }


        let handle = PreviewSessionHandle {
            id: session_id,
            session: Arc::new(session),
        };


        {
            let mut stats = self.stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.active_sessions += 1;
            stats.total_sessions += 1;
        }

        info!("Created preview session: {}", session_id);

        Ok(handle)
    }


    pub fn start_preview(&self, session_id: &Uuid) -> Result<()> {
        debug!("Starting preview for session: {}", session_id);

        let sessions = self.preview_sessions.read().map_err(|e| anyhow!("Sessions read lock error: {}", e))?;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;


        let session_handle = SessionHandle {
            id: *session_id,
            is_active: true,
            last_activity: Instant::now(),
        };


        {
            let mut active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;
            active_sessions.insert(*session_id, session_handle);
        }

        info!("Started preview for session: {}", session_id);

        Ok(())
    }


    pub fn stop_preview(&self, session_id: &Uuid) -> Result<()> {
        debug!("Stopping preview for session: {}", session_id);

        let mut active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;

        if active_sessions.remove(session_id).is_some() {
            info!("Stopped preview for session: {}", session_id);
        } else {
            warn!("Preview session not active: {}", session_id);
        }

        Ok(())
    }


    pub async fn request_frame(
        &self,
        session_id: &Uuid,
        frame_time: f64,
        force_quality: Option<PreviewQuality>,
    ) -> Result<PreviewFrame> {
        debug!("Requesting frame for session: {} at time: {:.3}", session_id, frame_time);

        let sessions = self.preview_sessions.read().map_err(|e| anyhow!("Sessions read lock error: {}", e))?;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;


        {
            let active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;
            if !active_sessions.contains_key(session_id) {
                return Err(anyhow!("Session not active: {}", session_id));
            }
        }


        let quality = force_quality.unwrap_or_else(|| session.get_current_quality());


        let render_task = RenderTask::new(*session_id, frame_time, quality)?;


        {
            let render_queue = self.render_queue.lock().map_err(|e| anyhow!("Render queue lock error: {}", e))?;
            render_queue.send(render_task)
                .map_err(|e| anyhow!("Failed to queue render task: {}", e))?;
        }


        let frame = PreviewFrame {
            session_id: *session_id,
            frame_time,
            quality,
            frame_buffer: session.get_frame_buffer_for_quality(quality)?,
            render_time: Duration::from_millis(16),
            timestamp: Instant::now(),
        };


        self.update_performance_metrics(session_id, &frame)?;

        debug!("Frame rendered for session: {} (quality: {:?})", session_id, quality);

        Ok(frame)
    }


    pub fn get_adaptive_quality(&self, session_id: &Uuid) -> Result<PreviewQuality> {
        let sessions = self.preview_sessions.read().map_err(|e| anyhow!("Sessions read lock error: {}", e))?;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        let performance = self.performance_monitor.lock().map_err(|e| anyhow!("Performance monitor lock error: {}", e))?;
        let session_performance = performance.get_session_performance(session_id);

        Ok(session.recommend_quality(session_performance))
    }


    pub fn update_session_quality(&self, session_id: &Uuid, quality: PreviewQuality) -> Result<()> {
        debug!("Updating session quality: {} -> {:?}", session_id, quality);

        let mut sessions = self.preview_sessions.write().map_err(|e| anyhow!("Sessions write lock error: {}", e))?;

        if let Some(session) = sessions.get_mut(session_id) {
            session.set_quality(quality);
            info!("Updated session quality: {} -> {:?}", session_id, quality);
        } else {
            return Err(anyhow!("Session not found: {}", session_id));
        }

        Ok(())
    }


    pub fn get_stats(&self) -> Result<PreviewStats> {
        let stats = self.stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
        let sessions = self.preview_sessions.read().map_err(|e| anyhow!("Sessions read lock error: {}", e))?;
        let active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;

        let mut current_stats = stats.clone();
        current_stats.active_sessions = active_sessions.len();
        current_stats.total_sessions = sessions.len();

        Ok(current_stats)
    }


    pub fn cleanup_inactive_sessions(&self, max_inactive_time: Duration) -> Result<usize> {
        debug!("Cleaning up inactive sessions (max inactive: {:?})", max_inactive_time);

        let mut active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;
        let mut to_remove = Vec::new();

        for (id, handle) in &active_sessions {
            if handle.last_activity.elapsed() > max_inactive_time {
                to_remove.push(*id);
            }
        }

        let removed_count = to_remove.len();

        for id in to_remove {
            active_sessions.remove(&id);
            debug!("Removed inactive session: {}", id);
        }

        if removed_count > 0 {
            info!("Cleaned up {} inactive sessions", removed_count);
        }

        Ok(removed_count)
    }


    fn start_render_workers(&self, mut render_rx: mpsc::UnboundedReceiver<RenderTask>) -> Result<()> {
        let worker_count = self.config.render_workers;
        let queue_semaphore = self.queue_semaphore.clone();
        let frame_buffer_manager = self.frame_buffer_manager.clone();
        let shader_system = self.shader_system.clone();
        let texture_pool = self.texture_pool.clone();
        let buffer_pool = self.buffer_pool.clone();
        let synchronization = self.synchronization.clone();
        let preview_sessions = self.preview_sessions.clone();
        let performance_monitor = self.performance_monitor.clone();
        let stats = self.stats.clone();

        for worker_id in 0..worker_count {
            let queue_semaphore = queue_semaphore.clone();
            let frame_buffer_manager = frame_buffer_manager.clone();
            let shader_system = shader_system.clone();
            let texture_pool = texture_pool.clone();
            let buffer_pool = buffer_pool.clone();
            let synchronization = synchronization.clone();
            let preview_sessions = preview_sessions.clone();
            let performance_monitor = performance_monitor.clone();
            let stats = stats.clone();

            std::thread::spawn(move || {
                debug!("Render worker {} started", worker_id);

                let worker = RenderWorker::new(worker_id);

                while let Some(task) = render_rx.blocking_recv() {

                    let _permit = queue_semaphore.blocking_acquire();

                    match worker.process_task(
                        task,
                        &frame_buffer_manager,
                        &shader_system,
                        &texture_pool,
                        &buffer_pool,
                        &synchronization,
                        &preview_sessions,
                        &performance_monitor,
                        &stats,
                    ) {
                        Ok(_) => {
                            debug!("Render worker {} completed frame", worker_id);
                        }
                        Err(e) => {
                            error!("Render worker {} failed: {}", worker_id, e);
                        }
                    }
                }

                debug!("Render worker {} stopped", worker_id);
            });
        }

        info!("Started {} render workers", worker_count);

        Ok(())
    }


    fn create_session_frame_buffers(&self, width: u32, height: u32) -> Result<HashMap<PreviewQuality, crate::gpu::FrameBufferHandle>> {
        let mut frame_buffers = HashMap::new();

        for quality in PreviewQuality::all() {
            let (scaled_width, scaled_height) = quality.scale_dimensions(width, height);

            let frame_buffer = self.frame_buffer_manager.create_frame_buffer(
                scaled_width,
                scaled_height,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            )?;

            frame_buffers.insert(quality, frame_buffer);
        }

        Ok(frame_buffers)
    }


    fn update_performance_metrics(&self, session_id: &Uuid, frame: &PreviewFrame) -> Result<()> {

        {
            let mut sessions = self.preview_sessions.write().map_err(|e| anyhow!("Sessions write lock error: {}", e))?;
            if let Some(session) = sessions.get_mut(session_id) {
                session.increment_frame_count();
                session.update_last_frame_time();


                if frame.timestamp.elapsed() > Duration::from_millis(100) {
                    session.increment_dropped_frames();
                }
            }
        }


        {
            let mut active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;
            if let Some(handle) = active_sessions.get_mut(session_id) {
                handle.last_activity = Instant::now();
            }
        }

        Ok(())
    }
}


#[derive(Debug)]
struct SessionHandle {
    id: Uuid,
    is_active: bool,
    last_activity: Instant,
}
