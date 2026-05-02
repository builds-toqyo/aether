use serde::{Serialize};
use tauri::State;
use uuid::Uuid;
use anyhow::Result;
use log::{debug, info};

use crate::state::AppState;
use aether_types::{Node, NodeType};

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
        node_type,
        name,
        position,
        success: true,
        message: "Node created successfully".to_string(),
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

/// Parse node type from string
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

/// Response for node operations
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
