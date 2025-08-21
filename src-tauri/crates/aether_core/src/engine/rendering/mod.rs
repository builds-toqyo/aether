mod export;
mod formats;
mod encoder;
mod gst_exporter;

pub use export::{Exporter, ExportOptions, ExportProgress, ExportCallback};
pub use formats::{VideoFormat, AudioFormat, ContainerFormat, get_available_formats};
pub use encoder::{EncoderPreset, EncoderOptions};
pub use gst_exporter::{GstExporter, ExportProgress as GstExportProgress, ExportOptions as GstExportOptions, ExportCallback as GstExportCallback};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use crate::engine::editing::types::EditingError;

/// Enum to represent the different types of exporters
pub enum ExporterType {
    /// FFmpeg-based exporter
    FFmpeg,
    /// GStreamer-based exporter
    GStreamer,
}

/// Enum to hold either type of exporter
pub enum ActiveExporter {
    /// FFmpeg-based exporter
    FFmpeg(Arc<Mutex<Exporter>>),
    /// GStreamer-based exporter
    GStreamer(Arc<Mutex<GstExporter>>),
}

pub struct RenderingEngine {
    initialized: bool,
    current_export: Option<ActiveExporter>,
    /// Default exporter type to use
    default_exporter_type: ExporterType,
}

impl RenderingEngine {
    pub fn new() -> Result<Self, EditingError> {    
        Ok(Self {
            initialized: true,
            current_export: None,
            default_exporter_type: ExporterType::FFmpeg, // Default to FFmpeg for backward compatibility
        })
    }
    
    /// Set the default exporter type
    pub fn set_default_exporter_type(&mut self, exporter_type: ExporterType) {
        self.default_exporter_type = exporter_type;
    }
    
    /// Create an FFmpeg-based exporter
    pub fn create_ffmpeg_export(&mut self, options: ExportOptions) -> Result<Arc<Mutex<Exporter>>, EditingError> {
        let exporter = Arc::new(Mutex::new(Exporter::new(options)?));
        self.current_export = Some(ActiveExporter::FFmpeg(exporter.clone()));
        
        Ok(exporter)
    }
    
    /// Create a GStreamer-based exporter
    pub fn create_gstreamer_export(&mut self, options: GstExportOptions) -> Result<Arc<Mutex<GstExporter>>, EditingError> {
        let exporter = Arc::new(Mutex::new(GstExporter::new(options)?));
        self.current_export = Some(ActiveExporter::GStreamer(exporter.clone()));
        
        Ok(exporter)
    }
    
    /// Create an exporter using the default exporter type
    pub fn create_export(&mut self, options: ExportOptions) -> Result<ActiveExporter, EditingError> {
        match self.default_exporter_type {
            ExporterType::FFmpeg => {
                let exporter = self.create_ffmpeg_export(options)?;
                Ok(ActiveExporter::FFmpeg(exporter))
            },
            ExporterType::GStreamer => {
                // Convert FFmpeg options to GStreamer options
                // This is a simplified conversion and might need more fields
                let gst_options = GstExportOptions {
                    timeline: ges::Timeline::new(), // This needs to be set by the caller
                    output_path: options.output_path,
                    container_format: options.container_format,
                    video_format: options.video_format,
                    audio_format: options.audio_format,
                    video_bitrate: options.video_bitrate,
                    audio_bitrate: options.audio_bitrate,
                    frame_rate: options.frame_rate,
                    width: options.width,
                    height: options.height,
                    encoder_preset: options.encoder_preset,
                    crf: options.crf,
                    hardware_acceleration: options.hardware_acceleration,
                    threads: options.threads,
                };
                
                let exporter = self.create_gstreamer_export(gst_options)?;
                Ok(ActiveExporter::GStreamer(exporter))
            },
        }
    }
    
    /// Get the current export if any
    pub fn current_export(&self) -> Option<&ActiveExporter> {
        self.current_export.as_ref()
    }
    
    /// Cancel the current export if any
    pub fn cancel_export(&mut self) -> Result<(), EditingError> {
        if let Some(exporter) = &self.current_export {
            match exporter {
                ActiveExporter::FFmpeg(ffmpeg_exporter) => {
                    ffmpeg_exporter.lock().unwrap().cancel()?;
                },
                ActiveExporter::GStreamer(gst_exporter) => {
                    gst_exporter.lock().unwrap().cancel_export()?;
                },
            }
            self.current_export = None;
        }
        
        Ok(())
    }
    
    pub fn shutdown(&mut self) -> Result<(), EditingError> {
        let _ = self.cancel_export();
        
        self.initialized = false;
        
        Ok(())
    }
}

impl Drop for RenderingEngine {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

pub fn create_rendering_engine() -> Result<RenderingEngine, EditingError> {
    RenderingEngine::new()
}
