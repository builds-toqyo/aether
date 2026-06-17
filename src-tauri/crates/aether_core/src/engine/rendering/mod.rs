pub mod export;
pub mod formats;
pub mod encoder;
pub mod gst_exporter;

pub use export::{Exporter, ExportOptions, ExportProgress, ExportCallback};
pub use formats::{VideoFormat, AudioFormat, ContainerFormat, get_available_formats};
pub use encoder::{EncoderPreset, EncoderOptions};
pub use gst_exporter::{GstExporter, ExportProgress as GstExportProgress, ExportOptions as GstExportOptions, ExportCallback as GstExportCallback};

use std::sync::{Arc, Mutex};
use anyhow::Result;
use crate::engine::editing::types::EditingError;

pub enum ExporterType {
    FFmpeg,
}

pub enum ActiveExporter {
    FFmpeg(Arc<Mutex<Exporter>>),
}

pub struct RenderingEngine {
    initialized: bool,
    current_export: Option<ActiveExporter>,
    default_exporter_type: ExporterType,
}

impl RenderingEngine {
    pub fn new() -> Result<Self, EditingError> {
        Ok(Self {
            initialized: true,
            current_export: None,
            default_exporter_type: ExporterType::FFmpeg,
        })
    }


    pub fn set_default_exporter_type(&mut self, exporter_type: ExporterType) {
        self.default_exporter_type = exporter_type;
    }


    pub fn create_ffmpeg_export(&mut self, options: ExportOptions) -> Result<Arc<Mutex<Exporter>>, EditingError> {
        let exporter = Arc::new(Mutex::new(Exporter::new(options)?));
        self.current_export = Some(ActiveExporter::FFmpeg(exporter.clone()));
        Ok(exporter)
    }

    pub fn create_export(&mut self, options: ExportOptions) -> Result<ActiveExporter, EditingError> {
        let exporter = self.create_ffmpeg_export(options)?;
        Ok(ActiveExporter::FFmpeg(exporter))
    }


    pub fn current_export(&self) -> Option<&ActiveExporter> {
        self.current_export.as_ref()
    }


    pub fn cancel_export(&mut self) -> Result<(), EditingError> {
        if let Some(ActiveExporter::FFmpeg(ffmpeg_exporter)) = &self.current_export {
            ffmpeg_exporter.lock().unwrap().cancel()?;
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
