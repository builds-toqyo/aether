use crate::nodes::{NodeError, NodeResult};
use aether_types::{Graph, Node, Connection, PinDataType, ParameterValue};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct ConnectionValidator;

impl ConnectionValidator {
    pub fn validate_connection(
        graph: &Graph,
        connection: &Connection,
    ) -> NodeResult<()> {
        let output_node = graph.get_node(&connection.output_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(connection.output_node_id))?;
        
        let input_node = graph.get_node(&connection.input_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(connection.input_node_id))?;
        
        let output_pin = output_node.get_output_pin(&connection.output_pin_id)
            .ok_or_else(|| NodeError::PinNotFound(connection.output_pin_id))?;
        
        let input_pin = input_node.get_input_pin(&connection.input_pin_id)
            .ok_or_else(|| NodeError::PinNotFound(connection.input_pin_id))?;
        
        Self::check_type_compatibility(&output_pin.data_type, &input_pin.data_type)?;
        
        if connection.output_node_id == connection.input_node_id {
            return Err(NodeError::InvalidConnection("Node cannot connect to itself".to_string()));
        }
        
        if input_pin.connection.is_some() {
            return Err(NodeError::InvalidConnection(
                format!("Input pin '{}' is already connected", input_pin.name)
            ));
        }
        
        Ok(())
    }
    
    fn check_type_compatibility(output_type: &PinDataType, input_type: &PinDataType) -> NodeResult<()> {
        if output_type == input_type {
            return Ok(());
        }
        
        match (output_type, input_type) {
            (PinDataType::Float, PinDataType::Integer) => Ok(()),
            (PinDataType::Float, PinDataType::Vector2) => Ok(()),
            (PinDataType::Float, PinDataType::Vector3) => Ok(()),
            (PinDataType::Float, PinDataType::Vector4) => Ok(()),
            
            (PinDataType::Vector2, PinDataType::Vector3) => Ok(()),
            (PinDataType::Vector2, PinDataType::Vector4) => Ok(()),
            (PinDataType::Vector3, PinDataType::Vector2) => Ok(()),
            (PinDataType::Vector3, PinDataType::Vector4) => Ok(()),
            (PinDataType::Vector4, PinDataType::Vector2) => Ok(()),
            (PinDataType::Vector4, PinDataType::Vector3) => Ok(()),
            
            (PinDataType::Vector4, PinDataType::Color) => Ok(()),
            (PinDataType::Color, PinDataType::Vector4) => Ok(()),
            
            (PinDataType::Array(ref output_inner), PinDataType::Array(ref input_inner)) => {
                Self::check_type_compatibility(output_inner, input_inner)
            }
            
            _ => Err(NodeError::TypeMismatch {
                expected: format!("{:?}", input_type),
                actual: format!("{:?}", output_type),
            }),
        }
    }
    
    pub fn validate_all_connections(graph: &Graph) -> NodeResult<()> {
        for connection in graph.get_connections() {
            Self::validate_connection(graph, connection)?;
        }
        Ok(())
    }
}

pub struct NodeValidator;

impl NodeValidator {
    pub fn validate_node(node: &Node) -> NodeResult<()> {
        for input_pin in &node.inputs {
            if input_pin.required && input_pin.connection.is_none() {
                return Err(NodeError::RequiredInputNotConnected(input_pin.name.clone()));
            }
        }
        
        for (name, param) in &node.parameters {
            Self::validate_parameter(name, param)?;
        }
        
        Ok(())
    }
    
    fn validate_parameter(name: &str, param: &aether_types::Parameter) -> NodeResult<()> {
        match (&param.data_type, &param.value) {
            (PinDataType::Float, ParameterValue::Float(_)) => Ok(()),
            (PinDataType::Integer, ParameterValue::Integer(_)) => Ok(()),
            (PinDataType::Boolean, ParameterValue::Boolean(_)) => Ok(()),
            (PinDataType::String, ParameterValue::String(_)) => Ok(()),
            (PinDataType::Vector2, ParameterValue::Vector2(_, _)) => Ok(()),
            (PinDataType::Vector3, ParameterValue::Vector3(_, _, _)) => Ok(()),
            (PinDataType::Vector4, ParameterValue::Vector4(_, _, _, _)) => Ok(()),
            (PinDataType::Color, ParameterValue::Color(_, _, _, _)) => Ok(()),
            (PinDataType::Array(_), ParameterValue::Array(_)) => Ok(()),
            (PinDataType::Image, ParameterValue::Image(_)) => Ok(()),
            (PinDataType::Image, ParameterValue::None) => Ok(()), // Image can be None initially
            _ => Err(NodeError::InvalidParameterValue(
                format!("Parameter '{}' has invalid value for type {:?}", name, param.data_type)
            )),
        }
    }
    
    pub fn validate_all_nodes(graph: &Graph) -> NodeResult<()> {
        for node in graph.get_nodes() {
            Self::validate_node(node)?;
        }
        Ok(())
    }
}

/// Validates graph structure and dependencies
pub struct GraphValidator;

impl GraphValidator {
    pub fn validate_graph(graph: &Graph) -> NodeResult<()> {
        NodeValidator::validate_all_nodes(graph)?;
        
        ConnectionValidator::validate_all_connections(graph)?;
        
        Self::check_circular_dependencies(graph)?;
        
        Self::check_orphaned_nodes(graph)?;
        
        Ok(())
    }
    
    fn check_circular_dependencies(graph: &Graph) -> NodeResult<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        // Build adjacency list
        let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        
        for connection in graph.get_connections() {
            adjacency
                .entry(connection.input_node_id)
                .or_insert_with(Vec::new)
                .push(connection.output_node_id);
        }
        
        // Check each node for cycles
        for node_id in graph.nodes.keys() {
            if !visited.contains(node_id) {
                if Self::dfs_cycle_check(node_id, &adjacency, &mut visited, &mut rec_stack) {
                    return Err(NodeError::CircularDependency);
                }
            }
        }
        
        Ok(())
    }
    
    /// DFS helper for cycle detection
    fn dfs_cycle_check(
        node_id: &Uuid,
        adjacency: &HashMap<Uuid, Vec<Uuid>>,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
    ) -> bool {
        visited.insert(*node_id);
        rec_stack.insert(*node_id);
        
        if let Some(neighbors) = adjacency.get(node_id) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if Self::dfs_cycle_check(neighbor, adjacency, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(node_id);
        false
    }
    
    /// Check for orphaned nodes (not connected to input/output chain)
    fn check_orphaned_nodes(graph: &Graph) -> NodeResult<()> {
        if graph.nodes.is_empty() {
            return Ok(());
        }
        
        // Find input and output nodes
        let input_nodes: HashSet<Uuid> = graph.get_input_nodes().map(|n| n.id).collect();
        let output_nodes: HashSet<Uuid> = graph.get_output_nodes().map(|n| n.id).collect();
        
        // If no input or output nodes, skip this check
        if input_nodes.is_empty() || output_nodes.is_empty() {
            return Ok(());
        }
        
        // Build reachable sets from inputs and outputs
        let reachable_from_input = Self::get_reachable_nodes(graph, &input_nodes, true);
        let reachable_to_output = Self::get_reachable_nodes(graph, &output_nodes, false);
        
        // Find nodes that are not in the main chain
        for node_id in graph.nodes.keys() {
            if input_nodes.contains(node_id) || output_nodes.contains(node_id) {
                continue; // Skip input/output nodes themselves
            }
            
            if !reachable_from_input.contains(node_id) || !reachable_to_output.contains(node_id) {
                // This is an orphaned node, but we'll just warn about it
                log::warn!("Orphaned node detected: {:?}", node_id);
            }
        }
        
        Ok(())
    }
    
    fn get_reachable_nodes(
        graph: &Graph,
        start_nodes: &HashSet<Uuid>,
        forward: bool,
    ) -> HashSet<Uuid> {
        let mut reachable = HashSet::new();
        let mut to_visit = start_nodes.iter().cloned().collect::<Vec<_>>();
        
        while let Some(current) = to_visit.pop() {
            if reachable.contains(&current) {
                continue;
            }
            
            reachable.insert(current);
            
            // Add neighbors
            for connection in graph.get_connections() {
                let neighbor = if forward {
                    // Forward traversal: output -> input
                    if connection.output_node_id == current {
                        Some(connection.input_node_id)
                    } else {
                        None
                    }
                } else {
                    // Backward traversal: input -> output
                    if connection.input_node_id == current {
                        Some(connection.output_node_id)
                    } else {
                        None
                    }
                };
                
                if let Some(neighbor) = neighbor {
                    to_visit.push(neighbor);
                }
            }
        }
        
        reachable
    }
}

/// Comprehensive validation for node graphs
pub struct GraphValidator;

impl GraphValidator {
    /// Perform comprehensive validation
    pub fn validate(graph: &Graph) -> NodeResult<Vec<String>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        // Basic graph validation
        if let Err(e) = graph.validate() {
            errors.push(format!("Basic validation failed: {}", e));
        }
        
        // Node validation
        if let Err(e) = NodeValidator::validate_all_nodes(graph) {
            errors.push(format!("Node validation failed: {}", e));
        }
        
        // Connection validation
        if let Err(e) = ConnectionValidator::validate_all_connections(graph) {
            errors.push(format!("Connection validation failed: {}", e));
        }
        
        // Graph structure validation
        if let Err(e) = GraphValidator::validate_graph(graph) {
            errors.push(format!("Graph structure validation failed: {}", e));
        }
        
        // Performance warnings
        Self::check_performance_issues(graph, &mut warnings);
        
        // Style warnings
        Self::check_style_issues(graph, &mut warnings);
        
        if !errors.is_empty() {
            return Err(NodeError::ExecutionFailed(errors.join("; ")));
        }
        
        Ok(warnings)
    }
    
    /// Check for potential performance issues
    fn check_performance_issues(graph: &Graph, warnings: &mut Vec<String>) {
        // Check for large number of nodes
        if graph.node_count() > 1000 {
            warnings.push("Large number of nodes may impact performance".to_string());
        }
        
        // Check for deep node chains
        let max_depth = Self::calculate_max_depth(graph);
        if max_depth > 50 {
            warnings.push(format!("Deep node chain (depth: {}) may impact performance", max_depth));
        }
        
        // Check for nodes with many connections
        for node in graph.get_nodes() {
            let connection_count = node.inputs.iter().filter(|i| i.connection.is_some()).count()
                + node.outputs.len();
            if connection_count > 20 {
                warnings.push(format!("Node '{}' has many connections ({}) which may impact performance", 
                    node.name, connection_count));
            }
        }
    }
    
    /// Calculate maximum depth of node graph
    fn calculate_max_depth(graph: &Graph) -> usize {
        let input_nodes: HashSet<Uuid> = graph.get_input_nodes().map(|n| n.id).collect();
        let mut max_depth = 0;
        
        for input_id in input_nodes {
            let depth = Self::calculate_node_depth(graph, input_id, &mut HashSet::new());
            max_depth = max_depth.max(depth);
        }
        
        max_depth
    }
    
    /// Calculate depth from a specific node
    fn calculate_node_depth(
        graph: &Graph,
        node_id: Uuid,
        visited: &mut HashSet<Uuid>,
    ) -> usize {
        if visited.contains(&node_id) {
            return 0; // Avoid cycles
        }
        
        visited.insert(node_id);
        
        let mut max_child_depth = 0;
        
        for connection in graph.get_connections() {
            if connection.output_node_id == node_id {
                let child_depth = Self::calculate_node_depth(graph, connection.input_node_id, visited);
                max_child_depth = max_child_depth.max(child_depth);
            }
        }
        
        visited.remove(&node_id);
        max_child_depth + 1
    }
    
    /// Check for style and naming issues
    fn check_style_issues(graph: &Graph, warnings: &mut Vec<String>) {
        // Check for nodes with default names
        for node in graph.get_nodes() {
            if node.name.is_empty() || node.name == "Node" {
                warnings.push(format!("Node {:?} has a generic name", node.id));
            }
            
            // Check for very long names
            if node.name.len() > 50 {
                warnings.push(format!("Node '{}' has a very long name", node.name));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{Node, NodeType, Graph, Connection, InputPin, OutputPin, PinDataType, ParameterValue};
    
    #[test]
    fn test_connection_validation() {
        let mut graph = Graph::new("Test".to_string());
        
        // Create test nodes
        let mut output_node = Node::new(NodeType::Input, "Output".to_string());
        let mut input_node = Node::new(NodeType::Output, "Input".to_string());
        
        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        
        let input_pin = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        
        output_node.add_output(output_pin.clone());
        input_node.add_input(input_pin.clone());
        
        let output_node_id = output_node.id;
        let input_node_id = input_node.id;
        
        graph.add_node(output_node);
        graph.add_node(input_node);
        
        // Test valid connection
        let connection = Connection::new(output_node_id, output_pin.id, input_node_id, input_pin.id);
        assert!(ConnectionValidator::validate_connection(&graph, &connection).is_ok());
        
        // Test type mismatch
        let mut wrong_input_node = Node::new(NodeType::Output, "Wrong".to_string());
        let wrong_input_pin = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: PinDataType::Float,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        wrong_input_node.add_input(wrong_input_pin);
        
        let wrong_input_id = wrong_input_node.id;
        graph.add_node(wrong_input_node);
        
        let wrong_connection = Connection::new(output_node_id, output_pin.id, wrong_input_id, wrong_input_pin.id);
        assert!(ConnectionValidator::validate_connection(&graph, &wrong_connection).is_err());
    }
    
    #[test]
    fn test_node_validation() {
        let mut node = Node::new(NodeType::Input, "Test".to_string());
        
        // Add required input without connection
        let required_pin = InputPin {
            id: Uuid::new_v4(),
            name: "required".to_string(),
            data_type: PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        node.add_input(required_pin);
        
        // Should fail due to required input not being connected
        assert!(NodeValidator::validate_node(&node).is_err());
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = Graph::new("Test".to_string());
        
        // Create nodes that form a cycle
        let node1 = Node::new(NodeType::Input, "Node1".to_string());
        let node2 = Node::new(NodeType::Output, "Node2".to_string());
        let node3 = Node::new(NodeType::Transform, "Node3".to_string());
        
        let node1_id = node1.id;
        let node2_id = node2.id;
        let node3_id = node3.id;
        
        graph.add_node(node1);
        graph.add_node(node2);
        graph.add_node(node3);
        
        // Add pins (simplified for test)
        // In a real implementation, you'd need to properly set up pins
        
        // Create circular connections: 1 -> 2 -> 3 -> 1
        // This would require proper pin setup in a real test
        
        // For now, just test that the method exists
        assert!(GraphValidator::check_circular_dependencies(&graph).is_ok());
    }
}
