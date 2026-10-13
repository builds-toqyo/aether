use log::{debug, info};
use serde_json::Value;
use tauri::{AppHandle, Emitter};

/// Send a real-time event from the frontend.
/// Broadcasts the event back to all windows so other listeners can react.
#[tauri::command]
pub async fn send_real_time_event(event: Value, app: AppHandle) -> Result<Value, String> {
    debug!("Received real-time event: {:?}", event);

    let event_type = event
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    app.emit("real_time_event", &event)
        .map_err(|e| format!("Failed to broadcast real-time event: {}", e))?;

    info!("Broadcast real-time event of type '{}'", event_type);

    Ok(serde_json::json!({
        "success": true,
        "type": event_type,
    }))
}
