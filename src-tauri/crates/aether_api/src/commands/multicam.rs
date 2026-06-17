use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use log::{debug, info, warn};
use tauri::State;
use crate::state::AppState;

/// Represents a single camera angle within a multicam group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraAngle {
    pub id: String,
    pub name: String,
    pub media_id: String,
    pub offset_ms: i64,
    pub enabled: bool,
    pub metadata: HashMap<String, String>,
}

impl CameraAngle {
    pub fn new(name: impl Into<String>, media_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            media_id: media_id.into(),
            offset_ms: 0,
            enabled: true,
            metadata: HashMap::new(),
        }
    }
}

/// A multicam clip groups multiple synchronized camera angles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MulticamClip {
    pub id: String,
    pub name: String,
    pub angles: Vec<CameraAngle>,
    pub active_angle_id: Option<String>,
    pub duration_ms: i64,
    pub metadata: HashMap<String, String>,
}

impl MulticamClip {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            angles: Vec::new(),
            active_angle_id: None,
            duration_ms: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn add_angle(&mut self, angle: CameraAngle) {
        if self.active_angle_id.is_none() {
            self.active_angle_id = Some(angle.id.clone());
        }
        self.angles.push(angle);
    }

    pub fn remove_angle(&mut self, angle_id: &str) {
        self.angles.retain(|a| a.id != angle_id);
        if self.active_angle_id.as_deref() == Some(angle_id) {
            self.active_angle_id = self.angles.first().map(|a| a.id.clone());
        }
    }

    pub fn set_active_angle(&mut self, angle_id: &str) -> Result<(), String> {
        if self.angles.iter().any(|a| a.id == angle_id) {
            self.active_angle_id = Some(angle_id.to_string());
            Ok(())
        } else {
            Err(format!("Angle '{}' not found in multicam clip", angle_id))
        }
    }

    pub fn get_active_angle(&self) -> Option<&CameraAngle> {
        self.active_angle_id.as_ref()
            .and_then(|id| self.angles.iter().find(|a| a.id == *id))
    }
}

/// In-memory registry for multicam clips
#[derive(Debug, Default)]
pub struct MulticamRegistry {
    pub clips: HashMap<String, MulticamClip>,
}

impl MulticamRegistry {
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MulticamResponse {
    pub success: bool,
    pub message: String,
    pub clip: Option<MulticamClip>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMulticamRequest {
    pub name: String,
    pub angle_media_ids: Vec<String>,
}

#[tauri::command]
pub async fn multicam_create(
    request: CreateMulticamRequest,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    debug!("Creating multicam clip: {}", request.name);

    let mut clip = MulticamClip::new(&request.name);
    for (i, media_id) in request.angle_media_ids.iter().enumerate() {
        let angle = CameraAngle::new(format!("Camera {}", i + 1), media_id.clone());
        clip.add_angle(angle);
    }

    let clip_id = clip.id.clone();
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;
    registry.clips.insert(clip_id.clone(), clip.clone());

    info!("Created multicam clip '{}' with {} angles", request.name, request.angle_media_ids.len());

    Ok(MulticamResponse {
        success: true,
        message: format!("Created multicam clip {}", clip_id),
        clip: Some(clip),
    })
}

#[tauri::command]
pub async fn multicam_add_angle(
    clip_id: String,
    name: String,
    media_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    let clip = registry.clips.get_mut(&clip_id)
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))?;

    let angle = CameraAngle::new(name, media_id);
    clip.add_angle(angle);

    Ok(MulticamResponse {
        success: true,
        message: format!("Added angle to multicam clip {}", clip_id),
        clip: Some(clip.clone()),
    })
}

#[tauri::command]
pub async fn multicam_remove_angle(
    clip_id: String,
    angle_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    let clip = registry.clips.get_mut(&clip_id)
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))?;

    clip.remove_angle(&angle_id);

    Ok(MulticamResponse {
        success: true,
        message: format!("Removed angle {} from multicam clip {}", angle_id, clip_id),
        clip: Some(clip.clone()),
    })
}

#[tauri::command]
pub async fn multicam_set_active_angle(
    clip_id: String,
    angle_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    let clip = registry.clips.get_mut(&clip_id)
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))?;

    clip.set_active_angle(&angle_id)?;

    Ok(MulticamResponse {
        success: true,
        message: format!("Set active angle {} on multicam clip {}", angle_id, clip_id),
        clip: Some(clip.clone()),
    })
}

#[tauri::command]
pub async fn multicam_get_clip(
    clip_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamClip, String> {
    let registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    registry.clips.get(&clip_id)
        .cloned()
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))
}

#[tauri::command]
pub async fn multicam_list_clips(
    state: State<'_, AppState>,
) -> Result<Vec<MulticamClip>, String> {
    let registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    Ok(registry.clips.values().cloned().collect())
}

#[tauri::command]
pub async fn multicam_delete_clip(
    clip_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    registry.clips.remove(&clip_id);

    Ok(MulticamResponse {
        success: true,
        message: format!("Deleted multicam clip {}", clip_id),
        clip: None,
    })
}

#[derive(Debug, Deserialize)]
pub struct Timecode {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub frames: u32,
    pub frame_rate: f64,
}

impl Timecode {
    pub fn from_string(tc_str: &str, frame_rate: f64) -> Result<Self, String> {
        let parts: Vec<&str> = tc_str.split(':').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid timecode format: {}. Expected HH:MM:SS:FF", tc_str));
        }

        let hours = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid hours: {}", e))?;
        let minutes = parts[1].parse::<u32>()
            .map_err(|e| format!("Invalid minutes: {}", e))?;
        let seconds = parts[2].parse::<u32>()
            .map_err(|e| format!("Invalid seconds: {}", e))?;
        let frames = parts[3].parse::<u32>()
            .map_err(|e| format!("Invalid frames: {}", e))?;

        if hours > 23 || minutes > 59 || seconds > 59 {
            return Err(format!("Invalid timecode values: {}", tc_str));
        }

        Ok(Timecode {
            hours,
            minutes,
            seconds,
            frames,
            frame_rate,
        })
    }

    pub fn to_milliseconds(&self) -> i64 {
        let total_seconds = self.hours as i64 * 3600 + self.minutes as i64 * 60 + self.seconds as i64;
        let frame_duration_ms = 1000.0 / self.frame_rate;
        total_seconds * 1000 + (self.frames as f64 * frame_duration_ms) as i64
    }

    pub fn parse_from_metadata(metadata: &HashMap<String, String>, frame_rate: f64) -> Option<Self> {
        // Try common timecode metadata keys
        let tc_keys = [
            "timecode",
            "timecode-stream",
            "timecode-source",
            "timecode-first-frame",
            "GST_TAG_TIMECODE",
        ];

        for key in &tc_keys {
            if let Some(tc_str) = metadata.get(*key) {
                if let Ok(tc) = Self::from_string(tc_str, frame_rate) {
                    return Some(tc);
                }
            }
        }

        None
    }
}

#[tauri::command]
pub async fn multicam_sync_by_timecode(
    clip_id: String,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    let clip = registry.clips.get_mut(&clip_id)
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))?;

    if clip.angles.is_empty() {
        return Err("No angles in multicam clip".to_string());
    }

    // Get file manager to extract media info with timecode metadata
    let file_manager = state.file_manager.lock()
        .map_err(|e| format!("Failed to lock file manager: {}", e))?;

    // Extract timecodes from all angles
    let mut timecodes: Vec<Option<Timecode>> = Vec::new();
    let mut frame_rates: Vec<f64> = Vec::new();

    for angle in &clip.angles {
        let media_path = std::path::PathBuf::from(&angle.media_id);
        if let Ok(media_info) = file_manager.get_media_info(&media_path) {
            let frame_rate = media_info.frame_rate.unwrap_or(30.0);
            frame_rates.push(frame_rate);

            let timecode = Timecode::parse_from_metadata(&media_info.metadata, frame_rate);
            timecodes.push(timecode);
        } else {
            timecodes.push(None);
            frame_rates.push(30.0);
        }
    }

    // Find the first angle with a valid timecode as reference
    let reference_timecode = timecodes.iter()
        .enumerate()
        .find(|(_, tc)| tc.is_some())
        .map(|(idx, tc)| (idx, tc.as_ref().unwrap()));

    if let Some((ref_idx, ref_tc)) = reference_timecode {
        let ref_ms = ref_tc.to_milliseconds();

        // Calculate offsets for all angles
        for (i, angle) in clip.angles.iter_mut().enumerate() {
            if let Some(tc) = &timecodes[i] {
                let angle_ms = tc.to_milliseconds();
                angle.offset_ms = angle_ms - ref_ms;
                info!("Angle {}: timecode offset = {} ms", i, angle.offset_ms);
            } else {
                // No timecode available, use zero offset
                angle.offset_ms = 0;
                warn!("Angle {}: no timecode metadata found, using zero offset", i);
            }
        }

        Ok(MulticamResponse {
            success: true,
            message: format!("Angles synchronized by timecode (reference: angle {})", ref_idx),
            clip: Some(clip.clone()),
        })
    } else {
        // No timecodes found, fall back to zero-offset sync
        for angle in &mut clip.angles {
            angle.offset_ms = 0;
        }

        warn!("multicam_sync_by_timecode: no timecode metadata found in any angle, using zero-offset sync");

        Ok(MulticamResponse {
            success: true,
            message: "No timecode metadata found, using zero-offset baseline".to_string(),
            clip: Some(clip.clone()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct WaveformSample {
    pub timestamp: f64,
    pub amplitude: f64,
}

#[derive(Debug, Clone)]
pub struct AudioWaveform {
    pub samples: Vec<WaveformSample>,
    pub sample_rate: u32,
    pub duration: f64,
}

impl AudioWaveform {
    pub fn new(sample_rate: u32, duration: f64) -> Self {
        let num_samples = (sample_rate as f64 * duration) as usize;
        let samples = Vec::with_capacity(num_samples);
        
        AudioWaveform {
            samples,
            sample_rate,
            duration,
        }
    }

    pub fn extract_from_audio(audio_path: &PathBuf, sample_rate: u32) -> Result<Self, String> {
        use gstreamer as gst;
        use gst::prelude::*;

        gst::init().map_err(|e| format!("Failed to initialize GStreamer: {}", e))?;

        let pipeline_str = format!(
            "filesrc location={} ! decodebin ! audioconvert ! audioresample ! audio/x-raw,rate={} ! level post-messages=true ! fakesink",
            audio_path.to_str().ok_or("Invalid path")?,
            sample_rate
        );

        let pipeline = gst::parse_launch(&pipeline_str)
            .map_err(|e| format!("Failed to create pipeline: {}", e))?;

        let bus = pipeline.bus().ok_or("Pipeline has no bus")?;
        pipeline.set_state(gst::State::Playing)
            .map_err(|e| format!("Failed to start pipeline: {}", e))?;

        let mut waveform = AudioWaveform::new(sample_rate, 0.0);
        let mut current_time = 0.0;
        let sample_duration = 1.0 / sample_rate as f64;

        let timeout = gst::ClockTime::from_seconds(60);
        while let Some(msg) = bus.timed_pop_filtered(timeout, &[gst::MessageType::Element, gst::MessageType::Eos, gst::MessageType::Error]) {
            match msg.view() {
                gst::MessageView::Element(element) => {
                    if let Some(structure) = element.structure() {
                        if structure.name() == "level" {
                            if let Ok(rms) = structure.get::<f64>("rms") {
                                if let Some(rms_value) = rms.first() {
                                    let amplitude = rms_value.abs();
                                    waveform.samples.push(WaveformSample {
                                        timestamp: current_time,
                                        amplitude,
                                    });
                                    current_time += sample_duration;
                                }
                            }
                        }
                    }
                }
                gst::MessageView::Eos(_) => {
                    break;
                }
                gst::MessageView::Error(err) => {
                    pipeline.set_state(gst::State::Null).ok();
                    return Err(format!("Pipeline error: {}", err.error()));
                }
                _ => {}
            }
        }

        pipeline.set_state(gst::State::Null)
            .map_err(|e| format!("Failed to stop pipeline: {}", e))?;

        waveform.duration = current_time;
        Ok(waveform)
    }

    pub fn compute_cross_correlation(&self, other: &AudioWaveform, max_offset_ms: f64) -> Option<i64> {
        if self.samples.is_empty() || other.samples.is_empty() {
            return None;
        }

        let max_offset_samples = (max_offset_ms / 1000.0 * self.sample_rate as f64) as usize;
        let max_offset_samples = max_offset_samples.min(self.samples.len().min(other.samples.len()));

        let mut best_offset = 0i64;
        let mut best_correlation = 0.0;

        for offset in 0..max_offset_samples {
            let mut correlation = 0.0;
            let mut count = 0;

            for i in 0..self.samples.len().saturating_sub(offset) {
                if i + offset < other.samples.len() {
                    correlation += self.samples[i].amplitude * other.samples[i + offset].amplitude;
                    count += 1;
                }
            }

            if count > 0 {
                correlation /= count as f64;
            }

            if correlation > best_correlation {
                best_correlation = correlation;
                best_offset = offset as i64;
            }
        }

        // Convert offset to milliseconds
        let offset_ms = (best_offset as f64 / self.sample_rate as f64) * 1000.0;
        Some(offset_ms as i64)
    }
}

#[tauri::command]
pub async fn multicam_sync_by_audio(
    clip_id: String,
    max_offset_ms: Option<f64>,
    state: State<'_, AppState>,
) -> Result<MulticamResponse, String> {
    let max_offset_ms = max_offset_ms.unwrap_or(5000.0); // Default 5 seconds

    let mut registry = state.multicam_registry.lock()
        .map_err(|e| format!("Failed to lock multicam registry: {}", e))?;

    let clip = registry.clips.get_mut(&clip_id)
        .ok_or_else(|| format!("Multicam clip '{}' not found", clip_id))?;

    if clip.angles.len() < 2 {
        return Err("At least 2 angles required for audio sync".to_string());
    }

    // Extract waveforms from all angles
    let mut waveforms: Vec<Option<AudioWaveform>> = Vec::new();
    let mut sample_rates: Vec<u32> = Vec::new();

    for angle in &clip.angles {
        let media_path = PathBuf::from(&angle.media_id);
        if media_path.exists() {
            let sample_rate = 48000; // Use standard sample rate
            match AudioWaveform::extract_from_audio(&media_path, sample_rate) {
                Ok(waveform) => {
                    info!("Extracted waveform from angle {} ({} samples)", angle.name, waveform.samples.len());
                    waveforms.push(Some(waveform));
                    sample_rates.push(sample_rate);
                }
                Err(e) => {
                    warn!("Failed to extract waveform from angle {}: {}", angle.name, e);
                    waveforms.push(None);
                    sample_rates.push(sample_rate);
                }
            }
        } else {
            warn!("Media file not found for angle {}: {:?}", angle.name, media_path);
            waveforms.push(None);
            sample_rates.push(48000);
        }
    }

    // Find the first angle with a valid waveform as reference
    let reference_waveform = waveforms.iter()
        .enumerate()
        .find(|(_, wf)| wf.is_some())
        .map(|(idx, wf)| (idx, wf.as_ref().unwrap()));

    if let Some((ref_idx, ref_wf)) = reference_waveform {
        // Calculate offsets for all angles using cross-correlation
        for (i, angle) in clip.angles.iter_mut().enumerate() {
            if i == ref_idx {
                angle.offset_ms = 0;
                continue;
            }

            if let Some(wf) = &waveforms[i] {
                if let Some(offset_ms) = ref_wf.compute_cross_correlation(wf, max_offset_ms) {
                    angle.offset_ms = offset_ms;
                    info!("Angle {}: audio sync offset = {} ms", i, offset_ms);
                } else {
                    // No correlation found, use zero offset
                    angle.offset_ms = 0;
                    warn!("Angle {}: no audio correlation found, using zero offset", i);
                }
            } else {
                // No waveform available, use zero offset
                angle.offset_ms = 0;
                warn!("Angle {}: no waveform available, using zero offset", i);
            }
        }

        Ok(MulticamResponse {
            success: true,
            message: format!("Angles synchronized by audio waveform (reference: angle {})", ref_idx),
            clip: Some(clip.clone()),
        })
    } else {
        // No waveforms found, fall back to zero-offset sync
        for angle in &mut clip.angles {
            angle.offset_ms = 0;
        }

        warn!("multicam_sync_by_audio: no audio waveforms extracted from any angle, using zero-offset sync");

        Ok(MulticamResponse {
            success: true,
            message: "No audio waveforms found, using zero-offset baseline".to_string(),
            clip: Some(clip.clone()),
        })
    }
}
