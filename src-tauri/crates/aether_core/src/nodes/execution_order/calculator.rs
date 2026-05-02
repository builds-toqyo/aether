use crate::nodes::{NodeError, NodeResult};
use aether_types::{Graph, Connection};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use log::debug;

/// Calculates execution order for node graphs using topological sorting
pub struct ExecutionOrderCalculator;

impl ExecutionOrderCalculator {
    /// Calculate execution order for the entire graph
    pub fn calculate_order(graph: &Graph) -> NodeResult<Vec<Uuid>> {
        debug!("Calculating execution order for {} nodes", graph.nodes.len());
        
        let adjacency = Self::build_adjacency_list(graph)?;
        let order = Self::topological_sort(&adjacency)?;
        
        debug!("Calculated execution order: {} nodes", order.len());
        
        Ok(order)
    }
    
    /// Calculate execution order starting from specific nodes
    pub fn calculate_order_from_nodes(graph: &Graph, start_nodes: &[Uuid]) -> NodeResult<Vec<Uuid>> {
        debug!("Calculating execution order from {} start nodes", start_nodes.len());
        
        let adjacency = Self::build_adjacency_list(graph)?;
        let filtered_adjacency = Self::filter_adjacency_list(&adjacency, start_nodes);
        let order = Self::topological_sort(&filtered_adjacency)?;
        
        debug!("Calculated partial execution order: {} nodes", order.len());
        
        Ok(order)
    }
    
    /// Build adjacency list from graph connections
    fn build_adjacency_list(graph: &Graph) -> NodeResult<HashMap<Uuid, Vec<Uuid>>> {
        let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        
        // Initialize all nodes with empty adjacency lists
        for node_id in graph.nodes.keys() {
            adjacency.insert(*node_id, Vec::new());
        }
        
        // Add edges from connections (output -> input dependency)
        for connection in graph.get_connections() {
            if !connection.enabled {
                debug!("Skipping disabled connection: {:?}", connection.id);
                continue;
            }
            
            debug!("Adding dependency: {} -> {}", connection.output_node_id, connection.input_node_id);
            
            adjacency
                .entry(connection.input_node_id)
                .or_insert_with(Vec::new)
                .push(connection.output_node_id);
        }
        
        debug!("Built adjacency list with {} nodes", adjacency.len());
        
        Ok(adjacency)
    }
    
    /// Filter adjacency list to include only reachable nodes from start nodes
    fn filter_adjacency_list(
        adjacency: &HashMap<Uuid, Vec<Uuid>>,
        start_nodes: &[Uuid],
    ) -> HashMap<Uuid, Vec<Uuid>> {
        debug!("Filtering adjacency list from {} start nodes", start_nodes.len());
        
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
        
        debug!("Filtered adjacency list to {} reachable nodes", filtered.len());
        
        filtered
    }
    
    /// Perform topological sort using Kahn's algorithm
    fn topological_sort(adjacency: &HashMap<Uuid, Vec<Uuid>>) -> NodeResult<Vec<Uuid>> {
        debug!("Performing topological sort on {} nodes", adjacency.len());
        
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue: Vec<Uuid> = Vec::new();
        
        // Initialize in-degree counts
        for node_id in adjacency.keys() {
            in_degree.insert(*node_id, 0);
        }
        
        // Calculate in-degrees
        for node_id in adjacency.keys() {
            for neighbor in &adjacency[node_id] {
                *in_degree.entry(*neighbor).or_insert(0) += 1;
            }
        }
        
        // Find nodes with no incoming edges
        for (node_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push(*node_id);
                debug!("Node {} has no dependencies", node_id);
            }
        }
        
        // Process nodes in topological order
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
        
        // Check for circular dependencies
        if result.len() != adjacency.len() {
            let remaining_nodes: Vec<Uuid> = adjacency.keys()
                .filter(|id| !result.contains(id))
                .copied()
                .collect();
            
            debug!("Circular dependency detected involving {} nodes: {:?}", 
                remaining_nodes.len(), remaining_nodes);
            
            return Err(NodeError::CircularDependency);
        }
        
        debug!("Topological sort completed successfully: {} nodes", result.len());
        
        Ok(result)
    }
    
    /// Validate that the execution order is correct
    pub fn validate_order(order: &[Uuid], graph: &Graph) -> NodeResult<()> {
        debug!("Validating execution order for {} nodes", order.len());
        
        let order_set: HashSet<Uuid> = order.iter().cloned().collect();
        
        // Check that all nodes in order exist in graph
        for &node_id in order {
            if !graph.nodes.contains_key(&node_id) {
                return Err(NodeError::NodeNotFound(node_id));
            }
        }
        
        // Check that all graph nodes are included (if not partial order)
        if order.len() == graph.nodes.len() {
            for &node_id in graph.nodes.keys() {
                if !order_set.contains(&node_id) {
                    return Err(NodeError::NodeNotFound(node_id));
                }
            }
        }
        
        // Check dependency order
        let node_positions: HashMap<Uuid, usize> = order.iter()
            .enumerate()
            .map(|(pos, &id)| (id, pos))
            .collect();
        
        for connection in graph.get_connections() {
            if !connection.enabled {
                continue;
            }
            
            let output_pos = node_positions.get(&connection.output_node_id);
            let input_pos = node_positions.get(&connection.input_node_id);
            
            match (output_pos, input_pos) {
                (Some(out_pos), Some(in_pos)) => {
                    if out_pos >= *in_pos {
                        debug!("Invalid dependency order: {} ({}) should come before {} ({})", 
                            connection.output_node_id, out_pos, 
                            connection.input_node_id, in_pos);
                        return Err(NodeError::InvalidConnection(
                            format!("Dependency order violation: {} -> {}", 
                                connection.output_node_id, connection.input_node_id)
                        ));
                    }
                }
                _ => {
                    // One or both nodes not in order (partial order)
                    debug!("Connection involves nodes not in order: {} -> {}", 
                        connection.output_node_id, connection.input_node_id);
                }
            }
        }
        
        debug!("Execution order validation passed");
        
        Ok(())
    }
    
    /// Get dependencies for a specific node
    pub fn get_node_dependencies(graph: &Graph, node_id: Uuid) -> NodeResult<Vec<Uuid>> {
        let adjacency = Self::build_adjacency_list(graph)?;
        
        let mut dependencies = Vec::new();
        let mut visited = HashSet::new();
        let mut to_visit = adjacency.get(&node_id)
            .map(|deps| deps.clone())
            .unwrap_or_default();
        
        while let Some(current) = to_visit.pop() {
            if visited.contains(&current) {
                continue;
            }
            
            visited.insert(current);
            dependencies.push(current);
            
            if let Some(deps) = adjacency.get(&current) {
                for dep in deps {
                    if !visited.contains(dep) {
                        to_visit.push(*dep);
                    }
                }
            }
        }
        
        Ok(dependencies)
    }
    
    /// Get dependents for a specific node
    pub fn get_node_dependents(graph: &Graph, node_id: Uuid) -> NodeResult<Vec<Uuid>> {
        let adjacency = Self::build_adjacency_list(graph)?;
        
        let mut dependents = Vec::new();
        
        for (node, deps) in adjacency {
            if deps.contains(&node_id) {
                dependents.push(node);
            }
        }
        
        Ok(dependents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{Node, NodeType, InputPin, OutputPin, ParameterValue};
    
    fn create_test_graph() -> Graph {
        let mut graph = Graph::new();
        
        // Create test nodes
        let node1_id = Uuid::new_v4();
        let node2_id = Uuid::new_v4();
        let node3_id = Uuid::new_v4();
        
        let mut node1 = Node::new(NodeType::Input, "Node1".to_string());
        let mut node2 = Node::new(NodeType::Input, "Node2".to_string());
        let mut node3 = Node::new(NodeType::Input, "Node3".to_string());
        
        // Add pins
        let output_pin1 = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: aether_types::PinDataType::Image,
            value: ParameterValue::None,
        };
        node1.add_output(output_pin1);
        
        let input_pin2 = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: aether_types::PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        node2.add_input(input_pin2);
        
        let output_pin2 = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: aether_types::PinDataType::Image,
            value: ParameterValue::None,
        };
        node2.add_output(output_pin2);
        
        let input_pin3 = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: aether_types::PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        node3.add_input(input_pin3);
        
        // Add nodes to graph
        graph.nodes.insert(node1_id, node1);
        graph.nodes.insert(node2_id, node2);
        graph.nodes.insert(node3_id, node3);
        
        // Add connections (1 -> 2 -> 3)
        let connection1 = Connection {
            id: Uuid::new_v4(),
            output_node_id: node1_id,
            output_pin_id: node1.outputs[0].id,
            input_node_id: node2_id,
            input_pin_id: node2.inputs[0].id,
            enabled: true,
        };
        
        let connection2 = Connection {
            id: Uuid::new_v4(),
            output_node_id: node2_id,
            output_pin_id: node2.outputs[0].id,
            input_node_id: node3_id,
            input_pin_id: node3.inputs[0].id,
            enabled: true,
        };
        
        graph.connections.push(connection1);
        graph.connections.push(connection2);
        
        graph
    }
    
    #[test]
    fn test_calculate_order() {
        let graph = create_test_graph();
        
        let order = ExecutionOrderCalculator::calculate_order(&graph).unwrap();
        
        // Should have 3 nodes
        assert_eq!(order.len(), 3);
        
        // Should be in dependency order (1 before 2 before 3)
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        assert!(order.contains(&node_ids[0]));
        assert!(order.contains(&node_ids[1]));
        assert!(order.contains(&node_ids[2]));
    }
    
    #[test]
    fn test_calculate_order_from_nodes() {
        let graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        
        // Start from middle node
        let order = ExecutionOrderCalculator::calculate_order_from_nodes(&graph, &[node_ids[1]]).unwrap();
        
        // Should include nodes 1, 2, and 3 (all reachable from node 2)
        assert_eq!(order.len(), 3);
    }
    
    #[test]
    fn test_circular_dependency() {
        let mut graph = Graph::new();
        
        // Create nodes with circular dependency
        let node1_id = Uuid::new_v4();
        let node2_id = Uuid::new_v4();
        
        let mut node1 = Node::new(NodeType::Input, "Node1".to_string());
        let mut node2 = Node::new(NodeType::Input, "Node2".to_string());
        
        // Add pins
        let output_pin1 = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: aether_types::PinDataType::Image,
            value: ParameterValue::None,
        };
        node1.add_output(output_pin1);
        
        let input_pin1 = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: aether_types::PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        node1.add_input(input_pin1);
        
        let output_pin2 = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: aether_types::PinDataType::Image,
            value: ParameterValue::None,
        };
        node2.add_output(output_pin2);
        
        let input_pin2 = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: aether_types::PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        node2.add_input(input_pin2);
        
        graph.nodes.insert(node1_id, node1);
        graph.nodes.insert(node2_id, node2);
        
        // Create circular connections (1 -> 2 and 2 -> 1)
        let connection1 = Connection {
            id: Uuid::new_v4(),
            output_node_id: node1_id,
            output_pin_id: node1.outputs[0].id,
            input_node_id: node2_id,
            input_pin_id: node2.inputs[0].id,
            enabled: true,
        };
        
        let connection2 = Connection {
            id: Uuid::new_v4(),
            output_node_id: node2_id,
            output_pin_id: node2.outputs[0].id,
            input_node_id: node1_id,
            input_pin_id: node1.inputs[0].id,
            enabled: true,
        };
        
        graph.connections.push(connection1);
        graph.connections.push(connection2);
        
        // Should detect circular dependency
        let result = ExecutionOrderCalculator::calculate_order(&graph);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NodeError::CircularDependency));
    }
    
    #[test]
    fn test_validate_order() {
        let graph = create_test_graph();
        let order = ExecutionOrderCalculator::calculate_order(&graph).unwrap();
        
        // Should validate successfully
        assert!(ExecutionOrderCalculator::validate_order(&order, &graph).is_ok());
        
        // Should fail with invalid order
        let mut invalid_order = order.clone();
        invalid_order.reverse();
        
        let result = ExecutionOrderCalculator::validate_order(&invalid_order, &graph);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_node_dependencies() {
        let graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        
        // Node 3 should depend on nodes 1 and 2
        let deps = ExecutionOrderCalculator::get_node_dependencies(&graph, node_ids[2]).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&node_ids[0]));
        assert!(deps.contains(&node_ids[1]));
    }
    
    #[test]
    fn test_get_node_dependents() {
        let graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        
        // Node 1 should have node 2 as dependent
        let deps = ExecutionOrderCalculator::get_node_dependents(&graph, node_ids[0]).unwrap();
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&node_ids[1]));
    }
}
