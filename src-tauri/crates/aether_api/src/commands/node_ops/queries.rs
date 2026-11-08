use log::debug;
use serde::Serialize;
use std::collections::HashMap;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn get_graph_info(state: State<'_, AppState>) -> Result<GraphInfoResponse, String> {
    debug!("Getting graph information");

    let graph = state
        .graph
        .lock()
        .map_err(|e| format!("Failed to lock graph: {}", e))?;

    let node_count = graph.nodes.len();
    let connection_count = graph.connections.len();

    let mut node_types = HashMap::new();
    for node in graph.get_nodes() {
        let type_name = format!("{:?}", node.node_type);
        *node_types.entry(type_name).or_insert(0) += 1;
    }

    let nodes: Vec<NodeInfo> = graph
        .get_nodes()
        .map(|node| NodeInfo {
            id: node.id.to_string(),
            name: node.name.clone(),
            node_type: format!("{:?}", node.node_type),
            input_count: node.inputs.len(),
            output_count: node.outputs.len(),
            enabled: node.enabled,
            position: (node.position.0, node.position.1),
            input_pins: node
                .inputs
                .iter()
                .map(|pin| PinInfo {
                    name: pin.name.clone(),
                    data_type: format!("{:?}", pin.data_type),
                })
                .collect(),
            output_pins: node
                .outputs
                .iter()
                .map(|pin| PinInfo {
                    name: pin.name.clone(),
                    data_type: format!("{:?}", pin.data_type),
                })
                .collect(),
            parameters: node
                .parameters
                .iter()
                .map(|(name, param)| {
                    (
                        name.clone(),
                        serde_json::to_value(&param.value).unwrap_or_default(),
                    )
                })
                .collect(),
        })
        .collect();

    // Resolve connection pin UUIDs to pin names (the API works in pin names)
    let pin_name = |node_id: &uuid::Uuid, pin_id: &uuid::Uuid, is_input: bool| -> String {
        graph
            .get_node(node_id)
            .and_then(|n| {
                if is_input {
                    n.inputs
                        .iter()
                        .find(|p| &p.id == pin_id)
                        .map(|p| p.name.clone())
                } else {
                    n.outputs
                        .iter()
                        .find(|p| &p.id == pin_id)
                        .map(|p| p.name.clone())
                }
            })
            .unwrap_or_else(|| pin_id.to_string())
    };

    let connections: Vec<ConnectionInfo> = graph
        .connections
        .values()
        .map(|conn| ConnectionInfo {
            id: conn.id.to_string(),
            output_node_id: conn.output_node_id.to_string(),
            output_pin_name: pin_name(&conn.output_node_id, &conn.output_pin_id, false),
            input_node_id: conn.input_node_id.to_string(),
            input_pin_name: pin_name(&conn.input_node_id, &conn.input_pin_id, true),
        })
        .collect();

    Ok(GraphInfoResponse {
        node_count,
        connection_count,
        node_types,
        nodes,
        connections,
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
    pub connections: Vec<ConnectionInfo>,
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
    pub position: (f32, f32),
    pub input_pins: Vec<PinInfo>,
    pub output_pins: Vec<PinInfo>,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct PinInfo {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub output_node_id: String,
    pub output_pin_name: String,
    pub input_node_id: String,
    pub input_pin_name: String,
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
            position: (0.0, 0.0),
            input_pins: vec![],
            output_pins: vec![PinInfo {
                name: "output_0".to_string(),
                data_type: "Image".to_string(),
            }],
            parameters: HashMap::new(),
        };

        assert_eq!(node_info.id, "test-id");
        assert_eq!(node_info.name, "Test Node");
        assert_eq!(node_info.node_type, "Input");
        assert_eq!(node_info.input_count, 0);
        assert_eq!(node_info.output_count, 1);
        assert!(node_info.enabled);
    }
}
