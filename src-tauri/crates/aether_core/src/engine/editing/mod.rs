mod timeline;
mod import;
mod preview;
mod effects;
mod export;
mod types;

pub use timeline::{Timeline, TimelineTrack, TimelineClip, TimelineEffect};
pub use import::{MediaImporter, ImportOptions};
pub use preview::{PreviewEngine, PreviewFrame};
pub use effects::{Effect, EffectType, Transition, TransitionType};
pub use export::{IntermediateExporter, ExportOptions, ExportProgress};
pub use types::{EditingError, MediaInfo, ClipInfo, TrackType};

use std::sync::{Arc, Mutex};
use anyhow::Result;
use gstreamer as gst;
use gstreamer_editing_services as ges;

pub struct EditingEngine {
    ges_timeline: Option<ges::Timeline>,
    ges_pipeline: Option<ges::Pipeline>,
    
    initialized: bool,
    project_path: Option<String>,
    
    importer: Arc<Mutex<MediaImporter>>,
    preview_engine: Arc<Mutex<PreviewEngine>>,
    timeline: Arc<Mutex<Timeline>>,
}

impl EditingEngine {
    pub fn new() -> Result<Self, EditingError> {
        gst::init()?;
        
        ges::init()?;
        
        let importer = Arc::new(Mutex::new(MediaImporter::new()?));
        let preview_engine = Arc::new(Mutex::new(PreviewEngine::new()?));
        let timeline = Arc::new(Mutex::new(Timeline::new()?));
        
        Ok(Self {
            ges_timeline: None,
            ges_pipeline: None,
            initialized: true,
            project_path: None,
            importer,
            preview_engine,
            timeline,
        })
    }
    
    pub fn init_project(&mut self, project_path: Option<String>) -> Result<(), EditingError> {
        let timeline = ges::Timeline::new_audio_video()?;
        
        let pipeline = ges::Pipeline::new()?;
        pipeline.set_timeline(&timeline)?;
        
        self.ges_timeline = Some(timeline);
        self.ges_pipeline = Some(pipeline);
        self.project_path = project_path;
        
        if let Some(timeline) = &self.ges_timeline {
            self.timeline.lock().unwrap().set_ges_timeline(timeline.clone())?;
            self.preview_engine.lock().unwrap().set_pipeline(self.ges_pipeline.clone())?;
        }
        
        Ok(())
    }
    
    pub fn timeline(&self) -> Arc<Mutex<Timeline>> {
        self.timeline.clone()
    }
    
    pub fn importer(&self) -> Arc<Mutex<MediaImporter>> {
        self.importer.clone()
    }
    
    pub fn preview(&self) -> Arc<Mutex<PreviewEngine>> {
        self.preview_engine.clone()
    }
    
    pub fn create_intermediate_export(&self, options: ExportOptions) -> Result<IntermediateExporter, EditingError> {
        let exporter = IntermediateExporter::new(
            self.ges_timeline.clone().ok_or(EditingError::NotInitialized)?,
            options
        )?;
        
        Ok(exporter)
    }
    
    pub fn shutdown(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.ges_pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
        
        self.ges_pipeline = None;
        self.ges_timeline = None;
        self.initialized = false;
        
        Ok(())
    }
}

impl Drop for EditingEngine {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

pub fn create_editing_engine() -> Result<EditingEngine, EditingError> {
    EditingEngine::new()
}
