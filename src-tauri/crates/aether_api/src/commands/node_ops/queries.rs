use serde::{Serialize};
use tauri::State;
use std::collections::HashMap;
use log::debug;

use crate::state::AppState;


#[tauri::command]
pub async fn get_graph_info(
    state: State<'_, AppState>,
) -> Result<GraphInfoResponse, String> {
    debug!("Getting graph information");

    let graph = state.graph.lock().map_err(|e| format!("Failed to lock graph: {}", e))?;

    let node_count = graph.nodes.len();
    let connection_count = graph.connections.len();


    let mut node_types = HashMap::new();
    for node in graph.get_nodes() {
        let type_name = format!("{:?}", node.node_type);
        *node_types.entry(type_name).or_insert(0) += 1;
    }


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
    fn test_node_info_serialization() {
        let node_info = NodeInfo {
            id: "test-id".to_string(),
            name: "Test Node".to_string(),
            node_type: "Input".to_string(),
            input_count: 0,
            output_count: 1,
            enabled: true,
        };


        assert_eq!(node_info.id, "test-id");
        assert_eq!(node_info.name, "Test Node");
        assert_eq!(node_info.node_type, "Input");
        assert_eq!(node_info.input_count, 0);
        assert_eq!(node_info.output_count, 1);
        assert!(node_info.enabled);
    }
}
