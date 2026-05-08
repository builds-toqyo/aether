use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;
use std::fs;
use std::path::Path;
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub created_at: i64,
    pub modified_at: i64,
    pub settings: ProjectSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSettings {
    pub resolution: (u32, u32),
    pub framerate: u32,
    pub audio_sample_rate: u32,
    pub auto_save: bool,
    pub auto_save_interval: u32,
    pub proxy_enabled: bool,
    pub proxy_resolution: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimelineClip {
    pub id: String,
    pub media_id: String,
    pub track_id: String,
    pub start_time: f64,
    pub duration: f64,
    pub offset: f64,
    pub speed: f64,
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimelineTrack {
    pub id: String,
    pub name: String,
    pub track_type: String,
    pub locked: bool,
    pub muted: bool,
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub media_type: String,
    pub duration: f64,
    pub size: u64,
    pub resolution: Option<(u32, u32)>,
    pub framerate: Option<f64>,
    pub codec: Option<String>,
    pub thumbnail_path: Option<String>,
}

pub struct EditingState {
    pub current_project: Option<Project>,
    pub timeline_tracks: Vec<TimelineTrack>,
    pub timeline_clips: Vec<TimelineClip>,
    pub media_items: Vec<MediaItem>,
}

impl EditingState {
    pub fn new() -> Self {
        EditingState {
            current_project: None,
            timeline_tracks: Vec::new(),
            timeline_clips: Vec::new(),
            media_items: Vec::new(),
        }
    }
}

// Project Management Commands

#[tauri::command]
pub fn create_project(name: String, path: Option<String>, state: State<Mutex<EditingState>>) -> Result<Project, String> {
    let project_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    
    let project = Project {
        id: project_id.clone(),
        name,
        path,
        created_at: now,
        modified_at: now,
        settings: ProjectSettings {
            resolution: (1920, 1080),
            framerate: 30,
            audio_sample_rate: 48000,
            auto_save: true,
            auto_save_interval: 300,
            proxy_enabled: false,
            proxy_resolution: "1080p".to_string(),
        },
    };
    
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.current_project = Some(project.clone());
    
    Ok(project)
}

#[tauri::command]
pub fn open_project(path: String, state: State<Mutex<EditingState>>) -> Result<Project, String> {
    // In a real implementation, this would load from disk
    // For now, we'll create a mock project
    let project_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    
    let project = Project {
        id: project_id.clone(),
        name: "Loaded Project".to_string(),
        path: Some(path),
        created_at: now - 86400, // 1 day ago
        modified_at: now,
        settings: ProjectSettings {
            resolution: (1920, 1080),
            framerate: 30,
            audio_sample_rate: 48000,
            auto_save: true,
            auto_save_interval: 300,
            proxy_enabled: false,
            proxy_resolution: "1080p".to_string(),
        },
    };
    
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.current_project = Some(project.clone());
    
    Ok(project)
}

#[tauri::command]
pub fn save_project(state: State<Mutex<EditingState>>) -> Result<Project, String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    
    if let Some(project) = state_guard.current_project.as_mut() {
        project.modified_at = chrono::Utc::now().timestamp();
        Ok(project.clone())
    } else {
        Err("No project is currently open".to_string())
    }
}

#[tauri::command]
pub fn get_current_project(state: State<Mutex<EditingState>>) -> Result<Option<Project>, String> {
    let state_guard = state.lock().map_err(|e| e.to_string())?;
    Ok(state_guard.current_project.clone())
}

#[tauri::command]
pub fn close_project(state: State<Mutex<EditingState>>) -> Result<(), String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.current_project = None;
    state_guard.timeline_tracks.clear();
    state_guard.timeline_clips.clear();
    state_guard.media_items.clear();
    Ok(())
}

#[tauri::command]
pub fn update_project_settings(
    project_id: String,
    settings: ProjectSettings,
    state: State<Mutex<EditingState>>
) -> Result<Project, String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    
    if let Some(project) = state_guard.current_project.as_mut() {
        if project.id == project_id {
            project.settings = settings;
            project.modified_at = chrono::Utc::now().timestamp();
            return Ok(project.clone());
        }
    }
    
    Err("Project not found".to_string())
}

// Timeline Commands

#[tauri::command]
pub fn create_track(name: String, track_type: String, state: State<Mutex<EditingState>>) -> Result<TimelineTrack, String> {
    let track_id = Uuid::new_v4().to_string();
    
    let track = TimelineTrack {
        id: track_id.clone(),
        name,
        track_type,
        locked: false,
        muted: false,
        volume: 1.0,
    };
    
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.timeline_tracks.push(track.clone());
    
    Ok(track)
}

#[tauri::command]
pub fn delete_track(track_id: String, state: State<Mutex<EditingState>>) -> Result<(), String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.timeline_tracks.retain(|t| t.id != track_id);
    state_guard.timeline_clips.retain(|c| c.track_id != track_id);
    Ok(())
}

#[tauri::command]
pub fn get_timeline_tracks(state: State<Mutex<EditingState>>) -> Result<Vec<TimelineTrack>, String> {
    let state_guard = state.lock().map_err(|e| e.to_string())?;
    Ok(state_guard.timeline_tracks.clone())
}

#[tauri::command]
pub fn update_track(
    track_id: String,
    name: Option<String>,
    locked: Option<bool>,
    muted: Option<bool>,
    volume: Option<f64>,
    state: State<Mutex<EditingState>>
) -> Result<TimelineTrack, String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    
    if let Some(track) = state_guard.timeline_tracks.iter_mut().find(|t| t.id == track_id) {
        if let Some(name) = name {
            track.name = name;
        }
        if let Some(locked) = locked {
            track.locked = locked;
        }
        if let Some(muted) = muted {
            track.muted = muted;
        }
        if let Some(volume) = volume {
            track.volume = volume;
        }
        return Ok(track.clone());
    }
    
    Err("Track not found".to_string())
}

#[tauri::command]
pub fn add_clip(
    media_id: String,
    track_id: String,
    start_time: f64,
    duration: f64,
    offset: f64,
    state: State<Mutex<EditingState>>
) -> Result<TimelineClip, String> {
    let clip_id = Uuid::new_v4().to_string();
    
    let clip = TimelineClip {
        id: clip_id.clone(),
        media_id,
        track_id,
        start_time,
        duration,
        offset,
        speed: 1.0,
        volume: 1.0,
    };
    
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.timeline_clips.push(clip.clone());
    
    Ok(clip)
}

#[tauri::command]
pub fn remove_clip(clip_id: String, state: State<Mutex<EditingState>>) -> Result<(), String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.timeline_clips.retain(|c| c.id != clip_id);
    Ok(())
}

#[tauri::command]
pub fn get_timeline_clips(state: State<Mutex<EditingState>>) -> Result<Vec<TimelineClip>, String> {
    let state_guard = state.lock().map_err(|e| e.to_string())?;
    Ok(state_guard.timeline_clips.clone())
}

#[tauri::command]
pub fn update_clip(
    clip_id: String,
    start_time: Option<f64>,
    duration: Option<f64>,
    offset: Option<f64>,
    speed: Option<f64>,
    volume: Option<f64>,
    state: State<Mutex<EditingState>>
) -> Result<TimelineClip, String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    
    if let Some(clip) = state_guard.timeline_clips.iter_mut().find(|c| c.id == clip_id) {
        if let Some(start_time) = start_time {
            clip.start_time = start_time;
        }
        if let Some(duration) = duration {
            clip.duration = duration;
        }
        if let Some(offset) = offset {
            clip.offset = offset;
        }
        if let Some(speed) = speed {
            clip.speed = speed;
        }
        if let Some(volume) = volume {
            clip.volume = volume;
        }
        return Ok(clip.clone());
    }
    
    Err("Clip not found".to_string())
}

#[tauri::command]
pub fn move_clip(clip_id: String, new_track_id: String, new_start_time: f64, state: State<Mutex<EditingState>>) -> Result<TimelineClip, String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    
    if let Some(clip) = state_guard.timeline_clips.iter_mut().find(|c| c.id == clip_id) {
        clip.track_id = new_track_id;
        clip.start_time = new_start_time;
        return Ok(clip.clone());
    }
    
    Err("Clip not found".to_string())
}


#[tauri::command]
pub fn import_media(path: String, analyze: bool, state: State<Mutex<EditingState>>) -> Result<MediaItem, String> {
    let media_id = Uuid::new_v4().to_string();
    
    // In a real implementation, this would analyze the media file
    // For now, we'll create a mock media item
    let media_item = MediaItem {
        id: media_id.clone(),
        name: path.split('/').last().unwrap_or("Unknown").to_string(),
        path: path.clone(),
        media_type: "video".to_string(), // Default to video
        duration: 10.0, // Mock duration
        size: 1024 * 1024 * 100, // Mock size (100MB)
        resolution: Some((1920, 1080)),
        framerate: Some(30.0),
        codec: Some("H.264".to_string()),
        thumbnail_path: None,
    };
    
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.media_items.push(media_item.clone());
    
    Ok(media_item)
}

#[tauri::command]
pub fn get_media_items(state: State<Mutex<EditingState>>) -> Result<Vec<MediaItem>, String> {
    let state_guard = state.lock().map_err(|e| e.to_string())?;
    Ok(state_guard.media_items.clone())
}

#[tauri::command]
pub fn remove_media(media_id: String, state: State<Mutex<EditingState>>) -> Result<(), String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.media_items.retain(|m| m.id != media_id);
    state_guard.timeline_clips.retain(|c| c.media_id != media_id);
    Ok(())
}

#[tauri::command]
pub fn analyze_media(path: String) -> Result<serde_json::Value, String> {
    // In a real implementation, this would use FFmpeg or similar to analyze the media
    // For now, return mock data
    let metadata = serde_json::json!({
        "duration": 10.0,
        "resolution": {
            "width": 1920,
            "height": 1080
        },
        "framerate": 30.0,
        "codec": "H.264",
        "bitrate": 5000000,
        "audio": {
            "sample_rate": 48000,
            "channels": 2,
            "codec": "AAC"
        }
    });
    
    Ok(metadata)
}
