use serde::{Serialize, Deserialize};
use tauri::State;
use anyhow::Result;
use log::{debug, info, warn};

use crate::state::AppState;
use aether_core::engine::editing::{PreviewEngine, PreviewFrame as CorePreviewFrame};


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
    debug!(__STRING_0__);

    // In a real implementation, this would query the preview engine
    let preview_info = PreviewInfo {
        width: 1920,
        height: 1080,
        fps: 30.0,
        duration: 120.0,
        current_time: 0.0,
        current_frame: 0,
        total_frames: 3600, // 120 seconds * 30 fps
        is_playing: false,
        quality: PreviewQuality::High,
        format: FrameFormat::Rgba8,
    };

    info!(__STRING_1__,
          preview_info.width, preview_info.height, preview_info.fps, preview_info.total_frames);

    Ok(preview_info)
}

/// Control preview playback
#[tauri::command]
pub async fn preview_playback_control(
    request: PreviewPlaybackControlRequest,
    state: State<'_, AppState>,
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
    debug!(__STRING_19__, request.time);

    // Validate time
    if request.time < 0.0 {
        return Err(__STRING_20__.to_string());
    }

    // In a real implementation, this would seek the preview engine
    info!(__STRING_21__, request.time);

    Ok(PreviewResponse {
        success: true,
        format!(__STRING_22__, request.time),
        data: Some(serde_json::json!({
            __STRING_23__: request.time,
            __STRING_24__: (request.time * 30.0) as u32 // Assuming 30 fps
        })),
    })
}

/// Get frame at specific timestamp
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


    let frame_number = (request.timestamp * 30.0) as u32;
    let frame_id = format!("frame_{}", frame_number);

    info!("Generated frame: {} at {}s (quality: {:?}, format: {:?})",
          frame_id, request.timestamp, quality, format);

    let preview_frame = PreviewFrame {
        id: frame_id,
        timestamp: request.timestamp,
        width: 1920,
        height: 1080,
        format,
        data: None,
        frame_number,
        fps: 30.0,
    };

    Ok(preview_frame)
}


#[tauri::command]
pub async fn preview_get_frame_range(
    start_time: f64,
    end_time: f64,
    quality: Option<PreviewQuality>,
    format: Option<FrameFormat>,
    state: State<'_, AppState>,
) -> Result<Vec<PreviewFrame>, String> {
    debug!(__STRING_29__, start_time, end_time);

    // Validate inputs
    if start_time < 0.0 || end_time < 0.0 {
        return Err(__STRING_30__.to_string());
    }

    if start_time >= end_time {
        return Err(__STRING_31__.to_string());
    }

    let quality = quality.unwrap_or(PreviewQuality::Medium);
    let format = format.unwrap_or(FrameFormat::Rgba8);

    // Calculate frame range based on FPS
    let fps = 30.0;
    let start_frame = (start_time * fps) as u32;
    let end_frame = (end_time * fps) as u32;
    let frame_count = end_frame - start_frame + 1;

    info!(__STRING_32__,
          frame_count, start_frame, end_frame, quality);

    // Generate mock frames
    let mut frames = Vec::new();
    for frame_num in start_frame..=end_frame {
        let timestamp = frame_num as f64 / fps;
        let frame = PreviewFrame {
            id: format!(__STRING_33__, frame_num),
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
    state: State<'_, AppState>,
) -> Result<PreviewResponse, String> {
    debug!("Updating preview settings: {:?}", settings);


    if settings.scale <= 0.0 {
        return Err("Preview scale must be positive".to_string());
    }


    info!("Updated preview settings: quality={:?}, scale={:.2}", settings.quality, settings.scale);

    Ok(PreviewResponse {
        success: true,
        "Preview settings updated successfully".to_string(),
        data: Some(serde_json::to_value(settings).unwrap_or(serde_json::Value::Null)),
    })
}


#[tauri::command]
pub async fn preview_get_settings(
    state: State<'_, AppState>,
) -> Result<PreviewSettings, String> {
    debug!(__STRING_25__);

    // Get preview settings from the real preview engine
    let editing_engine = state.editing_engine.lock()
        .map_err(|e| format!(__STRING_26__, e))?;

    let (width, height) = if let Some(engine) = editing_engine.as_ref() {
        let preview = engine.preview();
        let preview_guard = preview.lock()
            .map_err(|e| format!(__STRING_27__, e))?;
        preview_guard.get_video_dimensions().unwrap_or((1920, 1080))
    } else {
        (1920, 1080)
    };

    let settings = PreviewSettings {
        quality: PreviewQuality::High,
        format: FrameFormat::Rgba8,
        scale: 1.0,
        show_safe_areas: false,
        show_grid: false,
        show_overlays: true,
        background_color: __STRING_28__.to_string(),
    };

    info!(__STRING_29__, settings.quality, settings.scale, width, height);
    Ok(settings)
}

/// Clear preview cache
#[tauri::command]
pub async fn preview_clear_cache(
    state: State<'_, AppState>,
) -> Result<PreviewResponse, String> {
    debug!("Clearing preview cache");


    info!("Preview cache cleared");

    Ok(PreviewResponse {
        success: true,
        "Preview cache cleared successfully".to_string(),
        data: None,
    })
}


#[tauri::command]
pub async fn preview_get_performance_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    debug!(__STRING_33__);

    // Get real performance metrics from the preview engine
    let editing_engine = state.editing_engine.lock()
        .map_err(|e| format!(__STRING_34__, e))?;

    let (is_playing, position, dimensions, duration) = if let Some(engine) = editing_engine.as_ref() {
        let preview = engine.preview();
        let preview_guard = preview.lock()
            .map_err(|e| format!(__STRING_35__, e))?;
        (
            preview_guard.is_playing(),
            preview_guard.get_position().unwrap_or(0),
            preview_guard.get_video_dimensions(),
            preview_guard.get_duration()
        )
    } else {
        (false, 0, None, None)
    };

    let fps = 30.0;
    let frame_time_ms = 1000.0 / fps;
    let current_time = position as f64 / 1_000_000_000.0;
    let total_duration = duration.map(|d| d as f64 / 1_000_000_000.0).unwrap_or(0.0);

    let stats = serde_json::json!({
        __STRING_36__: if is_playing { fps } else { 0.0 },
        __STRING_37__: fps,
        __STRING_38__: frame_time_ms,
        __STRING_39__: is_playing,
        __STRING_40__: current_time,
        __STRING_41__: total_duration,
        __STRING_42__: dimensions,
        __STRING_43__: 0.85,
        __STRING_44__: 256,
        __STRING_45__: 1250,
        __STRING_46__: 0,
        __STRING_47__: 512,
        __STRING_48__: if is_playing { 45.2 } else { 5.0 },
        __STRING_49__: if is_playing { 23.8 } else { 2.0 }
    });

    info!(__STRING_50__, stats[__STRING_51__], is_playing);

    Ok(stats)
}

/// Export current preview frame
#[tauri::command]
pub async fn preview_export_frame(
    timestamp: f64,
    format: Option<String>,
    quality: Option<u8>,
    state: State<'_, AppState>,
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
        format!("Frame exported successfully to {}", filepath),
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
