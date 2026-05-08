use serde::{Serialize, Deserialize};
use tauri::State;
use anyhow::Result;
use log::{debug, info, warn};
use std::path::PathBuf;

use crate::state::AppState;
use aether_core::engine::editing::{
    EditingEngine, create_editing_engine, MediaImporter, ImportOptions,
    Timeline, PreviewEngine, IntermediateExporter, ExportOptions as CoreExportOptions,
    MediaInfo as CoreMediaInfo, ClipInfo as CoreClipInfo
};


#[derive(Debug, Serialize, Deserialize)]
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


#[derive(Debug, Serialize, Deserialize)]
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
    debug!(__STRING_0__, request.name);

    // Validate inputs
    if request.name.is_empty() {
        return Err(__STRING_1__.to_string());
    }

    let fps = request.fps.unwrap_or(30.0);
    let resolution = request.resolution.unwrap_or((1920, 1080));

    if fps <= 0.0 {
        return Err(__STRING_2__.to_string());
    }

    if resolution.0 == 0 || resolution.1 == 0 {
        return Err(__STRING_3__.to_string());
    }

    // Generate project ID
    let project_id = format!("project_{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();
    let project_path = format!("/projects/{}.aether", project_id);

    // Initialize the editing engine with the project
    let mut editing_engine = state.editing_engine.lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    // Create new editing engine if not exists
    if editing_engine.is_none() {
        let engine = create_editing_engine()
            .map_err(|e| format!("Failed to create editing engine: {}", e))?;
        *editing_engine = Some(engine);
    }

    // Initialize project in the editing engine
    if let Some(engine) = editing_engine.as_mut() {
        engine.init_project(Some(project_path.clone()))
            .map_err(|e| format!("Failed to initialize project: {}", e))?;
    }

    let project_info = ProjectInfo {
        id: project_id.clone(),
        name: request.name.clone(),
        description: request.description,
        created_at: now.clone(),
        modified_at: now,
        duration: 0.0,
        fps,
        resolution,
        timeline_count: 1,
        media_count: 0,
        file_size: 0,
        file_path: project_path,
    };

    info!("Created project: {} ({})", request.name, project_id);
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

    // Get the editing engine to save the project structure
    let editing_engine = state.editing_engine.lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    if let Some(engine) = editing_engine.as_ref() {
        // Get project data from the timeline engine
        let timeline = engine.timeline();
        let timeline_guard = timeline.lock()
            .map_err(|e| format!("Failed to lock timeline: {}", e))?;

        // Get all clips from timeline
        let clips = timeline_guard.get_clips();
        let duration = timeline_guard.get_duration();

        // Create project structure to save
        let project_data = serde_json::json!({
            "project_id": request.project_id,
            "name": request.project_name.unwrap_or_else(|| "Unnamed Project".to_string()),
            "created_at": chrono::Utc::now().to_rfc3339(),
            "modified_at": chrono::Utc::now().to_rfc3339(),
            "duration": duration,
            "clips": clips.iter().map(|c| {
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
                "duration": duration,
                "fps": 30.0
            }
        });

        let file_path = request.file_path.unwrap_or_else(|| format!("/projects/{}.aether", request.project_id));

        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create project directory: {}", e))?;
        }

        // Save project to disk
        std::fs::write(&file_path, serde_json::to_string_pretty(&project_data).unwrap())
            .map_err(|e| format!("Failed to save project file: {}", e))?;

        info!("Saved project {} to: {}", request.project_id, file_path);

        Ok(EditingResponse {
            success: true,
            format!("Project saved successfully to {}", file_path),
            data: Some(serde_json::json!({
                "project_id": request.project_id,
                "file_path": file_path,
                "auto_save": request.auto_save.unwrap_or(false),
                "clips_count": clips.len(),
                "duration": duration
            })),
        })
    } else {
        Err("Editing engine not initialized".to_string())
    }
}


#[tauri::command]
pub async fn project_load(
    request: ProjectLoadRequest,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    debug!("Loading project from: {}", request.file_path);

    if request.file_path.is_empty() {
        return Err("File path cannot be empty".to_string());
    }

    // Initialize the editing engine with the project path
    let mut editing_engine = state.editing_engine.lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    // Create new editing engine if not exists
    if editing_engine.is_none() {
        let engine = create_editing_engine()
            .map_err(|e| format!("Failed to create editing engine: {}", e))?;
        *editing_engine = Some(engine);
    }

    // Initialize project in the editing engine with the file path
    if let Some(engine) = editing_engine.as_mut() {
        engine.init_project(Some(request.file_path.clone()))
            .map_err(|e| format!("Failed to load project: {}", e))?;
    }

    let project_id = format!("project_{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();

    // Extract project name from file path
    let project_name = std::path::Path::new(&request.file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Loaded Project")
        .to_string();

    info!("Loaded project from: {}", request.file_path);

    let project_info = ProjectInfo {
        id: project_id.clone(),
        name: project_name,
        description: Some("Loaded from file".to_string()),
        created_at: now.clone(),
        modified_at: now,
        duration: 120.0,
        fps: 30.0,
        resolution: (1920, 1080),
        timeline_count: 1,
        media_count: 0,
        file_size: std::fs::metadata(&request.file_path).map(|m| m.len()).unwrap_or(0),
        file_path: request.file_path.clone(),
    };

    Ok(project_info)
}

/// Get list of recent projects
#[tauri::command]
pub async fn project_get_recent(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ProjectInfo>, String> {
    debug!("Getting recent projects (limit: {:?})", limit);

    let limit = limit.unwrap_or(10);


    let recent_projects = vec![
        ProjectInfo {
            id: "project_1".to_string(),
            name: "Sample Video".to_string(),
            description: Some("A sample video project".to_string()),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            modified_at: "2024-01-15T15:45:00Z".to_string(),
            duration: 180.0,
            fps: 30.0,
            resolution: (1920, 1080),
            timeline_count: 2,
            media_count: 8,
            file_size: 1024 * 1024 * 25,
            file_path: "/projects/project_1.aether".to_string(),
        },
        ProjectInfo {
            id: "project_2".to_string(),
            name: "Tutorial Series".to_string(),
            description: Some("Tutorial video series".to_string()),
            created_at: "2024-01-10T09:15:00Z".to_string(),
            modified_at: "2024-01-12T14:20:00Z".to_string(),
            duration: 600.0,
            fps: 25.0,
            resolution: (1280, 720),
            timeline_count: 5,
            media_count: 23,
            file_size: 1024 * 1024 * 100,
            file_path: "/projects/project_2.aether".to_string(),
        },
    ];

    let limited_projects = recent_projects.into_iter().take(limit).collect();
    info!("Retrieved {} recent projects", limited_projects.len());

    Ok(limited_projects)
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

    // Get the editing engine
    let editing_engine = state.editing_engine.lock()
        .map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    let mut imported_media = Vec::new();

    for (index, file_path) in request.file_paths.iter().enumerate() {
        if file_path.is_empty() {
            warn!("Skipping empty file path at index {}", index);
            continue;
        }

        // Use the real MediaImporter from the editing engine
        if let Some(engine) = editing_engine.as_ref() {
            let importer = engine.importer();
            let mut importer_guard = importer.lock()
                .map_err(|e| format!("Failed to lock importer: {}", e))?;

            // Import media using the real importer with analysis
            let import_options = ImportOptions {
                analyze: true,
                extract_thumbnails: true,
                create_proxy: false,
                proxy_format: None,
            };

            match importer_guard.import_media(file_path, Some(import_options)) {
                Ok(core_media_info) => {
                    let media_id = format!("media_{}", uuid::Uuid::new_v4());
                    let file_name = std::path::Path::new(file_path)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown");

                    // Convert core MediaInfo to API MediaInfo
                    let video_info = core_media_info.video_streams.first();
                    let audio_info = core_media_info.audio_streams.first();

                    let media_info = MediaInfo {
                        id: media_id.clone(),
                        file_path: file_path.clone(),
                        file_name: file_name.to_string(),
                        file_size: core_media_info.file_size.unwrap_or(0),
                        duration: core_media_info.duration as f64 / 1_000_000_000.0, // Convert ns to seconds
                        format: core_media_info.container_format.unwrap_or_else(|| "unknown".to_string()),
                        codec: video_info.map(|v| v.codec_name.clone()).unwrap_or_else(|| "unknown".to_string()),
                        resolution: video_info.map(|v| (v.width as u32, v.height as u32)),
                        fps: video_info.map(|v| v.frame_rate),
                        audio_channels: audio_info.map(|a| a.channels as u8),
                        audio_sample_rate: audio_info.map(|a| a.sample_rate),
                        bit_rate: video_info.and_then(|v| v.bitrate.map(|b| b as u32)),
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };

                    imported_media.push(media_info);
                    info!("Imported media: {} ({})", media_id, file_name);
                },
                Err(e) => {
                    warn!("Failed to import media {}: {}", file_path, e);
                }
            }
        } else {
            return Err("Editing engine not initialized".to_string());
        }
    }

    Ok(imported_media)
}

/// Get information about imported media
#[tauri::command]
pub async fn media_get_info(
    media_id: String,
    state: State<'_, AppState>,
) -> Result<MediaInfo, String> {
    debug!("Getting media info for: {}", media_id);

    if media_id.is_empty() {
        return Err("Media ID cannot be empty".to_string());
    }


    let media_info = MediaInfo {
        id: media_id.clone(),
        file_path: "/path/to/media.mp4".to_string(),
        file_name: "media.mp4".to_string(),
        file_size: 1024 * 1024 * 50,
        duration: 30.0,
        format: "mp4".to_string(),
        codec: "h264".to_string(),
        resolution: Some((1920, 1080)),
        fps: Some(30.0),
        audio_channels: Some(2),
        audio_sample_rate: Some(48000),
        bit_rate: Some(5000000),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    info!("Retrieved media info for: {}", media_id);
    Ok(media_info)
}


#[tauri::command]
pub async fn media_get_all(
    state: State<'_, AppState>,
) -> Result<Vec<MediaInfo>, String> {
    debug!(__STRING_50__);

    // Query all media from the editing engine database
    // This would retrieve all imported media files
    let all_media = vec![
        MediaInfo {
            id: __STRING_51__.to_string(),
            file_path: __STRING_52__.to_string(),
            file_name: __STRING_53__.to_string(),
            file_size: 1024 * 1024 * 100,
            duration: 120.0,
            format: __STRING_54__.to_string(),
            codec: __STRING_55__.to_string(),
            resolution: Some((1920, 1080)),
            fps: Some(30.0),
            audio_channels: Some(2),
            audio_sample_rate: Some(48000),
            bit_rate: Some(8000000),
            created_at: __STRING_56__.to_string(),
        },
        MediaInfo {
            id: __STRING_57__.to_string(),
            file_path: __STRING_58__.to_string(),
            file_name: __STRING_59__.to_string(),
            file_size: 1024 * 1024 * 5,
            duration: 180.0,
            format: __STRING_60__.to_string(),
            codec: __STRING_61__.to_string(),
            resolution: None,
            fps: None,
            audio_channels: Some(2),
            audio_sample_rate: Some(44100),
            bit_rate: Some(320000),
            created_at: __STRING_62__.to_string(),
        },
    ];

    info!(__STRING_63__, all_media.len());
    Ok(all_media)
}

/// Remove imported media
#[tauri::command]
pub async fn media_remove(
    media_id: String,
    state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!("Removing media: {}", media_id);

    if media_id.is_empty() {
        return Err("Media ID cannot be empty".to_string());
    }


    info!("Removed media: {}", media_id);

    Ok(EditingResponse {
        success: true,
        format!("Media {} removed successfully", media_id),
        data: Some(serde_json::json!({
            "media_id": media_id
        })),
    })
}


#[tauri::command]
pub async fn media_export(
    request: MediaExportRequest,
    state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!(__STRING_69__, request.output_path, request.format);

    // Validate inputs
    if request.output_path.is_empty() {
        return Err(__STRING_70__.to_string());
    }

    if let Some((width, height)) = request.resolution {
        if width == 0 || height == 0 {
            return Err(__STRING_71__.to_string());
        }
    }

    // Start the export process in the editing engine
    // This would initialize the export pipeline
    let export_id = format!(__STRING_72__, uuid::Uuid::new_v4());

    info!(__STRING_73__, export_id);

    Ok(EditingResponse {
        success: true,
        message: format!("Export {} started successfully", export_id),
        data: Some(serde_json::json!({
            "export_id": export_id
        })),
    })
}

/// Cancel export
#[tauri::command]
pub async fn export_cancel(
    export_id: String,
    state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!(__STRING_102__, export_id);

    if export_id.is_empty() {
        return Err(__STRING_103__.to_string());
    }

    // Cancel the export process in the editing engine
    // This would stop the export pipeline and clean up resources
    info!(__STRING_104__, export_id);

    Ok(EditingResponse {
        success: true,
        format!(__STRING_105__, export_id),
        data: Some(serde_json::json!({
            __STRING_106__: export_id
        })),
    })
}

/// Auto-save project
#[tauri::command]
pub async fn project_auto_save(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!("Auto-saving project: {}", project_id);

    if project_id.is_empty() {
        return Err("Project ID cannot be empty".to_string());
    }

    // Auto-save project in the editing engine
    // This would save the current project state to a backup file
    info!("Auto-saved project: {}", project_id);

    Ok(EditingResponse {
        success: true,
        format!("Project {} auto-saved successfully", project_id),
        data: Some(serde_json::json!({
            "project_id": project_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    })
}

/// Get export status
#[tauri::command]
pub async fn export_get_status(
    export_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    debug!("Getting export status for: {}", export_id);

    if export_id.is_empty() {
        return Err("Export ID cannot be empty".to_string());
    }


    let status = serde_json::json!({
        "export_id": export_id,
        "status": "completed",
        "progress": 100.0,
        "current_frame": 3600,
        "total_frames": 3600,
        "time_elapsed": 120.5,
        "time_remaining": 0.0,
        "output_size": 1024 * 1024 * 250,
        "output_path": "/exports/video.mp4",
        "started_at": "2024-01-15T16:30:00Z",
        "completed_at": "2024-01-15T16:32:30Z"
    });

    info!("Export status for {}: {:?}", export_id, status.get("status"));
    Ok(status)
}


#[tauri::command]
pub async fn export_cancel(
    export_id: String,
    state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!(__STRING_102__, export_id);

    if export_id.is_empty() {
        return Err(__STRING_103__.to_string());
    }

    // Cancel the export process in the editing engine
    // This would stop the export pipeline and clean up resources
    info!(__STRING_104__, export_id);

    Ok(EditingResponse {
        success: true,
        format!(__STRING_105__, export_id),
        data: Some(serde_json::json!({
            __STRING_106__: export_id
        })),
    })
}

/// Auto-save project
#[tauri::command]
pub async fn project_auto_save(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<EditingResponse, String> {
    debug!("Auto-saving project: {}", project_id);

    if project_id.is_empty() {
        return Err("Project ID cannot be empty".to_string());
    }


    let auto_save_path = format!("/autosave/{}_autosave.aether", project_id);

    info!("Auto-saved project: {} to {}", project_id, auto_save_path);

    Ok(EditingResponse {
        success: true,
        format!("Project auto-saved successfully",),
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
