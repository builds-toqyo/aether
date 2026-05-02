use crate::nodes::{NodeError, NodeResult};
use crate::nodes::validation::TypeChecker;
use aether_types::{Graph, Connection, PinDataType};
use uuid::Uuid;
use log::debug;

/// Validates node connections
pub struct ConnectionValidator;

impl ConnectionValidator {
    /// Validate a single connection
    pub fn validate_connection(
        graph: &Graph,
        connection: &Connection,
    ) -> NodeResult<()> {
        debug!("Validating connection: {} -> {}", 
            connection.output_node_id, connection.input_node_id);
        
        // Check that both nodes exist
        let output_node = graph.get_node(&connection.output_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(connection.output_node_id))?;
        
        let input_node = graph.get_node(&connection.input_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(connection.input_node_id))?;
        
        // Check that both pins exist
        let output_pin = output_node.get_output_pin(&connection.output_pin_id)
            .ok_or_else(|| NodeError::PinNotFound(connection.output_pin_id))?;
        
        let input_pin = input_node.get_input_pin(&connection.input_pin_id)
            .ok_or_else(|| NodeError::PinNotFound(connection.input_pin_id))?;
        
        // Check type compatibility
        TypeChecker::check_type_compatibility(&output_pin.data_type, &input_pin.data_type)?;
        
        // Check for self-connection
        if connection.output_node_id == connection.input_node_id {
            return Err(NodeError::InvalidConnection(
                "Node cannot connect to itself".to_string()
            ));
        }
        
        // Check if input pin is already connected
        if input_pin.connection.is_some() {
            return Err(NodeError::InvalidConnection(
                format!("Input pin '{}' is already connected", input_pin.name)
            ));
        }
        
        debug!("Connection validation passed: {} -> {}", 
            connection.output_node_id, connection.input_node_id);
        
        Ok(())
    }
    
    /// Validate all connections in a graph
    pub fn validate_all_connections(graph: &Graph) -> NodeResult<()> {
        debug!("Validating all {} connections", graph.connections.len());
        
        for connection in graph.get_connections() {
            if connection.enabled {
                Self::validate_connection(graph, connection)?;
            } else {
                debug!("Skipping disabled connection: {:?}", connection.id);
            }
        }
        
        debug!("All connections validated successfully");
        
        Ok(())
    }
    
    /// Check if two nodes can be connected
    pub fn can_connect(
        graph: &Graph,
        output_node_id: Uuid,
        output_pin_id: Uuid,
        input_node_id: Uuid,
        input_pin_id: Uuid,
    ) -> NodeResult<bool> {
        // Check that nodes exist
        let output_node = graph.get_node(&output_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(output_node_id))?;
        
        let input_node = graph.get_node(&input_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(input_node_id))?;
        
        // Check that pins exist
        let output_pin = output_node.get_output_pin(&output_pin_id)
            .ok_or_else(|| NodeError::PinNotFound(output_pin_id))?;
        
        let input_pin = input_node.get_input_pin(&input_pin_id)
            .ok_or_else(|| NodeError::PinNotFound(input_pin_id))?;
        
        // Check type compatibility
        TypeChecker::check_type_compatibility(&output_pin.data_type, &input_pin.data_type)?;
        
        // Check for self-connection
        if output_node_id == input_node_id {
            return Ok(false);
        }
        
        // Check if input pin is already connected
        if input_pin.connection.is_some() {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Get possible connections between two nodes
    pub fn get_possible_connections(
        graph: &Graph,
        output_node_id: Uuid,
        input_node_id: Uuid,
    ) -> NodeResult<Vec<(Uuid, Uuid)>> {
        let mut connections = Vec::new();
        
        // Check that nodes exist
        let output_node = graph.get_node(&output_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(output_node_id))?;
        
        let input_node = graph.get_node(&input_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(input_node_id))?;
        
        // Check all output pin and input pin combinations
        for output_pin in &output_node.outputs {
            for input_pin in &input_node.inputs {
                // Skip if input pin is already connected
                if input_pin.connection.is_some() {
                    continue;
                }
                
                // Check type compatibility
                if TypeChecker::are_types_compatible(&output_pin.data_type, &input_pin.data_type) {
                    connections.push((output_pin.id, input_pin.id));
                }
            }
        }
        
        Ok(connections)
    }
    
    /// Validate connection removal
    pub fn validate_connection_removal(
        graph: &Graph,
        connection: &Connection,
    ) -> NodeResult<()> {
        debug!("Validating connection removal: {:?}", connection.id);
        
        // Check that connection exists in graph
        if !graph.connections.iter().any(|c| c.id == connection.id) {
            return Err(NodeError::ConnectionNotFound(connection.id));
        }
        
        // Check that nodes exist
        graph.get_node(&connection.output_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(connection.output_node_id))?;
        
        graph.get_node(&connection.input_node_id)
            .ok_or_else(|| NodeError::NodeNotFound(connection.input_node_id))?;
        
        debug!("Connection removal validation passed");
        
        Ok(())
    }
    
    /// Check for duplicate connections
    pub fn check_duplicate_connections(graph: &Graph) -> NodeResult<()> {
        debug!("Checking for duplicate connections");
        
        let mut seen_connections = std::collections::HashSet::new();
        
        for connection in graph.get_connections() {
            if !connection.enabled {
                continue;
            }
            
            let key = (connection.output_node_id, connection.output_pin_id, 
                      connection.input_node_id, connection.input_pin_id);
            
            if seen_connections.contains(&key) {
                return Err(NodeError::InvalidConnection(
                    format!("Duplicate connection found: {} -> {}", 
                        connection.output_node_id, connection.input_node_id)
                ));
            }
            
            seen_connections.insert(key);
        }
        
        debug!("No duplicate connections found");
        
        Ok(())
    }
    
    /// Get connection statistics
    pub fn get_connection_stats(graph: &Graph) -> ConnectionStats {
        let total = graph.connections.len();
        let enabled = graph.connections.iter().filter(|c| c.enabled).count();
        let disabled = total - enabled;
        
        // Count connections by type
        let mut type_counts = std::collections::HashMap::new();
        
        for connection in graph.get_connections() {
            if !connection.enabled {
                continue;
            }
            
            if let (Some(output_node), Some(input_node)) = 
                (graph.get_node(&connection.output_node_id), 
                 graph.get_node(&connection.input_node_id)) {
                
                if let (Some(output_pin), Some(input_pin)) = 
                    (output_node.get_output_pin(&connection.output_pin_id),
                     input_node.get_input_pin(&connection.input_pin_id)) {
                    
                    let type_key = format!("{:?} -> {:?}", output_pin.data_type, input_pin.data_type);
                    *type_counts.entry(type_key).or_insert(0) += 1;
                }
            }
        }
        
        ConnectionStats {
            total,
            enabled,
            disabled,
            type_counts,
        }
    }
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Total number of connections
    pub total: usize,
    /// Number of enabled connections
    pub enabled: usize,
    /// Number of disabled connections
    pub disabled: usize,
    /// Count of connections by type
    pub type_counts: std::collections::HashMap<String, usize>,
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
        
        let mut node1 = Node::new(NodeType::Input, "Node1".to_string());
        let mut node2 = Node::new(NodeType::Input, "Node2".to_string());
        
        // Add pins
        let output_pin = OutputPin {
            id: Uuid::new_v4(),
            name: "output".to_string(),
            data_type: PinDataType::Image,
            value: ParameterValue::None,
        };
        node1.add_output(output_pin);
        
        let input_pin = InputPin {
            id: Uuid::new_v4(),
            name: "input".to_string(),
            data_type: PinDataType::Image,
            required: true,
            default_value: ParameterValue::None,
            current_value: ParameterValue::None,
            connection: None,
        };
        node2.add_input(input_pin);
        
        // Add nodes to graph
        graph.nodes.insert(node1_id, node1);
        graph.nodes.insert(node2_id, node2);
        
        graph
    }
    
    #[test]
    fn test_validate_connection() {
        let mut graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        
        // Create a valid connection
        let connection = Connection {
            id: Uuid::new_v4(),
            output_node_id: node_ids[0],
            output_pin_id: graph.nodes[&node_ids[0]].outputs[0].id,
            input_node_id: node_ids[1],
            input_pin_id: graph.nodes[&node_ids[1]].inputs[0].id,
            enabled: true,
        };
        
        // Should validate successfully
        assert!(ConnectionValidator::validate_connection(&graph, &connection).is_ok());
    }
    
    #[test]
    fn test_validate_connection_self_connection() {
        let mut graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        
        // Create a self-connection
        let connection = Connection {
            id: Uuid::new_v4(),
            output_node_id: node_ids[0],
            output_pin_id: graph.nodes[&node_ids[0]].outputs[0].id,
            input_node_id: node_ids[0],
            input_pin_id: graph.nodes[&node_ids[0]].inputs[0].id,
            enabled: true,
        };
        
        // Should fail validation
        let result = ConnectionValidator::validate_connection(&graph, &connection);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NodeError::InvalidConnection(_)));
    }
    
    #[test]
    fn test_validate_connection_nonexistent_node() {
        let mut graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        let fake_node_id = Uuid::new_v4();
        
        // Create connection to nonexistent node
        let connection = Connection {
            id: Uuid::new_v4(),
            output_node_id: fake_node_id,
            output_pin_id: Uuid::new_v4(),
            input_node_id: node_ids[1],
            input_pin_id: graph.nodes[&node_ids[1]].inputs[0].id,
            enabled: true,
        };
        
        // Should fail validation
        let result = ConnectionValidator::validate_connection(&graph, &connection);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NodeError::NodeNotFound(_)));
    }
    
    #[test]
    fn test_can_connect() {
        let graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        
        // Should be able to connect
        let can_connect = ConnectionValidator::can_connect(
            &graph,
            node_ids[0],
            graph.nodes[&node_ids[0]].outputs[0].id,
            node_ids[1],
            graph.nodes[&node_ids[1]].inputs[0].id,
        ).unwrap();
        
        assert!(can_connect);
        
        // Should not be able to connect to self
        let can_connect_self = ConnectionValidator::can_connect(
            &graph,
            node_ids[0],
            graph.nodes[&node_ids[0]].outputs[0].id,
            node_ids[0],
            graph.nodes[&node_ids[0]].inputs[0].id,
        ).unwrap();
        
        assert!(!can_connect_self);
    }
    
    #[test]
    fn test_get_possible_connections() {
        let graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        
        let connections = ConnectionValidator::get_possible_connections(
            &graph,
            node_ids[0],
            node_ids[1],
        ).unwrap();
        
        // Should find one possible connection
        assert_eq!(connections.len(), 1);
    }
    
    #[test]
    fn test_connection_stats() {
        let mut graph = create_test_graph();
        let node_ids: Vec<Uuid> = graph.nodes.keys().copied().collect();
        
        // Add a connection
        let connection = Connection {
            id: Uuid::new_v4(),
            output_node_id: node_ids[0],
            output_pin_id: graph.nodes[&node_ids[0]].outputs[0].id,
            input_node_id: node_ids[1],
            input_pin_id: graph.nodes[&node_ids[1]].inputs[0].id,
            enabled: true,
        };
        graph.connections.push(connection);
        
        let stats = ConnectionValidator::get_connection_stats(&graph);
        
        assert_eq!(stats.total, 1);
        assert_eq!(stats.enabled, 1);
        assert_eq!(stats.disabled, 0);
        assert_eq!(stats.type_counts.len(), 1);
    }
}
