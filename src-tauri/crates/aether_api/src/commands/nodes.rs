use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use std::collections::HashMap;
use anyhow::Result;
use log::{debug, info, warn, error};

use crate::state::AppState;
use aether_types::{Node, NodeType, PinDataType, ParameterValue, Graph, Connection, InputPin, OutputPin};
use aether_core::nodes::{NodeExecutor, ExecutionContext, NodeResult};

/// Command to create a new node in the graph
#[tauri::command]
pub async fn create_node(
    node_type: String,
    name: String,
    position: Option<(f64, f64)>,
    state: State<'_, AppState>,
) -> Result<NodeResponse, String> {
    debug!("Creating node: type={}, name={:?}", node_type, name);
    
    // Parse node type
    let node_type_enum = parse_node_type(&node_type)
        .map_err(|e| format!("Invalid node type: {}", e))?;
    
    // Create node
    let mut node = Node::new(node_type_enum, name.clone());
    
    // Set position if provided
    if let Some((x, y)) = position {
        // Store position in node metadata (you might want to add this to Node struct)
        debug!("Setting node position: ({}, {})", x, y);
    }
    
    // Add to graph
    let mut graph = state.graph.lock().map_err(|e| format!("Failed to lock graph: {}", e))?;
    let node_id = node.id;
    graph.nodes.insert(node_id, node);
    
    info!("Created node: {} ({})", name, node_id);
    
    Ok(NodeResponse {
        id: node_id,
        node_type: node_type,
        name,
        position,
        success: true,
        message: "Node created successfully".to_string(),
    })
}

/// Command to connect two nodes
#[tauri::command]
pub async fn connect_nodes(
    output_node_id: String,
    output_pin_name: String,
    input_node_id: String,
    input_pin_name: String,
    state: State<'_, AppState>,
) -> Result<ConnectionResponse, String> {
    debug!("Connecting nodes: {}:{} -> {}:{}", 
        output_node_id, output_pin_name, input_node_id, input_pin_name);
    
    // Parse UUIDs
    let output_id = Uuid::parse_str(&output_node_id)
        .map_err(|e| format!("Invalid output node ID: {}", e))?;
    let input_id = Uuid::parse_str(&input_node_id)
        .map_err(|e| format!("Invalid input node ID: {}", e))?;
    
    let mut graph = state.graph.lock().map_err(|e| format!("Failed to lock graph: {}", e))?;
    
    // Get nodes
    let output_node = graph.get_node(&output_id)
        .ok_or_else(|| format!("Output node not found: {}", output_id))?;
    let input_node = graph.get_node(&input_id)
        .ok_or_else(|| format!("Input node not found: {}", input_id))?;
    
    // Find pins
    let output_pin = output_node.get_output_pin_by_name(&output_pin_name)
        .ok_or_else(|| format!("Output pin '{}' not found", output_pin_name))?;
    let input_pin = input_node.get_input_pin_by_name(&input_pin_name)
        .ok_or_else(|| format!("Input pin '{}' not found", input_pin_name))?;
    
    // Check type compatibility
    if !are_pin_types_compatible(&output_pin.data_type, &input_pin.data_type) {
        return Err(format!("Incompatible pin types: {:?} -> {:?}", 
            output_pin.data_type, input_pin.data_type));
    }
    
    // Check if input pin is already connected
    if input_pin.connection.is_some() {
        return Err(format!("Input pin '{}' is already connected", input_pin_name));
    }
    
    // Create connection
    let connection = Connection {
        id: Uuid::new_v4(),
        output_node_id: output_id,
        output_pin_id: output_pin.id,
        input_node_id: input_id,
        input_pin_id: input_pin.id,
        enabled: true,
    };
    
    // Add connection to graph
    graph.connections.push(connection.clone());
    
    // Update input pin connection
    if let Some(input_node) = graph.nodes.get_mut(&input_id) {
        for pin in &mut input_node.inputs {
            if pin.id == input_pin.id {
                pin.connection = Some(connection.id);
                break;
            }
        }
    }
    
    info!("Connected nodes: {}:{} -> {}:{}", 
        output_node_id, output_pin_name, input_node_id, input_pin_name);
    
    Ok(ConnectionResponse {
        id: connection.id.to_string(),
        output_node_id: output_node_id,
        output_pin_name,
        input_node_id: input_node_id,
        input_pin_name,
        success: true,
        message: "Nodes connected successfully".to_string(),
    })
}

/// Command to execute the node graph
#[tauri::command]
pub async fn execute_graph(
    frame: Option<u64>,
    state: State<'_, AppState>,
) -> Result<ExecutionResponse, String> {
    debug!("Executing graph at frame: {:?}", frame);
    
    let graph = state.graph.lock().map_err(|e| format!("Failed to lock graph: {}", e))?;
    
    // Calculate execution order
    let execution_order = aether_core::nodes::execution_order::ExecutionOrderCalculator::calculate_order(&graph)
        .map_err(|e| format!("Failed to calculate execution order: {}", e))?;
    
    debug!("Execution order: {} nodes", execution_order.len());
    
    // Create execution context
    let frame_number = frame.unwrap_or(0);
    let mut context = ExecutionContext {
        frame: frame_number,
        inputs: HashMap::new(),
        outputs: HashMap::new(),
    };
    
    // Execute nodes in order
    let mut executed_nodes = Vec::new();
    let mut execution_errors = Vec::new();
    
    for node_id in &execution_order {
        if let Some(node) = graph.get_node(node_id) {
            debug!("Executing node: {} ({})", node.name, node_id);
            
            // Create node executor (this would need to be implemented based on node type)
            let executor = create_node_executor(node)?;
            
            // Execute node
            match executor.execute(&mut context) {
                Ok(_) => {
                    executed_nodes.push(node_id.to_string());
                    debug!("Node executed successfully: {}", node.name);
                }
                Err(e) => {
                    let error_msg = format!("Node {} failed: {}", node.name, e);
                    execution_errors.push(error_msg.clone());
                    error!("{}", error_msg);
                }
            }
        }
    }
    
    let success = execution_errors.is_empty();
    let message = if success {
        format!("Graph executed successfully: {} nodes", executed_nodes.len())
    } else {
        format!("Graph execution failed: {} errors", execution_errors.len())
    };
    
    info!("Graph execution completed: success={}, errors={}", success, execution_errors.len());
    
    Ok(ExecutionResponse {
        success,
        message,
        executed_nodes,
        execution_errors,
        frame_number,
        execution_time_ms: 0, // Would need to measure actual execution time
    })
}

/// Command to get the result of a specific node
#[tauri::command]
pub async fn get_node_result(
    node_id: String,
    pin_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<NodeResultResponse, String> {
    debug!("Getting node result: {} (pin: {:?})", node_id, pin_name);
    
    let node_uuid = Uuid::parse_str(&node_id)
        .map_err(|e| format!("Invalid node ID: {}", e))?;
    
    let graph = state.graph.lock().map_err(|e| format!("Failed to lock graph: {}", e))?;
    
    // Get node
    let node = graph.get_node(&node_uuid)
        .ok_or_else(|| format!("Node not found: {}", node_id))?;
    
    // Get execution results from state (this would need to be stored during execution)
    let execution_results = state.execution_results.lock().map_err(|e| format!("Failed to lock execution results: {}", e))?;
    
    // Get node outputs
    let mut outputs = HashMap::new();
    
    if let Some(pin_name) = pin_name {
        // Get specific pin output
        let output_pin = node.get_output_pin_by_name(&pin_name)
            .ok_or_else(|| format!("Output pin '{}' not found", pin_name))?;
        
        if let Some(value) = execution_results.get(&output_pin.id) {
            outputs.insert(pin_name.clone(), serialize_parameter_value(value));
        } else {
            outputs.insert(pin_name, "null".to_string());
        }
    } else {
        // Get all output pins
        for output_pin in &node.outputs {
            let value = execution_results.get(&output_pin.id)
                .map(|v| serialize_parameter_value(v))
                .unwrap_or_else(|| "null".to_string());
            outputs.insert(output_pin.name.clone(), value);
        }
    }
    
    Ok(NodeResultResponse {
        node_id,
        node_name: node.name.clone(),
        node_type: format!("{:?}", node.node_type),
        outputs,
        success: true,
        message: "Node result retrieved successfully".to_string(),
    })
}

/// Command to delete a node from the graph
#[tauri::command]
pub async fn delete_node(
    node_id: String,
    state: State<'_, AppState>,
) -> Result<NodeResponse, String> {
    debug!("Deleting node: {}", node_id);
    
    let node_uuid = Uuid::parse_str(&node_id)
        .map_err(|e| format!("Invalid node ID: {}", e))?;
    
    let mut graph = state.graph.lock().map_err(|e| format!("Failed to lock graph: {}", e))?;
    
    // Check if node exists
    let node = graph.get_node(&node_uuid)
        .ok_or_else(|| format!("Node not found: {}", node_id))?;
    
    let node_name = node.name.clone();
    let node_type = format!("{:?}", node.node_type);
    
    // Remove all connections to/from this node
    graph.connections.retain(|conn| {
        conn.output_node_id != node_uuid && conn.input_node_id != node_uuid
    });
    
    // Remove the node
    graph.nodes.remove(&node_uuid);
    
    info!("Deleted node: {} ({})", node_name, node_id);
    
    Ok(NodeResponse {
        id: node_uuid,
        node_type,
        name: node_name,
        position: None,
        success: true,
        message: "Node deleted successfully".to_string(),
    })
}

/// Command to disconnect nodes
#[tauri::command]
pub async fn disconnect_nodes(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<ConnectionResponse, String> {
    debug!("Disconnecting connection: {}", connection_id);
    
    let connection_uuid = Uuid::parse_str(&connection_id)
        .map_err(|e| format!("Invalid connection ID: {}", e))?;
    
    let mut graph = state.graph.lock().map_err(|e| format!("Failed to lock graph: {}", e))?;
    
    // Find connection
    let connection = graph.connections.iter()
        .find(|conn| conn.id == connection_uuid)
        .ok_or_else(|| format!("Connection not found: {}", connection_id))?;
    
    let output_node_id = connection.output_node_id.to_string();
    let input_node_id = connection.input_node_id.to_string();
    let output_pin_name = connection.output_pin_id.to_string(); // Would need to get actual pin name
    let input_pin_name = connection.input_pin_id.to_string(); // Would need to get actual pin name
    
    // Remove connection
    graph.connections.retain(|conn| conn.id != connection_uuid);
    
    // Clear input pin connection
    if let Some(input_node) = graph.nodes.get_mut(&connection.input_node_id) {
        for pin in &mut input_node.inputs {
            if pin.connection == Some(connection_uuid) {
                pin.connection = None;
                break;
            }
        }
    }
    
    info!("Disconnected connection: {}", connection_id);
    
    Ok(ConnectionResponse {
        id: connection_id,
        output_node_id,
        output_pin_name,
        input_node_id,
        input_pin_name,
        success: true,
        message: "Nodes disconnected successfully".to_string(),
    })
}

/// Command to get graph information
#[tauri::command]
pub async fn get_graph_info(
    state: State<'_, AppState>,
) -> Result<GraphInfoResponse, String> {
    debug!("Getting graph information");
    
    let graph = state.graph.lock().map_err(|e| format!("Failed to lock graph: {}", e))?;
    
    let node_count = graph.nodes.len();
    let connection_count = graph.connections.len();
    
    // Count nodes by type
    let mut node_types = HashMap::new();
    for node in graph.get_nodes() {
        let type_name = format!("{:?}", node.node_type);
        *node_types.entry(type_name).or_insert(0) += 1;
    }
    
    // Get node list
    let nodes: Vec<NodeInfo> = graph.get_nodes().iter().map(|node| NodeInfo {
        id: node.id.to_string(),
        name: node.name.clone(),
        node_type: format!("{:?}", node.node_type),
        input_count: node.inputs.len(),
        output_count: node.outputs.len(),
        enabled: node.enabled,
    }).collect();
    
    Ok(GraphInfoResponse {
        node_count,
        connection_count,
        node_types,
        nodes,
        success: true,
        message: "Graph information retrieved successfully".to_string(),
    })
}

// Helper functions

fn parse_node_type(node_type: &str) -> Result<NodeType, String> {
    match node_type.to_lowercase().as_str() {
        "input" => Ok(NodeType::Input),
        "output" => Ok(NodeType::Output),
        "transform" => Ok(NodeType::Transform),
        "merge" => Ok(NodeType::Merge),
        "color_correction" => Ok(NodeType::ColorCorrection),
        "blur" => Ok(NodeType::Blur),
        _ => Err(format!("Unknown node type: {}", node_type)),
    }
}

fn are_pin_types_compatible(output_type: &PinDataType, input_type: &PinDataType) -> bool {
    // Simplified compatibility check
    match (output_type, input_type) {
        (PinDataType::Image, PinDataType::Image) => true,
        (PinDataType::Float, PinDataType::Float) => true,
        (PinDataType::Vector2, PinDataType::Vector2) => true,
        (PinDataType::Vector3, PinDataType::Vector3) => true,
        (PinDataType::Vector4, PinDataType::Vector4) => true,
        (PinDataType::Color, PinDataType::Color) => true,
        // Allow some type conversions
        (PinDataType::Float, PinDataType::Vector2) => true,
        (PinDataType::Float, PinDataType::Vector3) => true,
        (PinDataType::Float, PinDataType::Vector4) => true,
        (PinDataType::Vector4, PinDataType::Color) => true,
        (PinDataType::Color, PinDataType::Vector4) => true,
        _ => false,
    }
}

fn create_node_executor(node: &Node) -> Result<Box<dyn NodeExecutor + Send + Sync>, String> {
    // This would need to be implemented based on your node system
    // For now, return a placeholder
    Err("Node executor creation not implemented".to_string())
}

fn serialize_parameter_value(value: &ParameterValue) -> String {
    match value {
        ParameterValue::None => "null".to_string(),
        ParameterValue::Float(f) => f.to_string(),
        ParameterValue::Integer(i) => i.to_string(),
        ParameterValue::Boolean(b) => b.to_string(),
        ParameterValue::String(s) => format!("\"{}\"", s),
        ParameterValue::Vector2(x, y) => format!("[{}, {}]", x, y),
        ParameterValue::Vector3(x, y, z) => format!("[{}, {}, {}]", x, y, z),
        ParameterValue::Vector4(x, y, z, w) => format!("[{}, {}, {}, {}]", x, y, z, w),
        ParameterValue::Color(r, g, b, a) => format!("[{}, {}, {}, {}]", r, g, b, a),
        ParameterValue::Array(arr) => format!("Array({} items)", arr.len()),
        ParameterValue::Image(id) => format!("Image({})", id),
    }
}

// Response types

#[derive(Debug, Serialize)]
pub struct NodeResponse {
    pub id: Uuid,
    pub node_type: String,
    pub name: String,
    pub position: Option<(f64, f64)>,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectionResponse {
    pub id: String,
    pub output_node_id: String,
    pub output_pin_name: String,
    pub input_node_id: String,
    pub input_pin_name: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutionResponse {
    pub success: bool,
    pub message: String,
    pub executed_nodes: Vec<String>,
    pub execution_errors: Vec<String>,
    pub frame_number: u64,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct NodeResultResponse {
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub outputs: HashMap<String, String>,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct GraphInfoResponse {
    pub node_count: usize,
    pub connection_count: usize,
    pub node_types: HashMap<String, usize>,
    pub nodes: Vec<NodeInfo>,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub input_count: usize,
    pub output_count: usize,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_node_type() {
        assert!(matches!(parse_node_type("input"), Ok(NodeType::Input)));
        assert!(matches!(parse_node_type("output"), Ok(NodeType::Output)));
        assert!(parse_node_type("invalid").is_err());
    }
    
    #[test]
    fn test_pin_type_compatibility() {
        assert!(are_pin_types_compatible(&PinDataType::Image, &PinDataType::Image));
        assert!(are_pin_types_compatible(&PinDataType::Float, &PinDataType::Vector2));
        assert!(!are_pin_types_compatible(&PinDataType::Image, &PinDataType::Float));
    }
    
    #[test]
    fn test_serialize_parameter_value() {
        assert_eq!(serialize_parameter_value(&ParameterValue::None), "null");
        assert_eq!(serialize_parameter_value(&ParameterValue::Float(3.14)), "3.14");
        assert_eq!(serialize_parameter_value(&ParameterValue::Boolean(true)), "true");
        assert_eq!(serialize_parameter_value(&ParameterValue::String("test")), "\"test\"");
    }
}
