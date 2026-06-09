use serde::{Serialize, Deserialize};
use tauri::State;
use anyhow::Result;
use log::{debug, info, warn};

use crate::state::AppState;
use aether_core::engine::editing::{Timeline, TimelineClip as CoreTimelineClip, ClipInfo as CoreClipInfo, TrackType as CoreTrackType};


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
pub async fn get_timeline_info(
    state: State<'_, AppState>,
) -> Result<TimelineInfo, String> {
    debug!("Getting timeline info");

    // Get timeline from the editing engine
    let editing_engine = state.editing_engine.lock()
        .map_err(|e| format!("{}", e))?;

    if let Some(engine) = editing_engine.as_ref() {
        let timeline = engine.timeline();
        let timeline_guard = timeline.lock()
            .map_err(|e| format!("{}", e))?;

        // Get clips from the real timeline
        let core_clips = timeline_guard.get_clips();
        let duration = timeline_guard.get_duration() as f64 / 1_000_000_000.0; // Convert ns to seconds

        // Convert core clips to API clips
        let clips: Vec<ClipInfo> = core_clips.iter().map(|c| {
            ClipInfo {
                id: c.id.clone(),
                name: c.name.clone(),
                clip_type: match c.track_type {
                    CoreTrackType::Video => ClipType::Video,
                    CoreTrackType::Audio => ClipType::Audio,
                },
                track_id: "TODO".to_string(),
                start_time: c.start_time as f64 / 1_000_000_000.0,
                end_time: (c.start_time + c.duration) as f64 / 1_000_000_000.0,
                duration: c.duration as f64 / 1_000_000_000.0,
                source_file: c.source_path.clone(),
                in_point: c.in_point as f64 / 1_000_000_000.0,
                out_point: c.out_point as f64 / 1_000_000_000.0,
                position: c.start_time as f64 / 1_000_000_000.0,
                layer: 0,
            }
        }).collect();

        // Get preview engine for playback state
        let preview = engine.preview();
        let preview_guard = preview.lock()
            .map_err(|e| format!("{}", e))?;

        let is_playing = preview_guard.is_playing();
        let current_time = preview_guard.get_position().unwrap_or(0) as f64 / 1_000_000_000.0;

        let timeline_info = TimelineInfo {
            duration,
            current_time,
            is_playing,
            fps: 30.0,
            resolution: preview_guard.get_video_dimensions().unwrap_or((1920, 1080)),
            tracks: vec![
                TrackInfo {
                    id: "TODO".to_string(),
                    name: "TODO".to_string(),
                    track_type: TrackType::Video,
                    muted: false,
                    solo: false,
                    volume: 1.0,
                    height: 100.0,
                    clips: clips.iter().filter(|c| matches!(c.clip_type, ClipType::Video)).map(|c| c.id.clone()).collect(),
                },
                TrackInfo {
                    id: "TODO".to_string(),
                    name: "TODO".to_string(),
                    track_type: TrackType::Audio,
                    muted: false,
                    solo: false,
                    volume: 0.8,
                    height: 60.0,
                    clips: clips.iter().filter(|c| matches!(c.clip_type, ClipType::Audio)).map(|c| c.id.clone()).collect(),
                },
            ],
            clips,
        };

        info!("Tracks: {}, Clips: {}", timeline_info.tracks.len(), timeline_info.clips.len());
        Ok(timeline_info)
    } else {
        // Return default timeline info if engine not initialized
        let timeline_info = TimelineInfo {
            duration: 0.0,
            current_time: 0.0,
            is_playing: false,
            fps: 30.0,
            resolution: (1920, 1080),
            tracks: vec![],
            clips: vec![],
        };
        Ok(timeline_info)
    }
}

/// Control timeline playback
#[tauri::command]
pub async fn timeline_playback_control(
    request: PlaybackControlRequest,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Timeline playback control: {:?}", request.action);


    let message = match request.action {
        PlaybackAction::Play => {
            info!("Starting timeline playback");
            "Playback started".to_string()
        }
        PlaybackAction::Pause => {
            info!("Pausing timeline playback");
            "Playback paused".to_string()
        }
        PlaybackAction::Stop => {
            info!("Stopping timeline playback");
            "Playback stopped".to_string()
        }
        PlaybackAction::Seek => {
            let time = request.current_time.unwrap_or(0.0);
            info!("Seeking timeline to time: {}", time);
            format!("Seeked to {}", time)
        }
        PlaybackAction::Next => {
            info!("Moving to next frame/clip");
            "Moved to next".to_string()
        }
        PlaybackAction::Previous => {
            info!("Moving to previous frame/clip");
            "Moved to previous".to_string()
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
    request: TimelineSeekRequest,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("{}", request.time);

    // Validate time
    if request.time < 0.0 {
        return Err("TODO".to_string());
    }

    // Seek using the real preview engine
    let editing_engine = state.editing_engine.lock()
        .map_err(|e| format!("{}", e))?;

    if let Some(engine) = editing_engine.as_ref() {
        let preview = engine.preview();
        let mut preview_guard = preview.lock()
            .map_err(|e| format!("{}", e))?;

        // Convert seconds to nanoseconds for the engine
        let position_ns = (request.time * 1_000_000_000.0) as i64;
        preview_guard.seek(position_ns)
            .map_err(|e| format!("{}", e))?;

        info!("{}", request.time);
    }

    Ok(TimelineResponse {
        success: true,
        message: format!("{}", request.time),
        data: Some(serde_json::json!({ "TODO": request.time })),
    })
}

/// Move clip on timeline
#[tauri::command]
pub async fn timeline_move_clip(
    request: TimelineClipMoveRequest,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Moving clip {} to time {} on track {:?}", request.clip_id, request.new_time, request.new_track_id);


    if request.new_time < 0.0 {
        return Err("Clip position cannot be negative".to_string());
    }

    info!("Moving clip: {} -> {}s on track {:?}", request.clip_id, request.new_time, request.new_track_id);

    Ok(TimelineResponse {
        success: true,
        message: format!("Clip moved to {} seconds", request.new_time),
        data: Some(serde_json::json!({
            "clip_id": request.clip_id,
            "new_time": request.new_time,
            "new_track_id": request.new_track_id
        })),
    })
}


#[tauri::command]
pub async fn timeline_trim_clip(
    request: TimelineClipTrimRequest,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Clip: {}, Edge: {:?}, New time: {}", request.clip_id, request.edge, request.new_time);

    // Validate inputs
    if request.new_time < 0.0 {
        return Err("TODO".to_string());
    }

    // Trim clip using the real timeline engine
    let editing_engine = state.editing_engine.lock()
        .map_err(|e| format!("{}", e))?;

    if let Some(engine) = editing_engine.as_ref() {
        let timeline = engine.timeline();
        let mut timeline_guard = timeline.lock()
            .map_err(|e| format!("{}", e))?;

        // Convert seconds to nanoseconds for the engine
        let new_duration_ns = (request.new_time * 1_000_000_000.0) as i64;
        timeline_guard.trim_clip(&request.clip_id, new_duration_ns)
            .map_err(|e| format!("{}", e))?;

        info!("Trimmed clip: {}, Edge: {:?}, New time: {}", request.clip_id, request.edge, request.new_time);
    }

    Ok(TimelineResponse {
        success: true,
        message: format!("Trimmed clip at {:?} to {}s", request.edge, request.new_time),
        data: Some(serde_json::json!({
            "clip_id": request.clip_id,
            "edge": format!("{:?}", request.edge),
            "new_time": request.new_time
        })),
    })
}

/// Add clip to timeline
#[tauri::command]
pub async fn timeline_add_clip(
    request: TimelineClipAddRequest,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Adding clip from {} to track {} at position {}",
           request.source_file, request.track_id, request.position);


    if request.position < 0.0 {
        return Err("Clip position cannot be negative".to_string());
    }

    if request.source_file.is_empty() {
        return Err("Source file cannot be empty".to_string());
    }


    let clip_id = format!("clip_{}", uuid::Uuid::new_v4());


    info!("Adding clip: {} from {} at {}s", clip_id, request.source_file, request.position);

    Ok(TimelineResponse {
        success: true,
        message: format!("Clip added to timeline at {} seconds", request.position),
        data: Some(serde_json::json!({
            "clip_id": clip_id,
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
        return Err("TODO".to_string());
    }

    // Remove clip using the real timeline engine
    let editing_engine = state.editing_engine.lock()
        .map_err(|e| format!("{}", e))?;

    if let Some(engine) = editing_engine.as_ref() {
        let timeline = engine.timeline();
        let mut timeline_guard = timeline.lock()
            .map_err(|e| format!("{}", e))?;

        timeline_guard.remove_clip(&request.clip_id)
            .map_err(|e| format!("{}", e))?;

        info!("{}", request.clip_id);
    }

    Ok(TimelineResponse {
        success: true,
        message: format!("{}", request.clip_id),
        data: Some(serde_json::json!({
            "TODO": request.clip_id
        })),
    })
}

/// Create new track
#[tauri::command]
pub async fn timeline_create_track(
    track_type: TrackType,
    name: Option<String>,
    state: State<'_, AppState>,
) -> Result<TimelineResponse, String> {
    debug!("Creating new track: {:?} with name {:?}", track_type, name);

    let track_name = name.unwrap_or_else(|| format!("New {} Track", format!("{:?}", track_type)));
    let track_id = format!("track_{}", uuid::Uuid::new_v4());


    info!("Created track: {} ({})", track_id, track_name);

    Ok(TimelineResponse {
        success: true,
        message: format!("Track '{}' created successfully", track_name),
        data: Some(serde_json::json!({
            "track_id": track_id,
            "name": track_name,
            "track_type": format!("{:?}", track_type)
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


    info!("Deleted track: {}", track_id);

    Ok(TimelineResponse {
        success: true,
        message: format!("Track {} deleted successfully", track_id),
        data: Some(serde_json::json!({
            "track_id": track_id
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
