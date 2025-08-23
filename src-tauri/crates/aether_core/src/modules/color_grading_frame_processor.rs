use anyhow::Result;
use gst::prelude::*;
use gst_app;
use log::{debug, error};
use std::sync::{Arc, Mutex};

use super::color_grading::ColorGradingEngine;

/// Frame processor for real-time color grading
pub struct ColorGradingFrameProcessor {
    /// The color grading engine
    engine: Arc<Mutex<ColorGradingEngine>>,
}

impl ColorGradingFrameProcessor {
    /// Create a new frame processor with the given color grading engine
    pub fn new(engine: ColorGradingEngine) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
        }
    }
    
    /// Process a video frame through the color grading pipeline
    pub fn process_frame(&self, frame: &[u8], width: u32, height: u32, format: &str) -> Result<Vec<u8>> {
        let mut engine = self.engine.lock().map_err(|_| anyhow::anyhow!("Failed to lock engine"))?;
        
        // Ensure engine is initialized
        if !engine.is_initialized() {
            engine.initialize()?;
        }
        
        // Ensure pipeline is in playing state
        engine.start()?;
        
        // Get the appsrc element
        let src = engine.get_element("src")
            .ok_or_else(|| anyhow::anyhow!("src element not found"))?;
        let appsrc = src.clone().dynamic_cast::<gst_app::AppSrc>()
            .map_err(|_| anyhow::anyhow!("Failed to cast to AppSrc"))?;
        
        // Create buffer from frame data
        let buffer = gst::Buffer::from_slice(frame.to_vec());
        
        // Push buffer to appsrc
        appsrc.push_buffer(buffer.clone())
            .map_err(|_| anyhow::anyhow!("Failed to push buffer to appsrc"))?;
        
        // Get processed frame from appsink
        self.pull_processed_frame(&engine)
    }
    
    /// Pull a processed frame from the appsink
    fn pull_processed_frame(&self, engine: &ColorGradingEngine) -> Result<Vec<u8>> {
        // Get the appsink element
        let sink = engine.get_element("sink")
            .ok_or_else(|| anyhow::anyhow!("sink element not found"))?;
        let appsink = sink.clone().dynamic_cast::<gst_app::AppSink>()
            .map_err(|_| anyhow::anyhow!("Failed to cast to AppSink"))?;
        
        // Try to pull a sample with timeout
        let timeout = std::time::Duration::from_millis(100);
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout {
            if let Some(sample) = appsink.try_pull_sample(gst::ClockTime::from_mseconds(10)) {
                // Get buffer from sample
                let buffer = sample.buffer()
                    .ok_or_else(|| anyhow::anyhow!("No buffer in sample"))?;
                
                // Map buffer for reading
                let map = buffer.map_readable()
                    .map_err(|_| anyhow::anyhow!("Cannot map buffer"))?;
                
                // Convert to Vec<u8>
                let processed_data = map.as_slice().to_vec();
                
                return Ok(processed_data);
            }
        }
        
        Err(anyhow::anyhow!("Timeout waiting for processed frame"))
    }
}
