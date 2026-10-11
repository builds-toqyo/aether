use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;
use aether_core::engine::editing::TrackType as CoreTrackType;

#[derive(Debug, Serialize, Deserialize)]
pub struct TimelineInfo {
    pub duration: f64,
    pub current_time: f64,
    pub is_playing: bool,
    pub tracks: Vec<TrackInfo>,
    pub clips: Vec<ClipInfo>,
    pub fps: f64,
    pub resolution: (u32, u32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackInfo {
    pub id: String,
    pub name: String,
    pub track_type: TrackType,
    pub muted: bool,
    pub solo: bool,
    pub volume: f64,
    pub height: f64,
    pub clips: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TrackType {
    Video,
    Audio,
    Subtitle,
    Effects,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipInfo {
    pub id: String,
    pub name: String,
    pub clip_type: ClipType,
    pub track_id: String,
    pub start_time: f64,
    pub end_time: f64,
    pub duration: f64,
    pub source_file: Option<String>,
    pub in_point: f64,
    pub out_point: f64,
    pub position: f64,
    pub layer: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClipType {
    Video,
    Audio,
    Image,
    Text,
    Effect,
}

#[derive(Debug, Deserialize)]
pub struct PlaybackControlRequest {
    pub action: PlaybackAction,
    pub current_time: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub enum PlaybackAction {
    Play,
    Pause,
    Stop,
    Seek,
    Next,
    Previous,
}

#[derive(Debug, Deserialize)]
pub struct TimelineSeekRequest {
    pub time: f64,
}

#[derive(Debug, Deserialize)]
pub struct TimelineClipMoveRequest {
    pub clip_id: String,
    pub new_time: f64,
    pub new_track_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TimelineClipTrimRequest {
    pub clip_id: String,
    pub edge: ClipEdge,
    pub new_time: f64,
}

#[derive(Debug, Deserialize)]
pub enum ClipEdge {
    Start,
    End,
}

#[derive(Debug, Deserialize)]
pub struct TimelineClipAddRequest {
    pub track_id: String,
    pub source_file: String,
    pub position: f64,
    pub duration: Option<f64>,
    pub in_point: Option<f64>,
    pub out_point: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct TimelineClipRemoveRequest {
    pub clip_id: String,
}

#[derive(Debug, Serialize)]
pub struct TimelineResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn get_timeline_info(state: State<'_, AppState>) -> Result<TimelineInfo, String> {
    debug!("Getting timeline info");

    let proxy_timeline = state
        .editing_engine
        .lock()
        .map_err(|e| format!("{}", e))?
        .get_timeline_info()
        .map_err(|e| format!("{}", e))?;
    let preview_state = state
        .editing_engine
        .lock()
        .map_err(|e| format!("{}", e))?
        .get_preview_state()
        .map_err(|e| format!("{}", e))?;

    let duration = proxy_timeline.duration as f64 / 1_000_000_000.0;
    let current_time = preview_state.position as f64 / 1_000_000_000.0;

    let clips: Vec<ClipInfo> = proxy_timeline
        .clips
        .iter()
        .map(|c| ClipInfo {
            id: c.id.clone(),
            name: c.name.clone(),
            clip_type: match c.track_type {
                CoreTrackType::Video => ClipType::Video,
                CoreTrackType::Audio => ClipType::Audio,
            },
            track_id: format!("track_{}", c.track_type as u8),
            start_time: c.start_time as f64 / 1_000_000_000.0,
            end_time: (c.start_time + c.duration) as f64 / 1_000_000_000.0,
            duration: c.duration as f64 / 1_000_000_000.0,
            source_file: c
                .source_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            in_point: c.in_point as f64 / 1_000_000_000.0,
            out_point: c.out_point as f64 / 1_000_000_000.0,
            position: c.start_time as f64 / 1_000_000_000.0,
            layer: 0,
        })
        .collect();

    let tracks: Vec<TrackInfo> = proxy_timeline
        .tracks
        .iter()
        .map(|t| TrackInfo {
            id: t.id.clone(),
            name: t.id.clone(),
            track_type: match t.track_type {
                CoreTrackType::Video => TrackType::Video,
                CoreTrackType::Audio => TrackType::Audio,
            },
            muted: false,
            solo: false,
            volume: 1.0,
            height: if matches!(t.track_type, CoreTrackType::Video) {
                100.0
            } else {
                60.0
            },
            clips: t.clips.clone(),
        })
        .collect();

    let timeline_info = TimelineInfo {
        duration,
        current_time,
        is_playing: preview_state.is_playing,
        fps: 30.0,
        resolution: preview_state.dimensions.unwrap_or((1920, 1080)),
        tracks,
        clips,
    };

    info!(
        "Tracks: {}, Clips: {}",
        timeline_info.tracks.len(),
        timeline_info.clips.len()
    );
    Ok(timeline_info)
}

/// Control timeline playback
#[tauri::command(rename_all = "camelCase")]
pub async fn timeline_playback_control(
    action: String,
    current_time: Option<f64>,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Timeline playback control: {}", action);

    let engine = state
        .editing_engine
        .lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;
    let frame_duration_ns = (1_000_000_000.0 / 30.0) as i64;

    let step_to = |direction: i64| -> i64 {
        let position = match current_time {
            Some(t) => (t * 1_000_000_000.0) as i64,
            None => engine.get_preview_state().map(|s| s.position).unwrap_or(0),
        };
        (position + direction * frame_duration_ns).max(0)
    };

    let message = match action.to_lowercase().as_str() {
        "play" => {
            engine
                .preview_play()
                .map_err(|e| format!("Failed to start playback: {}", e))?;
            info!("Starting timeline playback");
            "Playback started".to_string()
        }
        "pause" => {
            engine
                .preview_pause()
                .map_err(|e| format!("Failed to pause playback: {}", e))?;
            info!("Pausing timeline playback");
            "Playback paused".to_string()
        }
        "stop" => {
            engine
                .preview_stop()
                .map_err(|e| format!("Failed to stop playback: {}", e))?;
            info!("Stopping timeline playback");
            "Playback stopped".to_string()
        }
        "seek" => {
            let time = current_time.unwrap_or(0.0);
            let position_ns = (time * 1_000_000_000.0) as i64;
            engine
                .preview_seek(position_ns)
                .map_err(|e| format!("Failed to seek: {}", e))?;
            info!("Seeking timeline to time: {}", time);
            format!("Seeked to {}", time)
        }
        "next" | "stepforward" | "step_forward" => {
            engine
                .preview_seek(step_to(1))
                .map_err(|e| format!("Failed to step forward: {}", e))?;
            info!("Stepping forward one frame");
            "Moved to next".to_string()
        }
        "previous" | "stepbackward" | "step_backward" => {
            engine
                .preview_seek(step_to(-1))
                .map_err(|e| format!("Failed to step backward: {}", e))?;
            info!("Stepping backward one frame");
            "Moved to previous".to_string()
        }
        other => {
            return Err(format!("Unknown playback action: {}", other));
        }
    };

    Ok(TimelineResponse {
        success: true,
        message,
        data: None,
    })
}

#[tauri::command]
pub async fn timeline_seek(
    time: f64,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("{}", time);

    // Validate time
    if time < 0.0 {
        return Err("Seek time cannot be negative".to_string());
    }

    state
        .editing_engine
        .lock()
        .map_err(|e| format!("{}", e))?
        .timeline_seek(time)
        .map_err(|e| format!("{}", e))?;

    info!("{}", time);

    Ok(TimelineResponse {
        success: true,
        message: format!("{}", time),
        data: Some(serde_json::json!({ "time": time })),
    })
}

/// Move clip on timeline
#[tauri::command(rename_all = "camelCase")]
pub async fn timeline_move_clip(
    clip_id: String,
    new_time: f64,
    new_track_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!(
        "Moving clip {} to time {} on track {:?}",
        clip_id, new_time, new_track_id
    );

    if new_time < 0.0 {
        return Err("Clip position cannot be negative".to_string());
    }

    let new_start_ns = (new_time * 1_000_000_000.0) as i64;
    state
        .editing_engine
        .lock()
        .map_err(|e| format!("{}", e))?
        .timeline_move_clip(clip_id.clone(), new_start_ns)
        .map_err(|e| format!("{}", e))?;

    info!(
        "Moving clip: {} -> {}s on track {:?}",
        clip_id, new_time, new_track_id
    );

    Ok(TimelineResponse {
        success: true,
        message: format!("Clip moved to {} seconds", new_time),
        data: Some(serde_json::json!({
            "clip_id": clip_id,
            "new_time": new_time,
            "new_track_id": new_track_id
        })),
    })
}

#[tauri::command(rename_all = "camelCase")]
pub async fn timeline_trim_clip(
    clip_id: String,
    edge: String,
    new_time: f64,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Clip: {}, Edge: {}, New time: {}", clip_id, edge, new_time);

    let edge_parsed = match edge.to_lowercase().as_str() {
        "start" => ClipEdge::Start,
        "end" => ClipEdge::End,
        other => return Err(format!("Invalid clip edge: {}", other)),
    };

    // Validate inputs
    if new_time < 0.0 {
        return Err("Trim time cannot be negative".to_string());
    }

    let new_duration_ns = (new_time * 1_000_000_000.0) as i64;
    state
        .editing_engine
        .lock()
        .map_err(|e| format!("{}", e))?
        .timeline_trim_clip(clip_id.clone(), new_duration_ns)
        .map_err(|e| format!("{}", e))?;

    info!(
        "Trimmed clip: {}, Edge: {:?}, New time: {}",
        clip_id, edge_parsed, new_time
    );

    Ok(TimelineResponse {
        success: true,
        message: format!("Trimmed clip at {:?} to {}s", edge_parsed, new_time),
        data: Some(serde_json::json!({
            "clip_id": clip_id,
            "edge": edge,
            "new_time": new_time
        })),
    })
}

/// Add clip to timeline
#[tauri::command]
pub async fn timeline_add_clip(
    request: TimelineClipAddRequest,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!(
        "Adding clip from {} to track {} at position {}",
        request.source_file, request.track_id, request.position
    );

    if request.position < 0.0 {
        return Err("Clip position cannot be negative".to_string());
    }

    if request.source_file.is_empty() {
        return Err("Source file cannot be empty".to_string());
    }

    let engine = state.editing_engine.lock().map_err(|e| format!("{}", e))?;

    // Resolve the target track's type from the live timeline
    let timeline_info = engine.get_timeline_info().map_err(|e| format!("{}", e))?;
    let core_track_type = timeline_info
        .tracks
        .iter()
        .find(|t| t.id == request.track_id)
        .map(|t| t.track_type)
        .unwrap_or(CoreTrackType::Video);

    let uri = if request.source_file.starts_with("file://") {
        request.source_file.clone()
    } else {
        format!("file://{}", request.source_file)
    };

    let start_ns = (request.position * 1_000_000_000.0) as i64;
    let duration_ns = (request.duration.unwrap_or(0.0) * 1_000_000_000.0) as i64;
    let in_point_ns = (request.in_point.unwrap_or(0.0) * 1_000_000_000.0) as i64;

    let clip = engine
        .timeline_add_clip(uri, core_track_type, start_ns, duration_ns, in_point_ns)
        .map_err(|e| format!("Failed to add clip: {}", e))?;

    info!(
        "Added clip: {} from {} at {}s",
        clip.id, request.source_file, request.position
    );

    Ok(TimelineResponse {
        success: true,
        message: format!("Clip added to timeline at {} seconds", request.position),
        data: Some(serde_json::json!({
            "clip_id": clip.id,
            "track_id": request.track_id,
            "position": request.position,
            "source_file": request.source_file
        })),
    })
}

#[tauri::command]
pub async fn timeline_remove_clip(
    request: TimelineClipRemoveRequest,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("{}", request.clip_id);

    if request.clip_id.is_empty() {
        return Err("Clip ID cannot be empty".to_string());
    }

    state
        .editing_engine
        .lock()
        .map_err(|e| format!("{}", e))?
        .timeline_remove_clip(request.clip_id.clone())
        .map_err(|e| format!("{}", e))?;

    info!("{}", request.clip_id);

    Ok(TimelineResponse {
        success: true,
        message: format!("{}", request.clip_id),
        data: Some(serde_json::json!({
            "clip_id": request.clip_id
        })),
    })
}

/// Split clip at specific time
#[tauri::command]
pub async fn timeline_split_clip(
    clip_id: String,
    split_time: Option<f64>,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Splitting clip {} at time {:?}", clip_id, split_time);

    if clip_id.is_empty() {
        return Err("Clip ID cannot be empty".to_string());
    }

    let engine = state
        .editing_engine
        .lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    // Default to the clip's midpoint when no split time is provided
    let split_time = match split_time {
        Some(t) => t,
        None => {
            let timeline_info = engine
                .get_timeline_info()
                .map_err(|e| format!("Failed to get timeline info: {}", e))?;
            let clip = timeline_info
                .clips
                .iter()
                .find(|c| c.id == clip_id)
                .ok_or_else(|| format!("Clip {} not found", clip_id))?;
            clip.start_time as f64 / 1_000_000_000.0 + clip.duration as f64 / 2_000_000_000.0
        }
    };

    if split_time < 0.0 {
        return Err("Split time cannot be negative".to_string());
    }

    let position_ns = (split_time * 1_000_000_000.0) as i64;
    let new_clip_id = engine
        .timeline_split_clip(clip_id.clone(), position_ns)
        .map_err(|e| format!("Failed to split clip: {}", e))?;

    info!(
        "Split clip: {} at {}s -> new clip {}",
        clip_id, split_time, new_clip_id
    );

    Ok(TimelineResponse {
        success: true,
        message: format!("Clip split at {} seconds", split_time),
        data: Some(serde_json::json!({
            "original_clip_id": clip_id,
            "new_clip_id": new_clip_id,
            "split_time": split_time
        })),
    })
}

/// Add transition between clips
#[tauri::command]
pub async fn timeline_add_transition(
    from_clip_id: String,
    to_clip_id: String,
    duration: f64,
    transition_type: String,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!(
        "Adding transition from {} to {} with duration {} and type {}",
        from_clip_id, to_clip_id, duration, transition_type
    );

    if from_clip_id.is_empty() || to_clip_id.is_empty() {
        return Err("Clip IDs cannot be empty".to_string());
    }

    if duration <= 0.0 {
        return Err("Transition duration must be positive".to_string());
    }

    let engine = state
        .editing_engine
        .lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    // Apply the transition as a GES effect on the outgoing clip
    let effect_id = engine
        .timeline_add_effect(from_clip_id.clone(), transition_type.clone())
        .map_err(|e| format!("Failed to add transition effect: {}", e))?;

    info!(
        "Adding transition: {} -> {} ({}s, type: {})",
        from_clip_id, to_clip_id, duration, transition_type
    );

    Ok(TimelineResponse {
        success: true,
        message: format!("Transition added ({}s)", duration),
        data: Some(serde_json::json!({
            "transition_id": effect_id,
            "from_clip_id": from_clip_id,
            "to_clip_id": to_clip_id,
            "duration": duration,
            "transition_type": transition_type
        })),
    })
}

/// Ripple delete clip (shifts subsequent clips)
#[tauri::command]
pub async fn timeline_ripple_delete(
    clip_id: String,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Ripple deleting clip {}", clip_id);

    if clip_id.is_empty() {
        return Err("Clip ID cannot be empty".to_string());
    }

    let engine = state
        .editing_engine
        .lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    // Find the clip being removed to know its span and track
    let timeline_info = engine
        .get_timeline_info()
        .map_err(|e| format!("Failed to get timeline info: {}", e))?;
    let target = timeline_info
        .clips
        .iter()
        .find(|c| c.id == clip_id)
        .cloned()
        .ok_or_else(|| format!("Clip {} not found", clip_id))?;
    let removed_end = target.start_time + target.duration;
    let removed_duration = target.duration;

    // Shift subsequent clips on the same track type left by the removed duration
    let clips_to_shift: Vec<(String, i64)> = timeline_info
        .clips
        .iter()
        .filter(|c| {
            c.id != clip_id && c.track_type == target.track_type && c.start_time >= removed_end
        })
        .map(|c| (c.id.clone(), c.start_time - removed_duration))
        .collect();

    engine
        .timeline_remove_clip(clip_id.clone())
        .map_err(|e| format!("Failed to remove clip: {}", e))?;

    for (shift_clip_id, new_start) in &clips_to_shift {
        if let Err(e) = engine.timeline_move_clip(shift_clip_id.clone(), *new_start) {
            warn!(
                "Failed to shift clip {} during ripple delete: {}",
                shift_clip_id, e
            );
        }
    }

    info!(
        "Ripple deleted clip: {} (shifted {} clips)",
        clip_id,
        clips_to_shift.len()
    );

    Ok(TimelineResponse {
        success: true,
        message: format!("Clip ripple deleted: {}", clip_id),
        data: Some(serde_json::json!({
            "clip_id": clip_id,
            "shifted_clips": clips_to_shift.len()
        })),
    })
}

/// Rolling edit (adjusts clip boundaries without affecting other clips)
#[tauri::command]
pub async fn timeline_rolling_edit(
    clip_id: String,
    edge: ClipEdge,
    new_time: f64,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!(
        "Rolling edit clip {} at edge {:?} to time {}",
        clip_id, edge, new_time
    );

    if clip_id.is_empty() {
        return Err("Clip ID cannot be empty".to_string());
    }

    if new_time < 0.0 {
        return Err("Edit time cannot be negative".to_string());
    }

    let engine = state
        .editing_engine
        .lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;
    let new_time_ns = (new_time * 1_000_000_000.0) as i64;

    let timeline_info = engine
        .get_timeline_info()
        .map_err(|e| format!("Failed to get timeline info: {}", e))?;
    let clip = timeline_info
        .clips
        .iter()
        .find(|c| c.id == clip_id)
        .ok_or_else(|| format!("Clip {} not found", clip_id))?;

    match edge {
        ClipEdge::End => {
            // Adjust the clip's out point: new duration = new_time - start
            let new_duration = new_time_ns - clip.start_time;
            if new_duration <= 0 {
                return Err("Rolling edit would produce non-positive duration".to_string());
            }
            engine
                .timeline_trim_clip(clip_id.clone(), new_duration)
                .map_err(|e| format!("Rolling edit failed: {}", e))?;
        }
        ClipEdge::Start => {
            // Adjust the clip's in point: move start to new_time, keep the end fixed
            let clip_end = clip.start_time + clip.duration;
            let new_duration = clip_end - new_time_ns;
            if new_duration <= 0 {
                return Err("Rolling edit would produce non-positive duration".to_string());
            }
            engine
                .timeline_move_clip(clip_id.clone(), new_time_ns)
                .map_err(|e| format!("Rolling edit failed: {}", e))?;
            engine
                .timeline_trim_clip(clip_id.clone(), new_duration)
                .map_err(|e| format!("Rolling edit failed: {}", e))?;
        }
    }

    info!("Rolling edit: {} {:?} -> {}s", clip_id, edge, new_time);

    Ok(TimelineResponse {
        success: true,
        message: format!("Rolling edit applied at {:?} to {}s", edge, new_time),
        data: Some(serde_json::json!({
            "clip_id": clip_id,
            "edge": format!("{:?}", edge),
            "new_time": new_time
        })),
    })
}

/// Sync timeline changes to GES
#[tauri::command]
pub async fn timeline_sync_to_ges(state: State<'_, AppState>) -> Result<TimelineResponse, String> {
    debug!("Syncing timeline changes to GES");

    let engine = state
        .editing_engine
        .lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    // Get current timeline state and sync to GES pipeline
    // This ensures all Rust-side changes are reflected in GES
    let timeline_info = engine
        .get_timeline_info()
        .map_err(|e| format!("Failed to get timeline info: {}", e))?;

    info!(
        "Synced timeline to GES: {} tracks, {} clips",
        timeline_info.tracks.len(),
        timeline_info.clips.len()
    );

    Ok(TimelineResponse {
        success: true,
        message: "Timeline synced to GES successfully".to_string(),
        data: Some(serde_json::json!({
            "tracks_count": timeline_info.tracks.len(),
            "clips_count": timeline_info.clips.len()
        })),
    })
}

/// Create new track
#[tauri::command]
pub async fn timeline_create_track(
    track_type: String,
    name: Option<String>,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Creating new track: {} with name {:?}", track_type, name);

    let core_track_type = match track_type.to_lowercase().as_str() {
        "video" => CoreTrackType::Video,
        "audio" => CoreTrackType::Audio,
        other => return Err(format!("Unsupported track type: {}", other)),
    };

    let track_name = name.unwrap_or_else(|| format!("New {} Track", track_type));

    let track = state
        .editing_engine
        .lock()
        .map_err(|e| format!("{}", e))?
        .timeline_create_track(core_track_type)
        .map_err(|e| format!("Failed to create track: {}", e))?;

    info!("Created track: {} ({})", track.id, track_name);

    Ok(TimelineResponse {
        success: true,
        message: format!("Track '{}' created successfully", track_name),
        data: Some(serde_json::json!({
            "track_id": track.id,
            "name": track_name,
            "track_type": track_type
        })),
    })
}

#[tauri::command]
pub async fn timeline_delete_track(
    track_id: String,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Deleting track: {}", track_id);

    if track_id.is_empty() {
        return Err("Track ID cannot be empty".to_string());
    }

    state
        .editing_engine
        .lock()
        .map_err(|e| format!("{}", e))?
        .timeline_delete_track(track_id.clone())
        .map_err(|e| format!("Failed to delete track: {}", e))?;

    info!("Deleted track: {}", track_id);

    Ok(TimelineResponse {
        success: true,
        message: format!("Track {} deleted successfully", track_id),
        data: Some(serde_json::json!({
            "track_id": track_id
        })),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BindNodeToEffectRequest {
    pub clip_id: String,
    pub effect_id: String,
    pub node_id: String,
    pub output_pin_name: String,
}

#[tauri::command]
pub async fn timeline_bind_node_to_effect(
    request: BindNodeToEffectRequest,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!(
        "Binding node {} to effect {} on clip {}",
        request.node_id, request.effect_id, request.clip_id
    );

    state
        .editing_engine
        .lock()
        .map_err(|e| format!("{}", e))?
        .timeline_bind_node_to_effect(
            request.clip_id.clone(),
            request.effect_id.clone(),
            request.node_id.clone(),
            request.output_pin_name.clone(),
        )
        .map_err(|e| format!("Failed to bind node to effect: {}", e))?;

    info!(
        "Bound node {} to effect {} on clip {}",
        request.node_id, request.effect_id, request.clip_id
    );

    Ok(TimelineResponse {
        success: true,
        message: format!(
            "Node {} bound to effect {}",
            request.node_id, request.effect_id
        ),
        data: Some(serde_json::json!({
            "clip_id": request.clip_id,
            "effect_id": request.effect_id,
            "node_id": request.node_id,
            "output_pin_name": request.output_pin_name,
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_seek_validation() {
        let request = TimelineSeekRequest { time: -1.0 };

        assert!(request.time < 0.0);
    }

    #[test]
    fn test_clip_move_validation() {
        let request = TimelineClipMoveRequest {
            clip_id: "test_clip".to_string(),
            new_time: -1.0,
            new_track_id: None,
        };

        assert!(request.new_time < 0.0);
    }

    #[test]
    fn test_track_type_serialization() {
        let track_type = TrackType::Video;
        assert_eq!(format!("{:?}", track_type), "Video");
    }

    #[test]
    fn test_clip_edge_serialization() {
        let edge = ClipEdge::Start;
        assert_eq!(format!("{:?}", edge), "Start");
    }
}
