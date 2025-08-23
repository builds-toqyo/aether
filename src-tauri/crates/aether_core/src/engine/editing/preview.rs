use std::sync::Arc;
use std::panic;
use log::{error, warn, debug};
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
    
    frame_callback: Option<Arc<dyn Fn(PreviewFrame) + Send + Sync + 'static>>,
    
    /// Stores the latest frame for asynchronous access
    latest_frame: Arc<std::sync::Mutex<Option<PreviewFrame>>>,
    
    /// Video dimensions from the pipeline
    video_dimensions: Option<(u32, u32)>,
    
    /// Video duration from the pipeline
    video_duration: Option<i64>,
}

impl PreviewEngine {
    pub fn new() -> Result<Self, EditingError> {
        Ok(Self {
            pipeline: None,
            video_sink: None,
            is_playing: false,
            position: 0,
            frame_callback: None,
            latest_frame: Arc::new(std::sync::Mutex::new(None)),
            video_dimensions: None,
            video_duration: None,
        })
    }
    
    pub fn set_pipeline(&mut self, pipeline: Option<ges::Pipeline>) -> Result<(), EditingError> {
        // Clean up existing resources first
        self.cleanup_resources();
        
        // Set up new pipeline if provided
        if let Some(pipeline) = pipeline {
            self.setup_preview_pipeline(&pipeline)?;
            self.pipeline = Some(pipeline);
        }
        
        Ok(())
    }
    
    /// Clean up all resources associated with the current pipeline
    fn cleanup_resources(&mut self) {
        // First remove the video sink from the pipeline if it exists
        if let (Some(pipeline), Some(video_sink)) = (&self.pipeline, &self.video_sink) {
            // Try to remove the video sink from the pipeline
            if let Err(err) = pipeline.set_video_sink(None) {
                error!("Failed to remove video sink from pipeline: {:?}", err);
            }
        }
        
        // Set pipeline to NULL state to release resources
        if let Some(pipeline) = &self.pipeline {
            if let Err(err) = pipeline.set_state(gst::State::Null) {
                error!("Failed to set pipeline to NULL state: {:?}", err);
            }
            
            // Wait for the state change to complete
            // Wait for state change with proper error handling
        if let Err(err) = pipeline.get_state(gst::ClockTime::from_seconds(1)) {
            warn!("Failed to wait for pipeline state change: {:?}", err);
        }
        }
        
        // Clear our references
        self.pipeline = None;
        self.video_sink = None;
        self.is_playing = false;
    }
    
    fn setup_preview_pipeline(&mut self, pipeline: &ges::Pipeline) -> Result<(), EditingError> {
        // Extract video properties from the pipeline
        self.update_video_properties(pipeline);
        let video_sink = gst::ElementFactory::make("appsink")
            .name("preview_sink")
            .build()
            .map_err(|_| EditingError::PreviewError("Failed to create appsink".to_string()))?;
        
        let appsink = video_sink.downcast_ref::<gst_app::AppSink>()
            .ok_or(EditingError::PreviewError("Failed to downcast to AppSink".to_string()))?;
        
        // Support multiple pixel formats to reduce unnecessary conversions
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", &gst::List::new(["RGB", "RGBA", "BGRx", "BGRA"]))
            .build();
        
        appsink.set_caps(Some(&caps));
        appsink.set_drop(true);
        appsink.set_max_buffers(1);
        
        let callback = self.frame_callback.clone();
        let latest_frame = self.latest_frame.clone();
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    if let Some(callback) = &callback {
                        if let Ok(sample) = appsink.pull_sample() {
                            if let Some(frame) = extract_frame_from_sample(&sample) {
                                // Use catch_unwind to prevent callback panics from crashing the pipeline
                                // Store the frame in latest_frame for asynchronous access
                                if let Ok(mut latest_frame) = latest_frame.lock() {
                                    *latest_frame = Some(frame.clone());
                                }
                                
                                // Call the callback
                                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                                    callback(frame);
                                })) {
                                    error!("Preview callback panicked: {:?}", e);
                                }
                            } else {
                                warn!("Failed to extract frame from sample");
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
        F: Fn(PreviewFrame) + Send + Sync + 'static,
    {
        self.frame_callback = Some(Arc::new(callback));
    }
    
    pub fn play(&mut self) -> Result<(), EditingError> {
        let pipeline = self.pipeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;
        
        // Set state and wait for state change to complete
        pipeline.set_state(gst::State::Playing)?;
        
        // Verify state change was successful
        let (state_change, new_state, _) = pipeline.state(gst::ClockTime::from_seconds(1));
        if state_change == gst::StateChangeReturn::Failure || new_state != gst::State::Playing {
            return Err(EditingError::PreviewError(format!("Failed to set pipeline to Playing state, current state: {:?}", new_state)));
        }
        
        self.is_playing = true;
        debug!("Pipeline successfully set to Playing state");
        
        Ok(())
    }
    
    pub fn pause(&mut self) -> Result<(), EditingError> {
        let pipeline = self.pipeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;
        
        pipeline.set_state(gst::State::Paused)?;
        
        // Verify state change was successful
        let (state_change, new_state, _) = pipeline.state(gst::ClockTime::from_seconds(1));
        if state_change == gst::StateChangeReturn::Failure {
            return Err(EditingError::PreviewError(format!("Failed to set pipeline to Paused state, current state: {:?}", new_state)));
        }
        
        self.is_playing = false;
        debug!("Pipeline successfully set to Paused state");
        
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
                .unwrap_or_else(|| self.position);
            
            Ok(position)
        } else {
            Ok(self.position)
        }
    }
    
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
    
    pub fn get_frame(&self) -> Result<Option<PreviewFrame>, EditingError> {
        // Return a clone of the latest frame if available
        if let Ok(latest_frame) = self.latest_frame.lock() {
            return Ok(latest_frame.clone());
        }
        
        // Return error if we couldn't acquire the lock
        Err(EditingError::PreviewError("Failed to access latest frame".to_string()))
    }
    
    /// Get the video dimensions (width, height) if available
    pub fn get_video_dimensions(&self) -> Option<(u32, u32)> {
        self.video_dimensions
    }
    
    /// Get the video duration in nanoseconds if available
    pub fn get_duration(&self) -> Option<i64> {
        self.video_duration
    }
    
    /// Update video properties from the pipeline
    fn update_video_properties(&mut self, pipeline: &ges::Pipeline) -> Result<(), EditingError> {
        // Get video dimensions from the pipeline
        if let Some(timeline) = pipeline.timeline() {
            // Try to get dimensions from timeline
            let width = timeline.width();
            let height = timeline.height();
            
            if width > 0 && height > 0 {
                self.video_dimensions = Some((width as u32, height as u32));
                debug!("Video dimensions: {}x{}", width, height);
            }
            
            // Try to get duration from timeline
            if let Some(duration) = timeline.duration() {
                self.video_duration = Some(duration.nseconds() as i64);
                debug!("Video duration: {} ns", duration.nseconds());
            }
        }
        
        Ok(())
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
