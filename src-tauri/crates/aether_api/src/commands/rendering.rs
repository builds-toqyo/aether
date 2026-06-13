use serde::{Serialize, Deserialize};
use tauri::State;
use anyhow::Result;
use log::{debug, info, warn};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::state::AppState;
use aether_core::engine::rendering::{
    RenderingEngine, ExportOptions, ExportProgress,
    VideoFormat, AudioFormat, ContainerFormat, EncoderPreset
};
use aether_core::engine::editing::types::EditingError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenderingJob {
    pub id: String,
    pub name: String,
    pub status: RenderingStatus,
    pub progress: f64,
    pub current_frame: u32,
    pub total_frames: u32,
    pub start_time: String,
    pub end_time: Option<String>,
    pub output_path: String,
    pub format: RenderFormat,
    pub quality: RenderQuality,
    pub resolution: (u32, u32),
    pub fps: f64,
    pub bitrate: u32,
    pub estimated_size: u64,
    pub actual_size: Option<u64>,
    pub error_message: Option<String>,
}

pub struct ActiveRenderingJob {
    pub job: RenderingJob,
    pub exporter: Arc<Mutex<Box<dyn ExporterTrait>>>,
    pub progress: Arc<Mutex<ExportProgress>>,
}

pub trait ExporterTrait: Send + Sync {
    fn get_progress(&self) -> ExportProgress;
    fn cancel(&mut self) -> Result<(), EditingError>;
    fn is_complete(&self) -> bool;
    fn has_error(&self) -> bool;
    fn get_error(&self) -> Option<String>;
    fn pause(&mut self) -> Result<(), EditingError>;
    fn resume(&mut self) -> Result<(), EditingError>;
    fn is_paused(&self) -> bool;
}

impl ExporterTrait for aether_core::engine::rendering::Exporter {
    fn get_progress(&self) -> ExportProgress {
        self.get_progress()
    }

    fn cancel(&mut self) -> Result<(), EditingError> {
        self.cancel()
    }

    fn is_complete(&self) -> bool {
        self.is_complete()
    }

    fn has_error(&self) -> bool {
        self.has_error()
    }

    fn get_error(&self) -> Option<String> {
        self.get_error()
    }

    fn pause(&mut self) -> Result<(), EditingError> {
        self.pause()
    }

    fn resume(&mut self) -> Result<(), EditingError> {
        self.resume()
    }

    fn is_paused(&self) -> bool {
        self.is_paused()
    }
}

impl ExporterTrait for crate::render_proxy::GstExporterProxy {
    fn get_progress(&self) -> ExportProgress {
        let gst_progress = self.get_progress();
        ExportProgress {
            current_frame: gst_progress.current_frame,
            total_frames: gst_progress.total_frames,
            current_time: gst_progress.current_time,
            total_duration: gst_progress.total_duration,
            percent: gst_progress.percent,
            complete: gst_progress.complete,
            error: gst_progress.error,
        }
    }

    fn cancel(&mut self) -> Result<(), EditingError> {
        self.cancel_export()
    }

    fn is_complete(&self) -> bool {
        self.is_complete()
    }

    fn has_error(&self) -> bool {
        self.has_error()
    }

    fn get_error(&self) -> Option<String> {
        self.get_error()
    }

    fn pause(&mut self) -> Result<(), EditingError> {
        warn!("GStreamer exporter pause requested - stub");
        Ok(())
    }

    fn resume(&mut self) -> Result<(), EditingError> {
        warn!("GStreamer exporter resume requested - stub");
        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.is_paused()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RenderingStatus {
    Pending,
    Preparing,
    Rendering,
    Encoding,
    Uploading,
    Completed,
    Failed,
    Cancelled,
    Paused,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RenderFormat {
    Mp4,
    Avi,
    Mov,
    Mkv,
    Webm,
    Gif,
    PngSequence,
    JpegSequence,
    AudioOnly,
    Custom(String),
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
    Custom {
        bitrate: u32,
        preset: RenderPreset,
        profile: Option<String>,
    },
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RenderPreset {
    UltraFast,
    SuperFast,
    VeryFast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    VerySlow,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AudioRenderSettings {
    pub codec: AudioCodec,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub volume: f64,
    pub normalize: bool,
    pub fade_in: Option<f64>,
    pub fade_out: Option<f64>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AudioCodec {
    Aac,
    Mp3,
    Opus,
    Flac,
    Wav,
    Ac3,
    Dts,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct VideoRenderSettings {
    pub codec: VideoCodec,
    pub bitrate: u32,
    pub preset: RenderPreset,
    pub profile: Option<String>,
    pub level: Option<String>,
    pub gop_size: Option<u32>,
    pub b_frames: Option<u32>,
    pub max_b_frames: Option<u32>,
    pub pixel_format: Option<String>,
    pub color_space: Option<String>,
    pub color_range: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VideoCodec {
    H264,
    H265,
    Vp9,
    Av1,
    Mpeg2,
    Mpeg4,
    ProRes,
    Dnxhd,
}


#[derive(Debug, Deserialize)]
pub struct RenderingRequest {
    pub name: String,
    pub output_path: String,
    pub format: RenderFormat,
    pub quality: RenderQuality,
    pub resolution: Option<(u32, u32)>,
    pub fps: Option<f64>,
    pub start_time: Option<f64>,
    pub end_time: Option<f64>,
    pub video_settings: Option<VideoRenderSettings>,
    pub audio_settings: Option<AudioRenderSettings>,
    pub export_range: Option<ExportRange>,
    pub metadata: Option<RenderMetadata>,
}


#[derive(Debug, Deserialize)]
pub struct ExportRange {
    pub start_time: f64,
    pub end_time: f64,
    pub include_markers: bool,
    pub marker_filter: Option<Vec<String>>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RenderMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub copyright: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_at: Option<String>,
    pub software: Option<String>,
}


#[derive(Debug, Serialize)]
pub struct RenderingResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}


#[derive(Debug, Clone)]
pub struct QueuedJob {
    pub id: String,
    pub name: String,
    pub output_path: String,
    pub format: RenderFormat,
    pub quality: RenderQuality,
    pub resolution: (u32, u32),
    pub fps: f64,
    pub bitrate: u32,
    pub estimated_size: u64,
    pub created_at: String,
    pub priority: JobPriority,
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Urgent = 4,
}

impl Default for JobPriority {
    fn default() -> Self {
        JobPriority::Normal
    }
}


pub struct RenderingState {
    pub engine: RenderingEngine,
    pub active_jobs: HashMap<String, ActiveRenderingJob>,
    pub job_queue: Vec<QueuedJob>,
    pub max_concurrent_jobs: usize,
    pub paused_jobs: HashMap<String, ActiveRenderingJob>,
}

impl RenderingState {
    pub fn new() -> Result<Self, EditingError> {
        Ok(Self {
            engine: RenderingEngine::new()?,
            active_jobs: HashMap::new(),
            job_queue: Vec::new(),
            max_concurrent_jobs: 2,
            paused_jobs: HashMap::new(),
        })
    }

    pub fn add_queued_job(&mut self, job: QueuedJob) {
        self.job_queue.push(job);

        self.job_queue.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn remove_queued_job(&mut self, job_id: &str) -> Option<QueuedJob> {
        let index = self.job_queue.iter().position(|job| job.id == job_id)?;
        Some(self.job_queue.remove(index))
    }

    pub fn move_to_paused(&mut self, job_id: &str) -> Result<(), EditingError> {
        if let Some(active_job) = self.active_jobs.remove(job_id) {
            self.paused_jobs.insert(job_id.to_string(), active_job);
            Ok(())
        } else {
            Err(EditingError::ExportError("Job not found in active jobs".to_string()))
        }
    }

    pub fn move_from_paused(&mut self, job_id: &str) -> Result<(), EditingError> {
        if let Some(paused_job) = self.paused_jobs.remove(job_id) {
            self.active_jobs.insert(job_id.to_string(), paused_job);
            Ok(())
        } else {
            Err(EditingError::ExportError("Job not found in paused jobs".to_string()))
        }
    }
}

impl Default for RenderingState {
    fn default() -> Self {
        Self::new().expect("Failed to create rendering state")
    }
}


fn convert_render_format(format: &RenderFormat) -> ContainerFormat {
    match format {
        RenderFormat::Mp4 => ContainerFormat::Mp4,
        RenderFormat::Avi => ContainerFormat::Avi,
        RenderFormat::Mov => ContainerFormat::Mov,
        RenderFormat::Mkv => ContainerFormat::Mkv,
        RenderFormat::Webm => ContainerFormat::Webm,
        RenderFormat::Gif => ContainerFormat::Gif,
        RenderFormat::PngSequence => ContainerFormat::PngSequence,
        RenderFormat::JpegSequence => ContainerFormat::JpegSequence,
        RenderFormat::AudioOnly => ContainerFormat::Mp4,
        RenderFormat::Custom(_) => ContainerFormat::Mp4,
    }
}


fn convert_render_quality(quality: &RenderQuality) -> EncoderPreset {
    match quality {
        RenderQuality::Low => EncoderPreset::Fast,
        RenderQuality::Medium => EncoderPreset::Medium,
        RenderQuality::High => EncoderPreset::Slow,
        RenderQuality::Ultra => EncoderPreset::VerySlow,
        RenderQuality::Custom { preset, .. } => match preset {
            RenderPreset::UltraFast => EncoderPreset::UltraFast,
            RenderPreset::SuperFast => EncoderPreset::SuperFast,
            RenderPreset::VeryFast => EncoderPreset::VeryFast,
            RenderPreset::Faster => EncoderPreset::Faster,
            RenderPreset::Fast => EncoderPreset::Fast,
            RenderPreset::Medium => EncoderPreset::Medium,
            RenderPreset::Slow => EncoderPreset::Slow,
            RenderPreset::Slower => EncoderPreset::Slower,
            RenderPreset::VerySlow => EncoderPreset::VerySlow,
        },
    }
}


fn convert_progress(progress: &ExportProgress) -> RenderingProgress {
    RenderingProgress {
        job_id: String::new(),
        status: if progress.complete {
            if progress.error.is_some() {
                RenderingStatus::Failed
            } else {
                RenderingStatus::Completed
            }
        } else {
            RenderingStatus::Rendering
        },
        progress: progress.percent,
        current_frame: progress.current_frame as u32,
        total_frames: progress.total_frames as u32,
        fps: 30.0,
        time_elapsed: progress.current_time,
        time_remaining: if progress.percent > 0.0 {
            Some((progress.total_duration - progress.current_time) / (progress.percent / 100.0))
        } else {
            None
        },
        current_stage: "Rendering".to_string(),
        estimated_size: 0,
        actual_size: None,
    }
}


#[derive(Debug, Serialize)]
pub struct RenderingProgress {
    pub job_id: String,
    pub status: RenderingStatus,
    pub progress: f64,
    pub current_frame: u32,
    pub total_frames: u32,
    pub fps: f64,
    pub time_elapsed: f64,
    pub time_remaining: Option<f64>,
    pub current_stage: String,
    pub estimated_size: u64,
    pub actual_size: Option<u64>,
}


#[derive(Debug, Serialize)]
pub struct RenderingQueue {
    pub active_jobs: Vec<RenderingJob>,
    pub queued_jobs: Vec<RenderingJob>,
    pub completed_jobs: Vec<RenderingJob>,
    pub failed_jobs: Vec<RenderingJob>,
    pub max_concurrent_jobs: usize,
    pub total_capacity: usize,
}


#[tauri::command]
pub async fn rendering_start_job(
    request: RenderingRequest,
    state: State<'_, AppState>,
) -> Result<RenderingJob, String> {
    debug!("{}", request.name);

    // Validate inputs
    if request.name.is_empty() {
        return Err("Render job name cannot be empty".to_string());
    }

    if request.output_path.is_empty() {
        return Err("Output path cannot be empty".to_string());
    }

    if let Some((width, height)) = request.resolution {
        if width == 0 || height == 0 {
            return Err("Resolution cannot be zero".to_string());
        }
    }

    if let Some(fps) = request.fps {
        if fps <= 0.0 {
            return Err("FPS must be greater than 0".to_string());
        }
    }

    // Generate job ID
    let job_id = format!("{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();

    // Create export options for the rendering engine
    let export_options = ExportOptions {
        input_path: std::path::PathBuf::from("/timeline/current"), // Input path derived from active timeline
        output_path: std::path::PathBuf::from(&request.output_path),
        container_format: convert_render_format(&request.format),
        video_format: VideoFormat::H264, // Convert from request.video_settings
        audio_format: AudioFormat::Aac,   // Convert from request.audio_settings
        video_bitrate: request.video_settings.as_ref().map(|v| v.bitrate).unwrap_or(5000000),
        audio_bitrate: 128000, // Default audio bitrate
        frame_rate: request.fps.unwrap_or(30.0),
        width: request.resolution.unwrap_or((1920, 1080)).0,
        height: request.resolution.unwrap_or((1920, 1080)).1,
        encoder_preset: convert_render_quality(&request.quality),
        crf: 23, // Default CRF
        hardware_acceleration: false,
        threads: 0, // Auto-detect
    };

    // Query timeline duration from engine for realistic frame/size estimates
    let timeline_duration_ns = state.editing_engine.lock()
        .map_err(|e| format!("{}", e))?
        .get_timeline_info()
        .map(|t| t.duration)
        .unwrap_or(0);
    let duration_seconds = timeline_duration_ns as f64 / 1_000_000_000.0;
    let fps = request.fps.unwrap_or(30.0);
    let total_frames = if duration_seconds > 0.0 {
        (duration_seconds * fps) as u32
    } else {
        0
    };
    let bitrate = request.video_settings.as_ref().map(|v| v.bitrate).unwrap_or(5_000_000u32);
    let estimated_size = if duration_seconds > 0.0 {
        (bitrate as u64 * duration_seconds as u64) / 8
    } else {
        1024 * 1024 * 250 // fallback estimate
    };

    // Get rendering state
    let mut rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("{}", e))?;

    // Create exporter directly (bypassing aether_core::ActiveExporter which required unsafe Send/Sync)
    let exporter: Box<dyn ExporterTrait> = {
        let ffmpeg_exporter = aether_core::engine::rendering::Exporter::new(export_options)
            .map_err(|e| format!("{}", e))?;
        Box::new(ffmpeg_exporter)
    };

    // Create progress tracking
    let progress = Arc::new(Mutex::new(ExportProgress {
        current_frame: 0,
        total_frames: 0,
        current_time: 0.0,
        total_duration: 0.0,
        percent: 0.0,
        complete: false,
        error: None,
    }));

    // Create active job
    let active_job = ActiveRenderingJob {
        job: RenderingJob {
            id: job_id.clone(),
            name: request.name.clone(),
            status: RenderingStatus::Preparing,
            progress: 0.0,
            current_frame: 0,
            total_frames,
            start_time: now.clone(),
            end_time: None,
            output_path: request.output_path.clone(),
            format: request.format.clone(),
            quality: request.quality.clone(),
            resolution: request.resolution.unwrap_or((1920, 1080)),
            fps,
            bitrate,
            estimated_size,
            actual_size: None,
            error_message: None,
        },
        exporter: Arc::new(Mutex::new(exporter)),
        progress: progress.clone(),
    };

    // Add to active jobs
    rendering_state.active_jobs.insert(job_id.clone(), active_job);

    info!("Started job: {} ({})", request.name, job_id);
    Ok(rendering_state.active_jobs[&job_id].job.clone())
}

/// Cancel a rendering job
#[tauri::command]
pub async fn rendering_cancel_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<RenderingResponse, String> {
    debug!("Cancelling rendering job: {}", job_id);

    if job_id.is_empty() {
        return Err("Job ID cannot be empty".to_string());
    }


    let mut rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("Failed to lock rendering state: {}", e))?;

    let active_job = rendering_state.active_jobs.get_mut(&job_id)
        .ok_or_else(|| format!("Job not found: {}", job_id))?;


    {
        let mut exporter = active_job.exporter.lock()
            .map_err(|e| format!("Failed to lock exporter: {}", e))?;
        exporter.cancel()
            .map_err(|e| format!("Failed to cancel job: {}", e))?;
    }


    active_job.job.status = RenderingStatus::Cancelled;
    active_job.job.end_time = Some(chrono::Utc::now().to_rfc3339());

    let response = RenderingResponse {
        success: true,
        message: format!("Job {} cancelled successfully", job_id),
        data: None,
    };

    info!("Cancelled rendering job: {}", job_id);
    Ok(response)
}

#[tauri::command]
pub async fn rendering_pause_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<RenderingResponse, String> {
    debug!("{}", job_id);

    if job_id.is_empty() {
        return Err("Job ID cannot be empty".to_string());
    }

    // Get rendering state and find the job
    let mut rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("{}", e))?;

    {
        let active_job = rendering_state.active_jobs.get_mut(&job_id)
            .ok_or_else(|| format!("Job not found in active jobs: {}", job_id))?;

        match active_job.exporter.lock().unwrap().pause() {
            Ok(_) => {
                active_job.job.status = RenderingStatus::Paused;
            }
            Err(e) => {
                let response = RenderingResponse {
                    success: false,
                    message: format!("Job {} pause failed: {}", job_id, e),
                    data: None,
                };
                warn!("Job {} pause failed: {}", job_id, e);
                return Ok(response);
            }
        }
    }

    // Move job from active to paused jobs
    let job = rendering_state.active_jobs.remove(&job_id).unwrap();
    rendering_state.paused_jobs.insert(job_id.clone(), job);

    let response = RenderingResponse {
        success: true,
        message: format!("Job {} paused successfully", job_id),
        data: None,
    };

    info!("Job {} paused", job_id);
    Ok(response)
}

/// Resume a rendering job
#[tauri::command]
pub async fn rendering_resume_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<RenderingResponse, String> {
    debug!("Resuming rendering job: {}", job_id);

    if job_id.is_empty() {
        return Err("Job ID cannot be empty".to_string());
    }


    let mut rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("Failed to lock rendering state: {}", e))?;

    {
        let paused_job = rendering_state.paused_jobs.get_mut(&job_id)
            .ok_or_else(|| format!("Job not found in paused jobs: {}", job_id))?;

        match paused_job.exporter.lock().unwrap().resume() {
            Ok(_) => {
                paused_job.job.status = RenderingStatus::Rendering;
            }
            Err(e) => {
                let response = RenderingResponse {
                    success: false,
                    message: format!("Failed to resume job {}: {}", job_id, e),
                    data: None,
                };
                warn!("Failed to resume rendering job {}: {}", job_id, e);
                return Ok(response);
            }
        }
    }

    let job = rendering_state.paused_jobs.remove(&job_id).unwrap();
    rendering_state.active_jobs.insert(job_id.clone(), job);

    let response = RenderingResponse {
        success: true,
        message: format!("Job {} resumed successfully", job_id),
        data: None,
    };

    info!("Resumed rendering job: {}", job_id);
    Ok(response)
}


#[tauri::command]
pub async fn rendering_get_job_status(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<RenderingJob, String> {
    debug!("{}", job_id);

    if job_id.is_empty() {
        return Err("Job ID cannot be empty".to_string());
    }

    // Get rendering state and find the job
    let rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("{}", e))?;

    if let Some(active_job) = rendering_state.active_jobs.get(&job_id) {
        // Get current progress from exporter
        let progress = {
            let exporter = active_job.exporter.lock()
                .map_err(|e| format!("{}", e))?;
            exporter.get_progress()
        };

        let mut job = active_job.job.clone();
        job.progress = progress.percent;
        job.current_frame = progress.current_frame as u32;
        job.total_frames = progress.total_frames as u32;

        if progress.complete {
            if progress.error.is_some() {
                job.status = RenderingStatus::Failed;
                job.error_message = progress.error.clone();
                job.end_time = Some(chrono::Utc::now().to_rfc3339());
            } else {
                job.status = RenderingStatus::Completed;
                job.end_time = Some(chrono::Utc::now().to_rfc3339());
            }
        } else {
            job.status = RenderingStatus::Rendering;
        }

        info!("Retrieved job status: {} ({})", job.name, job_id);
        Ok(job)
    } else if let Some(paused_job) = rendering_state.paused_jobs.get(&job_id) {
        let mut job = paused_job.job.clone();
        job.status = RenderingStatus::Paused;
        info!("Retrieved paused job status: {} ({})", job.name, job_id);
        Ok(job)
    } else {
        Err(format!("Job not found: {}", job_id))
    }
}

/// Get rendering job progress
#[tauri::command]
pub async fn rendering_get_job_progress(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<RenderingProgress, String> {
    debug!("Getting rendering job progress: {}", job_id);

    if job_id.is_empty() {
        return Err("Job ID cannot be empty".to_string());
    }


    let rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("Failed to lock rendering state: {}", e))?;

    let active_job = rendering_state.active_jobs.get(&job_id)
        .ok_or_else(|| format!("Job not found: {}", job_id))?;


    let progress = {
        let exporter = active_job.exporter.lock()
            .map_err(|e| format!("Failed to lock exporter: {}", e))?;
        exporter.get_progress()
    };


    let mut api_progress = convert_progress(&progress);
    api_progress.job_id = job_id.clone();

    info!("Retrieved rendering progress: {} - {:.1}%", job_id, api_progress.progress);
    Ok(api_progress)
}


#[tauri::command]
pub async fn rendering_get_all_jobs(
    state: State<'_, AppState>,
) -> Result<Vec<RenderingJob>, String> {
    debug!("Getting all rendering jobs");

    // Get rendering state
    let rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("{}", e))?;

    // Collect all active jobs with updated status
    let mut jobs = Vec::new();
    for (_job_id, active_job) in &rendering_state.active_jobs {
        // Get current progress from exporter
        let progress = {
            let exporter = active_job.exporter.lock()
                .map_err(|e| format!("{}", e))?;
            exporter.get_progress()
        };

        // Update job status based on progress
        let mut job = active_job.job.clone();
        job.progress = progress.percent;
        job.current_frame = progress.current_frame as u32;
        job.total_frames = progress.total_frames as u32;

        if progress.complete {
            if progress.error.is_some() {
                job.status = RenderingStatus::Failed;
                job.error_message = progress.error.clone();
                job.end_time = Some(chrono::Utc::now().to_rfc3339());
            } else {
                job.status = RenderingStatus::Completed;
                job.end_time = Some(chrono::Utc::now().to_rfc3339());
            }
        } else {
            job.status = RenderingStatus::Rendering;
        }

        jobs.push(job);
    }

    // Include paused jobs
    for (_job_id, paused_job) in &rendering_state.paused_jobs {
        let mut job = paused_job.job.clone();
        job.status = RenderingStatus::Paused;
        jobs.push(job);
    }

    info!("{}", jobs.len());
    Ok(jobs)
}

/// Get rendering queue information
#[tauri::command]
pub async fn rendering_get_queue(
    state: State<'_, AppState>,
) -> Result<RenderingQueue, String> {
    debug!("Getting rendering queue information");


    let rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("Failed to lock rendering state: {}", e))?;


    let mut active_jobs = Vec::new();
    let mut queued_jobs = Vec::new();
    let mut completed_jobs = Vec::new();
    let mut failed_jobs = Vec::new();

    for (_job_id, active_job) in &rendering_state.active_jobs {

        let progress = {
            let exporter = active_job.exporter.lock()
                .map_err(|e| format!("Failed to lock exporter: {}", e))?;
            exporter.get_progress()
        };


        let mut job = active_job.job.clone();
        job.progress = progress.percent;
        job.current_frame = progress.current_frame as u32;
        job.total_frames = progress.total_frames as u32;

        if progress.complete {
            if progress.error.is_some() {
                job.status = RenderingStatus::Failed;
                job.error_message = progress.error.clone();
                job.end_time = Some(chrono::Utc::now().to_rfc3339());
                failed_jobs.push(job);
            } else {
                job.status = RenderingStatus::Completed;
                job.end_time = Some(chrono::Utc::now().to_rfc3339());
                completed_jobs.push(job);
            }
        } else {
            job.status = RenderingStatus::Rendering;
            active_jobs.push(job);
        }
    }


    for queued_job in &rendering_state.job_queue {
        queued_jobs.push(RenderingJob {
            id: queued_job.id.clone(),
            name: queued_job.name.clone(),
            status: RenderingStatus::Pending,
            progress: 0.0,
            current_frame: 0,
            total_frames: 0,
            start_time: queued_job.created_at.clone(),
            end_time: None,
            output_path: queued_job.output_path.clone(),
            format: queued_job.format.clone(),
            quality: queued_job.quality.clone(),
            resolution: queued_job.resolution,
            fps: queued_job.fps,
            bitrate: queued_job.bitrate,
            estimated_size: queued_job.estimated_size,
            actual_size: None,
            error_message: None,
        });
    }


    for (_job_id, paused_job) in &rendering_state.paused_jobs {
        let mut job = paused_job.job.clone();
        job.status = RenderingStatus::Paused;
        active_jobs.push(job);
    }

    let queue = RenderingQueue {
        active_jobs,
        queued_jobs,
        completed_jobs,
        failed_jobs,
        max_concurrent_jobs: rendering_state.max_concurrent_jobs,
        total_capacity: rendering_state.max_concurrent_jobs * 2,
    };

    info!("Retrieved rendering queue: {} active, {} queued, {} completed",
          queue.active_jobs.len(), queue.queued_jobs.len(), queue.completed_jobs.len());
    Ok(queue)
}


#[tauri::command]
pub async fn rendering_get_formats(
    _state: State<'_, AppState>,
) -> Result<Vec<RenderFormatInfo>, String> {
    debug!("Getting render formats");

    let core_formats = aether_core::engine::rendering::get_available_formats();

    let formats: Vec<RenderFormatInfo> = core_formats.into_iter().map(|fi| {
        let (max_res, max_fps, max_bitrate) = match fi.container {
            aether_core::engine::rendering::ContainerFormat::Mp4 => (Some((7680, 4320)), Some(120.0), Some(50000000)),
            aether_core::engine::rendering::ContainerFormat::Mov => (Some((7680, 4320)), Some(120.0), Some(100000000)),
            aether_core::engine::rendering::ContainerFormat::Webm => (Some((3840, 2160)), Some(60.0), Some(20000000)),
            aether_core::engine::rendering::ContainerFormat::Gif => (Some((1280, 720)), Some(30.0), None),
            _ => (Some((3840, 2160)), Some(60.0), Some(50000000)),
        };

        RenderFormatInfo {
            format: map_container_format(fi.container),
            name: fi.container.display_name().to_string(),
            description: fi.use_case.clone(),
            extensions: vec![fi.container.extension().to_string()],
            supports_video: !fi.video_formats.is_empty(),
            supports_audio: !fi.audio_formats.is_empty() && fi.container != aether_core::engine::rendering::ContainerFormat::Gif,
            recommended_for: if fi.web_friendly {
                vec!["web".to_string(), "streaming".to_string()]
            } else if fi.container == aether_core::engine::rendering::ContainerFormat::Mov {
                vec!["editing".to_string(), "mastering".to_string()]
            } else {
                vec!["general".to_string()]
            },
            max_resolution: max_res,
            max_fps,
            max_bitrate,
        }
    }).collect();

    info!("{}", formats.len());
    Ok(formats)
}

fn map_container_format(core: aether_core::engine::rendering::ContainerFormat) -> RenderFormat {
    use aether_core::engine::rendering::ContainerFormat as Core;
    match core {
        Core::Mp4 => RenderFormat::Mp4,
        Core::Mov => RenderFormat::Mov,
        Core::Webm => RenderFormat::Webm,
        Core::Avi => RenderFormat::Avi,
        Core::Mkv => RenderFormat::Mkv,
        Core::Gif => RenderFormat::Gif,
        Core::PngSequence => RenderFormat::PngSequence,
        Core::JpegSequence => RenderFormat::JpegSequence,
        Core::Flv => RenderFormat::Custom("flv".to_string()),
        Core::Wmv => RenderFormat::Custom("wmv".to_string()),
        Core::Mpg => RenderFormat::Custom("mpg".to_string()),
        Core::Ts => RenderFormat::Custom("ts".to_string()),
        Core::Mxf => RenderFormat::Custom("mxf".to_string()),
    }
}

/// Get rendering presets
#[tauri::command]
pub async fn rendering_get_presets(
    _state: State<'_, AppState>,
) -> Result<Vec<RenderPresetInfo>, String> {
    debug!("Getting rendering presets");

    let presets = vec![
        RenderPresetInfo {
            name: "YouTube 1080p".to_string(),
            description: "Optimized for YouTube uploads at 1080p".to_string(),
            format: RenderFormat::Mp4,
            resolution: (1920, 1080),
            fps: 30.0,
            bitrate: 8000000,
            quality: RenderQuality::High,
            preset: RenderPreset::Medium,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
        },
        RenderPresetInfo {
            name: "Instagram Story".to_string(),
            description: "Optimized for Instagram stories (9:16 aspect ratio)".to_string(),
            format: RenderFormat::Mp4,
            resolution: (1080, 1920),
            fps: 30.0,
            bitrate: 4000000,
            quality: RenderQuality::Medium,
            preset: RenderPreset::Fast,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
        },
        RenderPresetInfo {
            name: "TikTok".to_string(),
            description: "Optimized for TikTok vertical videos".to_string(),
            format: RenderFormat::Mp4,
            resolution: (1080, 1920),
            fps: 30.0,
            bitrate: 6000000,
            quality: RenderQuality::High,
            preset: RenderPreset::Medium,
            video_codec: VideoCodec::H264,
            audio_codec: AudioCodec::Aac,
        },
        RenderPresetInfo {
            name: "4K Master".to_string(),
            description: "High quality 4K export for professional use".to_string(),
            format: RenderFormat::Mov,
            resolution: (3840, 2160),
            fps: 30.0,
            bitrate: 50000000,
            quality: RenderQuality::Ultra,
            preset: RenderPreset::Slow,
            video_codec: VideoCodec::H265,
            audio_codec: AudioCodec::Flac,
        },
    ];

    info!("Retrieved {} rendering presets", presets.len());
    Ok(presets)
}


#[tauri::command]
pub async fn rendering_estimate_time(
    resolution: (u32, u32),
    fps: f64,
    duration: f64,
    quality: RenderQuality,
    format: RenderFormat,
    _state: State<'_, AppState>,
) -> Result<RenderingTimeEstimate, String> {
    debug!("Resolution: {}x{}, FPS: {}, Duration: {}, Quality: {:?}, Format: {:?}",
           resolution.0, resolution.1, fps, duration, quality, format);

    // Validate inputs
    if resolution.0 == 0 || resolution.1 == 0 {
        return Err("Resolution cannot be zero".to_string());
    }

    if fps <= 0.0 {
        return Err("FPS must be greater than 0".to_string());
    }

    if duration <= 0.0 {
        return Err("Duration must be greater than 0".to_string());
    }

    // Calculate frame count and pixel complexity
    let pixel_count = resolution.0 * resolution.1;
    let frame_count = (duration * fps) as u32;
    let total_pixels = pixel_count as u64 * frame_count as u64;

    // Base rendering benchmarks (frames per second) for different quality levels
    let base_fps = match &quality {
        RenderQuality::Low => 60.0,      // Fast rendering
        RenderQuality::Medium => 30.0,   // Balanced
        RenderQuality::High => 15.0,     // Slower but higher quality
        RenderQuality::Ultra => 8.0,     // Very slow, highest quality
        RenderQuality::Custom { preset, .. } => match preset {
            RenderPreset::UltraFast => 120.0,
            RenderPreset::SuperFast => 90.0,
            RenderPreset::VeryFast => 60.0,
            RenderPreset::Faster => 45.0,
            RenderPreset::Fast => 30.0,
            RenderPreset::Medium => 20.0,
            RenderPreset::Slow => 12.0,
            RenderPreset::Slower => 8.0,
            RenderPreset::VerySlow => 4.0,
        },
    };

    // Adjust for resolution complexity (compared to 1080p baseline)
    let baseline_pixels = 1920u64 * 1080u64;
    let resolution_factor = (total_pixels as f64 / baseline_pixels as f64).sqrt();

    // Adjust for format complexity
    let format_factor = match format {
        RenderFormat::Mp4 => 1.0,
        RenderFormat::Avi => 0.9,
        RenderFormat::Mov => 1.1,
        RenderFormat::Mkv => 1.0,
        RenderFormat::Webm => 1.2,
        RenderFormat::Gif => 0.5,
        RenderFormat::PngSequence => 0.3,
        RenderFormat::JpegSequence => 0.4,
        RenderFormat::AudioOnly => 0.1,
        RenderFormat::Custom(_) => 1.0,
    };

    // Calculate effective rendering speed
    let effective_fps = base_fps / resolution_factor / format_factor;

    // Calculate estimated time
    let estimated_render_time = duration * fps / effective_fps;
    let estimated_encode_time = estimated_render_time * 0.3; // Encoding is typically 30% of render time
    let estimated_total_time = estimated_render_time + estimated_encode_time;

    // Calculate estimated file size (rough estimate)
    let bitrate = match &quality {
        RenderQuality::Low => 2000000,      // 2 Mbps
        RenderQuality::Medium => 5000000,   // 5 Mbps
        RenderQuality::High => 10000000,    // 10 Mbps
        RenderQuality::Ultra => 25000000,   // 25 Mbps
        RenderQuality::Custom { bitrate, .. } => *bitrate as u64,
    };
    let _estimated_size = (bitrate as u64 * duration as u64) / 8; // Convert to bytes

    // Calculate confidence based on complexity
    let confidence: f64 = if resolution_factor > 2.0 || format_factor > 1.5 {
        0.7 // Lower confidence for complex scenarios
    } else if resolution_factor < 0.5 {
        0.9 // Higher confidence for simple scenarios
    } else {
        0.8 // Standard confidence
    };

    let estimate = RenderingTimeEstimate {
        estimated_seconds: estimated_total_time as f64,
        estimated_minutes: (estimated_total_time / 60.0) as f64,
        estimated_hours: (estimated_total_time / 3600.0) as f64,
        confidence: confidence as f64,
        factors_used: vec![
            format!("resolution_factor: {}", resolution_factor),
            format!("format_factor: {}", format_factor),
            format!("base_fps: {}", base_fps),
            format!("effective_fps: {}", effective_fps),
        ],
    };

    info!("Render: {:.1}s, Encode: {:.1}s, Total: {:.1}s, Confidence: {:.0}%",
          estimated_render_time, estimated_encode_time, estimated_total_time, confidence * 100.0);
    Ok(estimate)
}

/// Get rendering performance statistics
#[tauri::command]
pub async fn rendering_get_performance_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    debug!("Getting rendering performance statistics");


    let rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("Failed to lock rendering state: {}", e))?;


    let mut total_frames_rendered = 0u64;
    let mut total_frames = 0u64;
    let mut current_fps: f64 = 0.0;
    let mut render_time_per_frame = 0.0;
    let mut active_jobs_count = 0;

    for (_job_id, active_job) in &rendering_state.active_jobs {

        let progress = {
            let exporter = active_job.exporter.lock()
                .map_err(|e| format!("Failed to lock exporter: {}", e))?;
            exporter.get_progress()
        };

        if !progress.complete && progress.total_frames > 0 {
            active_jobs_count += 1;
            total_frames_rendered += progress.current_frame;
            total_frames += progress.total_frames;


            if progress.current_time > 0.0 {
                let fps = progress.current_frame as f64 / progress.current_time;
                current_fps = current_fps.max(fps);


                render_time_per_frame = progress.current_time / progress.current_frame as f64;
            }
        }
    }


    let average_fps = if active_jobs_count > 0 && total_frames_rendered > 0 {
        current_fps / active_jobs_count as f64
    } else {
        0.0
    };

    let stats = serde_json::json!({
        "current_fps": current_fps,
        "average_fps": average_fps,
        "render_time_per_frame_ms": render_time_per_frame * 1000.0,
        "frames_rendered": total_frames_rendered,
        "total_frames": total_frames,
        "active_jobs": active_jobs_count,
        "estimated_completion_time": if total_frames > 0 && total_frames_rendered > 0 {
            let remaining_frames = total_frames - total_frames_rendered;
            let estimated_seconds = remaining_frames as f64 / current_fps.max(1.0);
            let completion_time = chrono::Utc::now() + chrono::Duration::seconds(estimated_seconds as i64);
            Some(completion_time.to_rfc3339())
        } else {
            None
        }
    });

    info!("Rendering performance stats: {:.1} fps, {:.2}ms per frame, {} active jobs",
          stats["current_fps"], render_time_per_frame * 1000.0, active_jobs_count);
    Ok(stats)
}

#[tauri::command]
pub async fn rendering_apply_lut(
    lut_path: String,
    format: String,
    state: State<'_, AppState>,
) -> Result<RenderingResponse, String> {
    debug!("Applying LUT: {} format: {}", lut_path, format);

    if lut_path.is_empty() {
        return Err("LUT path cannot be empty".to_string());
    }

    let path = std::path::PathBuf::from(&lut_path);
    if !path.exists() {
        return Err(format!("LUT file not found: {}", lut_path));
    }

    let lut_format = match format.to_lowercase().as_str() {
        "cube" => aether_core::modules::color_grading::LutFormat::Cube,
        "3dl" => aether_core::modules::color_grading::LutFormat::ThreeDL,
        "look" => aether_core::modules::color_grading::LutFormat::Look,
        _ => return Err(format!("Unsupported LUT format: {}", format)),
    };

    let mut color_engine = state.color_grading_engine.lock()
        .map_err(|e| format!("Failed to lock color grading engine: {}", e))?;

    color_engine.load_lut(&path, lut_format)
        .map_err(|e| format!("Failed to load LUT: {}", e))?;

    info!("LUT applied successfully: {}", lut_path);
    Ok(RenderingResponse {
        success: true,
        message: format!("LUT applied: {}", lut_path),
        data: Some(serde_json::json!({
            "lut_path": lut_path,
            "format": format
        })),
    })
}

#[tauri::command]
pub async fn rendering_remove_lut(
    state: State<'_, AppState>,
) -> Result<RenderingResponse, String> {
    debug!("Removing LUT from color grading pipeline");

    let mut color_engine = state.color_grading_engine.lock()
        .map_err(|e| format!("Failed to lock color grading engine: {}", e))?;

    color_engine.clear_lut()
        .map_err(|e| format!("Failed to clear LUT: {}", e))?;

    info!("LUT removed successfully");
    Ok(RenderingResponse {
        success: true,
        message: "LUT removed successfully".to_string(),
        data: None,
    })
}

#[tauri::command]
pub async fn rendering_start_batch_job(
    requests: Vec<RenderingRequest>,
    state: State<'_, AppState>,
) -> Result<RenderingResponse, String> {
    debug!("Starting batch rendering with {} jobs", requests.len());

    if requests.is_empty() {
        return Err("Batch request cannot be empty".to_string());
    }

    let mut rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("Failed to lock rendering state: {}", e))?;

    let mut job_ids = Vec::new();
    let mut failed = 0;

    for request in requests {
        let job_id = format!("render_{}", uuid::Uuid::new_v4());
        let now = chrono::Utc::now().to_rfc3339();

        let export_options = ExportOptions {
            input_path: std::path::PathBuf::from("/timeline/current"),
            output_path: std::path::PathBuf::from(&request.output_path),
            container_format: convert_render_format(&request.format),
            video_format: VideoFormat::H264,
            audio_format: AudioFormat::Aac,
            video_bitrate: request.video_settings.as_ref().map(|v| v.bitrate).unwrap_or(5000000),
            audio_bitrate: 128000,
            frame_rate: request.fps.unwrap_or(30.0),
            width: request.resolution.unwrap_or((1920, 1080)).0,
            height: request.resolution.unwrap_or((1920, 1080)).1,
            encoder_preset: convert_render_quality(&request.quality),
            crf: 23,
            hardware_acceleration: false,
            threads: 0,
        };

        let exporter: Box<dyn ExporterTrait> = {
            let ffmpeg_exporter = aether_core::engine::rendering::Exporter::new(export_options)
                .map_err(|e| format!("Failed to create exporter: {}", e))?;
            Box::new(ffmpeg_exporter)
        };

        let progress = Arc::new(Mutex::new(ExportProgress {
            current_frame: 0,
            total_frames: 0,
            current_time: 0.0,
            total_duration: 0.0,
            percent: 0.0,
            complete: false,
            error: None,
        }));

        let active_job = ActiveRenderingJob {
            job: RenderingJob {
                id: job_id.clone(),
                name: request.name.clone(),
                status: RenderingStatus::Pending,
                progress: 0.0,
                current_frame: 0,
                total_frames: 0,
                start_time: now.clone(),
                end_time: None,
                output_path: request.output_path.clone(),
                format: request.format.clone(),
                quality: request.quality.clone(),
                resolution: request.resolution.unwrap_or((1920, 1080)),
                fps: request.fps.unwrap_or(30.0),
                bitrate: request.video_settings.as_ref().map(|v| v.bitrate).unwrap_or(5000000),
                estimated_size: 1024 * 1024 * 250,
                actual_size: None,
                error_message: None,
            },
            exporter: Arc::new(Mutex::new(exporter)),
            progress: progress.clone(),
        };

        rendering_state.queued_jobs.insert(job_id.clone(), active_job);
        job_ids.push(job_id.clone());
    }

    info!("Batch rendering queued {} jobs", job_ids.len());
    Ok(RenderingResponse {
        success: true,
        message: format!("Batch rendering queued {} jobs", job_ids.len()),
        data: Some(serde_json::json!({
            "job_ids": job_ids,
            "total": job_ids.len(),
            "failed": failed
        })),
    })
}

#[tauri::command]
pub async fn rendering_cleanup_completed(
    older_than_hours: Option<u32>,
    keep_count: Option<u32>,
    state: State<'_, AppState>,
) -> Result<RenderingResponse, String> {
    debug!("Cleaning up completed rendering jobs");

    let older_than = older_than_hours.unwrap_or(24);
    let keep = keep_count.unwrap_or(10);
    let cutoff = chrono::Utc::now() - chrono::Duration::hours(older_than as i64);

    let mut rendering_state = state.rendering_state.lock()
        .map_err(|e| format!("Failed to lock rendering state: {}", e))?;

    let mut removed = 0;
    let mut kept = 0;
    let job_ids: Vec<String> = rendering_state.active_jobs.keys().cloned().collect();

    for job_id in job_ids {
        if let Some(job) = rendering_state.active_jobs.get(&job_id) {
            let is_complete = {
                let exporter = job.exporter.lock()
                    .map_err(|e| format!("Failed to lock exporter: {}", e))?;
                exporter.is_complete()
            };

            if is_complete {
                let end_time = job.job.end_time.as_ref()
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));

                let should_remove = end_time.map(|t| t < cutoff).unwrap_or(true);
                if should_remove && kept >= keep {
                    rendering_state.active_jobs.remove(&job_id);
                    removed += 1;
                } else {
                    kept += 1;
                }
            }
        }
    }

    info!("Cleaned up {} completed rendering jobs older than {} hours, kept {}",
          removed, older_than, kept);

    Ok(RenderingResponse {
        success: true,
        message: format!("Cleaned up {} completed jobs, kept {}", removed, kept),
        data: Some(serde_json::json!({
            "removed": removed,
            "kept": kept,
            "older_than_hours": older_than,
            "keep_count": keep
        })),
    })
}


#[derive(Debug, Serialize)]
pub struct RenderFormatInfo {
    pub format: RenderFormat,
    pub name: String,
    pub description: String,
    pub extensions: Vec<String>,
    pub supports_video: bool,
    pub supports_audio: bool,
    pub recommended_for: Vec<String>,
    pub max_resolution: Option<(u32, u32)>,
    pub max_fps: Option<f64>,
    pub max_bitrate: Option<u32>,
}


#[derive(Debug, Serialize)]
pub struct RenderPresetInfo {
    pub name: String,
    pub description: String,
    pub format: RenderFormat,
    pub resolution: (u32, u32),
    pub fps: f64,
    pub bitrate: u32,
    pub quality: RenderQuality,
    pub preset: RenderPreset,
    pub video_codec: VideoCodec,
    pub audio_codec: AudioCodec,
}


#[derive(Debug, Serialize)]
pub struct RenderingTimeEstimate {
    pub estimated_seconds: f64,
    pub estimated_minutes: f64,
    pub estimated_hours: f64,
    pub confidence: f64,
    pub factors_used: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendering_request_validation() {

        let request = RenderingRequest {
            name: "".to_string(),
            output_path: "/exports/video.mp4".to_string(),
            format: RenderFormat::Mp4,
            quality: RenderQuality::Medium,
            resolution: None,
            fps: None,
            start_time: None,
            end_time: None,
            video_settings: None,
            audio_settings: None,
            export_range: None,
            metadata: None,
        };
        assert!(request.name.is_empty());
    }

    #[test]
    fn test_rendering_time_estimation() {
        let estimate = RenderingTimeEstimate {
            estimated_seconds: 120.0,
            estimated_minutes: 2.0,
            estimated_hours: 0.033,
            confidence: 0.85,
            factors_used: vec!["resolution".to_string(), "fps".to_string()],
        };

        assert!(estimate.estimated_seconds > 0.0);
        assert!(estimate.estimated_minutes > 0.0);
        assert!(estimate.confidence > 0.0);
    }

    #[test]
    fn test_render_format_serialization() {
        let format = RenderFormat::Mp4;
        assert_eq!(format!("{:?}", format), "Mp4");
    }

    #[test]
    fn test_render_quality_serialization() {
        let quality = RenderQuality::High;
        assert_eq!(format!("{:?}", quality), "High");
    }

    #[test]
    fn test_render_status_serialization() {
        let status = RenderingStatus::Rendering;
        assert_eq!(format!("{:?}", status), "Rendering");
    }
}
