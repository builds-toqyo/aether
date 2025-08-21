mod export;
mod formats;
mod encoder;

pub use export::{Exporter, ExportOptions, ExportProgress, ExportCallback};
pub use formats::{VideoFormat, AudioFormat, ContainerFormat, get_available_formats};
pub use encoder::{EncoderPreset, EncoderOptions};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use crate::engine::editing::types::EditingError;

pub struct RenderingEngine {
    initialized: bool,
    current_export: Option<Arc<Mutex<Exporter>>>,
}

impl RenderingEngine {
    pub fn new() -> Result<Self, EditingError> {    
        Ok(Self {
            initialized: true,
            current_export: None,
        })
    }
    
    pub fn create_export(&mut self, options: ExportOptions) -> Result<Arc<Mutex<Exporter>>, EditingError> {
        let exporter = Arc::new(Mutex::new(Exporter::new(options)?));
        self.current_export = Some(exporter.clone());
        
        Ok(exporter)
    }
    
    pub fn current_export(&self) -> Option<Arc<Mutex<Exporter>>> {
        self.current_export.clone()
    }
    
    pub fn cancel_export(&mut self) -> Result<(), EditingError> {
        if let Some(exporter) = &self.current_export {
            exporter.lock().unwrap().cancel()?;
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
