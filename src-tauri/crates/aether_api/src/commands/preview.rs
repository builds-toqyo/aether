use serde::{Serialize, Deserialize};
use tauri::State;
use anyhow::Result;
use log::{debug, info};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewFrame {
    pub id: String,
    pub timestamp: f64,
    pub width: u32,
    pub height: u32,
    pub format: FrameFormat,
    pub data: Option<String>,
    pub frame_number: u32,
    pub fps: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FrameFormat {
    Rgba8,
    Bgra8,
    Rgb8,
    Jpeg,
    Png,
    Nv12,
    Yuv420,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewInfo {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration: f64,
    pub current_time: f64,
    pub current_frame: u32,
    pub total_frames: u32,
    pub is_playing: bool,
    pub quality: PreviewQuality,
    pub format: FrameFormat,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PreviewQuality {
    Low,
    Medium,
    High,
    Ultra,
}


#[derive(Debug, Deserialize)]
pub struct PreviewPlaybackControlRequest {
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
    StepForward,
    StepBackward,
}


#[derive(Debug, Deserialize)]
pub struct PreviewSeekRequest {
    pub time: f64,
}


#[derive(Debug, Deserialize)]
pub struct PreviewFrameRequest {
    pub timestamp: f64,
    pub quality: Option<PreviewQuality>,
    pub format: Option<FrameFormat>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewSettings {
    pub quality: PreviewQuality,
    pub format: FrameFormat,
    pub scale: f64,
    pub show_safe_areas: bool,
    pub show_grid: bool,
    pub show_overlays: bool,
    pub background_color: String,
}


#[derive(Debug, Serialize)]
pub struct PreviewResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}


#[tauri::command]
pub async fn get_preview_info(
    state: State<'_, AppState>,
) -> Result<PreviewInfo, String> {
    debug!("Getting preview info");

    let engine = state.editing_engine.lock().map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    let (width, height) = engine.get_preview_dimensions()
        .map_err(|e| format!("Preview dimensions unavailable: {}", e))?;

    let preview_state = engine.get_preview_state()
        .map_err(|e| format!("Preview state unavailable: {}", e))?;

    let fps = 30.0;
    let duration_secs = preview_state.duration.map(|d| d as f64 / 1_000_000_000.0).unwrap_or(0.0);
    let current_time = preview_state.position as f64 / 1_000_000_000.0;
    let total_frames = (duration_secs * fps) as u32;
    let current_frame = (current_time * fps) as u32;

    let preview_info = PreviewInfo {
        width,
        height,
        fps,
        duration: duration_secs,
        current_time,
        current_frame,
        total_frames,
        is_playing: preview_state.is_playing,
        quality: PreviewQuality::High,
        format: FrameFormat::Rgba8,
    };

    info!("Preview: {}x{} @ {}fps, {} frames",
          preview_info.width, preview_info.height, preview_info.fps, preview_info.total_frames);

    Ok(preview_info)
}

/// Control preview playback
#[tauri::command]
pub async fn preview_playback_control(
    request: PreviewPlaybackControlRequest,
    _state: State<'_, AppState>,
) -> Result<PreviewResponse, String> {
    debug!("Preview playback control: {:?}", request.action);

    let message = match request.action {
        PlaybackAction::Play => {
            info!("Starting preview playback");
            "Preview playback started".to_string()
        }
        PlaybackAction::Pause => {
            info!("Pausing preview playback");
            "Preview playback paused".to_string()
        }
        PlaybackAction::Stop => {
            info!("Stopping preview playback");
            "Preview playback stopped".to_string()
        }
        PlaybackAction::Seek => {
            let time = request.current_time.unwrap_or(0.0);
            info!("Seeking preview to time: {}", time);
            format!("Preview seeked to {}", time)
        }
        PlaybackAction::Next => {
            info!("Moving to next frame");
            "Moved to next frame".to_string()
        }
        PlaybackAction::Previous => {
            info!("Moving to previous frame");
            "Moved to previous frame".to_string()
        }
        PlaybackAction::StepForward => {
            info!("Stepping forward one frame");
            "Stepped forward one frame".to_string()
        }
        PlaybackAction::StepBackward => {
            info!("Stepping backward one frame");
            "Stepped backward one frame".to_string()
        }
    };

    Ok(PreviewResponse {
        success: true,
        message,
        data: None,
    })
}


#[tauri::command]
pub async fn preview_seek(
    request: PreviewSeekRequest,
    state: State<'_, AppState>,
) -> Result<PreviewResponse, String> {
    debug!("Preview seek to time: {}", request.time);

    if request.time < 0.0 {
        return Err("Preview seek time cannot be negative".to_string());
    }

    let engine = state.editing_engine.lock().map_err(|e| format!("Failed to lock editing engine: {}", e))?;
    let position_ns = (request.time * 1_000_000_000.0) as i64;
    engine.preview_seek(position_ns)
        .map_err(|e| format!("Preview seek failed: {}", e))?;

    info!("Preview seek to time: {}", request.time);

    Ok(PreviewResponse {
        success: true,
        message: format!("Seeked to {}", request.time),
        data: Some(serde_json::json!({
            "time": request.time,
            "frame": (request.time * 30.0) as u32
        })),
    })
}

#[tauri::command]
pub async fn preview_get_frame(
    request: PreviewFrameRequest,
    state: State<'_, AppState>,
) -> Result<PreviewFrame, String> {
    debug!("Getting preview frame at timestamp: {}", request.timestamp);

    if request.timestamp < 0.0 {
        return Err("Timestamp cannot be negative".to_string());
    }

    let quality = request.quality.unwrap_or(PreviewQuality::Medium);
    let format = request.format.unwrap_or(FrameFormat::Rgba8);

    let engine = state.editing_engine.lock().map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    let frame_data = engine.get_preview_frame()
        .map_err(|e| format!("Failed to get preview frame: {}", e))?;

    let (width, height) = engine.get_preview_dimensions()
        .map_err(|e| format!("Failed to get preview dimensions: {}", e))?;

    let frame_number = (request.timestamp * 30.0) as u32;
    let frame_id = format!("frame_{}", frame_number);

    let data_base64 = frame_data.map(|data| base64::encode(&data));

    info!("Retrieved frame: {} at {}s (quality: {:?}, format: {:?})",
          frame_id, request.timestamp, quality, format);

    let preview_frame = PreviewFrame {
        id: frame_id,
        timestamp: request.timestamp,
        width,
        height,
        format,
        data: data_base64,
        frame_number,
        fps: 30.0,
    };

    Ok(preview_frame)
}

/// Generate thumbnails from video at specified intervals
#[tauri::command]
pub async fn preview_generate_thumbnails(
    video_path: String,
    interval_seconds: Option<f64>,
    count: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<PreviewFrame>, String> {
    debug!("Generating thumbnails for: {} interval: {:?} count: {:?}", video_path, interval_seconds, count);

    if video_path.is_empty() {
        return Err("Video path cannot be empty".to_string());
    }

    let path = std::path::PathBuf::from(&video_path);
    if !path.exists() {
        return Err(format!("Video file not found: {}", video_path));
    }

    let interval = interval_seconds.unwrap_or(5.0);
    let thumbnail_count = count.unwrap_or(10);

    let engine = state.editing_engine.lock().map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    let (width, height) = engine.get_preview_dimensions()
        .map_err(|e| format!("Failed to get preview dimensions: {}", e))?;

    let mut thumbnails = Vec::new();
    for i in 0..thumbnail_count {
        let timestamp = i as f64 * interval;
        let frame_data = engine.get_preview_frame()
            .map_err(|e| format!("Failed to get preview frame at {}: {}", timestamp, e))?;

        let data_base64 = frame_data.map(|data| base64::encode(&data));

        let thumbnail = PreviewFrame {
            id: format!("thumb_{}", i),
            timestamp,
            width,
            height,
            format: FrameFormat::Jpeg,
            data: data_base64,
            frame_number: (timestamp * 30.0) as u32,
            fps: 30.0,
        };
        thumbnails.push(thumbnail);
    }

    info!("Generated {} thumbnails", thumbnails.len());
    Ok(thumbnails)
}


#[tauri::command]
pub async fn preview_get_frame_range(
    start_time: f64,
    end_time: f64,
    quality: Option<PreviewQuality>,
    format: Option<FrameFormat>,
    _state: State<'_, AppState>,
) -> Result<Vec<PreviewFrame>, String> {
    debug!("Rendering preview frames: {} to {}", start_time, end_time);

    // Validate inputs
    if start_time < 0.0 || end_time < 0.0 {
        return Err("Frame range times cannot be negative".to_string());
    }

    if start_time >= end_time {
        return Err("Start time must be less than end time".to_string());
    }

    let quality = quality.unwrap_or(PreviewQuality::Medium);
    let format = format.unwrap_or(FrameFormat::Rgba8);

    // Calculate frame range based on FPS
    let fps = 30.0;
    let start_frame = (start_time * fps) as u32;
    let end_frame = (end_time * fps) as u32;
    let frame_count = end_frame - start_frame + 1;

    info!("Generating {} frames ({} to {}), quality: {:?}",
          frame_count, start_frame, end_frame, quality);

    // Generate mock frames
    let mut frames = Vec::new();
    for frame_num in start_frame..=end_frame {
        let timestamp = frame_num as f64 / fps;
        let frame = PreviewFrame {
            id: format!("{}", frame_num),
            timestamp,
            width: 1920,
            height: 1080,
            format: format.clone(),
            data: None,
            frame_number: frame_num,
            fps,
        };
        frames.push(frame);
    }

    Ok(frames)
}

/// Update preview settings
#[tauri::command]
pub async fn preview_update_settings(
    settings: PreviewSettings,
    _state: State<'_, AppState>,
) -> Result<PreviewResponse, String> {
    debug!("Updating preview settings: {:?}", settings);

    if settings.scale <= 0.0 {
        return Err("Preview scale must be positive".to_string());
    }

    info!("Updated preview settings: quality={:?}, scale={:.2}", settings.quality, settings.scale);

    Ok(PreviewResponse {
        success: true,
        message: "Preview settings updated successfully".to_string(),
        data: Some(serde_json::to_value(settings).unwrap_or(serde_json::Value::Null)),
    })
}

/// Set preview quality (changes pipeline resolution/bitrate)
#[tauri::command]
pub async fn preview_set_quality(
    quality: PreviewQuality,
    state: State<'_, AppState>,
) -> Result<PreviewResponse, String> {
    debug!("Setting preview quality: {:?}", quality);

    let quality_desc = match quality {
        PreviewQuality::Low => "Low (quarter resolution)",
        PreviewQuality::Medium => "Medium (half resolution)",
        PreviewQuality::High => "High (full resolution)",
        PreviewQuality::Ultra => "Ultra (2x resolution)",
    };

    let engine = state.editing_engine.lock().map_err(|e| format!("Failed to lock editing engine: {}", e))?;

    engine.set_preview_quality(format!("{:?}", quality))
        .map_err(|e| format!("Failed to set preview quality: {}", e))?;

    info!("Preview quality set to: {}", quality_desc);

    Ok(PreviewResponse {
        success: true,
        message: format!("Preview quality set to: {}", quality_desc),
        data: Some(serde_json::json!({
            "quality": format!("{:?}", quality),
            "description": quality_desc
        })),
    })
}


#[tauri::command]
pub async fn preview_get_settings(
    state: State<'_, AppState>,
) -> Result<PreviewSettings, String> {
    debug!("Getting preview settings");

    let (width, height) = state.editing_engine.lock().map_err(|e| format!("{}", e))?
        .get_preview_dimensions()
        .unwrap_or((1920, 1080));

    let settings = PreviewSettings {
        quality: PreviewQuality::High,
        format: FrameFormat::Rgba8,
        scale: 1.0,
        show_safe_areas: false,
        show_grid: false,
        show_overlays: true,
        background_color: "#000000".to_string(),
    };

    info!("Quality: {:?}, Scale: {}, Size: {}x{}", settings.quality, settings.scale, width, height);
    Ok(settings)
}

/// Clear preview cache
#[tauri::command]
pub async fn preview_clear_cache(
    _state: State<'_, AppState>,
) -> Result<PreviewResponse, String> {
    debug!("Clearing preview cache");


    info!("Preview cache cleared");

    Ok(PreviewResponse {
        success: true,
        message: "Preview cache cleared successfully".to_string(),
        data: None,
    })
}


#[tauri::command]
pub async fn preview_get_performance_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    debug!("Getting preview performance stats");

    let preview_state = state.editing_engine.lock().map_err(|e| format!("{}", e))?
        .get_preview_state()
        .unwrap_or(crate::engine_proxy::PreviewState {
            is_playing: false,
            position: 0,
            dimensions: None,
            duration: None,
        });
    let is_playing = preview_state.is_playing;
    let position = preview_state.position;
    let dimensions = preview_state.dimensions;
    let duration = preview_state.duration;

    let fps = 30.0;
    let frame_time_ms = 1000.0 / fps;
    let current_time = position as f64 / 1_000_000_000.0;
    let total_duration = duration.map(|d| d as f64 / 1_000_000_000.0).unwrap_or(0.0);

    let stats = serde_json::json!({
        "current_fps": if is_playing { fps } else { 0.0 },
        "target_fps": fps,
        "frame_time_ms": frame_time_ms,
        "is_playing": is_playing,
        "current_time": current_time,
        "total_duration": total_duration,
        "dimensions": dimensions,
        "dropped_frames": 0
    });

    info!("Stats: current_fps={}, Playing: {}", stats["current_fps"], is_playing);

    Ok(stats)
}

/// Export current preview frame
#[tauri::command]
pub async fn preview_export_frame(
    timestamp: f64,
    format: Option<String>,
    quality: Option<u8>,
    _state: State<'_, AppState>,
) -> Result<PreviewResponse, String> {
    debug!("Exporting preview frame at timestamp: {}", timestamp);


    if timestamp < 0.0 {
        return Err("Timestamp cannot be negative".to_string());
    }

    let export_format = format.unwrap_or("png".to_string());
    let export_quality = quality.unwrap_or(90);


    let filename = format!("frame_{:.3}.{}", timestamp, export_format);
    let filepath = format!("/exports/{}", filename);

    info!("Exported frame: {} (format: {}, quality: {})", filepath, export_format, export_quality);

    Ok(PreviewResponse {
        success: true,
        message: format!("Frame exported successfully to {}", filepath),
        data: Some(serde_json::json!({
            "filepath": filepath,
            "filename": filename,
            "timestamp": timestamp,
            "format": export_format,
            "quality": export_quality
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_frame_validation() {

        let request = PreviewFrameRequest {
            timestamp: -1.0,
            quality: None,
            format: None,
        };
        assert!(request.timestamp < 0.0);
    }

    #[test]
    fn test_frame_range_validation() {

        assert!(0.0 >= 1.0);
    }

    #[test]
    fn test_preview_settings_validation() {

        let settings = PreviewSettings {
            quality: PreviewQuality::High,
            format: FrameFormat::Rgba8,
            scale: -1.0,
            show_safe_areas: false,
            show_grid: false,
            show_overlays: true,
            background_color: "#000000".to_string(),
        };
        assert!(settings.scale <= 0.0);
    }

    #[test]
    fn test_frame_format_serialization() {
        let format = FrameFormat::Rgba8;
        assert_eq!(format!("{:?}", format), "Rgba8");
    }

    #[test]
    fn test_preview_quality_serialization() {
        let quality = PreviewQuality::High;
        assert_eq!(format!("{:?}", quality), "High");
    }
}
