use crate::nodes::{NodeError, NodeResult};
use crate::nodes::execution_order::{ExecutionOrderCalculator, ExecutionCache};
use aether_types::Graph;
use uuid::Uuid;
use log::debug;

/// Manages execution order calculation and caching
pub struct ExecutionOrderManager {
    cache: ExecutionCache,
    calculator: ExecutionOrderCalculator,
}

impl ExecutionOrderManager {
    /// Create a new execution order manager
    pub fn new() -> Self {
        Self {
            cache: ExecutionCache::new(),
            calculator: ExecutionOrderCalculator,
        }
    }
    
    /// Get execution order for the entire graph
    pub fn get_execution_order(&mut self, graph: &Graph) -> NodeResult<Vec<Uuid>> {
        debug!("Getting execution order for graph with {} nodes", graph.nodes.len());
        
        // Check cache first
        if let Some(cached_order) = self.cache.get_cached_order(graph) {
            debug!("Using cached execution order");
            return Ok(cached_order);
        }
        
        // Calculate new order
        let order = self.calculator.calculate_order(graph)?;
        
        // Validate order
        self.calculator.validate_order(&order, graph)?;
        
        // Cache the result
        self.cache.cache_order(graph, &order);
        
        debug!("Calculated and cached new execution order");
        
        Ok(order)
    }
    
    /// Get execution order starting from specific nodes
    pub fn get_partial_execution_order(&mut self, graph: &Graph, start_nodes: &[Uuid]) -> NodeResult<Vec<Uuid>> {
        debug!("Getting partial execution order from {} start nodes", start_nodes.len());
        
        // For partial orders, we don't cache (too many variations)
        let order = self.calculator.calculate_order_from_nodes(graph, start_nodes)?;
        
        // Validate order
        self.calculator.validate_order(&order, graph)?;
        
        debug!("Calculated partial execution order: {} nodes", order.len());
        
        Ok(order)
    }
    
    /// Force recalculation of execution order (bypass cache)
    pub fn force_recalculate(&mut self, graph: &Graph) -> NodeResult<Vec<Uuid>> {
        debug!("Force recalculating execution order");
        
        // Clear cache for this graph
        self.cache.invalidate_cache(graph);
        
        // Calculate new order
        self.get_execution_order(graph)
    }
    
    /// Check if cached order is valid for the graph
    pub fn is_cache_valid(&self, graph: &Graph) -> bool {
        self.cache.is_valid_for_graph(graph)
    }
    
    /// Clear all cached orders
    pub fn clear_cache(&mut self) {
        debug!("Clearing all execution order cache");
        self.cache.clear_all();
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_orders: self.cache.len(),
            cache_hits: self.cache.get_hit_count(),
            cache_misses: self.cache.get_miss_count(),
        }
    }
    
    /// Get dependencies for a specific node
    pub fn get_node_dependencies(&self, graph: &Graph, node_id: Uuid) -> NodeResult<Vec<Uuid>> {
        self.calculator.get_node_dependencies(graph, node_id)
    }
    
    /// Get dependents for a specific node
    pub fn get_node_dependents(&self, graph: &Graph, node_id: Uuid) -> NodeResult<Vec<Uuid>> {
        self.calculator.get_node_dependents(graph, node_id)
    }
    
    /// Check if node execution order is valid
    pub fn validate_node_order(&self, graph: &Graph, node_order: &[Uuid]) -> NodeResult<()> {
        self.calculator.validate_order(node_order, graph)
    }
    
    /// Get execution order with performance monitoring
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

/// Result of execution order calculation with timing information
#[derive(Debug, Clone)]
pub struct ExecutionOrderResult {
    /// The calculated execution order
    pub order: Vec<Uuid>,
    /// Time taken to calculate the order
    pub calculation_time: std::time::Duration,
    /// Whether the result was from cache
    pub cache_hit: bool,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached orders
    pub cached_orders: usize,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
}

impl CacheStats {
    /// Get cache hit ratio
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
        
        // First calculation should compute and cache
        let order1 = manager.get_execution_order(&graph).unwrap();
        assert_eq!(order1.len(), 3);
        assert!(!manager.is_cache_valid(&graph));
        
        // Second calculation should use cache
        let order2 = manager.get_execution_order(&graph).unwrap();
        assert_eq!(order1, order2);
        assert!(manager.is_cache_valid(&graph));
        
        // Force recalculation should recompute
        let order3 = manager.force_recalculate(&graph).unwrap();
        assert_eq!(order1, order3);
    }
    
    #[test]
    fn test_partial_execution_order() {
        let mut manager = ExecutionOrderManager::new();
        let graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        
        let partial_order = manager.get_partial_execution_order(&graph, &[node_ids[1]]).unwrap();
        
        // Should include all reachable nodes
        assert_eq!(partial_order.len(), 3);
    }
    
    #[test]
    fn test_cache_operations() {
        let mut manager = ExecutionOrderManager::new();
        let graph = create_test_graph();
        
        // Initially no cache
        assert!(!manager.is_cache_valid(&graph));
        
        // Calculate to populate cache
        manager.get_execution_order(&graph).unwrap();
        assert!(manager.is_cache_valid(&graph));
        
        // Clear cache
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
        
        // First calculation (miss)
        manager.get_execution_order(&graph).unwrap();
        let stats = manager.get_cache_stats();
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_ratio(), 0.0);
        
        // Second calculation (hit)
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
        assert!(!result.cache_hit); // First calculation
        assert!(result.calculation_time.as_nanos() > 0);
        
        // Second calculation should be cached
        let result2 = manager.get_execution_order_with_timing(&graph).unwrap();
        assert_eq!(result.order, result2.order);
        assert!(result2.cache_hit); // Second calculation
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
        
        // Valid order should pass
        assert!(manager.validate_node_order(&graph, &order).is_ok());
        
        // Invalid order should fail
        let mut invalid_order = order.clone();
        invalid_order.reverse();
        assert!(manager.validate_node_order(&graph, &invalid_order).is_err());
    }
}
