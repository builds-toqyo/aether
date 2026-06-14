use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use log::{debug, info, warn};
use tauri::State;
use crate::state::AppState;

/// Represents a single camera angle within a multicam group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraAngle {
    pub id: String,
    pub name: String,
    pub media_id: String,
    pub offset_ms: i64,
    pub enabled: bool,
    pub metadata: HashMap<String, String>,
}

impl CameraAngle {
    pub fn new(name: impl Into<String>, media_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            media_id: media_id.into(),
            offset_ms: 0,
            enabled: true,
            metadata: HashMap::new(),
        }
    }
}

/// A multicam clip groups multiple synchronized camera angles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MulticamClip {
    pub id: String,
    pub name: String,
    pub angles: Vec<CameraAngle>,
    pub active_angle_id: Option<String>,
    pub duration_ms: i64,
    pub metadata: HashMap<String, String>,
}

impl MulticamClip {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            angles: Vec::new(),
            active_angle_id: None,
            duration_ms: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn add_angle(&mut self, angle: CameraAngle) {
        if self.active_angle_id.is_none() {
            self.active_angle_id = Some(angle.id.clone());
        }
        self.angles.push(angle);
    }

    pub fn remove_angle(&mut self, angle_id: &str) {
        self.angles.retain(|a| a.id != angle_id);
        if self.active_angle_id.as_deref() == Some(angle_id) {
            self.active_angle_id = self.angles.first().map(|a| a.id.clone());
        }
    }

    pub fn set_active_angle(&mut self, angle_id: &str) -> Result<(), String> {
        if self.angles.iter().any(|a| a.id == angle_id) {
            self.active_angle_id = Some(angle_id.to_string());
            Ok(())
        } else {
            Err(format!("Angle '{}' not found in multicam clip", angle_id))
        }
    }

    pub fn get_active_angle(&self) -> Option<&CameraAngle> {
        self.active_angle_id.as_ref()
            .and_then(|id| self.angles.iter().find(|a| a.id == *id))
    }
}

/// In-memory registry for multicam clips
#[derive(Debug, Default)]
pub struct MulticamRegistry {
    pub clips: HashMap<String, MulticamClip>,
}

impl MulticamRegistry {
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MulticamResponse {
    pub success: bool,
    pub message: String,
    pub clip: Option<MulticamClip>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMulticamRequest {
    pub name: String,
    pub angle_media_ids: Vec<String>,
}

#[tauri::command]
pub async fn multicam_create(
    request: CreateMulticamRequest,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    debug!("Creating multicam clip: {}", request.name);

    let mut clip = MulticamClip::new(&request.name);
    for (i, media_id) in request.angle_media_ids.iter().enumerate() {
        let angle = CameraAngle::new(format!("Camera {}", i + 1), media_id.clone());
        clip.add_angle(angle);
    }

    let clip_id = clip.id.clone();
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;
    registry.clips.insert(clip_id.clone(), clip.clone());

    info!("Created multicam clip '{}' with {} angles", request.name, request.angle_media_ids.len());

    Ok(MulticamResponse {
        success: true,
        message: format!("Created multicam clip {}", clip_id),
        clip: Some(clip),
    })
}

#[tauri::command]
pub async fn multicam_add_angle(
    clip_id: String,
    name: String,
    media_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    let clip = registry.clips.get_mut(&clip_id)
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))?;

    let angle = CameraAngle::new(name, media_id);
    clip.add_angle(angle);

    Ok(MulticamResponse {
        success: true,
        message: format!("Added angle to multicam clip {}", clip_id),
        clip: Some(clip.clone()),
    })
}

#[tauri::command]
pub async fn multicam_remove_angle(
    clip_id: String,
    angle_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    let clip = registry.clips.get_mut(&clip_id)
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))?;

    clip.remove_angle(&angle_id);

    Ok(MulticamResponse {
        success: true,
        message: format!("Removed angle {} from multicam clip {}", angle_id, clip_id),
        clip: Some(clip.clone()),
    })
}

#[tauri::command]
pub async fn multicam_set_active_angle(
    clip_id: String,
    angle_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    let clip = registry.clips.get_mut(&clip_id)
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))?;

    clip.set_active_angle(&angle_id)?;

    Ok(MulticamResponse {
        success: true,
        message: format!("Set active angle {} on multicam clip {}", angle_id, clip_id),
        clip: Some(clip.clone()),
    })
}

#[tauri::command]
pub async fn multicam_get_clip(
    clip_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamClip, String> {
    let registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    registry.clips.get(&clip_id)
        .cloned()
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))
}

#[tauri::command]
pub async fn multicam_list_clips(
    state: State<'_, AppState>,
) -> Result<Vec<MulticamClip>, String> {
    let registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    Ok(registry.clips.values().cloned().collect())
}

#[tauri::command]
pub async fn multicam_delete_clip(
    clip_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    registry.clips.remove(&clip_id);

    Ok(MulticamResponse {
        success: true,
        message: format!("Deleted multicam clip {}", clip_id),
        clip: None,
    })
}

#[tauri::command]
pub async fn multicam_sync_by_timecode(
    clip_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    let clip = registry.clips.get_mut(&clip_id)
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))?;

    // For MVP: reset all offsets to 0 (real implementation would parse timecode from media)
    for angle in &mut clip.angles {
        angle.offset_ms = 0;
    }

    warn!("multicam_sync_by_timecode: using zero-offset sync (timecode parsing not yet implemented)");

    Ok(MulticamResponse {
        success: true,
        message: "Angles synchronized by timecode (zero-offset baseline)".to_string(),
        clip: Some(clip.clone()),
    })
}
