use std::path::PathBuf;
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum EditingError {
    #[error("GStreamer initialization failed: {0}")]
    GstreamerInitError(String),
    
    #[error("GES initialization failed: {0}")]
    GesInitError(String),
    
    #[error("Media import failed: {0}")]
    ImportError(String),
    
    #[error("Timeline operation failed: {0}")]
    TimelineError(String),
    
    #[error("Preview operation failed: {0}")]
    PreviewError(String),
    
    #[error("Export operation failed: {0}")]
    ExportError(String),
    
    #[error("Effect application failed: {0}")]
    EffectError(String),
    
    #[error("Engine not initialized")]
    NotInitialized,
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("GStreamer error: {0}")]
    GstreamerError(String),
}

impl From<gstreamer::glib::Error> for EditingError {
    fn from(err: gstreamer::glib::Error) -> Self {
        EditingError::GstreamerError(err.to_string())
    }
}

impl From<gstreamer::glib::BoolError> for EditingError {
    fn from(err: gstreamer::glib::BoolError) -> Self {
        EditingError::GstreamerError(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub path: PathBuf,
    
    pub duration: i64,
    
    pub title: Option<String>,
    
    pub media_type: MediaType,
    
    pub video_streams: Vec<VideoStreamInfo>,
    
    pub audio_streams: Vec<AudioStreamInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStreamInfo {
    pub index: i32,
    
    pub width: i32,
    
    pub height: i32,
    
    pub frame_rate: f64,
    
    pub codec_name: String,
    
    pub pixel_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamInfo {
    pub index: i32,
    
    pub sample_rate: i32,
    
    pub channels: i32,
    
    pub codec_name: String,
    
    pub bit_depth: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipInfo {
    pub id: String,
    
    pub name: String,
    
    pub source_path: Option<PathBuf>,
    
    pub start_time: i64,
    
    pub duration: i64,
    
    pub in_point: i64,
    
    pub out_point: i64,
    
    pub track_type: TrackType,
    
    pub effects: Vec<EffectInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackType {
    Video,
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectInfo {
    pub id: String,
    
    pub name: String,
    
    pub effect_type: String,
    
    pub parameters: std::collections::HashMap<String, String>,
    
    pub start_time: i64,
    
    pub duration: i64,
}
