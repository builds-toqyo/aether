use std::sync::{Arc, Mutex};
use anyhow::Result;
use gstreamer as gst;
use gstreamer_video as gst_video;
use gstreamer_editing_services as ges;
use crate::engine::editing::types::EditingError;

#[derive(Clone)]
pub struct PreviewFrame {
    pub width: u32,
    
    pub height: u32,
    
    pub data: Vec<u8>,
    
    pub pts: i64,
    
    pub duration: i64,
}

pub struct PreviewEngine {
    pipeline: Option<ges::Pipeline>,
    
    video_sink: Option<gst::Element>,
    
    is_playing: bool,
    
    position: i64,
    
    frame_callback: Option<Arc<Mutex<dyn Fn(PreviewFrame) + Send + 'static>>>,
}

impl PreviewEngine {
    pub fn new() -> Result<Self, EditingError> {
        Ok(Self {
            pipeline: None,
            video_sink: None,
            is_playing: false,
            position: 0,
            frame_callback: None,
        })
    }
    
    pub fn set_pipeline(&mut self, pipeline: Option<ges::Pipeline>) -> Result<(), EditingError> {
        if let Some(pipeline) = pipeline {
            self.setup_preview_pipeline(&pipeline)?;
            self.pipeline = Some(pipeline);
        } else {
            if let Some(pipeline) = &self.pipeline {
                let _ = pipeline.set_state(gst::State::Null);
            }
            self.pipeline = None;
            self.video_sink = None;
        }
        
        Ok(())
    }
    
    fn setup_preview_pipeline(&mut self, pipeline: &ges::Pipeline) -> Result<(), EditingError> {
        let video_sink = gst::ElementFactory::make("appsink")
            .name("preview_sink")
            .build()
            .map_err(|_| EditingError::PreviewError("Failed to create appsink".to_string()))?;
        
        let appsink = video_sink.downcast_ref::<gst_app::AppSink>()
            .ok_or(EditingError::PreviewError("Failed to downcast to AppSink".to_string()))?;
        
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "RGB")
            .build();
        
        appsink.set_caps(Some(&caps));
        appsink.set_drop(true);
        appsink.set_max_buffers(1);
        
        let callback = self.frame_callback.clone();
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    if let Some(callback) = &callback {
                        if let Ok(sample) = appsink.pull_sample() {
                            if let Some(frame) = extract_frame_from_sample(&sample) {
                                callback.lock().unwrap()(frame);
                            }
                        }
                    }
                    Ok(gst::FlowSuccess::Ok)
                })
                .build()
        );
        
        pipeline.set_video_sink(Some(&video_sink))?;
        self.video_sink = Some(video_sink);
        
        Ok(())
    }
    
    pub fn set_frame_callback<F>(&mut self, callback: F)
    where
        F: Fn(PreviewFrame) + Send + 'static,
    {
        self.frame_callback = Some(Arc::new(Mutex::new(callback)));
    }
    
    pub fn play(&mut self) -> Result<(), EditingError> {
        let pipeline = self.pipeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;
        
        pipeline.set_state(gst::State::Playing)?;
        self.is_playing = true;
        
        Ok(())
    }
    
    pub fn pause(&mut self) -> Result<(), EditingError> {
        let pipeline = self.pipeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;
        
        pipeline.set_state(gst::State::Paused)?;
        self.is_playing = false;
        
        Ok(())
    }
    
    pub fn stop(&mut self) -> Result<(), EditingError> {
        let pipeline = self.pipeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;
        
        pipeline.set_state(gst::State::Ready)?;
        self.is_playing = false;
        self.position = 0;
        
        Ok(())
    }
    
    pub fn seek(&mut self, position: i64) -> Result<(), EditingError> {
        let pipeline = self.pipeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;
        
        let seek_flags = gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE;
        
        pipeline.seek_simple(gst::Format::Time, seek_flags, position)?;
        self.position = position;
        
        Ok(())
    }
    
    pub fn get_position(&self) -> Result<i64, EditingError> {
        if let Some(pipeline) = &self.pipeline {
            let position = pipeline.query_position::<gst::ClockTime>()
                .map(|p| p.nseconds() as i64)
                .unwrap_or(self.position);
            
            Ok(position)
        } else {
            Ok(self.position)
        }
    }
    
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
    
    pub fn get_frame(&self) -> Result<Option<PreviewFrame>, EditingError> {
        // This would require implementing a synchronous frame grabbing mechanism
        // For now, we rely on the callback mechanism
        Err(EditingError::NotSupported("Synchronous frame grabbing not implemented".to_string()))
    }
}

fn extract_frame_from_sample(sample: &gst::Sample) -> Option<PreviewFrame> {
    let buffer = sample.buffer()?;
    let caps = sample.caps()?;
    let structure = caps.structure(0)?;
    
    let width = structure.get::<i32>("width").ok()? as u32;
    let height = structure.get::<i32>("height").ok()? as u32;
    
    let map = buffer.map_readable().ok()?;
    let data = map.as_slice().to_vec();
    
    let pts = buffer.pts().map(|t| t.nseconds() as i64).unwrap_or(0);
    let duration = buffer.duration().map(|d| d.nseconds() as i64).unwrap_or(0);
    
    Some(PreviewFrame {
        width,
        height,
        data,
        pts,
        duration,
    })
}
