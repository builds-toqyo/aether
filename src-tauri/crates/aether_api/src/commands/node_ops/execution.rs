use serde::{Serialize};
use tauri::State;
use std::collections::HashMap;
use anyhow::Result;
use log::{debug, info, error};

use crate::state::AppState;
use aether_types::{ParameterValue};
use aether_core::nodes::{NodeExecutor, ExecutionContext};

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
    
    let node_uuid = uuid::Uuid::parse_str(&node_id)
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

/// Create node executor based on node type
fn create_node_executor(node: &aether_types::Node) -> Result<Box<dyn NodeExecutor + Send + Sync>, String> {
    // This would need to be implemented based on your node system
    // For now, return a placeholder
    Err("Node executor creation not implemented".to_string())
}

/// Serialize parameter value to string
pub fn serialize_parameter_value(value: &ParameterValue) -> String {
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

/// Response for graph execution
#[derive(Debug, Serialize)]
pub struct ExecutionResponse {
    pub success: bool,
    pub message: String,
    pub executed_nodes: Vec<String>,
    pub execution_errors: Vec<String>,
    pub frame_number: u64,
    pub execution_time_ms: u64,
}

/// Response for node result queries
#[derive(Debug, Serialize)]
pub struct NodeResultResponse {
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub outputs: HashMap<String, String>,
    pub success: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_serialize_parameter_value() {
        assert_eq!(serialize_parameter_value(&ParameterValue::None), "null");
        assert_eq!(serialize_parameter_value(&ParameterValue::Float(3.14)), "3.14");
        assert_eq!(serialize_parameter_value(&ParameterValue::Boolean(true)), "true");
        assert_eq!(serialize_parameter_value(&ParameterValue::String("test")), "\"test\"");
    }
}
