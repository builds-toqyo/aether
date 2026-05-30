use anyhow::Result;
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_app::AppSink;
use gstreamer_app::AppSrc;
use log::{debug, error};
use std::sync::{Arc, Mutex};

use super::color_grading::ColorGradingEngine;


pub struct ColorGradingFrameProcessor {

    engine: Arc<Mutex<ColorGradingEngine>>,
}

impl ColorGradingFrameProcessor {

    pub fn new(engine: ColorGradingEngine) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
        }
    }


    pub fn process_frame(&self, frame: &[u8], width: u32, height: u32, format: &str) -> Result<Vec<u8>> {
        let mut engine = self.engine.lock().map_err(|_| anyhow::anyhow!("Failed to lock engine"))?;


        if !engine.is_initialized() {
            engine.initialize()?;
        }


        engine.start()?;


        let src = engine.get_element("src")
            .ok_or_else(|| anyhow::anyhow!("src element not found"))?;
        let appsrc = src.clone().dynamic_cast::<gst_app::AppSrc>()
            .map_err(|_| anyhow::anyhow!("Failed to cast to AppSrc"))?;


        let buffer = gst::Buffer::from_slice(frame.to_vec());


        appsrc.push_buffer(buffer.clone())
            .map_err(|_| anyhow::anyhow!("Failed to push buffer to appsrc"))?;


        self.pull_processed_frame(&engine)
    }


    fn pull_processed_frame(&self, engine: &ColorGradingEngine) -> Result<Vec<u8>> {

        let sink = engine.get_element("sink")
            .ok_or_else(|| anyhow::anyhow!("sink element not found"))?;
        let appsink = sink.clone().dynamic_cast::<gst_app::AppSink>()
            .map_err(|_| anyhow::anyhow!("Failed to cast to AppSink"))?;


        let timeout = std::time::Duration::from_millis(100);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout {
            if let Some(sample) = appsink.try_pull_sample(gst::ClockTime::from_mseconds(10)) {

                let buffer = sample.buffer()
                    .ok_or_else(|| anyhow::anyhow!("No buffer in sample"))?;


                let map = buffer.map_readable()
                    .map_err(|_| anyhow::anyhow!("Cannot map buffer"))?;


                let processed_data = map.as_slice().to_vec();

                return Ok(processed_data);
            }
        }

        Err(anyhow::anyhow!("Timeout waiting for processed frame"))
    }
}
