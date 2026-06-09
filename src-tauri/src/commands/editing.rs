use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::{Arc, Mutex};
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


#[tauri::command]
pub fn create_project(name: String, path: Option<String>, state: State<Arc<Mutex<EditingState>>>) -> Result<Project, String> {
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
pub fn open_project(path: String, state: State<Arc<Mutex<EditingState>>>) -> Result<Project, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("Project file not found: {}", path));
    }

    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read project file: {}", e))?;

    let mut project: Project = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse project file: {}", e))?;

    project.path = Some(path);
    project.modified_at = chrono::Utc::now().timestamp();

    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.current_project = Some(project.clone());

    Ok(project)
}

#[tauri::command]
pub fn save_project(state: State<Arc<Mutex<EditingState>>>) -> Result<Project, String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;

    if let Some(project) = state_guard.current_project.as_mut() {
        project.modified_at = chrono::Utc::now().timestamp();

        if let Some(ref path) = project.path {
            let content = serde_json::to_string_pretty(&project)
                .map_err(|e| format!("Failed to serialize project: {}", e))?;

            fs::write(path, content)
                .map_err(|e| format!("Failed to write project file: {}", e))?;
        }

        Ok(project.clone())
    } else {
        Err("No project is currently open".to_string())
    }
}

#[tauri::command]
pub fn get_current_project(state: State<Arc<Mutex<EditingState>>>) -> Result<Option<Project>, String> {
    let state_guard = state.lock().map_err(|e| e.to_string())?;
    Ok(state_guard.current_project.clone())
}

#[tauri::command]
pub fn close_project(state: State<Arc<Mutex<EditingState>>>) -> Result<(), String> {
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
    state: State<Arc<Mutex<EditingState>>>
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

#[tauri::command]
pub fn create_track(name: String, track_type: String, state: State<Arc<Mutex<EditingState>>>) -> Result<TimelineTrack, String> {
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
pub fn delete_track(track_id: String, state: State<Arc<Mutex<EditingState>>>) -> Result<(), String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.timeline_tracks.retain(|t| t.id != track_id);
    state_guard.timeline_clips.retain(|c| c.track_id != track_id);
    Ok(())
}

#[tauri::command]
pub fn get_timeline_tracks(state: State<Arc<Mutex<EditingState>>>) -> Result<Vec<TimelineTrack>, String> {
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
    state: State<Arc<Mutex<EditingState>>>
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
    state: State<Arc<Mutex<EditingState>>>
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
pub fn remove_clip(clip_id: String, state: State<Arc<Mutex<EditingState>>>) -> Result<(), String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.timeline_clips.retain(|c| c.id != clip_id);
    Ok(())
}

#[tauri::command]
pub fn get_timeline_clips(state: State<Arc<Mutex<EditingState>>>) -> Result<Vec<TimelineClip>, String> {
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
    state: State<Arc<Mutex<EditingState>>>
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
pub fn move_clip(clip_id: String, new_track_id: String, new_start_time: f64, state: State<Arc<Mutex<EditingState>>>) -> Result<TimelineClip, String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;

    if let Some(clip) = state_guard.timeline_clips.iter_mut().find(|c| c.id == clip_id) {
        clip.track_id = new_track_id;
        clip.start_time = new_start_time;
        return Ok(clip.clone());
    }

    Err("Clip not found".to_string())
}


#[tauri::command]
pub fn import_media(path: String, analyze: bool, state: State<Arc<Mutex<EditingState>>>) -> Result<MediaItem, String> {
    let media_id = Uuid::new_v4().to_string();
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("Media file not found: {}", path));
    }

    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let extension = file_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let media_type = match extension.as_str() {
        "mp4" | "mov" | "avi" | "mkv" | "webm" | "wmv" | "flv" => "video",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" => "image",
        "mp3" | "wav" | "aac" | "flac" | "ogg" | "m4a" => "audio",
        _ => "video",
    }.to_string();

    let file_size = fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0);

    let mut media_item = MediaItem {
        id: media_id.clone(),
        name: file_name,
        path: path.clone(),
        media_type,
        duration: 0.0,
        size: file_size,
        resolution: None,
        framerate: None,
        codec: None,
        thumbnail_path: None,
    };

    if analyze {
        if let Ok(metadata) = analyze_media_internal(&path) {
            if let Some(duration) = metadata.get("duration").and_then(|v| v.as_f64()) {
                media_item.duration = duration;
            }
            if let Some(resolution) = metadata.get("resolution") {
                let width = resolution.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let height = resolution.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                if width > 0 && height > 0 {
                    media_item.resolution = Some((width, height));
                }
            }
            if let Some(framerate) = metadata.get("framerate").and_then(|v| v.as_f64()) {
                media_item.framerate = Some(framerate);
            }
            if let Some(codec) = metadata.get("codec").and_then(|v| v.as_str()) {
                media_item.codec = Some(codec.to_string());
            }
        }
    }

    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.media_items.push(media_item.clone());

    Ok(media_item)
}

#[tauri::command]
pub fn get_media_items(state: State<Arc<Mutex<EditingState>>>) -> Result<Vec<MediaItem>, String> {
    let state_guard = state.lock().map_err(|e| e.to_string())?;
    Ok(state_guard.media_items.clone())
}

#[tauri::command]
pub fn remove_media(media_id: String, state: State<Arc<Mutex<EditingState>>>) -> Result<(), String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.media_items.retain(|m| m.id != media_id);
    state_guard.timeline_clips.retain(|c| c.media_id != media_id);
    Ok(())
}

fn analyze_media_internal(path: &str) -> Result<serde_json::Value, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            path
        ])
        .output()
        .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

    if !output.status.success() {
        return Err("ffprobe failed to analyze media".to_string());
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let probe_data: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let mut result = serde_json::json!({});

    if let Some(format) = probe_data.get("format") {
        if let Some(duration) = format.get("duration").and_then(|v| v.as_str()) {
            if let Ok(dur) = duration.parse::<f64>() {
                result["duration"] = serde_json::json!(dur);
            }
        }
        if let Some(bitrate) = format.get("bit_rate").and_then(|v| v.as_str()) {
            if let Ok(br) = bitrate.parse::<u64>() {
                result["bitrate"] = serde_json::json!(br);
            }
        }
    }

    if let Some(streams) = probe_data.get("streams").and_then(|v| v.as_array()) {
        for stream in streams {
            let codec_type = stream.get("codec_type").and_then(|v| v.as_str()).unwrap_or("");

            if codec_type == "video" {
                if let Some(width) = stream.get("width").and_then(|v| v.as_u64()) {
                    if let Some(height) = stream.get("height").and_then(|v| v.as_u64()) {
                        result["resolution"] = serde_json::json!({
                            "width": width,
                            "height": height
                        });
                    }
                }

                if let Some(codec) = stream.get("codec_name").and_then(|v| v.as_str()) {
                    result["codec"] = serde_json::json!(codec);
                }

                if let Some(fps) = stream.get("r_frame_rate").and_then(|v| v.as_str()) {
                    let parts: Vec<&str> = fps.split('/').collect();
                    if parts.len() == 2 {
                        if let (Ok(num), Ok(den)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                            if den > 0.0 {
                                result["framerate"] = serde_json::json!(num / den);
                            }
                        }
                    }
                }
            } else if codec_type == "audio" {
                let mut audio = serde_json::json!({});

                if let Some(sample_rate) = stream.get("sample_rate").and_then(|v| v.as_str()) {
                    if let Ok(sr) = sample_rate.parse::<u32>() {
                        audio["sample_rate"] = serde_json::json!(sr);
                    }
                }

                if let Some(channels) = stream.get("channels").and_then(|v| v.as_u64()) {
                    audio["channels"] = serde_json::json!(channels);
                }

                if let Some(codec) = stream.get("codec_name").and_then(|v| v.as_str()) {
                    audio["codec"] = serde_json::json!(codec);
                }

                result["audio"] = audio;
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn analyze_media(path: String) -> Result<serde_json::Value, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("Media file not found: {}", path));
    }

    analyze_media_internal(&path)
}
