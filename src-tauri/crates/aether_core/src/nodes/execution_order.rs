use crate::nodes::{NodeError, NodeResult};
use aether_types::{Graph, Connection};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Calculates execution order for node graphs using topological sorting
pub struct ExecutionOrderCalculator;

impl ExecutionOrderCalculator {
    pub fn calculate_order(graph: &Graph) -> NodeResult<Vec<Uuid>> {
        let adjacency = Self::build_adjacency_list(graph)?;

        let order = Self::topological_sort(&adjacency)?;
        
        Ok(order)
    }
    
    pub fn calculate_order_from_nodes(graph: &Graph, start_nodes: &[Uuid]) -> NodeResult<Vec<Uuid>> {
        let adjacency = Self::build_adjacency_list(graph)?;
        
        let filtered_adjacency = Self::filter_adjacency_list(&adjacency, start_nodes);
        
        let order = Self::topological_sort(&filtered_adjacency)?;
        
        Ok(order)
    }
    
    fn build_adjacency_list(graph: &Graph) -> NodeResult<HashMap<Uuid, Vec<Uuid>>> {
        let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        
        for node_id in graph.nodes.keys() {
            adjacency.insert(*node_id, Vec::new());
        }
        
        for connection in graph.get_connections() {
            if !connection.enabled {
                continue;
            }
            
            adjacency
                .entry(connection.input_node_id)
                .or_insert_with(Vec::new)
                .push(connection.output_node_id);
        }
        
        Ok(adjacency)
    }
    
    fn filter_adjacency_list(
        adjacency: &HashMap<Uuid, Vec<Uuid>>,
        start_nodes: &[Uuid],
    ) -> HashMap<Uuid, Vec<Uuid>> {
        let mut filtered = HashMap::new();
        let mut visited = HashSet::new();
        let mut to_visit = start_nodes.iter().cloned().collect::<Vec<_>>();
        
        while let Some(current) = to_visit.pop() {
            if visited.contains(&current) {
                continue;
            }
            
            visited.insert(current);
            
            if let Some(neighbors) = adjacency.get(&current) {
                filtered.insert(current, neighbors.clone());
                
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        to_visit.push(*neighbor);
                    }
                }
            }
        }
        
        filtered
    }
    
    fn topological_sort(adjacency: &HashMap<Uuid, Vec<Uuid>>) -> NodeResult<Vec<Uuid>> {
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue: Vec<Uuid> = Vec::new();
        
        for node_id in adjacency.keys() {
            in_degree.insert(*node_id, 0);
        }
        
        for node_id in adjacency.keys() {
            for neighbor in &adjacency[node_id] {
                *in_degree.entry(*neighbor).or_insert(0) += 1;
            }
        }
        
        for (node_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push(*node_id);
            }
        }
        
        while let Some(current) = queue.pop() {
            result.push(current);
            
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    
                    if *degree == 0 {
                        queue.push(*neighbor);
                    }
                }
            }
        }
        
        if result.len() != adjacency.len() {
            return Err(NodeError::CircularDependency);
        }
        
        Ok(result)
    }
}

pub struct ExecutionOrderManager {
    cached_order: Option<Vec<Uuid>>,
    cache_version: u64,
    current_version: u64,
}

impl ExecutionOrderManager {
    pub fn new() -> Self {
        Self {
            cached_order: None,
            cache_version: 0,
            current_version: 0,
        }
    }
    
    pub fn get_execution_order(&mut self, graph: &Graph) -> NodeResult<Vec<Uuid>> {
        if self.cached_order.is_some() && self.cache_version == self.current_version {
            return Ok(self.cached_order.clone().unwrap());
        }
        
        // Calculate new order
        let order = ExecutionOrderCalculator::calculate_order(graph)?;
        
        // Update cache
        self.cached_order = Some(order.clone());
        self.cache_version = self.current_version;
        
        Ok(order)
    }
    
    pub fn get_execution_order_from_nodes(&mut self, graph: &Graph, start_nodes: &[Uuid]) -> NodeResult<Vec<Uuid>> {
        // For now, always calculate fresh for start nodes
        // In a more sophisticated implementation, we could cache multiple orders
        ExecutionOrderCalculator::calculate_order_from_nodes(graph, start_nodes)
    }
    
    pub fn invalidate_cache(&mut self) {
        self.current_version += 1;
    }
    
    pub fn refresh_cache(&mut self, graph: &Graph) -> NodeResult<()> {
        self.cached_order = None;
        let _order = self.get_execution_order(graph)?;
        Ok(())
    }
}

impl Default for ExecutionOrderManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ExecutionOrderAnalyzer;

impl ExecutionOrderAnalyzer {
    pub fn analyze_order(graph: &Graph, order: &[Uuid]) -> ExecutionOrderAnalysis {
        let mut analysis = ExecutionOrderAnalysis::default();
        
        // Calculate depth levels
        analysis.depth_levels = Self::calculate_depth_levels(graph, order);
        
        // Find parallelizable groups
        analysis.parallel_groups = Self::find_parallel_groups(graph, order);
        
        // Calculate critical path
        analysis.critical_path = Self::calculate_critical_path(graph, order);
        
        // Identify bottlenecks
        analysis.bottlenecks = Self::identify_bottlenecks(graph, order);
        
        analysis
    }
    
    fn calculate_depth_levels(graph: &Graph, order: &[Uuid]) -> HashMap<Uuid, usize> {
        let mut depth_levels = HashMap::new();
        let mut processed = HashSet::new();
        
        for node_id in order {
            let depth = Self::calculate_node_depth(graph, *node_id, &mut processed);
            depth_levels.insert(*node_id, depth);
        }
        
        depth_levels
    }
    
    /// Calculate depth for a specific node
    fn calculate_node_depth(graph: &Graph, node_id: Uuid, processed: &mut HashSet<Uuid>) -> usize {
        if processed.contains(&node_id) {
            return 0; // Avoid cycles
        }
        
        processed.insert(node_id);
        
        let mut max_input_depth = 0;
        
        for connection in graph.get_connections() {
            if connection.input_node_id == node_id {
                let input_depth = Self::calculate_node_depth(graph, connection.output_node_id, processed);
                max_input_depth = max_input_depth.max(input_depth);
            }
        }
        
        processed.remove(&node_id);
        max_input_depth + 1
    }
    
    /// Find groups of nodes that can be executed in parallel
    fn find_parallel_groups(graph: &Graph, order: &[Uuid]) -> Vec<Vec<Uuid>> {
        let mut groups = Vec::new();
        let mut current_group = Vec::new();
        let mut current_depth = 0;
        
        let depth_levels = Self::calculate_depth_levels(graph, order);
        
        for node_id in order {
            let depth = depth_levels.get(node_id).copied().unwrap_or(0);
            
            if depth == current_depth {
                current_group.push(*node_id);
            } else {
                if !current_group.is_empty() {
                    groups.push(current_group.clone());
                    current_group.clear();
                }
                current_group.push(*node_id);
                current_depth = depth;
            }
        }
        
        if !current_group.is_empty() {
            groups.push(current_group);
        }
        
        groups
    }
    
    /// Calculate the critical path (longest chain of dependencies)
    fn calculate_critical_path(graph: &Graph, order: &[Uuid]) -> Vec<Uuid> {
        let mut path = Vec::new();
        let mut max_path = Vec::new();
        let mut max_length = 0;
        
        // Start from output nodes
        let output_nodes: Vec<Uuid> = graph.get_output_nodes().map(|n| n.id).collect();
        
        for output_id in output_nodes {
            Self::find_longest_path(graph, output_id, &mut path, &mut HashSet::new());
            
            if path.len() > max_length {
                max_length = path.len();
                max_path = path.clone();
            }
            
            path.clear();
        }
        
        max_path
    }
    
    /// Find longest path from a node using DFS
    fn find_longest_path(
        graph: &Graph,
        node_id: Uuid,
        current_path: &mut Vec<Uuid>,
        visited: &mut HashSet<Uuid>,
    ) {
        if visited.contains(&node_id) {
            return; // Avoid cycles
        }
        
        visited.insert(node_id);
        current_path.push(node_id);
        
        // Find all input connections to this node
        let mut input_nodes = Vec::new();
        for connection in graph.get_connections() {
            if connection.input_node_id == node_id {
                input_nodes.push(connection.output_node_id);
            }
        }
        
        if input_nodes.is_empty() {
            // This is an input node, path is complete
            return;
        }
        
        // Continue with the longest input chain
        let mut max_sub_path = Vec::new();
        let mut max_length = 0;
        
        for input_id in input_nodes {
            let mut sub_path = Vec::new();
            Self::find_longest_path(graph, input_id, &mut sub_path, visited);
            
            if sub_path.len() > max_length {
                max_length = sub_path.len();
                max_sub_path = sub_path;
            }
        }
        
        current_path.extend(max_sub_path);
        visited.remove(&node_id);
    }
    
    /// Identify potential bottlenecks in execution order
    fn identify_bottlenecks(graph: &Graph, order: &[Uuid]) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Find nodes with many outputs (fan-out)
        let mut fan_out_counts: HashMap<Uuid, usize> = HashMap::new();
        for connection in graph.get_connections() {
            *fan_out_counts.entry(connection.output_node_id).or_insert(0) += 1;
        }
        
        for (node_id, count) in &fan_out_counts {
            if *count > 5 {
                if let Some(node) = graph.get_node(node_id) {
                    bottlenecks.push(Bottleneck {
                        node_id: *node_id,
                        node_name: node.name.clone(),
                        bottleneck_type: BottleneckType::HighFanOut,
                        severity: (*count as f32 / 10.0).min(1.0),
                        description: format!("Node has {} output connections", count),
                    });
                }
            }
        }
        
        // Find nodes with many inputs (fan-in)
        let mut fan_in_counts: HashMap<Uuid, usize> = HashMap::new();
        for connection in graph.get_connections() {
            *fan_in_counts.entry(connection.input_node_id).or_insert(0) += 1;
        }
        
        for (node_id, count) in &fan_in_counts {
            if *count > 5 {
                if let Some(node) = graph.get_node(node_id) {
                    bottlenecks.push(Bottleneck {
                        node_id: *node_id,
                        node_name: node.name.clone(),
                        bottleneck_type: BottleneckType::HighFanIn,
                        severity: (*count as f32 / 10.0).min(1.0),
                        description: format!("Node has {} input connections", count),
                    });
                }
            }
        }
        
        bottlenecks
    }
}

/// Analysis results for execution order
#[derive(Debug, Default)]
pub struct ExecutionOrderAnalysis {
    /// Depth levels for each node
    pub depth_levels: HashMap<Uuid, usize>,
    /// Groups of nodes that can be executed in parallel
    pub parallel_groups: Vec<Vec<Uuid>>,
    /// Critical path through the graph
    pub critical_path: Vec<Uuid>,
    /// Identified bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
}

/// Types of execution bottlenecks
#[derive(Debug, Clone)]
pub enum BottleneckType {
    HighFanOut,
    HighFanIn,
    ComplexNode,
    MemoryIntensive,
}

/// Identified bottleneck
#[derive(Debug, Clone)]
pub struct Bottleneck {
    /// Node ID
    pub node_id: Uuid,
    /// Node name
    pub node_name: String,
    /// Type of bottleneck
    pub bottleneck_type: BottleneckType,
    /// Severity (0.0 to 1.0)
    pub severity: f32,
    /// Description
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{Node, NodeType, Graph, Connection, InputPin, OutputPin, PinDataType, ParameterValue};
    
    #[test]
    fn test_simple_execution_order() {
        let mut graph = Graph::new("Test".to_string());
        
        // Create a simple chain: input -> transform -> output
        let input_node = Node::new(NodeType::Input, "Input".to_string());
        let transform_node = Node::new(NodeType::Transform, "Transform".to_string());
        let output_node = Node::new(NodeType::Output, "Output".to_string());
        
        let input_id = input_node.id;
        let transform_id = transform_node.id;
        let output_id = output_node.id;
        
        graph.add_node(input_node);
        graph.add_node(transform_node);
        graph.add_node(output_node);
        
        // Add connections (simplified - in real implementation would need proper pins)
        // For this test, we'll just add the connections directly to the graph
        let connection1 = Connection::new(input_id, Uuid::new_v4(), transform_id, Uuid::new_v4());
        let connection2 = Connection::new(transform_id, Uuid::new_v4(), output_id, Uuid::new_v4());
        
        graph.add_connection(connection1);
        graph.add_connection(connection2);
        
        // Calculate execution order
        let order = ExecutionOrderCalculator::calculate_order(&graph).unwrap();
        
        // Should have all three nodes
        assert_eq!(order.len(), 3);
        
        // Input should come before transform, transform before output
        let input_pos = order.iter().position(|&id| id == input_id).unwrap();
        let transform_pos = order.iter().position(|&id| id == transform_id).unwrap();
        let output_pos = order.iter().position(|&id| id == output_id).unwrap();
        
        assert!(input_pos < transform_pos);
        assert!(transform_pos < output_pos);
    }
    
    #[test]
    fn test_parallel_execution_groups() {
        let mut graph = Graph::new("Test".to_string());
        
        // Create two parallel branches from one input
        let input_node = Node::new(NodeType::Input, "Input".to_string());
        let branch1_node = Node::new(NodeType::Transform, "Branch1".to_string());
        let branch2_node = Node::new(NodeType::Transform, "Branch2".to_string());
        let output_node = Node::new(NodeType::Output, "Output".to_string());
        
        let input_id = input_node.id;
        let branch1_id = branch1_node.id;
        let branch2_id = branch2_node.id;
        let output_id = output_node.id;
        
        graph.add_node(input_node);
        graph.add_node(branch1_node);
        graph.add_node(branch2_node);
        graph.add_node(output_node);
        
        // Add connections: input -> branch1, input -> branch2, branch1 -> output, branch2 -> output
        let conn1 = Connection::new(input_id, Uuid::new_v4(), branch1_id, Uuid::new_v4());
        let conn2 = Connection::new(input_id, Uuid::new_v4(), branch2_id, Uuid::new_v4());
        let conn3 = Connection::new(branch1_id, Uuid::new_v4(), output_id, Uuid::new_v4());
        let conn4 = Connection::new(branch2_id, Uuid::new_v4(), output_id, Uuid::new_v4());
        
        graph.add_connection(conn1);
        graph.add_connection(conn2);
        graph.add_connection(conn3);
        graph.add_connection(conn4);
        
        // Calculate execution order
        let order = ExecutionOrderCalculator::calculate_order(&graph).unwrap();
        
        // Analyze for parallel groups
        let analysis = ExecutionOrderAnalyzer::analyze_order(&graph, &order);
        
        // Should have parallel groups
        assert!(!analysis.parallel_groups.is_empty());
        
        // Check that branch1 and branch2 are in the same or adjacent groups
        let branch1_group = analysis.parallel_groups.iter()
            .position(|group| group.contains(&branch1_id));
        let branch2_group = analysis.parallel_groups.iter()
            .position(|group| group.contains(&branch2_id));
        
        assert!(branch1_group.is_some());
        assert!(branch2_group.is_some());
    }
    
    #[test]
    fn test_execution_order_manager() {
        let mut manager = ExecutionOrderManager::new();
        let mut graph = Graph::new("Test".to_string());
        
        // Add a simple node
        let node = Node::new(NodeType::Input, "Test".to_string());
        graph.add_node(node);
        
        // First call should calculate order
        let order1 = manager.get_execution_order(&graph).unwrap();
        assert_eq!(order1.len(), 1);
        
        // Second call should use cache
        let order2 = manager.get_execution_order(&graph).unwrap();
        assert_eq!(order1, order2);
        
        // Invalidate cache
        manager.invalidate_cache();
        
        // Next call should recalculate
        let order3 = manager.get_execution_order(&graph).unwrap();
        assert_eq!(order1, order3); // Should be same order but recalculated
    }
}
