use crate::nodes::{
    NodeManager, NodeExecutor, ExecutionContext, NodeError, NodeResult,
    validation::GraphValidator, execution_order::ExecutionOrderManager
};
use aether_types::{Graph, ParameterValue, Uuid};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;
use parking_lot::RwLock;
use log::{debug, info, warn, error};

/// Main node execution engine
pub struct NodeExecutorEngine {
    /// Node manager for handling nodes
    node_manager: NodeManager,
    /// Execution order manager
    order_manager: ExecutionOrderManager,
    /// Performance metrics
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Execution configuration
    config: ExecutionConfig,
    /// Frame cache for caching results
    frame_cache: Arc<RwLock<FrameCache>>,
}

/// Configuration for node execution
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Enable parallel execution
    pub enable_parallel: bool,
    /// Maximum number of worker threads
    pub max_workers: usize,
    /// Enable frame caching
    pub enable_caching: bool,
    /// Maximum cache size in frames
    pub max_cache_size: usize,
    /// Performance monitoring interval
    pub perf_monitoring_interval: Duration,
    /// Timeout for individual node execution
    pub node_timeout: Duration,
    /// Enable GPU acceleration
    pub enable_gpu: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            enable_parallel: true,
            max_workers: num_cpus::get(),
            enable_caching: true,
            max_cache_size: 100,
            perf_monitoring_interval: Duration::from_secs(1),
            node_timeout: Duration::from_secs(5),
            enable_gpu: false,
        }
    }
}

/// Performance metrics for execution monitoring
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    /// Total execution time for last frame
    pub total_frame_time: Duration,
    /// Individual node execution times
    pub node_times: HashMap<Uuid, Duration>,
    /// Frame count processed
    pub frame_count: u64,
    /// Average FPS over last N frames
    pub average_fps: f32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Cache hit rate
    pub cache_hit_rate: f32,
    /// Number of parallel executions
    pub parallel_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
}

/// Frame cache for caching node results
#[derive(Debug, Default)]
pub struct FrameCache {
    /// Cached frame results
    cached_frames: HashMap<(Uuid, u64), ParameterValue>,
    /// Cache access statistics
    cache_hits: u64,
    cache_misses: u64,
    /// Maximum cache size
    max_size: usize,
}

impl FrameCache {
    /// Create a new frame cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cached_frames: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            max_size,
        }
    }

    /// Get cached result for a node and frame
    pub fn get(&self, node_id: Uuid, frame: u64) -> Option<&ParameterValue> {
        self.cached_frames.get(&(node_id, frame))
            .map(|value| {
                // Note: In a real implementation, we'd increment cache_hits here
                // but since this is a read-only reference, we can't modify it
                value
            })
    }

    /// Set cached result for a node and frame
    pub fn set(&mut self, node_id: Uuid, frame: u64, value: ParameterValue) {
        // Check cache size limit
        if self.cached_frames.len() >= self.max_size {
            // Simple LRU: remove oldest entries
            let keys_to_remove: Vec<_> = self.cached_frames.keys().take(self.max_size / 4).cloned().collect();
            for key in keys_to_remove {
                self.cached_frames.remove(&key);
            }
        }

        self.cached_frames.insert((node_id, frame), value);
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.cached_frames.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f32 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f32 / total as f32
        }
    }
}

impl NodeExecutorEngine {
    /// Create a new node execution engine
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            node_manager: NodeManager::new(),
            order_manager: ExecutionOrderManager::new(),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            config,
            frame_cache: Arc::new(RwLock::new(FrameCache::new(config.max_cache_size))),
        }
    }

    /// Get the node manager
    pub fn node_manager(&mut self) -> &mut NodeManager {
        &mut self.node_manager
    }

    /// Execute a graph for a specific frame
    pub fn execute_graph(&mut self, graph: &Graph, frame: u64) -> NodeResult<ExecutionContext> {
        let start_time = Instant::now();

        // Validate graph
        GraphValidator::validate_graph(graph)?;

        // Calculate execution order
        let execution_order = self.order_manager.get_execution_order(graph)?;

        // Create execution context
        let time = frame as f64 / 30.0; // Assuming 30 FPS
        let mut context = ExecutionContext::new(frame, time, 30.0, (1920, 1080));

        // Set up GPU context if enabled
        if self.config.enable_gpu {
            context.gpu_context = Some(GpuContext {
                device: 1, // Dummy device ID
                command_queue: 1, // Dummy queue ID
                available_memory: 1024 * 1024 * 1024, // 1GB
            });
        }

        // Execute nodes in order
        if self.config.enable_parallel {
            self.execute_parallel(&execution_order, &mut context, frame)?;
        } else {
            self.execute_sequential(&execution_order, &mut context, frame)?;
        }

        // Update performance metrics
        self.update_performance_metrics(start_time, frame);

        Ok(context)
    }

    /// Execute nodes sequentially
    fn execute_sequential(
        &mut self,
        execution_order: &[Uuid],
        context: &mut ExecutionContext,
        frame: u64,
    ) -> NodeResult<()> {
        for &node_id in execution_order {
            self.execute_node_with_cache(node_id, context, frame)?;
        }
        Ok(())
    }

    /// Execute nodes in parallel where possible
    fn execute_parallel(
        &mut self,
        execution_order: &[Uuid],
        context: &mut ExecutionContext,
        frame: u64,
    ) -> NodeResult<()> {
        // For now, implement a simple parallel execution
        // In a more sophisticated implementation, we'd analyze the graph
        // to identify truly parallelizable nodes

        let mut current_batch = Vec::new();
        let mut executed_nodes = std::collections::HashSet::new();

        for &node_id in execution_order {
            // Check if all dependencies are executed
            let can_execute = self.check_dependencies_executed(node_id, &executed_nodes);

            if can_execute {
                current_batch.push(node_id);
            } else {
                // Execute current batch
                if !current_batch.is_empty() {
                    self.execute_node_batch(&current_batch, context, frame)?;
                    executed_nodes.extend(current_batch.drain(..));
                }
                // Add current node to next batch
                current_batch.push(node_id);
            }
        }

        // Execute remaining batch
        if !current_batch.is_empty() {
            self.execute_node_batch(&current_batch, context, frame)?;
        }

        Ok(())
    }

    /// Execute a batch of nodes in parallel
    fn execute_node_batch(
        &mut self,
        node_ids: &[Uuid],
        context: &mut ExecutionContext,
        frame: u64,
    ) -> NodeResult<()> {
        use std::thread;

        if node_ids.len() == 1 {
            // Single node, execute sequentially
            self.execute_node_with_cache(node_ids[0], context, frame)?;
            return Ok(());
        }

        // Multiple nodes, execute in parallel
        let mut handles = Vec::new();

        for &node_id in node_ids {
            let context_clone = unsafe { std::ptr::read(context) };
            let frame_cache = self.frame_cache.clone();
            let config = self.config.clone();

            let handle = thread::spawn(move || {
                let mut local_context = context_clone;
                let result = Self::execute_node_internal(node_id, &mut local_context, frame, &frame_cache, &config);
                (node_id, result, local_context)
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            match handle.join() {
                Ok((node_id, result, local_context)) => {
                    if let Err(e) = result {
                        error!("Node {} execution failed: {}", node_id, e);
                        return Err(e);
                    }
                    // Merge context results
                    context.outputs.extend(local_context.outputs);
                }
                Err(e) => {
                    error!("Thread panic during node execution: {:?}", e);
                    return Err(NodeError::ExecutionFailed("Thread panic during execution".to_string()));
                }
            }
        }

        Ok(())
    }

    /// Check if all dependencies of a node are executed
    fn check_dependencies_executed(&self, node_id: Uuid, executed_nodes: &std::collections::HashSet<Uuid>) -> bool {
        // This is a simplified check
        // In a real implementation, we'd analyze the graph structure
        true
    }

    /// Execute a single node with caching
    fn execute_node_with_cache(
        &mut self,
        node_id: Uuid,
        context: &mut ExecutionContext,
        frame: u64,
    ) -> NodeResult<()> {
        // Check cache first
        if self.config.enable_caching {
            let cache = self.frame_cache.read();
            if let Some(cached_result) = cache.get(node_id, frame) {
                context.set_output(node_id, cached_result.clone());
                return Ok(());
            }
        }

        // Execute node
        let result = Self::execute_node_internal(node_id, context, frame, &self.frame_cache, &self.config);

        // Cache result if successful
        if self.config.enable_caching && result.is_ok() {
            if let Some(output) = context.outputs.get(&node_id) {
                let mut cache = self.frame_cache.write();
                cache.set(node_id, frame, output.clone());
            }
        }

        result
    }

    /// Internal node execution
    fn execute_node_internal(
        node_id: Uuid,
        context: &mut ExecutionContext,
        frame: u64,
        frame_cache: &Arc<RwLock<FrameCache>>,
        config: &ExecutionConfig,
    ) -> NodeResult<()> {
        let start_time = Instant::now();

        // This is a simplified execution
        // In a real implementation, we'd get the actual node from the manager
        debug!("Executing node {} for frame {}", node_id, frame);

        // Simulate node execution time
        std::thread::sleep(Duration::from_millis(1));

        let execution_time = start_time.elapsed();
        debug!("Node {} executed in {:?}", node_id, execution_time);

        // Check timeout
        if execution_time > config.node_timeout {
            warn!("Node {} execution timeout after {:?}", node_id, execution_time);
            return Err(NodeError::ExecutionFailed("Node execution timeout".to_string()));
        }

        Ok(())
    }

    /// Update performance metrics
    fn update_performance_metrics(&self, start_time: Instant, frame: u64) {
        let execution_time = start_time.elapsed();
        let mut metrics = self.performance_metrics.write();

        metrics.total_frame_time = execution_time;
        metrics.frame_count = frame;

        // Calculate average FPS (simplified)
        if execution_time.as_secs_f32() > 0.0 {
            metrics.average_fps = 1.0 / execution_time.as_secs_f32();
        }

        // Update cache hit rate
        {
            let cache = self.frame_cache.read();
            metrics.cache_hit_rate = cache.hit_rate();
        }

        info!("Frame {} executed in {:?} (FPS: {:.2})", frame, execution_time, metrics.average_fps);
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().clone()
    }

    /// Clear frame cache
    pub fn clear_cache(&self) {
        let mut cache = self.frame_cache.write();
        cache.clear();
        info!("Frame cache cleared");
    }

    /// Invalidate execution order cache
    pub fn invalidate_order_cache(&mut self) {
        self.order_manager.invalidate_cache();
        info!("Execution order cache invalidated");
    }

    /// Get memory usage statistics
    pub fn get_memory_usage(&self) -> MemoryUsage {
        let cache = self.frame_cache.read();
        MemoryUsage {
            cache_size: cache.cached_frames.len(),
            cache_memory_bytes: cache.cached_frames.len() * std::mem::size_of::<(Uuid, u64, ParameterValue)>(),
            total_memory_bytes: cache.cached_frames.len() * std::mem::size_of::<(Uuid, u64, ParameterValue)>(),
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Number of cached frames
    pub cache_size: usize,
    /// Cache memory usage in bytes
    pub cache_memory_bytes: usize,
    /// Total memory usage in bytes
    pub total_memory_bytes: usize,
}

/// GPU execution context
#[derive(Debug, Clone)]
pub struct GpuContext {
    /// GPU device handle
    pub device: u64,
    /// Command queue
    pub command_queue: u64,
    /// Available memory
    pub available_memory: u64,
}

/// Streaming execution engine for real-time processing
pub struct StreamingExecutor {
    /// Base execution engine
    engine: NodeExecutorEngine,
    /// Streaming configuration
    streaming_config: StreamingConfig,
    /// Frame buffer for streaming
    frame_buffer: Vec<ExecutionContext>,
}

/// Configuration for streaming execution
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Buffer size for streaming
    pub buffer_size: usize,
    /// Target FPS
    pub target_fps: f32,
    /// Adaptive quality enabled
    pub adaptive_quality: bool,
    /// Drop frames if behind schedule
    pub drop_frames: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10,
            target_fps: 30.0,
            adaptive_quality: true,
            drop_frames: true,
        }
    }
}

impl StreamingExecutor {
    /// Create a new streaming executor
    pub fn new(config: ExecutionConfig, streaming_config: StreamingConfig) -> Self {
        Self {
            engine: NodeExecutorEngine::new(config),
            streaming_config,
            frame_buffer: Vec::with_capacity(streaming_config.buffer_size),
        }
    }

    /// Start streaming execution
    pub fn start_streaming(&mut self, graph: &Graph) -> NodeResult<()> {
        info!("Starting streaming execution at {} FPS", self.streaming_config.target_fps);

        let frame_interval = Duration::from_secs_f32(1.0 / self.streaming_config.target_fps);
        let mut frame_counter = 0u64;

        loop {
            let frame_start = Instant::now();

            // Execute frame
            match self.engine.execute_graph(graph, frame_counter) {
                Ok(context) => {
                    // Add to buffer
                    self.frame_buffer.push(context);

                    // Keep buffer size limited
                    if self.frame_buffer.len() > self.streaming_config.buffer_size {
                        self.frame_buffer.remove(0);
                    }

                    debug!("Streamed frame {}", frame_counter);
                }
                Err(e) => {
                    error!("Frame {} execution failed: {}", frame_counter, e);
                    if self.streaming_config.drop_frames {
                        warn!("Dropping frame {} due to error", frame_counter);
                    }
                }
            }

            frame_counter += 1;

            // Wait for next frame
            let elapsed = frame_start.elapsed();
            if elapsed < frame_interval {
                std::thread::sleep(frame_interval - elapsed);
            } else {
                warn!("Frame {} took longer than target interval", frame_counter);
            }
        }
    }

    /// Get latest frame from buffer
    pub fn get_latest_frame(&self) -> Option<&ExecutionContext> {
        self.frame_buffer.last()
    }

    /// Get frame buffer size
    pub fn buffer_size(&self) -> usize {
        self.frame_buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{Node, NodeType, Graph};

    #[test]
    fn test_execution_engine_creation() {
        let config = ExecutionConfig::default();
        let engine = NodeExecutorEngine::new(config);
        assert_eq!(engine.node_manager().node_count(), 0);
    }

    #[test]
    fn test_frame_cache() {
        let mut cache = FrameCache::new(10);
        
        // Test empty cache
        assert!(cache.get(uuid::Uuid::new_v4(), 0).is_none());
        assert_eq!(cache.hit_rate(), 0.0);

        // Test cache set/get
        let node_id = uuid::Uuid::new_v4();
        let value = ParameterValue::Float(1.0);
        cache.set(node_id, 0, value.clone());
        
        assert!(cache.get(node_id, 0).is_some());
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::default();
        
        metrics.total_frame_time = Duration::from_millis(33);
        metrics.frame_count = 1;
        
        assert_eq!(metrics.average_fps, 30.3); // ~30 FPS
    }

    #[test]
    fn test_execution_config() {
        let config = ExecutionConfig::default();
        
        assert!(config.enable_parallel);
        assert!(config.enable_caching);
        assert_eq!(config.max_workers, num_cpus::get());
        assert_eq!(config.max_cache_size, 100);
    }

    #[test]
    fn test_streaming_executor() {
        let exec_config = ExecutionConfig::default();
        let stream_config = StreamingConfig::default();
        let executor = StreamingExecutor::new(exec_config, stream_config);
        
        assert_eq!(executor.buffer_size(), 0);
        assert!(executor.get_latest_frame().is_none());
    }
}
