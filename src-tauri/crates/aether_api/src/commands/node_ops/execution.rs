use serde::{Serialize};
use tauri::State;
use std::collections::HashMap;
use anyhow::Result;
use log::{debug, info, error};

use crate::state::AppState;
use aether_types::{ParameterValue};
use aether_core::nodes::{NodeExecutor, ExecutionContext};
use aether_core::nodes::core::factory::CoreNodes;


#[tauri::command]
pub async fn execute_graph(
    frame: Option<u64>,
    state: State<'_, AppState>,
) -> Result<ExecutionResponse, String> {
    debug!("Executing graph, frame: {:?}", frame);

    let graph = state.graph.lock().map_err(|e| format!("{}", e))?;

    // Calculate execution order
    let execution_order = aether_core::nodes::execution_order::ExecutionOrderCalculator::calculate_order(&graph)
        .map_err(|e| format!("{}", e))?;

    debug!("Execution order length: {}", execution_order.len());

    // Create execution context
    let frame_number = frame.unwrap_or(0);
    let mut context = ExecutionContext::new(
        frame_number,
        frame_number as f64 / 30.0,
        30.0,
        (1920, 1080),
    );

    // Execute nodes in order
    let mut executed_nodes = Vec::new();
    let mut execution_errors = Vec::new();

    for node_id in &execution_order {
        if let Some(node) = graph.get_node(node_id) {
            debug!("{} ({})", node.name, node_id);

            // Create node executor (this would need to be implemented based on node type)
            let mut executor = create_node_executor(node)?;

            // Execute node
            match executor.execute(&mut context) {
                Ok(_) => {
                    executed_nodes.push(node_id.to_string());
                    debug!("{}", node.name);
                }
                Err(e) => {
                    let error_msg = format!("{} failed: {}", node.name, e);
                    execution_errors.push(error_msg.clone());
                    error!("Node execution error: {}", error_msg);
                }
            }
        }
    }

    let success = execution_errors.is_empty();
    let message = if success {
        format!("Executed {} nodes", executed_nodes.len())
    } else {
        format!("{} errors during execution", execution_errors.len())
    };

    info!("Success: {}, Errors: {}", success, execution_errors.len());

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


    let node = graph.get_node(&node_uuid)
        .ok_or_else(|| format!("Node not found: {}", node_id))?;

    let execution_results = state.execution_results.lock().map_err(|e| format!("Failed to lock execution results: {}", e))?;

    let mut outputs = HashMap::new();

    if let Some(pin_name) = pin_name {

        let output_pin = node.get_output_pin_by_name(&pin_name)
            .ok_or_else(|| format!("Output pin '{}' not found", pin_name))?;

        if let Some(value) = execution_results.get(&output_pin.id) {
            outputs.insert(pin_name.clone(), serialize_parameter_value(value));
        } else {
            outputs.insert(pin_name, "null".to_string());
        }
    } else {

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

#[tauri::command]
pub async fn connect_node_to_clip(
    node_id: String,
    output_pin_name: String,
    clip_id: String,
    state: State<'_, AppState>,
) -> Result<NodeClipConnectionResponse, String> {
    debug!("Connecting node {} output {} to clip {}", node_id, output_pin_name, clip_id);

    let node_uuid = uuid::Uuid::parse_str(&node_id)
        .map_err(|e| format!("Invalid node ID: {}", e))?;

    let graph = state.graph.lock().map_err(|e| format!("Failed to lock graph: {}", e))?;

    let node = graph.get_node(&node_uuid)
        .ok_or_else(|| format!("Node not found: {}", node_id))?;

    let output_pin = node.get_output_pin_by_name(&output_pin_name)
        .ok_or_else(|| format!("Output pin '{}' not found", output_pin_name))?;

    let mut execution_results = state.execution_results.lock().map_err(|e| format!("Failed to lock execution results: {}", e))?;

    let connection_key = format!("{}_{}", node_id, output_pin_name);
    execution_results.insert(
        uuid::Uuid::new_v4(),
        ParameterValue::String(format!("clip:{}", clip_id))
    );

    info!("Connected node {} output {} to clip {}", node_id, output_pin_name, clip_id);

    Ok(NodeClipConnectionResponse {
        node_id,
        output_pin_name,
        clip_id,
        success: true,
        message: format!("Node output connected to clip {}", clip_id),
    })
}

fn create_node_executor(node: &aether_types::Node) -> Result<Box<dyn NodeExecutor + Send + Sync>, String> {
    CoreNodes::create_node_by_type(node.node_type.clone(), node.clone())
        .ok_or_else(|| format!("Unsupported node type: {:?}", node.node_type))
}


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
        ParameterValue::Binary(data) => format!("Binary({} bytes)", data.len()),
    }
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
pub struct NodeClipConnectionResponse {
    pub node_id: String,
    pub output_pin_name: String,
    pub clip_id: String,
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
