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


pub struct NodeExecutorEngine {

    node_manager: NodeManager,

    order_manager: ExecutionOrderManager,

    performance_metrics: Arc<RwLock<PerformanceMetrics>>,

    config: ExecutionConfig,

    frame_cache: Arc<RwLock<FrameCache>>,
}


#[derive(Debug, Clone)]
pub struct ExecutionConfig {

    pub enable_parallel: bool,

    pub max_workers: usize,

    pub enable_caching: bool,

    pub max_cache_size: usize,

    pub perf_monitoring_interval: Duration,

    pub node_timeout: Duration,

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


#[derive(Debug, Default)]
pub struct PerformanceMetrics {

    pub total_frame_time: Duration,

    pub node_times: HashMap<Uuid, Duration>,

    pub frame_count: u64,

    pub average_fps: f32,

    pub memory_usage: u64,

    pub cache_hit_rate: f32,

    pub parallel_executions: u64,

    pub failed_executions: u64,
}


#[derive(Debug, Default)]
pub struct FrameCache {

    cached_frames: HashMap<(Uuid, u64), ParameterValue>,

    cache_hits: u64,
    cache_misses: u64,

    max_size: usize,
}

impl FrameCache {

    pub fn new(max_size: usize) -> Self {
        Self {
            cached_frames: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            max_size,
        }
    }


    pub fn get(&self, node_id: Uuid, frame: u64) -> Option<&ParameterValue> {
        self.cached_frames.get(&(node_id, frame))
            .map(|value| {

                value
            })
    }


    pub fn set(&mut self, node_id: Uuid, frame: u64, value: ParameterValue) {

        if self.cached_frames.len() >= self.max_size {

            let keys_to_remove: Vec<_> = self.cached_frames.keys().take(self.max_size / 4).cloned().collect();
            for key in keys_to_remove {
                self.cached_frames.remove(&key);
            }
        }

        self.cached_frames.insert((node_id, frame), value);
    }


    pub fn clear(&mut self) {
        self.cached_frames.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }


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

    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            node_manager: NodeManager::new(),
            order_manager: ExecutionOrderManager::new(),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            config,
            frame_cache: Arc::new(RwLock::new(FrameCache::new(config.max_cache_size))),
        }
    }


    pub fn node_manager(&mut self) -> &mut NodeManager {
        &mut self.node_manager
    }


    pub fn execute_graph(&mut self, graph: &Graph, frame: u64) -> NodeResult<ExecutionContext> {
        let start_time = Instant::now();


        GraphValidator::validate_graph(graph)?;


        let execution_order = self.order_manager.get_execution_order(graph)?;


        let time = frame as f64 / 30.0;
        let mut context = ExecutionContext::new(frame, time, 30.0, (1920, 1080));


        if self.config.enable_gpu {
            context.gpu_context = Some(GpuContext {
                device: 1,
                command_queue: 1,
                available_memory: 1024 * 1024 * 1024,
            });
        }


        if self.config.enable_parallel {
            self.execute_parallel(&execution_order, &mut context, frame, graph)?;
        } else {
            self.execute_sequential(&execution_order, &mut context, frame)?;
        }


        self.update_performance_metrics(start_time, frame);

        Ok(context)
    }


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


    fn execute_parallel(
        &mut self,
        execution_order: &[Uuid],
        context: &mut ExecutionContext,
        frame: u64,
        graph: &Graph,
    ) -> NodeResult<()> {

        let mut current_batch = Vec::new();
        let mut executed_nodes = std::collections::HashSet::new();

        for &node_id in execution_order {

            let can_execute = self.check_dependencies_executed(node_id, &executed_nodes, graph);

            if can_execute {
                current_batch.push(node_id);
            } else {

                if !current_batch.is_empty() {
                    self.execute_node_batch(&current_batch, context, frame)?;
                    executed_nodes.extend(current_batch.drain(..));
                }

                current_batch.push(node_id);
            }
        }


        if !current_batch.is_empty() {
            self.execute_node_batch(&current_batch, context, frame)?;
        }

        Ok(())
    }


    fn execute_node_batch(
        &mut self,
        node_ids: &[Uuid],
        context: &mut ExecutionContext,
        frame: u64,
    ) -> NodeResult<()> {
        use std::thread;

        if node_ids.len() == 1 {

            self.execute_node_with_cache(node_ids[0], context, frame)?;
            return Ok(());
        }


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


        for handle in handles {
            match handle.join() {
                Ok((node_id, result, local_context)) => {
                    if let Err(e) = result {
                        error!("Node {} execution failed: {}", node_id, e);
                        return Err(e);
                    }

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


    fn check_dependencies_executed(&self, node_id: Uuid, executed_nodes: &std::collections::HashSet<Uuid>, graph: &Graph) -> bool {

        if let Some(node) = self.node_manager.get_node_metadata(&node_id) {

            for input_pin in &node.inputs {
                if let Some(connection_id) = &input_pin.connection {

                    if let Some(connection) = graph.get_connection(connection_id) {
                        if !executed_nodes.contains(&connection.output_node_id) {
                            return false;
                        }
                    }
                }
            }
        }


        true
    }


    fn execute_node_with_cache(
        &mut self,
        node_id: Uuid,
        context: &mut ExecutionContext,
        frame: u64,
    ) -> NodeResult<()> {

        if self.config.enable_caching {
            let cache = self.frame_cache.read();
            if let Some(cached_result) = cache.get(node_id, frame) {
                context.set_output(node_id, cached_result.clone());
                return Ok(());
            }
        }


        let result = Self::execute_node_internal(node_id, context, frame, &self.frame_cache, &self.config);


        if self.config.enable_caching && result.is_ok() {
            if let Some(output) = context.outputs.get(&node_id) {
                let mut cache = self.frame_cache.write();
                cache.set(node_id, frame, output.clone());
            }
        }

        result
    }


    fn execute_node_internal(
        node_id: Uuid,
        context: &mut ExecutionContext,
        frame: u64,
        frame_cache: &Arc<RwLock<FrameCache>>,
        config: &ExecutionConfig,
    ) -> NodeResult<()> {
        let start_time = Instant::now();


        debug!("Executing node {} for frame {}", node_id, frame);


        std::thread::sleep(Duration::from_millis(1));

        let execution_time = start_time.elapsed();
        debug!("Node {} executed in {:?}", node_id, execution_time);


        if execution_time > config.node_timeout {
            warn!("Node {} execution timeout after {:?}", node_id, execution_time);
            return Err(NodeError::ExecutionFailed("Node execution timeout".to_string()));
        }

        Ok(())
    }


    fn update_performance_metrics(&self, start_time: Instant, frame: u64) {
        let execution_time = start_time.elapsed();
        let mut metrics = self.performance_metrics.write();

        metrics.total_frame_time = execution_time;
        metrics.frame_count = frame;


        if execution_time.as_secs_f32() > 0.0 {
            metrics.average_fps = 1.0 / execution_time.as_secs_f32();
        }


        {
            let cache = self.frame_cache.read();
            metrics.cache_hit_rate = cache.hit_rate();
        }

        info!("Frame {} executed in {:?} (FPS: {:.2})", frame, execution_time, metrics.average_fps);
    }


    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().clone()
    }


    pub fn clear_cache(&self) {
        let mut cache = self.frame_cache.write();
        cache.clear();
        info!("Frame cache cleared");
    }


    pub fn invalidate_order_cache(&mut self) {
        self.order_manager.invalidate_cache();
        info!("Execution order cache invalidated");
    }


    pub fn get_memory_usage(&self) -> MemoryUsage {
        let cache = self.frame_cache.read();
        MemoryUsage {
            cache_size: cache.cached_frames.len(),
            cache_memory_bytes: cache.cached_frames.len() * std::mem::size_of::<(Uuid, u64, ParameterValue)>(),
            total_memory_bytes: cache.cached_frames.len() * std::mem::size_of::<(Uuid, u64, ParameterValue)>(),
        }
    }
}


#[derive(Debug, Clone)]
pub struct MemoryUsage {

    pub cache_size: usize,

    pub cache_memory_bytes: usize,

    pub total_memory_bytes: usize,
}


#[derive(Debug, Clone)]
pub struct GpuContext {

    pub device: u64,

    pub command_queue: u64,

    pub available_memory: u64,
}


pub struct StreamingExecutor {

    engine: NodeExecutorEngine,

    streaming_config: StreamingConfig,

    frame_buffer: Vec<ExecutionContext>,
}


#[derive(Debug, Clone)]
pub struct StreamingConfig {

    pub buffer_size: usize,

    pub target_fps: f32,

    pub adaptive_quality: bool,

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

    pub fn new(config: ExecutionConfig, streaming_config: StreamingConfig) -> Self {
        Self {
            engine: NodeExecutorEngine::new(config),
            streaming_config,
            frame_buffer: Vec::with_capacity(streaming_config.buffer_size),
        }
    }


    pub fn start_streaming(&mut self, graph: &Graph) -> NodeResult<()> {
        info!("Starting streaming execution at {} FPS", self.streaming_config.target_fps);

        let frame_interval = Duration::from_secs_f32(1.0 / self.streaming_config.target_fps);
        let mut frame_counter = 0u64;

        loop {
            let frame_start = Instant::now();


            match self.engine.execute_graph(graph, frame_counter) {
                Ok(context) => {

                    self.frame_buffer.push(context);


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


            let elapsed = frame_start.elapsed();
            if elapsed < frame_interval {
                std::thread::sleep(frame_interval - elapsed);
            } else {
                warn!("Frame {} took longer than target interval", frame_counter);
            }
        }
    }


    pub fn get_latest_frame(&self) -> Option<&ExecutionContext> {
        self.frame_buffer.last()
    }


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


        assert!(cache.get(uuid::Uuid::new_v4(), 0).is_none());
        assert_eq!(cache.hit_rate(), 0.0);


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

        assert_eq!(metrics.average_fps, 30.3);
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
