use anyhow::Result;
use log::{debug, info};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::state::AppState;
use aether_types::NodeType;

#[tauri::command(rename_all = "camelCase")]
pub async fn create_node(
    node_type: String,
    name: String,
    position: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<NodeResponse, String> {
    debug!("{}, {}", node_type, name);

    // Parse node type
    let node_type_enum = parse_node_type(&node_type).map_err(|e| e.to_string())?;

    // Create node with default pins for its type (input/output/merge get
    // type-specific pin layouts; others default to 1-in/1-out)
    let (input_count, output_count) = default_pin_counts(&node_type_enum);
    let mut node = aether_core::nodes::basic::create_basic_node_with_pins(
        node_type_enum,
        name.clone(),
        input_count,
        output_count,
    );

    // Accept position as either {x, y} object or [x, y] tuple
    let position_tuple = position.as_ref().and_then(|p| match p {
        serde_json::Value::Object(_) => Some((
            p.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
            p.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
        )),
        serde_json::Value::Array(arr) if arr.len() >= 2 => Some((
            arr[0].as_f64().unwrap_or(0.0),
            arr[1].as_f64().unwrap_or(0.0),
        )),
        _ => None,
    });

    // Persist position on the node so it survives graph reloads
    if let Some((x, y)) = position_tuple {
        debug!("Setting position: {}, {}", x, y);
        node.set_position(x as f32, y as f32);
    }

    // Add to graph
    let mut graph = state.graph.lock().map_err(|e| format!("{}", e))?;
    let node_id = node.id;
    graph.nodes.insert(node_id, node);

    info!("{}, {}", name, node_id);

    Ok(NodeResponse {
        id: node_id,
        node_type,
        name,
        position: position_tuple,
        success: true,
        message: "Node created successfully".to_string(),
    })
}

/// Command to delete a node from the graph
#[tauri::command(rename_all = "camelCase")]
pub async fn delete_node(
    node_id: String,
    state: State<'_, AppState>,
) -> Result<NodeResponse, String> {
    debug!("Deleting node: {}", node_id);

    let node_uuid = Uuid::parse_str(&node_id).map_err(|e| format!("Invalid node ID: {}", e))?;

    let mut graph = state
        .graph
        .lock()
        .map_err(|e| format!("Failed to lock graph: {}", e))?;

    let node = graph
        .get_node(&node_uuid)
        .ok_or_else(|| format!("Node not found: {}", node_id))?;

    let node_name = node.name.clone();
    let node_type = format!("{:?}", node.node_type);

    graph
        .connections
        .retain(|_, conn| conn.output_node_id != node_uuid && conn.input_node_id != node_uuid);

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

/// Update a node's canvas position
#[tauri::command(rename_all = "camelCase")]
pub async fn update_node_position(
    node_id: String,
    x: f32,
    y: f32,
    state: State<'_, AppState>,
) -> Result<NodeResponse, String> {
    let node_uuid = Uuid::parse_str(&node_id).map_err(|e| format!("Invalid node ID: {}", e))?;

    let mut graph = state
        .graph
        .lock()
        .map_err(|e| format!("Failed to lock graph: {}", e))?;

    let node = graph
        .get_node_mut(&node_uuid)
        .ok_or_else(|| format!("Node not found: {}", node_id))?;

    node.set_position(x, y);

    let node_type = format!("{:?}", node.node_type);
    let name = node.name.clone();

    Ok(NodeResponse {
        id: node_uuid,
        node_type,
        name,
        position: Some((x as f64, y as f64)),
        success: true,
        message: "Node position updated".to_string(),
    })
}

pub fn parse_node_type(node_type: &str) -> Result<NodeType, String> {
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

fn default_pin_counts(node_type: &NodeType) -> (usize, usize) {
    match node_type {
        NodeType::Input => (0, 1),
        NodeType::Output => (1, 0),
        NodeType::Merge => (2, 1),
        _ => (1, 1),
    }
}

#[derive(Debug, Serialize)]
pub struct NodeResponse {
    pub id: Uuid,
    pub node_type: String,
    pub name: String,
    pub position: Option<(f64, f64)>,
    pub success: bool,
    pub message: String,
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
}
