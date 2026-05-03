use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::thread;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::{mpsc, oneshot, Semaphore};
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};

use crate::gpu::{
    FrameBufferManager, FrameBufferHandle, ShaderSystem,
    TexturePool, BufferPool, GpuCpuSynchronization
};
use crate::nodes::{Graph, NodeExecutor, ExecutionContext};

/// Real-time preview system with adaptive quality and background processing
pub struct PreviewSystem {
    config: PreviewConfig,
    frame_buffer_manager: Arc<FrameBufferManager>,
    shader_system: Arc<ShaderSystem>,
    texture_pool: Arc<Mutex<TexturePool>>,
    buffer_pool: Arc<Mutex<BufferPool>>,
    synchronization: Arc<GpuCpuSynchronization>,
    
    // Preview state
    preview_sessions: Arc<RwLock<HashMap<Uuid, PreviewSession>>>,
    active_sessions: Arc<Mutex<HashMap<Uuid, SessionHandle>>>,
    
    // Performance monitoring
    performance_monitor: Arc<Mutex<PerformanceMonitor>>,
    
    // Background processing
    render_queue: Arc<Mutex<mpsc::UnboundedSender<RenderTask>>>,
    queue_semaphore: Arc<Semaphore>,
    
    // Statistics
    stats: Arc<Mutex<PreviewStats>>,
}

impl PreviewSystem {
    /// Create a new preview system
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
        
        // Start background render workers
        system.start_render_workers(render_rx)?;
        
        info!("Preview system created successfully");
        
        Ok(system)
    }
    
    /// Create a new preview session
    pub fn create_session(
        &self,
        graph: Arc<Graph>,
        width: u32,
        height: u32,
        quality: PreviewQuality,
    ) -> Result<PreviewSessionHandle> {
        debug!("Creating preview session: {}x{} @ {:?}", width, height, quality);
        
        let session_id = Uuid::new_v4();
        
        // Create frame buffers for the session
        let frame_buffers = self.create_session_frame_buffers(width, height)?;
        
        // Create preview session
        let session = PreviewSession {
            id: session_id,
            graph,
            width,
            height,
            current_quality: quality,
            target_quality: quality,
            frame_buffers,
            created_at: Instant::now(),
            last_frame_time: Instant::now(),
            frame_count: 0,
            dropped_frames: 0,
            adaptive_quality: AdaptiveQualityController::new(self.config.adaptive_config.clone()),
        };
        
        // Store session
        {
            let mut sessions = self.preview_sessions.write().map_err(|e| anyhow!("Sessions write lock error: {}", e))?;
            sessions.insert(session_id, session.clone());
        }
        
        // Create session handle
        let handle = PreviewSessionHandle {
            id: session_id,
            session: Arc::new(session),
        };
        
        // Update stats
        {
            let mut stats = self.stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.active_sessions += 1;
            stats.total_sessions += 1;
        }
        
        info!("Created preview session: {}", session_id);
        
        Ok(handle)
    }
    
    /// Start real-time preview for a session
    pub fn start_preview(&self, session_id: &Uuid) -> Result<()> {
        debug!("Starting preview for session: {}", session_id);
        
        let sessions = self.preview_sessions.read().map_err(|e| anyhow!("Sessions read lock error: {}", e))?;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;
        
        // Create session handle for background processing
        let session_handle = SessionHandle {
            id: *session_id,
            is_active: true,
            last_activity: Instant::now(),
        };
        
        // Store session handle
        {
            let mut active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;
            active_sessions.insert(*session_id, session_handle);
        }
        
        info!("Started preview for session: {}", session_id);
        
        Ok(())
    }
    
    /// Stop preview for a session
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
    
    /// Request a frame for a session
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
        
        // Check if session is active
        {
            let active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;
            if !active_sessions.contains_key(session_id) {
                return Err(anyhow!("Session not active: {}", session_id));
            }
        }
        
        // Determine quality
        let quality = force_quality.unwrap_or(session.current_quality);
        
        // Create render task
        let (response_tx, response_rx) = oneshot::channel();
        let render_task = RenderTask {
            session_id: *session_id,
            frame_time,
            quality,
            response_tx,
            timestamp: Instant::now(),
        };
        
        // Queue render task
        {
            let render_queue = self.render_queue.lock().map_err(|e| anyhow!("Render queue lock error: {}", e))?;
            render_queue.send(render_task)
                .map_err(|e| anyhow!("Failed to queue render task: {}", e))?;
        }
        
        // Wait for completion
        let frame = response_rx.await
            .map_err(|e| anyhow!("Render task failed: {}", e))??;
        
        // Update performance metrics
        self.update_performance_metrics(session_id, &frame)?;
        
        debug!("Frame rendered for session: {} (quality: {:?})", session_id, quality);
        
        Ok(frame)
    }
    
    /// Get adaptive quality recommendation
    pub fn get_adaptive_quality(&self, session_id: &Uuid) -> Result<PreviewQuality> {
        let sessions = self.preview_sessions.read().map_err(|e| anyhow!("Sessions read lock error: {}", e))?;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;
        
        let performance = self.performance_monitor.lock().map_err(|e| anyhow!("Performance monitor lock error: {}", e))?;
        let session_performance = performance.get_session_performance(session_id);
        
        Ok(session.adaptive_quality.recommend_quality(session_performance))
    }
    
    /// Update session quality
    pub fn update_session_quality(&self, session_id: &Uuid, quality: PreviewQuality) -> Result<()> {
        debug!("Updating session quality: {} -> {:?}", session_id, quality);
        
        let mut sessions = self.preview_sessions.write().map_err(|e| anyhow!("Sessions write lock error: {}", e))?;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.target_quality = quality;
            session.current_quality = quality;
            info!("Updated session quality: {} -> {:?}", session_id, quality);
        } else {
            return Err(anyhow!("Session not found: {}", session_id));
        }
        
        Ok(())
    }
    
    /// Get preview statistics
    pub fn get_stats(&self) -> Result<PreviewStats> {
        let stats = self.stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
        let sessions = self.preview_sessions.read().map_err(|e| anyhow!("Sessions read lock error: {}", e))?;
        let active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;
        
        let mut current_stats = stats.clone();
        current_stats.active_sessions = active_sessions.len();
        current_stats.total_sessions = sessions.len();
        
        Ok(current_stats)
    }
    
    /// Clean up inactive sessions
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
    
    /// Start background render workers
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
            
            thread::spawn(move || {
                debug!("Render worker {} started", worker_id);
                
                while let Some(task) = render_rx.blocking_recv() {
                    // Acquire semaphore permit
                    let _permit = queue_semaphore.blocking_acquire();
                    
                    let start_time = Instant::now();
                    
                    match Self::process_render_task(
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
                            let render_time = start_time.elapsed();
                            debug!("Render worker {} completed frame in {:?}", worker_id, render_time);
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
    
    /// Process a single render task
    fn process_render_task(
        task: RenderTask,
        frame_buffer_manager: &Arc<FrameBufferManager>,
        shader_system: &Arc<ShaderSystem>,
        texture_pool: &Arc<Mutex<TexturePool>>,
        buffer_pool: &Arc<Mutex<BufferPool>>,
        synchronization: &Arc<GpuCpuSynchronization>,
        preview_sessions: &Arc<RwLock<HashMap<Uuid, PreviewSession>>>,
        performance_monitor: &Arc<Mutex<PerformanceMonitor>>,
        stats: &Arc<Mutex<PreviewStats>>,
    ) -> Result<()> {
        let start_time = Instant::now();
        
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
        
        // Execute node graph
        let node_executor = NodeExecutor::new(
            texture_pool.clone(),
            buffer_pool.clone(),
            synchronization.clone(),
        );
        
        let result = node_executor.execute_graph(&session.graph, &context)?;
        
        // Create preview frame
        let frame = PreviewFrame {
            session_id: task.session_id,
            frame_time: task.frame_time,
            quality: task.quality,
            frame_buffer,
            render_time: start_time.elapsed(),
            timestamp: Instant::now(),
        };
        
        // Send response
        if let Err(e) = task.response_tx.send(Ok(frame)) {
            debug!("Failed to send render response: {}", e);
        }
        
        // Update stats
        {
            let mut stats = stats.lock().map_err(|e| anyhow!("Stats lock error: {}", e))?;
            stats.frames_rendered += 1;
            stats.total_render_time += start_time.elapsed();
        }
        
        // Update performance monitor
        {
            let mut monitor = performance_monitor.lock().map_err(|e| anyhow!("Performance monitor lock error: {}", e))?;
            monitor.record_frame_render(task.session_id, start_time.elapsed(), task.quality);
        }
        
        Ok(())
    }
    
    /// Create frame buffers for a session
    fn create_session_frame_buffers(&self, width: u32, height: u32) -> Result<HashMap<PreviewQuality, FrameBufferHandle>> {
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
    
    /// Update performance metrics
    fn update_performance_metrics(&self, session_id: &Uuid, frame: &PreviewFrame) -> Result<()> {
        // Update session frame count
        {
            let mut sessions = self.preview_sessions.write().map_err(|e| anyhow!("Sessions write lock error: {}", e))?;
            if let Some(session) = sessions.get_mut(session_id) {
                session.frame_count += 1;
                session.last_frame_time = Instant::now();
                
                // Check if frame was dropped (too old)
                if frame.timestamp.elapsed() > Duration::from_millis(100) {
                    session.dropped_frames += 1;
                }
            }
        }
        
        // Update active session activity
        {
            let mut active_sessions = self.active_sessions.lock().map_err(|e| anyhow!("Active sessions lock error: {}", e))?;
            if let Some(handle) = active_sessions.get_mut(session_id) {
                handle.last_activity = Instant::now();
            }
        }
        
        Ok(())
    }
}

/// Preview configuration
#[derive(Debug, Clone)]
pub struct PreviewConfig {
    pub render_workers: usize,
    pub max_concurrent_renders: usize,
    pub adaptive_config: AdaptiveQualityConfig,
    pub cleanup_interval: Duration,
    pub max_inactive_time: Duration,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            render_workers: num_cpus::get(),
            max_concurrent_renders: 4,
            adaptive_config: AdaptiveQualityConfig::default(),
            cleanup_interval: Duration::from_secs(30),
            max_inactive_time: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Preview quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreviewQuality {
    Low,     // 1/4 resolution
    Medium,  // 1/2 resolution
    High,    // Full resolution
    Ultra,   // 2x resolution (for future use)
}

impl PreviewQuality {
    /// Get all quality levels
    pub fn all() -> Vec<Self> {
        vec![Self::Low, Self::Medium, Self::High, Self::Ultra]
    }
    
    /// Scale dimensions based on quality
    pub fn scale_dimensions(&self, width: u32, height: u32) -> (u32, u32) {
        match self {
            Self::Low => (width / 4, height / 4),
            Self::Medium => (width / 2, height / 2),
            Self::High => (width, height),
            Self::Ultra => (width * 2, height * 2),
        }
    }
    
    /// Convert to float for shader parameters
    pub fn to_f32(&self) -> f32 {
        match self {
            Self::Low => 0.25,
            Self::Medium => 0.5,
            Self::High => 1.0,
            Self::Ultra => 2.0,
        }
    }
}

/// Preview session
#[derive(Debug, Clone)]
pub struct PreviewSession {
    pub id: Uuid,
    pub graph: Arc<Graph>,
    pub width: u32,
    pub height: u32,
    pub current_quality: PreviewQuality,
    pub target_quality: PreviewQuality,
    pub frame_buffers: HashMap<PreviewQuality, FrameBufferHandle>,
    pub created_at: Instant,
    pub last_frame_time: Instant,
    pub frame_count: u64,
    pub dropped_frames: u64,
    pub adaptive_quality: AdaptiveQualityController,
}

impl PreviewSession {
    /// Get frame buffer for quality level
    pub fn get_frame_buffer_for_quality(&self, quality: PreviewQuality) -> Result<FrameBufferHandle> {
        self.frame_buffers.get(&quality)
            .cloned()
            .ok_or_else(|| anyhow!("Frame buffer not available for quality: {:?}", quality))
    }
    
    /// Get frame rate
    pub fn frame_rate(&self) -> f64 {
        let elapsed = self.created_at.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.frame_count as f64 / elapsed
        } else {
            0.0
        }
    }
    
    /// Get drop rate
    pub fn drop_rate(&self) -> f64 {
        if self.frame_count > 0 {
            self.dropped_frames as f64 / self.frame_count as f64
        } else {
            0.0
        }
    }
}

/// Preview session handle
#[derive(Debug, Clone)]
pub struct PreviewSessionHandle {
    pub id: Uuid,
    pub session: Arc<PreviewSession>,
}

/// Render task for background processing
#[derive(Debug)]
struct RenderTask {
    session_id: Uuid,
    frame_time: f64,
    quality: PreviewQuality,
    response_tx: oneshot::Sender<Result<PreviewFrame>>,
    timestamp: Instant,
}

/// Preview frame result
#[derive(Debug, Clone)]
pub struct PreviewFrame {
    pub session_id: Uuid,
    pub frame_time: f64,
    pub quality: PreviewQuality,
    pub frame_buffer: FrameBufferHandle,
    pub render_time: Duration,
    pub timestamp: Instant,
}

/// Session handle for active preview
#[derive(Debug)]
struct SessionHandle {
    id: Uuid,
    is_active: bool,
    last_activity: Instant,
}

/// Adaptive quality controller
#[derive(Debug, Clone)]
pub struct AdaptiveQualityController {
    config: AdaptiveQualityConfig,
    current_quality: PreviewQuality,
    performance_history: Vec<Duration>,
    last_adjustment: Instant,
}

impl AdaptiveQualityController {
    fn new(config: AdaptiveQualityConfig) -> Self {
        Self {
            config,
            current_quality: PreviewQuality::Medium,
            performance_history: Vec::new(),
            last_adjustment: Instant::now(),
        }
    }
    
    fn recommend_quality(&mut self, performance: SessionPerformance) -> PreviewQuality {
        // Add recent performance to history
        self.performance_history.push(performance.average_render_time);
        
        // Keep only recent history
        if self.performance_history.len() > self.config.history_size {
            self.performance_history.remove(0);
        }
        
        // Check if we should adjust quality
        if self.last_adjustment.elapsed() < self.config.adjustment_interval {
            return self.current_quality;
        }
        
        let avg_render_time = if self.performance_history.is_empty() {
            Duration::from_millis(16) // Default to 60 FPS target
        } else {
            let total: Duration = self.performance_history.iter().sum();
            total / self.performance_history.len() as u32
        };
        
        let target_frame_time = Duration::from_millis(1000 / self.config.target_fps);
        let quality = if avg_render_time > target_frame_time * 2 {
            // Too slow, decrease quality
            match self.current_quality {
                PreviewQuality::High => PreviewQuality::Medium,
                PreviewQuality::Medium => PreviewQuality::Low,
                PreviewQuality::Low => PreviewQuality::Low,
                PreviewQuality::Ultra => PreviewQuality::High,
            }
        } else if avg_render_time < target_frame_time / 2 {
            // Very fast, can increase quality
            match self.current_quality {
                PreviewQuality::Low => PreviewQuality::Medium,
                PreviewQuality::Medium => PreviewQuality::High,
                PreviewQuality::High => PreviewQuality::Ultra,
                PreviewQuality::Ultra => PreviewQuality::Ultra,
            }
        } else {
            // Good performance, keep current quality
            self.current_quality
        };
        
        if quality != self.current_quality {
            self.current_quality = quality;
            self.last_adjustment = Instant::now();
            debug!("Adjusted preview quality to: {:?}", quality);
        }
        
        self.current_quality
    }
}

/// Adaptive quality configuration
#[derive(Debug, Clone)]
pub struct AdaptiveQualityConfig {
    pub target_fps: u32,
    pub adjustment_interval: Duration,
    pub history_size: usize,
}

impl Default for AdaptiveQualityConfig {
    fn default() -> Self {
        Self {
            target_fps: 30,
            adjustment_interval: Duration::from_secs(2),
            history_size: 10,
        }
    }
}

/// Performance monitor for preview sessions
#[derive(Debug)]
pub struct PerformanceMonitor {
    session_performance: HashMap<Uuid, SessionPerformance>,
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            session_performance: HashMap::new(),
        }
    }
    
    fn record_frame_render(&mut self, session_id: Uuid, render_time: Duration, quality: PreviewQuality) {
        let performance = self.session_performance.entry(session_id).or_insert_with(|| SessionPerformance::new());
        performance.record_frame(render_time, quality);
    }
    
    fn get_session_performance(&self, session_id: &Uuid) -> SessionPerformance {
        self.session_performance.get(session_id).cloned().unwrap_or_else(|| SessionPerformance::new())
    }
}

/// Session performance metrics
#[derive(Debug, Clone)]
pub struct SessionPerformance {
    pub frame_count: u64,
    pub total_render_time: Duration,
    pub recent_render_times: Vec<Duration>,
    pub quality_distribution: HashMap<PreviewQuality, u64>,
}

impl SessionPerformance {
    fn new() -> Self {
        Self {
            frame_count: 0,
            total_render_time: Duration::ZERO,
            recent_render_times: Vec::new(),
            quality_distribution: HashMap::new(),
        }
    }
    
    fn record_frame(&mut self, render_time: Duration, quality: PreviewQuality) {
        self.frame_count += 1;
        self.total_render_time += render_time;
        
        self.recent_render_times.push(render_time);
        if self.recent_render_times.len() > 20 {
            self.recent_render_times.remove(0);
        }
        
        *self.quality_distribution.entry(quality).or_insert(0) += 1;
    }
    
    fn average_render_time(&self) -> Duration {
        if self.frame_count == 0 {
            Duration::ZERO
        } else {
            self.total_render_time / self.frame_count as u32
        }
    }
    
    fn recent_average_render_time(&self) -> Duration {
        if self.recent_render_times.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.recent_render_times.iter().sum();
            total / self.recent_render_times.len() as u32
        }
    }
}

/// Preview system statistics
#[derive(Debug, Clone)]
pub struct PreviewStats {
    pub active_sessions: usize,
    pub total_sessions: usize,
    pub frames_rendered: u64,
    pub total_render_time: Duration,
    pub average_fps: f64,
}

impl PreviewStats {
    fn new() -> Self {
        Self {
            active_sessions: 0,
            total_sessions: 0,
            frames_rendered: 0,
            total_render_time: Duration::ZERO,
            average_fps: 0.0,
        }
    }
    
    /// Format statistics for display
    pub fn format(&self) -> String {
        format!(
            "Sessions: {}/{} | Frames: {} | Avg FPS: {:.1} | Total Render Time: {:.2}s",
            self.active_sessions,
            self.total_sessions,
            self.frames_rendered,
            self.average_fps,
            self.total_render_time.as_secs_f64()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preview_quality() {
        assert_eq!(PreviewQuality::Low.scale_dimensions(1920, 1080), (480, 270));
        assert_eq!(PreviewQuality::Medium.scale_dimensions(1920, 1080), (960, 540));
        assert_eq!(PreviewQuality::High.scale_dimensions(1920, 1080), (1920, 1080));
        assert_eq!(PreviewQuality::Ultra.scale_dimensions(1920, 1080), (3840, 2160));
    }
    
    #[test]
    fn test_adaptive_quality() {
        let config = AdaptiveQualityConfig::default();
        let mut controller = AdaptiveQualityController::new(config);
        
        let performance = SessionPerformance::new();
        let quality = controller.recommend_quality(performance);
        
        // Should default to Medium
        assert_eq!(quality, PreviewQuality::Medium);
    }
    
    #[test]
    fn test_preview_stats() {
        let stats = PreviewStats::new();
        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.frames_rendered, 0);
        
        let formatted = stats.format();
        assert!(formatted.contains("Sessions: 0/0"));
        assert!(formatted.contains("Frames: 0"));
    }
}
