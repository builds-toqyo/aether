use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use gstreamer as gst;
use gstreamer_editing_services as ges;
use crate::engine::editing::types::EditingError;

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub output_path: PathBuf,
    
    pub container: String,
    
    pub video_codec: String,
    
    pub audio_codec: String,
    
    pub video_bitrate: u32,
    
    pub audio_bitrate: u32,
    
    pub frame_rate: f64,
    
    pub width: u32,
    
    pub height: u32,
    
    pub hardware_acceleration: bool,
    
    pub start_time: i64,
    
    pub end_time: i64,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            output_path: PathBuf::new(),
            container: "mkv".to_string(),
            video_codec: "libx264".to_string(),
            audio_codec: "flac".to_string(),
            video_bitrate: 0,
            audio_bitrate: 0,
            frame_rate: 30.0,
            width: 0,
            height: 0,
            hardware_acceleration: false,
            start_time: 0,
            end_time: -1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportProgress {
    pub position: i64,
    
    pub duration: i64,
    
    pub percent: f64,
    
    pub complete: bool,
    
    pub error: Option<String>,
}

pub struct IntermediateExporter {
    timeline: ges::Timeline,
    
    options: ExportOptions,
    
    pipeline: Option<gst::Pipeline>,
    
    progress: Arc<Mutex<ExportProgress>>,
    
    progress_callback: Option<Arc<Mutex<dyn Fn(ExportProgress) + Send + 'static>>>,
}

impl IntermediateExporter {
    pub fn new(timeline: ges::Timeline, options: ExportOptions) -> Result<Self, EditingError> {
        let progress = Arc::new(Mutex::new(ExportProgress {
            position: 0,
            duration: 0,
            percent: 0.0,
            complete: false,
            error: None,
        }));
        
        Ok(Self {
            timeline,
            options,
            pipeline: None,
            progress,
            progress_callback: None,
        })
    }
    
    pub fn set_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(ExportProgress) + Send + 'static,
    {
        self.progress_callback = Some(Arc::new(Mutex::new(callback)));
    }
    
    pub fn start_export(&mut self) -> Result<(), EditingError> {
        let output_uri = gst::filename_to_uri(&self.options.output_path)?;
        
        let profile = self.create_encoding_profile()?;
        
        let pipeline = gst::Pipeline::new(None);
        
        let filesink = gst::ElementFactory::make("filesink")
            .name("export_sink")
            .property("location", &self.options.output_path.to_string_lossy().to_string())
            .build()
            .map_err(|_| EditingError::ExportError("Failed to create filesink".to_string()))?;
        
        // Create encodebin
        let encodebin = gst::ElementFactory::make("encodebin")
            .name("encoder")
            .property("profile", &profile)
            .build()
            .map_err(|_| EditingError::ExportError("Failed to create encodebin".to_string()))?;
        
        // Add elements to pipeline
        pipeline.add_many(&[&encodebin, &filesink])?;
        gst::Element::link_many(&[&encodebin, &filesink])?;
        
        let ges_pipeline = ges::Pipeline::new()?;
        ges_pipeline.set_timeline(&self.timeline)?;
        
        let src_pad = ges_pipeline.get_video_pad()?;
        let sink_pad = encodebin.static_pad("video_0").unwrap();
        src_pad.link(&sink_pad)?;
        
        let src_pad = ges_pipeline.get_audio_pad()?;
        let sink_pad = encodebin.static_pad("audio_0").unwrap();
        src_pad.link(&sink_pad)?;
        
        let progress = self.progress.clone();
        let callback = self.progress_callback.clone();
        
        let bus = pipeline.bus().unwrap();
        let _watch_id = bus.add_watch(move |_, msg| {
            match msg.view() {
                gst::MessageView::Eos(..) => {
                    // Export complete
                    let mut progress = progress.lock().unwrap();
                    progress.complete = true;
                    progress.percent = 100.0;
                    
                    if let Some(callback) = &callback {
                        callback.lock().unwrap()(progress.clone());
                    }
                },
                gst::MessageView::Error(err) => {
                    let mut progress = progress.lock().unwrap();
                    progress.error = Some(format!("{}: {}", err.error(), err.debug().unwrap_or_default()));
                    
                    if let Some(callback) = &callback {
                        callback.lock().unwrap()(progress.clone());
                    }
                },
                gst::MessageView::StateChanged(state_changed) => {
                    // Only interested in pipeline state changes
                    if state_changed.src().map(|s| s == pipeline.upcast_ref::<gst::Object>()).unwrap_or(false) {
                        if state_changed.current() == gst::State::Playing {
                            // Pipeline started playing
                        }
                    }
                },
                _ => (),
            }
            
            glib::Continue(true)
        })
        .expect("Failed to add bus watch");
        
        let progress = self.progress.clone();
        let callback = self.progress_callback.clone();
        let timeline_duration = self.timeline.get_duration();
        
        let _timeout_id = glib::timeout_add_seconds(1, move || {
            if let Some(position) = pipeline.query_position::<gst::ClockTime>() {
                let mut progress_guard = progress.lock().unwrap();
                progress_guard.position = position.nseconds() as i64;
                progress_guard.duration = timeline_duration;
                
                if timeline_duration > 0 {
                    progress_guard.percent = (progress_guard.position as f64 / timeline_duration as f64) * 100.0;
                }
                
                if let Some(callback) = &callback {
                    callback.lock().unwrap()(progress_guard.clone());
                }
            }
            
            glib::Continue(true)
        });
        
        pipeline.set_state(gst::State::Playing)?;
        
        self.pipeline = Some(pipeline);
        
        Ok(())
    }
    
    fn create_encoding_profile(&self) -> Result<gst_pbutils::EncodingContainerProfile, EditingError> {
        let container_caps = gst::Caps::builder(&format!("video/{}", self.options.container)).build();
        let container_profile = gst_pbutils::EncodingContainerProfile::new(
            Some("export-profile"),
            Some("Export Profile"),
            &container_caps,
            None,
        ).ok_or(EditingError::ExportError("Failed to create container profile".to_string()))?;
        
        let video_caps = gst::Caps::builder("video/x-raw")
            .field("format", "I420")
            .build();
        
        let video_codec_caps = gst::Caps::builder(&format!("video/{}", self.options.video_codec)).build();
        let video_profile = gst_pbutils::EncodingVideoProfile::new(
            &video_codec_caps,
            None,
            &video_caps,
            1,
        ).ok_or(EditingError::ExportError("Failed to create video profile".to_string()))?;
        
        if self.options.video_bitrate > 0 {
            video_profile.set_bitrate(self.options.video_bitrate);
        }
        
        let audio_caps = gst::Caps::builder("audio/x-raw")
            .field("format", "S16LE")
            .build();
        
        let audio_codec_caps = gst::Caps::builder(&format!("audio/{}", self.options.audio_codec)).build();
        let audio_profile = gst_pbutils::EncodingAudioProfile::new(
            &audio_codec_caps,
            None,
            &audio_caps,
            1,
        ).ok_or(EditingError::ExportError("Failed to create audio profile".to_string()))?;
        
        if self.options.audio_bitrate > 0 {
            audio_profile.set_bitrate(self.options.audio_bitrate);
        }
        
        container_profile.add_profile(&video_profile.upcast())?;
        container_profile.add_profile(&audio_profile.upcast())?;
        
        Ok(container_profile)
    }
    
    pub fn cancel_export(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null)?;
            
            let mut progress = self.progress.lock().unwrap();
            progress.complete = true;
            progress.error = Some("Export cancelled".to_string());
            
            if let Some(callback) = &self.progress_callback {
                callback.lock().unwrap()(progress.clone());
            }
        }
        
        self.pipeline = None;
        
        Ok(())
    }
    
    pub fn get_progress(&self) -> ExportProgress {
        self.progress.lock().unwrap().clone()
    }
}

impl Drop for IntermediateExporter {
    fn drop(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }
}
