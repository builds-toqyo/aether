use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;
use std::sync::Arc;
use std::process::{Command, Stdio, Child};
use std::io::{BufRead, BufReader};
use std::thread;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportRequest {
    pub project_id: String,
    pub output_path: String,
    pub format: ExportFormat,
    pub video_settings: VideoSettings,
    pub audio_settings: AudioSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportFormat {
    pub container: String,
    pub video_codec: String,
    pub audio_codec: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoSettings {
    pub resolution: (u32, u32),
    pub framerate: u32,
    pub bitrate: u64,
    pub profile: Option<String>,
    pub level: Option<String>,
    pub pixel_format: Option<String>,
    pub color_space: Option<String>,
    pub pass: u32,
    pub hardware_acceleration: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioSettings {
    pub sample_rate: u32,
    pub channels: u32,
    pub bitrate: u64,
    pub codec: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportProgress {
    pub export_id: String,
    pub status: ExportStatus,
    pub progress: f64,
    pub current_frame: u32,
    pub total_frames: u32,
    pub speed: Option<f64>,
    pub time_remaining: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExportStatus {
    Idle,
    Preparing,
    Rendering,
    Encoding,
    Finalizing,
    Completed,
    Failed,
}

pub struct RenderingState {
    pub active_exports: HashMap<String, ExportProgress>,
}

impl RenderingState {
    pub fn new() -> Self {
        RenderingState {
            active_exports: HashMap::new(),
        }
    }
}

fn build_ffmpeg_args(request: &ExportRequest, input_path: &str) -> Vec<String> {
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        input_path.to_string(),
        "-progress".to_string(),
        "pipe:1".to_string(),
    ];
    
    let video_codec = match request.format.video_codec.as_str() {
        "H.264" => "libx264",
        "H.265" | "HEVC" => "libx265",
        "VP9" => "libvpx-vp9",
        "AV1" => "libaom-av1",
        "ProRes 422" | "ProRes 422 HQ" => "prores_ks",
        "DNxHD" => "dnxhd",
        _ => "libx264",
    };
    
    args.push("-c:v".to_string());
    args.push(video_codec.to_string());
    
    args.push("-s".to_string());
    args.push(format!("{}x{}", request.video_settings.resolution.0, request.video_settings.resolution.1));
    
    args.push("-r".to_string());
    args.push(request.video_settings.framerate.to_string());
    
    if request.video_settings.bitrate > 0 {
        args.push("-b:v".to_string());
        args.push(format!("{}", request.video_settings.bitrate));
    }
    
    if let Some(ref profile) = request.video_settings.profile {
        args.push("-profile:v".to_string());
        args.push(profile.clone());
    }
    
    if request.video_settings.hardware_acceleration {
        args.push("-hwaccel".to_string());
        args.push("auto".to_string());
    }
    
    let audio_codec = match request.format.audio_codec.as_str() {
        "AAC" => "aac",
        "MP3" => "libmp3lame",
        "PCM" => "pcm_s16le",
        "FLAC" => "flac",
        "Opus" => "libopus",
        "Vorbis" => "libvorbis",
        _ => "aac",
    };
    
    args.push("-c:a".to_string());
    args.push(audio_codec.to_string());
    
    args.push("-ar".to_string());
    args.push(request.audio_settings.sample_rate.to_string());
    
    args.push("-ac".to_string());
    args.push(request.audio_settings.channels.to_string());
    
    if request.audio_settings.bitrate > 0 {
        args.push("-b:a".to_string());
        args.push(format!("{}", request.audio_settings.bitrate));
    }
    
    args.push(request.output_path.clone());
    
    args
}

fn parse_ffmpeg_progress(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

#[tauri::command]
pub fn start_rendering(
    request: ExportRequest,
    input_path: String,
    state: State<Mutex<RenderingState>>
) -> Result<String, String> {
    let export_id = Uuid::new_v4().to_string();
    
    let args = build_ffmpeg_args(&request, &input_path);
    
    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start FFmpeg: {}", e))?;
    
    let progress = ExportProgress {
        export_id: export_id.clone(),
        status: ExportStatus::Rendering,
        progress: 0.0,
        current_frame: 0,
        total_frames: calculate_total_frames(&request),
        speed: None,
        time_remaining: None,
        error: None,
    };
    
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    state_guard.active_exports.insert(export_id.clone(), progress);
    drop(state_guard);
    
    let export_id_clone = export_id.clone();
    let state_arc = Arc::new(state.inner().clone());
    
    thread::spawn(move || {
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut current_frame: u32 = 0;
            let mut speed: f64 = 0.0;
            
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Some((key, value)) = parse_ffmpeg_progress(&line) {
                        match key.as_str() {
                            "frame" => {
                                if let Ok(f) = value.parse::<u32>() {
                                    current_frame = f;
                                }
                            }
                            "speed" => {
                                let speed_str = value.trim_end_matches('x');
                                if let Ok(s) = speed_str.parse::<f64>() {
                                    speed = s;
                                }
                            }
                            "progress" => {
                                if let Ok(mut state_guard) = state_arc.lock() {
                                    if let Some(progress) = state_guard.active_exports.get_mut(&export_id_clone) {
                                        progress.current_frame = current_frame;
                                        progress.speed = Some(speed);
                                        
                                        if progress.total_frames > 0 {
                                            progress.progress = (current_frame as f64 / progress.total_frames as f64) * 100.0;
                                            
                                            if speed > 0.0 {
                                                let remaining_frames = progress.total_frames - current_frame;
                                                progress.time_remaining = Some((remaining_frames as f64 / (30.0 * speed)) as u64);
                                            }
                                        }
                                        
                                        if value == "end" {
                                            progress.status = ExportStatus::Completed;
                                            progress.progress = 100.0;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        let status = child.wait();
        if let Ok(mut state_guard) = state_arc.lock() {
            if let Some(progress) = state_guard.active_exports.get_mut(&export_id_clone) {
                match status {
                    Ok(exit_status) if exit_status.success() => {
                        progress.status = ExportStatus::Completed;
                        progress.progress = 100.0;
                    }
                    _ => {
                        progress.status = ExportStatus::Failed;
                        progress.error = Some("FFmpeg process failed".to_string());
                    }
                }
            }
        }
    });
    
    Ok(export_id)
}

fn calculate_total_frames(request: &ExportRequest) -> u32 {
    (request.video_settings.framerate * 60) as u32
}

#[tauri::command]
pub fn get_export_progress(export_id: String, state: State<Mutex<RenderingState>>) -> Result<ExportProgress, String> {
    let state_guard = state.lock().map_err(|e| e.to_string())?;
    
    match state_guard.active_exports.get(&export_id) {
        Some(progress) => Ok(progress.clone()),
        None => Err("Export not found".to_string()),
    }
}

#[tauri::command]
pub fn cancel_export(export_id: String, state: State<Mutex<RenderingState>>) -> Result<(), String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    
    match state_guard.active_exports.get_mut(&export_id) {
        Some(progress) => {
            progress.status = ExportStatus::Failed;
            progress.error = Some("Export cancelled".to_string());
            Ok(())
        }
        None => Err("Export not found".to_string()),
    }
}

#[tauri::command]
pub fn get_active_exports(state: State<Mutex<RenderingState>>) -> Result<Vec<ExportProgress>, String> {
    let state_guard = state.lock().map_err(|e| e.to_string())?;
    Ok(state_guard.active_exports.values().cloned().collect())
}

#[tauri::command]
pub fn get_supported_formats() -> Result<Vec<serde_json::Value>, String> {
    let formats = vec![
        serde_json::json!({
            "name": "MP4",
            "container": "mp4",
            "video_codecs": ["H.264", "H.265", "AV1"],
            "audio_codecs": ["AAC", "MP3"],
            "description": "Universal format with wide compatibility"
        }),
        serde_json::json!({
            "name": "MOV",
            "container": "mov",
            "video_codecs": ["H.264", "H.265", "ProRes"],
            "audio_codecs": ["AAC", "PCM"],
            "description": "Apple QuickTime format, good for editing"
        }),
        serde_json::json!({
            "name": "AVI",
            "container": "avi",
            "video_codecs": ["H.264", "Xvid"],
            "audio_codecs": ["MP3", "AC3"],
            "description": "Legacy format with good compatibility"
        }),
        serde_json::json!({
            "name": "MKV",
            "container": "mkv",
            "video_codecs": ["H.264", "H.265", "VP9", "AV1"],
            "audio_codecs": ["AAC", "FLAC", "Opus"],
            "description": "Matroska container, supports many codecs"
        }),
        serde_json::json!({
            "name": "WebM",
            "container": "webm",
            "video_codecs": ["VP9", "AV1"],
            "audio_codecs": ["Opus", "Vorbis"],
            "description": "Web-optimized format for streaming"
        }),
    ];
    
    Ok(formats)
}

#[tauri::command]
pub fn get_export_presets() -> Result<Vec<serde_json::Value>, String> {
    let presets = vec![
        serde_json::json!({
            "name": "YouTube 1080p",
            "description": "Optimized for YouTube upload at 1080p",
            "format": {
                "container": "mp4",
                "video_codec": "H.264",
                "audio_codec": "AAC"
            },
            "video_settings": {
                "resolution": [1920, 1080],
                "framerate": 30,
                "bitrate": 8000000,
                "profile": "high",
                "level": "4.0"
            },
            "audio_settings": {
                "sample_rate": 48000,
                "channels": 2,
                "bitrate": 320000
            }
        }),
        serde_json::json!({
            "name": "YouTube 4K",
            "description": "Optimized for YouTube upload at 4K",
            "format": {
                "container": "mp4",
                "video_codec": "H.265",
                "audio_codec": "AAC"
            },
            "video_settings": {
                "resolution": [3840, 2160],
                "framerate": 30,
                "bitrate": 45000000,
                "profile": "main",
                "level": "5.0"
            },
            "audio_settings": {
                "sample_rate": 48000,
                "channels": 2,
                "bitrate": 320000
            }
        }),
        serde_json::json!({
            "name": "Vimeo 1080p",
            "description": "Optimized for Vimeo upload at 1080p",
            "format": {
                "container": "mp4",
                "video_codec": "H.264",
                "audio_codec": "AAC"
            },
            "video_settings": {
                "resolution": [1920, 1080],
                "framerate": 30,
                "bitrate": 10000000,
                "profile": "high",
                "level": "4.0"
            },
            "audio_settings": {
                "sample_rate": 48000,
                "channels": 2,
                "bitrate": 320000
            }
        }),
        serde_json::json!({
            "name": "Proxy 720p",
            "description": "Low-resolution proxy for editing",
            "format": {
                "container": "mp4",
                "video_codec": "H.264",
                "audio_codec": "AAC"
            },
            "video_settings": {
                "resolution": [1280, 720],
                "framerate": 30,
                "bitrate": 2000000,
                "profile": "main",
                "level": "3.1"
            },
            "audio_settings": {
                "sample_rate": 48000,
                "channels": 2,
                "bitrate": 128000
            }
        }),
        serde_json::json!({
            "name": "Master ProRes",
            "description": "High-quality ProRes for mastering",
            "format": {
                "container": "mov",
                "video_codec": "ProRes 422 HQ",
                "audio_codec": "PCM"
            },
            "video_settings": {
                "resolution": [1920, 1080],
                "framerate": 30,
                "bitrate": 0, // VBR
                "profile": None,
                "level": None
            },
            "audio_settings": {
                "sample_rate": 48000,
                "channels": 2,
                "bitrate": 0 // Uncompressed
            }
        }),
        serde_json::json!({
            "name": "Web Optimized",
            "description": "Optimized for web streaming",
            "format": {
                "container": "mp4",
                "video_codec": "H.264",
                "audio_codec": "AAC"
            },
            "video_settings": {
                "resolution": [1920, 1080],
                "framerate": 30,
                "bitrate": 5000000,
                "profile": "main",
                "level": "3.1"
            },
            "audio_settings": {
                "sample_rate": 48000,
                "channels": 2,
                "bitrate": 192000
            }
        }),
    ];
    
    Ok(presets)
}

#[tauri::command]
pub fn get_video_codecs() -> Result<Vec<serde_json::Value>, String> {
    let codecs = vec![
        serde_json::json!({
            "name": "H.264",
            "description": "Most widely compatible codec",
            "profiles": ["baseline", "main", "high"],
            "hardware_acceleration": true
        }),
        serde_json::json!({
            "name": "H.265 (HEVC)",
            "description": "More efficient compression, less compatible",
            "profiles": ["main", "main10"],
            "hardware_acceleration": true
        }),
        serde_json::json!({
            "name": "VP9",
            "description": "Open-source codec, good for web",
            "profiles": ["profile0", "profile1", "profile2", "profile3"],
            "hardware_acceleration": false
        }),
        serde_json::json!({
            "name": "AV1",
            "description": "Next-generation codec, best compression",
            "profiles": ["main", "high", "professional"],
            "hardware_acceleration": true
        }),
        serde_json::json!({
            "name": "ProRes 422",
            "description": "Professional editing codec",
            "profiles": ["Proxy", "LT", "422", "422 HQ", "4444"],
            "hardware_acceleration": false
        }),
        serde_json::json!({
            "name": "DNxHD",
            "description": "Avid codec for editing",
            "profiles": ["36", "115", "145", "175", "220"],
            "hardware_acceleration": false
        }),
    ];
    
    Ok(codecs)
}

#[tauri::command]
pub fn get_audio_codecs() -> Result<Vec<serde_json::Value>, String> {
    let codecs = vec![
        serde_json::json!({
            "name": "AAC",
            "description": "Most widely compatible audio codec",
            "sample_rates": [44100, 48000, 96000],
            "channels": [1, 2, 6, 8]
        }),
        serde_json::json!({
            "name": "MP3",
            "description": "Legacy audio codec, universal compatibility",
            "sample_rates": [44100, 48000],
            "channels": [1, 2]
        }),
        serde_json::json!({
            "name": "PCM",
            "description": "Uncompressed audio, highest quality",
            "sample_rates": [44100, 48000, 96000],
            "channels": [1, 2, 6, 8]
        }),
        serde_json::json!({
            "name": "FLAC",
            "description": "Lossless compression",
            "sample_rates": [44100, 48000, 96000],
            "channels": [1, 2, 6, 8]
        }),
        serde_json::json!({
            "name": "Opus",
            "description": "Modern codec, excellent for web",
            "sample_rates": [48000],
            "channels": [1, 2, 6, 8]
        }),
        serde_json::json!({
            "name": "Vorbis",
            "description": "Open-source codec, good for web",
            "sample_rates": [44100, 48000],
            "channels": [1, 2, 6, 8]
        }),
    ];
    
    Ok(codecs)
}

#[tauri::command]
pub fn simulate_export_progress(export_id: String, state: State<Mutex<RenderingState>>) -> Result<(), String> {
    let mut state_guard = state.lock().map_err(|e| e.to_string())?;
    
    if let Some(progress) = state_guard.active_exports.get_mut(&export_id) {
        match progress.status {
            ExportStatus::Preparing => {
                progress.status = ExportStatus::Rendering;
                progress.progress = 0.0;
            }
            ExportStatus::Rendering => {
                progress.progress = (progress.progress + 10.0).min(100.0);
                progress.current_frame = ((progress.progress / 100.0) * progress.total_frames as f64) as u32;
                progress.speed = Some(30.0); // Mock 30 fps
                progress.time_remaining = Some(((100.0 - progress.progress) / 10.0 * 2.0) as u64);
                
                if progress.progress >= 100.0 {
                    progress.status = ExportStatus::Encoding;
                }
            }
            ExportStatus::Encoding => {
                progress.status = ExportStatus::Finalizing;
                progress.progress = 100.0;
            }
            ExportStatus::Finalizing => {
                progress.status = ExportStatus::Completed;
            }
            _ => {}
        }
        return Ok(());
    }
    
    Err("Export not found".to_string())
}
