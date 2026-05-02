use aether_types::Graph;
use std::collections::HashMap;
use uuid::Uuid;
use log::debug;

/// Cache for execution order calculations
pub struct ExecutionCache {
    cached_orders: HashMap<GraphHash, Vec<Uuid>>,
    hit_count: u64,
    miss_count: u64,
}

impl ExecutionCache {
    /// Create a new execution cache
    pub fn new() -> Self {
        Self {
            cached_orders: HashMap::new(),
            hit_count: 0,
            miss_count: 0,
        }
    }
    
    /// Get cached execution order for a graph
    pub fn get_cached_order(&mut self, graph: &Graph) -> Option<Vec<Uuid>> {
        let graph_hash = self.calculate_graph_hash(graph);
        
        if let Some(order) = self.cached_orders.get(&graph_hash) {
            self.hit_count += 1;
            debug!("Cache hit for graph hash: {:?}", graph_hash);
            Some(order.clone())
        } else {
            self.miss_count += 1;
            debug!("Cache miss for graph hash: {:?}", graph_hash);
            None
        }
    }
    
    /// Cache execution order for a graph
    pub fn cache_order(&mut self, graph: &Graph, order: &[Uuid]) {
        let graph_hash = self.calculate_graph_hash(graph);
        
        debug!("Caching execution order for graph hash: {:?}", graph_hash);
        
        self.cached_orders.insert(graph_hash, order.to_vec());
    }
    
    /// Check if cache is valid for the graph
    pub fn is_valid_for_graph(&self, graph: &Graph) -> bool {
        let graph_hash = self.calculate_graph_hash(graph);
        self.cached_orders.contains_key(&graph_hash)
    }
    
    /// Invalidate cache for a specific graph
    pub fn invalidate_cache(&mut self, graph: &Graph) {
        let graph_hash = self.calculate_graph_hash(graph);
        
        if self.cached_orders.remove(&graph_hash).is_some() {
            debug!("Invalidated cache for graph hash: {:?}", graph_hash);
        }
    }
    
    /// Clear all cached orders
    pub fn clear_all(&mut self) {
        debug!("Clearing all execution order cache ({} entries)", self.cached_orders.len());
        
        self.cached_orders.clear();
        self.hit_count = 0;
        self.miss_count = 0;
    }
    
    /// Get number of cached orders
    pub fn len(&self) -> usize {
        self.cached_orders.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cached_orders.is_empty()
    }
    
    /// Get cache hit count
    pub fn get_hit_count(&self) -> u64 {
        self.hit_count
    }
    
    /// Get cache miss count
    pub fn get_miss_count(&self) -> u64 {
        self.miss_count
    }
    
    /// Calculate hash for a graph
    fn calculate_graph_hash(&self, graph: &Graph) -> GraphHash {
        // Simple hash based on node count, connection count, and enabled status
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        
        // Hash node count and IDs
        std::hash::Hash::hash(&graph.nodes.len(), &mut hasher);
        for node_id in graph.nodes.keys() {
            std::hash::Hash::hash(node_id, &mut hasher);
        }
        
        // Hash connection count and enabled connections
        std::hash::Hash::hash(&graph.connections.len(), &mut hasher);
        for connection in &graph.connections {
            if connection.enabled {
                std::hash::Hash::hash(&connection.output_node_id, &mut hasher);
                std::hash::Hash::hash(&connection.input_node_id, &mut hasher);
                std::hash::Hash::hash(&connection.output_pin_id, &mut hasher);
                std::hash::Hash::hash(&connection.input_pin_id, &mut hasher);
            }
        }
        
        GraphHash(std::hash::Hasher::finish(&hasher))
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            cached_orders: self.cached_orders.len(),
            hit_count: self.hit_count,
            miss_count: self.miss_count,
        }
    }
    
    /// Prune old cache entries (keep most recent)
    pub fn prune_cache(&mut self, max_entries: usize) {
        if self.cached_orders.len() <= max_entries {
            return;
        }
        
        debug!("Pruning cache from {} to {} entries", self.cached_orders.len(), max_entries);
        
        // Simple pruning: remove oldest entries (this is a basic implementation)
        // In a real implementation, you might want LRU or time-based eviction
        let entries_to_remove = self.cached_orders.len() - max_entries;
        let keys_to_remove: Vec<GraphHash> = self.cached_orders.keys()
            .take(entries_to_remove)
            .copied()
            .collect();
        
        for key in keys_to_remove {
            self.cached_orders.remove(&key);
        }
        
        debug!("Cache pruned to {} entries", self.cached_orders.len());
    }
    
    /// Get memory usage estimate
    pub fn estimate_memory_usage(&self) -> usize {
        // Rough estimate: each UUID is 16 bytes, each Vec has overhead
        let uuid_size = 16;
        let vec_overhead = 24; // Approximate
        let hashmap_overhead = 24; // Approximate per entry
        
        let total_uuids: usize = self.cached_orders.values()
            .map(|order| order.len())
            .sum();
        
        let uuid_memory = total_uuids * uuid_size;
        let vec_memory = self.cached_orders.len() * vec_overhead;
        let hashmap_memory = self.cached_orders.len() * hashmap_overhead;
        
        uuid_memory + vec_memory + hashmap_memory
    }
}

impl Default for ExecutionCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash key for graph caching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GraphHash(u64);

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached orders
    pub cached_orders: usize,
    /// Number of cache hits
    pub hit_count: u64,
    /// Number of cache misses
    pub miss_count: u64,
}

impl CacheStats {
    /// Get cache hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
    
    /// Get total number of requests
    pub fn total_requests(&self) -> u64 {
        self.hit_count + self.miss_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::execution_order::tests::create_test_graph;
    
    #[test]
    fn test_execution_cache() {
        let mut cache = ExecutionCache::new();
        let graph = create_test_graph();
        let test_order = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        
        // Initially empty
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert!(!cache.is_valid_for_graph(&graph));
        
        // Cache miss
        assert_eq!(cache.get_cached_order(&graph), None);
        assert_eq!(cache.get_miss_count(), 1);
        assert_eq!(cache.get_hit_count(), 0);
        
        // Cache order
        cache.cache_order(&graph, &test_order);
        assert_eq!(cache.len(), 1);
        assert!(cache.is_valid_for_graph(&graph));
        
        // Cache hit
        let cached_order = cache.get_cached_order(&graph).unwrap();
        assert_eq!(cached_order, test_order);
        assert_eq!(cache.get_hit_count(), 1);
        assert_eq!(cache.get_miss_count(), 1);
        
        // Invalidate cache
        cache.invalidate_cache(&graph);
        assert!(!cache.is_valid_for_graph(&graph));
        assert_eq!(cache.len(), 0);
    }
    
    #[test]
    fn test_cache_stats() {
        let mut cache = ExecutionCache::new();
        let graph = create_test_graph();
        let test_order = vec![Uuid::new_v4()];
        
        let stats = cache.get_stats();
        assert_eq!(stats.cached_orders, 0);
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 0);
        assert_eq!(stats.hit_ratio(), 0.0);
        
        // Cache miss
        cache.get_cached_order(&graph);
        let stats = cache.get_stats();
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_ratio(), 0.0);
        
        // Cache order and hit
        cache.cache_order(&graph, &test_order);
        cache.get_cached_order(&graph);
        let stats = cache.get_stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_ratio(), 0.5);
    }
    
    #[test]
    fn test_clear_all() {
        let mut cache = ExecutionCache::new();
        let graph = create_test_graph();
        let test_order = vec![Uuid::new_v4()];
        
        // Add some entries
        cache.cache_order(&graph, &test_order);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get_hit_count(), 0);
        assert_eq!(cache.get_miss_count(), 0);
        
        // Clear all
        cache.clear_all();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get_hit_count(), 0);
        assert_eq!(cache.get_miss_count(), 0);
    }
    
    #[test]
    fn test_prune_cache() {
        let mut cache = ExecutionCache::new();
        
        // Add multiple entries
        for i in 0..10 {
            let mut graph = create_test_graph();
            // Modify graph slightly to create different hashes
            graph.connections.clear(); // This will change the hash
            let test_order = vec![Uuid::new_v4()];
            cache.cache_order(&graph, &test_order);
        }
        
        assert_eq!(cache.len(), 10);
        
        // Prune to 5 entries
        cache.prune_cache(5);
        assert_eq!(cache.len(), 5);
        
        // Prune to same size (no change)
        cache.prune_cache(5);
        assert_eq!(cache.len(), 5);
        
        // Prune to larger size (no change)
        cache.prune_cache(10);
        assert_eq!(cache.len(), 5);
    }
    
    #[test]
    fn test_estimate_memory_usage() {
        let mut cache = ExecutionCache::new();
        let graph = create_test_graph();
        
        // Initially no memory usage
        assert_eq!(cache.estimate_memory_usage(), 0);
        
        // Add some entries
        cache.cache_order(&graph, &vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]);
        
        let usage = cache.estimate_memory_usage();
        assert!(usage > 0);
        
        // Should be roughly: 3 * 16 (UUIDs) + overhead
        let expected_min = 3 * 16; // Minimum for 3 UUIDs
        assert!(usage >= expected_min);
    }
    
    #[test]
    fn test_graph_hash_consistency() {
        let mut cache = ExecutionCache::new();
        let graph = create_test_graph();
        
        let hash1 = cache.calculate_graph_hash(&graph);
        let hash2 = cache.calculate_graph_hash(&graph);
        
        // Same graph should produce same hash
        assert_eq!(hash1, hash2);
        
        // Modified graph should produce different hash
        let mut modified_graph = graph.clone();
        modified_graph.connections.clear();
        let hash3 = cache.calculate_graph_hash(&modified_graph);
        
        assert_ne!(hash1, hash3);
    }
}
