use crate::nodes::{NodeError, NodeResult};
use crate::nodes::execution_order::{ExecutionOrderCalculator, ExecutionCache};
use aether_types::Graph;
use uuid::Uuid;
use log::debug;


pub struct ExecutionOrderManager {
    cache: ExecutionCache,
    calculator: ExecutionOrderCalculator,
}

impl ExecutionOrderManager {

    pub fn new() -> Self {
        Self {
            cache: ExecutionCache::new(),
            calculator: ExecutionOrderCalculator,
        }
    }

    pub fn get_execution_order(&mut self, graph: &Graph) -> NodeResult<Vec<Uuid>> {
        debug!("Getting execution order for graph with {} nodes", graph.nodes.len());

        if let Some(cached_order) = self.cache.get_cached_order(graph) {
            debug!("Using cached execution order");
            return Ok(cached_order);
        }

        let order = ExecutionOrderCalculator::calculate_order(graph)?;

        ExecutionOrderCalculator::validate_order(&order, graph)?;

        self.cache.cache_order(graph, &order);

        debug!("Calculated and cached new execution order");

        Ok(order)
    }

    pub fn get_partial_execution_order(&mut self, graph: &Graph, start_nodes: &[Uuid]) -> NodeResult<Vec<Uuid>> {
        debug!("Getting partial execution order from {} start nodes", start_nodes.len());

        let order = ExecutionOrderCalculator::calculate_order_from_nodes(graph, start_nodes)?;

        ExecutionOrderCalculator::validate_order(&order, graph)?;

        debug!("Calculated partial execution order: {} nodes", order.len());

        Ok(order)
    }


    pub fn force_recalculate(&mut self, graph: &Graph) -> NodeResult<Vec<Uuid>> {
        debug!("Force recalculating execution order");
        self.cache.invalidate_cache(graph);

        self.get_execution_order(graph)
    }


    pub fn is_cache_valid(&self, graph: &Graph) -> bool {
        self.cache.is_valid_for_graph(graph)
    }


    pub fn clear_cache(&mut self) {
        debug!("Clearing all execution order cache");
        self.cache.clear_all();
    }


    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_orders: self.cache.len(),
            cache_hits: self.cache.get_hit_count(),
            cache_misses: self.cache.get_miss_count(),
        }
    }


    pub fn get_node_dependencies(&self, graph: &Graph, node_id: Uuid) -> NodeResult<Vec<Uuid>> {
        ExecutionOrderCalculator::get_node_dependencies(graph, node_id)
    }


    pub fn get_node_dependents(&self, graph: &Graph, node_id: Uuid) -> NodeResult<Vec<Uuid>> {
        ExecutionOrderCalculator::get_node_dependents(graph, node_id)
    }


    pub fn validate_node_order(&self, graph: &Graph, node_order: &[Uuid]) -> NodeResult<()> {
        ExecutionOrderCalculator::validate_order(node_order, graph)
    }


    pub fn get_execution_order_with_timing(&mut self, graph: &Graph) -> NodeResult<ExecutionOrderResult> {
        let start_time = std::time::Instant::now();

        let order = self.get_execution_order(graph)?;

        let elapsed = start_time.elapsed();

        debug!("Execution order calculation took {:?}", elapsed);

        Ok(ExecutionOrderResult {
            order,
            calculation_time: elapsed,
            cache_hit: self.cache.is_valid_for_graph(graph),
        })
    }
}

impl Default for ExecutionOrderManager {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone)]
pub struct ExecutionOrderResult {

    pub order: Vec<Uuid>,

    pub calculation_time: std::time::Duration,

    pub cache_hit: bool,
}


#[derive(Debug, Clone)]
pub struct CacheStats {

    pub cached_orders: usize,

    pub cache_hits: u64,

    pub cache_misses: u64,
}

impl CacheStats {

    pub fn hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::execution_order::tests::create_test_graph;

    #[test]
    fn test_execution_order_manager() {
        let mut manager = ExecutionOrderManager::new();
        let graph = create_test_graph();


        let order1 = manager.get_execution_order(&graph).unwrap();
        assert_eq!(order1.len(), 3);
        assert!(!manager.is_cache_valid(&graph));


        let order2 = manager.get_execution_order(&graph).unwrap();
        assert_eq!(order1, order2);
        assert!(manager.is_cache_valid(&graph));


        let order3 = manager.force_recalculate(&graph).unwrap();
        assert_eq!(order1, order3);
    }

    #[test]
    fn test_partial_execution_order() {
        let mut manager = ExecutionOrderManager::new();
        let graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();

        let partial_order = manager.get_partial_execution_order(&graph, &[node_ids[1]]).unwrap();


        assert_eq!(partial_order.len(), 3);
    }

    #[test]
    fn test_cache_operations() {
        let mut manager = ExecutionOrderManager::new();
        let graph = create_test_graph();


        assert!(!manager.is_cache_valid(&graph));


        manager.get_execution_order(&graph).unwrap();
        assert!(manager.is_cache_valid(&graph));


        manager.clear_cache();
        assert!(!manager.is_cache_valid(&graph));
    }

    #[test]
    fn test_cache_stats() {
        let mut manager = ExecutionOrderManager::new();
        let graph = create_test_graph();

        let stats = manager.get_cache_stats();
        assert_eq!(stats.cached_orders, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.hit_ratio(), 0.0);


        manager.get_execution_order(&graph).unwrap();
        let stats = manager.get_cache_stats();
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_ratio(), 0.0);


        manager.get_execution_order(&graph).unwrap();
        let stats = manager.get_cache_stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_ratio(), 0.5);
    }

    #[test]
    fn test_execution_order_with_timing() {
        let mut manager = ExecutionOrderManager::new();
        let graph = create_test_graph();

        let result = manager.get_execution_order_with_timing(&graph).unwrap();

        assert_eq!(result.order.len(), 3);
        assert!(!result.cache_hit);
        assert!(result.calculation_time.as_nanos() > 0);


        let result2 = manager.get_execution_order_with_timing(&graph).unwrap();
        assert_eq!(result.order, result2.order);
        assert!(result2.cache_hit);
    }

    #[test]
    fn test_node_dependencies() {
        let manager = ExecutionOrderManager::new();
        let graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();

        let deps = manager.get_node_dependencies(&graph, node_ids[2]).unwrap();
        assert_eq!(deps.len(), 2);

        let dependents = manager.get_node_dependents(&graph, node_ids[0]).unwrap();
        assert_eq!(dependents.len(), 1);
    }

    #[test]
    fn test_validate_node_order() {
        let manager = ExecutionOrderManager::new();
        let graph = create_test_graph();
        let order = manager.get_execution_order(&graph).unwrap();


        assert!(manager.validate_node_order(&graph, &order).is_ok());


        let mut invalid_order = order.clone();
        invalid_order.reverse();
        assert!(manager.validate_node_order(&graph, &invalid_order).is_err());
    }
}
