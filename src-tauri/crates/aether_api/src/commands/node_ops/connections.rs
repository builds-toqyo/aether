use serde::{Serialize};
use tauri::State;
use uuid::Uuid;
use log::{debug, info};

use crate::state::AppState;
use aether_types::{Connection, PinDataType};


#[tauri::command]
pub async fn connect_nodes(
    output_node_id: String,
    output_pin_name: String,
    input_node_id: String,
    input_pin_name: String,
    state: State<'_, AppState>,
) -> Result<ConnectionResponse, String> {
    debug!("Connecting {}.{} -> {}.{}",
        output_node_id, output_pin_name, input_node_id, input_pin_name);

    // Parse UUIDs
    let output_id = Uuid::parse_str(&output_node_id)
        .map_err(|e| format!("{}", e))?;
    let input_id = Uuid::parse_str(&input_node_id)
        .map_err(|e| format!("{}", e))?;

    let mut graph = state.graph.lock().map_err(|e| format!("{}", e))?;

    // Get pin IDs and validate
    let (output_pin_id, input_pin_id) = {
        let output_node = graph.get_node(&output_id)
            .ok_or_else(|| format!("Node not found: {}", output_id))?;
        let input_node = graph.get_node(&input_id)
            .ok_or_else(|| format!("Node not found: {}", input_id))?;

        let output_pin = output_node.get_output_pin_by_name(&output_pin_name)
            .ok_or_else(|| format!("Output pin not found: {}", output_pin_name))?;
        let input_pin = input_node.get_input_pin_by_name(&input_pin_name)
            .ok_or_else(|| format!("Input pin not found: {}", input_pin_name))?;

        if !are_pin_types_compatible(&output_pin.data_type, &input_pin.data_type) {
            return Err(format!("Incompatible pin types: {:?} and {:?}",
                output_pin.data_type, input_pin.data_type));
        }

        if input_pin.connection.is_some() {
            return Err(format!("Input pin already connected: {}", input_pin_name));
        }

        (output_pin.id, input_pin.id)
    };

    // Create connection
    let connection = Connection {
        id: Uuid::new_v4(),
        output_node_id: output_id,
        output_pin_id: output_pin_id,
        input_node_id: input_id,
        input_pin_id: input_pin_id,
        enabled: true,
    };

    // Add connection to graph
    graph.connections.insert(connection.id, connection.clone());

    // Update input pin connection
    if let Some(input_node) = graph.nodes.get_mut(&input_id) {
        for pin in &mut input_node.inputs {
            if pin.id == input_pin_id {
                pin.connection = Some(connection.id);
                break;
            }
        }
    }

    info!("Connected {}.{} -> {}.{}",
        output_node_id, output_pin_name, input_node_id, input_pin_name);

    Ok(ConnectionResponse {
        id: connection.id.to_string(),
        output_node_id: output_node_id,
        output_pin_name,
        input_node_id: input_node_id,
        input_pin_name,
        success: true,
        message: "TODO".to_string(),
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


    let (output_node_id, input_node_id, output_pin_name, input_pin_name, input_node_uuid) = {
        let connection = graph.connections.values()
            .find(|conn| conn.id == connection_uuid)
            .ok_or_else(|| format!("Connection not found: {}", connection_id))?;

        (
            connection.output_node_id.to_string(),
            connection.input_node_id.to_string(),
            connection.output_pin_id.to_string(),
            connection.input_pin_id.to_string(),
            connection.input_node_id,
        )
    };

    graph.connections.retain(|_, conn| conn.id != connection_uuid);

    if let Some(input_node) = graph.nodes.get_mut(&input_node_uuid) {
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


pub fn are_pin_types_compatible(output_type: &PinDataType, input_type: &PinDataType) -> bool {

    match (output_type, input_type) {
        (PinDataType::Image, PinDataType::Image) => true,
        (PinDataType::Float, PinDataType::Float) => true,
        (PinDataType::Vector2, PinDataType::Vector2) => true,
        (PinDataType::Vector3, PinDataType::Vector3) => true,
        (PinDataType::Vector4, PinDataType::Vector4) => true,
        (PinDataType::Color, PinDataType::Color) => true,

        (PinDataType::Float, PinDataType::Vector2) => true,
        (PinDataType::Float, PinDataType::Vector3) => true,
        (PinDataType::Float, PinDataType::Vector4) => true,
        (PinDataType::Vector4, PinDataType::Color) => true,
        (PinDataType::Color, PinDataType::Vector4) => true,
        _ => false,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_type_compatibility() {
        assert!(are_pin_types_compatible(&PinDataType::Image, &PinDataType::Image));
        assert!(are_pin_types_compatible(&PinDataType::Float, &PinDataType::Vector2));
        assert!(!are_pin_types_compatible(&PinDataType::Image, &PinDataType::Float));
    }
}
