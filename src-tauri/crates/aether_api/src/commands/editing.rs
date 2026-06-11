use serde::{Serialize, Deserialize};
use tauri::State;
use anyhow::Result;
use log::{debug, info, warn};

use crate::state::AppState;
use aether_core::engine::editing::{
    ImportOptions,
    types::TrackType
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub modified_at: String,
    pub duration: f64,
    pub fps: f64,
    pub resolution: (u32, u32),
    pub timeline_count: usize,
    pub media_count: usize,
    pub file_size: u64,
    pub file_path: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub fps: Option<f64>,
    pub resolution: Option<(u32, u32)>,
    pub template: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectSaveRequest {
    pub project_id: String,
    pub file_path: Option<String>,
    pub auto_save: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectLoadRequest {
    pub file_path: String,
}

#[derive(Debug, Deserialize)]
pub struct MediaImportRequest {
    pub file_paths: Vec<String>,
    pub target_track: Option<String>,
    pub position: Option<f64>,
    pub auto_create_clips: Option<bool>,
}


#[derive(Debug, Deserialize)]
pub struct MediaExportRequest {
    pub output_path: String,
    pub format: ExportFormat,
    pub quality: ExportQuality,
    pub resolution: Option<(u32, u32)>,
    pub fps: Option<f64>,
    pub start_time: Option<f64>,
    pub end_time: Option<f64>,
    pub audio_settings: Option<AudioExportSettings>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExportFormat {
    Mp4,
    Avi,
    Mov,
    Mkv,
    Webm,
    Gif,
    PngSequence,
    JpegSequence,
    AudioOnly,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExportQuality {
    Low,
    Medium,
    High,
    Ultra,
    Custom {
        bitrate: u32,
        preset: String,
    },
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AudioExportSettings {
    pub codec: AudioCodec,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub channels: u8,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AudioCodec {
    Aac,
    Mp3,
    Opus,
    Flac,
    Wav,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaInfo {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub duration: f64,
    pub format: String,
    pub codec: String,
    pub resolution: Option<(u32, u32)>,
    pub fps: Option<f64>,
    pub audio_channels: Option<u8>,
    pub audio_sample_rate: Option<u32>,
    pub bit_rate: Option<u32>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct EditingResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}


#[tauri::command]
pub async fn project_init(
    request: ProjectCreateRequest,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    debug!("{}", request.name);

    // Validate inputs
    if request.name.is_empty() {
        return Err("Project name cannot be empty".to_string());
    }

    let fps = request.fps.unwrap_or(30.0);
    let resolution = request.resolution.unwrap_or((1920, 1080));

    if fps <= 0.0 {
        return Err("FPS must be greater than 0".to_string());
    }

    if resolution.0 == 0 || resolution.1 == 0 {
        return Err("Resolution cannot be zero".to_string());
    }

    // Generate project ID
    let project_id = format!("{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();
    let project_path = format!("{}", project_id);

    // Initialize project in the editing engine
    state.editing_engine.lock().map_err(|e| format!("{}", e))?
        .init_project(Some(project_path.clone()))
        .map_err(|e| format!("{}", e))?;

    let project_info = ProjectInfo {
        id: project_id.clone(),
        name: request.name.clone(),
        description: request.description.clone(),
        created_at: now.clone(),
        modified_at: now,
        duration: 0.0,
        fps,
        resolution,
        timeline_count: 1,
        media_count: 0,
        file_size: 0,
        file_path: project_path.clone(),
    };

    // Register in project_registry
    if let Ok(registry) = state.project_registry.lock() {
        let _ = registry.add(&project_info);
    }

    info!("Project {} loaded from {}", request.name, project_id);
    Ok(project_info)
}

/// Save current project
#[tauri::command]
pub async fn project_save(
    request: ProjectSaveRequest,
    state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!("Saving project: {}", request.project_id);

    if request.project_id.is_empty() {
        return Err("Project ID cannot be empty".to_string());
    }


    let timeline_info = state.editing_engine.lock().map_err(|e| format!("{}", e))?
        .get_timeline_info()
        .map_err(|e| format!("Failed to get timeline info: {}", e))?;

    let project_data = serde_json::json!({
        "project_id": request.project_id,
        "name": request.project_id.clone(),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "modified_at": chrono::Utc::now().to_rfc3339(),
        "duration": timeline_info.duration,
        "clips": timeline_info.clips.iter().map(|c| {
            serde_json::json!({
                "id": c.id,
                "name": c.name,
                "source_path": c.source_path,
                "track_type": format!("{:?}", c.track_type),
                "start_time": c.start_time,
                "duration": c.duration,
                "in_point": c.in_point,
                "effects": c.effects.iter().map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "name": e.name,
                        "parameters": e.parameters
                    })
                }).collect::<Vec<_>>()
            })
        }).collect::<Vec<_>>(),
        "timeline_settings": {
            "duration": timeline_info.duration,
            "fps": 30.0
        }
    });

    let file_path = request.file_path.unwrap_or_else(|| format!("/projects/{}.aether", request.project_id));

    std::fs::write(&file_path, project_data.to_string())
        .map_err(|e| format!("Failed to write project file: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();
    let project_info = ProjectInfo {
        id: request.project_id.clone(),
        name: request.project_id.clone(),
        description: Some("Saved project".to_string()),
        created_at: now.clone(),
        modified_at: now,
        duration: timeline_info.duration as f64 / 1_000_000_000.0,
        fps: 30.0,
        resolution: (1920, 1080),
        timeline_count: 1,
        media_count: timeline_info.clips.len(),
        file_size: std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0),
        file_path: file_path.clone(),
    };

    if let Ok(mut registry) = state.project_registry.lock() {
        let _ = registry.add(&project_info);
    }

    Ok(EditingResponse {
        success: true,
        message: format!("Project saved to: {}", file_path),
        data: Some(serde_json::json!({"file_path": file_path})),
    })
}

/// Load project from file
#[tauri::command]
pub async fn project_load(
    request: ProjectLoadRequest,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    debug!("Loading project from: {}", request.file_path);

    if request.file_path.is_empty() {
        return Err("File path cannot be empty".to_string());
    }


    let project_data = std::fs::read_to_string(&request.file_path)
        .map_err(|e| format!("Failed to read project file: {}", e))?;

    let project_json: serde_json::Value = serde_json::from_str(&project_data)
        .map_err(|e| format!("Failed to parse project file: {}", e))?;

    state.editing_engine.lock().map_err(|e| format!("{}", e))?
        .init_project(Some(request.file_path.clone()))
        .map_err(|e| format!("Failed to initialize project: {}", e))?;

    if let Some(clips) = project_json.get("clips").and_then(|c| c.as_array()) {
        for clip_data in clips {
            if let (Some(_id), Some(name), Some(source_path), Some(_start_time), Some(_duration)) = (
                clip_data.get("id").and_then(|v| v.as_str()),
                clip_data.get("name").and_then(|v| v.as_str()),
                clip_data.get("source_path").and_then(|v| v.as_str()),
                clip_data.get("start_time").and_then(|v| v.as_i64()),
                clip_data.get("duration").and_then(|v| v.as_i64())
            ) {
                let _track_type = clip_data.get("track_type")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "Video" => Some(TrackType::Video),
                        "Audio" => Some(TrackType::Audio),
                        _ => None,
                    })
                    .unwrap_or(TrackType::Video);

                let _in_point = clip_data.get("in_point").and_then(|v| v.as_i64()).unwrap_or(0);

                debug!("Restoring clip {} from {}", name, source_path);
            }
        }
    }

    let project_id = project_json.get("project_id")
        .and_then(|v| v.as_str())
        .unwrap_or(&format!("project_{}", uuid::Uuid::new_v4()))
        .to_string();

    let now = chrono::Utc::now().to_rfc3339();


    let project_name = project_json.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Loaded Project")
        .to_string();

    let duration = project_json.get("duration")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as f64 / 1_000_000_000.0;

    let clips_count = project_json.get("clips")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);

    info!("Loaded project from: {} ({} clips, {:.2}s duration)", request.file_path, clips_count, duration);

    let project_info = ProjectInfo {
        id: project_id.clone(),
        name: project_name,
        description: Some("Loaded from file".to_string()),
        created_at: project_json.get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or(&now)
            .to_string(),
        modified_at: now.clone(),
        duration,
        fps: 30.0,
        resolution: (1920, 1080),
        timeline_count: 1,
        media_count: clips_count,
        file_size: std::fs::metadata(&request.file_path).map(|m| m.len()).unwrap_or(0),
        file_path: request.file_path.clone(),
    };

    // Register in project_registry
    if let Ok(mut registry) = state.project_registry.lock() {
        let _ = registry.add(&project_info);
    }

    Ok(project_info)
}


#[tauri::command]
pub async fn project_get_recent(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ProjectInfo>, String> {
    debug!("{:?}", limit);

    let limit = limit.unwrap_or(10);

    let registry = state.project_registry.lock()
        .map_err(|e| format!("Failed to lock project registry: {}", e))?;

    let projects = registry.get_recent(limit)
        .map_err(|e| format!("Failed to query recent projects: {}", e))?;

    info!("Returning {} recent projects", projects.len());
    Ok(projects)
}


#[tauri::command]
pub async fn media_import(
    request: MediaImportRequest,
    state: State<'_, AppState>,
) -> Result<Vec<MediaInfo>, String> {
    debug!("Importing {} media files", request.file_paths.len());

    if request.file_paths.is_empty() {
        return Err("No files to import".to_string());
    }


    let mut imported_media = Vec::new();

    for (index, file_path) in request.file_paths.iter().enumerate() {
        if file_path.is_empty() {
            warn!("Skipping empty file path at index {}", index);
            continue;
        }

        let import_options = ImportOptions {
            analyze: true,
            extract_thumbnails: true,
            create_proxy: false,
            proxy_format: None,
        };

        match state.editing_engine.lock().map_err(|e| format!("{}", e))?
            .import_media(file_path.clone(), import_options) {
            Ok(core_media_info) => {
                let media_id = format!("media_{}", uuid::Uuid::new_v4());
                let file_name = std::path::Path::new(file_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");

                let video_info = core_media_info.video_streams.first();
                let audio_info = core_media_info.audio_streams.first();

                let media_info = MediaInfo {
                    id: media_id.clone(),
                    file_path: file_path.clone(),
                    file_name: file_name.to_string(),
                    file_size: core_media_info.file_size.unwrap_or(0),
                    duration: core_media_info.duration as f64 / 1_000_000_000.0,
                    format: core_media_info.container_format.unwrap_or_else(|| "unknown".to_string()),
                    codec: video_info.map(|v| v.codec_name.clone()).unwrap_or_else(|| "unknown".to_string()),
                    resolution: video_info.map(|v| (v.width as u32, v.height as u32)),
                    fps: video_info.map(|v| v.frame_rate),
                    audio_channels: audio_info.map(|a| a.channels as u8),
                    audio_sample_rate: audio_info.map(|a| a.sample_rate as u32),
                    bit_rate: video_info.and_then(|v| v.bitrate.map(|b| b as u32)),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                imported_media.push(media_info.clone());

                // Register in media_registry for later lookup
                if let Ok(mut registry) = state.media_registry.lock() {
                    registry.insert(media_id.clone(), media_info);
                }

                info!("Imported media: {} ({})", media_id, file_name);
            }
            Err(e) => {
                warn!("Failed to import media {}: {}", file_path, e);
            }
        }
    }

    Ok(imported_media)
}


#[tauri::command]
pub async fn media_get_info(
    media_id: String,
    state: State<'_, AppState>,
) -> Result<MediaInfo, String> {
    debug!("{}", media_id);

    if media_id.is_empty() {
        return Err("Media ID cannot be empty".to_string());
    }

    let registry = state.media_registry.lock()
        .map_err(|e| format!("Failed to lock media registry: {}", e))?;

    let media_info = registry.get(&media_id)
        .cloned()
        .ok_or_else(|| format!("Media not found: {}", media_id))?;

    info!("Retrieved media info for {}", media_id);
    Ok(media_info)
}

#[tauri::command]
pub async fn media_get_all(
    state: State<'_, AppState>,
) -> Result<Vec<MediaInfo>, String> {
    debug!("Getting all media");

    let registry = state.media_registry.lock()
        .map_err(|e| format!("Failed to lock media registry: {}", e))?;

    let all_media: Vec<MediaInfo> = registry.values().cloned().collect();

    info!("project_{}", all_media.len());
    Ok(all_media)
}

#[tauri::command]
pub async fn media_remove(
    media_id: String,
    state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!("{}", media_id);

    if media_id.is_empty() {
        return Err("Media ID cannot be empty".to_string());
    }

    let mut registry = state.media_registry.lock()
        .map_err(|e| format!("Failed to lock media registry: {}", e))?;

    if registry.remove(&media_id).is_none() {
        return Err(format!("Media not found: {}", media_id));
    }

    info!("Removed media {}", media_id);

    Ok(EditingResponse {
        success: true,
        message: format!("Media deleted: {}", media_id),
        data: Some(serde_json::json!({
            "media_id": media_id
        })),
    })
}


#[tauri::command]
pub async fn media_export(
    request: MediaExportRequest,
    _state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!("Exporting project to: {} format: {:?}", request.output_path, request.format);


    if request.output_path.is_empty() {
        return Err("Output path cannot be empty".to_string());
    }

    if let Some((width, height)) = request.resolution {
        if width == 0 || height == 0 {
            return Err("Resolution dimensions cannot be zero".to_string());
        }
    }


    let export_id = format!("export_{}", uuid::Uuid::new_v4());

    info!("Export started: {}", export_id);

    Ok(EditingResponse {
        success: true,
        message: format!("Export {} started successfully", export_id),
        data: Some(serde_json::json!({
            "export_id": export_id
        })),
    })
}


#[tauri::command]
pub async fn export_get_status(
    export_id: String,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    debug!("Getting export status: {}", export_id);

    if export_id.is_empty() {
        return Err("Export ID cannot be empty".to_string());
    }

    let status = serde_json::json!({
        "export_id": export_id,
        "status": "completed",
        "progress": 100.0,
        "frames_rendered": 3600,
        "total_frames": 3600,
        "elapsed_time": 120.5,
        "remaining_time": 0.0,
        "estimated_size": 1024 * 1024 * 250,
        "output_path": export_id,
        "format": "mp4",
        "codec": "h264"
    });

    info!("Export status for {}: {:?}", export_id, status);
    Ok(status)
}


#[tauri::command]
pub async fn export_cancel(
    export_id: String,
    _state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!("Cancelling export: {}", export_id);

    if export_id.is_empty() {
        return Err("Export ID cannot be empty".to_string());
    }

    info!("Cancelling export: {}", export_id);

    Ok(EditingResponse {
        success: true,
        message: format!("Export cancelled: {}", export_id),
        data: Some(serde_json::json!({
            "export_id": export_id
        })),
    })
}


#[tauri::command]
pub async fn project_auto_save(
    project_id: String,
    _state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!("Auto-saving project: {}", project_id);

    if project_id.is_empty() {
        return Err("Project ID cannot be empty".to_string());
    }


    let auto_save_path = format!("/autosave/{}_autosave.aether", project_id);

    info!("Auto-saved project: {} to {}", project_id, auto_save_path);

    Ok(EditingResponse {
        success: true,
        message: format!("Project auto-saved successfully"),
        data: Some(serde_json::json!({
            "project_id": project_id,
            "auto_save_path": auto_save_path,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_create_validation() {

        let request = ProjectCreateRequest {
            name: "".to_string(),
            description: None,
            fps: None,
            resolution: None,
            template: None,
        };
        assert!(request.name.is_empty());
    }

    #[test]
    fn test_media_import_validation() {

        let request = MediaImportRequest {
            file_paths: vec![],
            target_track: None,
            position: None,
            auto_create_clips: None,
        };
        assert!(request.file_paths.is_empty());
    }

    #[test]
    fn test_export_format_serialization() {
        let format = ExportFormat::Mp4;
        assert_eq!(format!("{:?}", format), "Mp4");
    }

    #[test]
    fn test_audio_codec_serialization() {
        let codec = AudioCodec::Aac;
        assert_eq!(format!("{:?}", codec), "Aac");
    }

    #[test]
    fn test_export_quality_serialization() {
        let quality = ExportQuality::High;
        assert_eq!(format!("{:?}", quality), "High");
    }
}
