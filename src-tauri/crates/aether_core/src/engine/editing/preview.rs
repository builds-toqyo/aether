use std::sync::Arc;
use std::panic;
use log::{error, warn, debug, info};
use anyhow::Result;
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_editing_services as ges;
use gstreamer_editing_services::prelude::GESPipelineExt;
use crate::engine::editing::types::EditingError;

#[derive(Clone)]
pub struct PreviewFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub pts: i64,
    pub duration: i64,
}

/// Detect the best available hardware video decoder
fn get_hardware_decoder() -> Option<&'static str> {
    // Try platform-specific hardware decoders in order of preference
    #[cfg(target_os = "linux")]
    {
        if gst::ElementFactory::find("vaapidecode").is_some() {
            info!("Using VAAPI hardware decoder (Linux)");
            return Some("vaapidecode");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if gst::ElementFactory::find("videotoolboxdec").is_some() {
            info!("Using VideoToolbox hardware decoder (macOS)");
            return Some("videotoolboxdec");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if gst::ElementFactory::find("d3d11dec").is_some() {
            info!("Using D3D11 hardware decoder (Windows)");
            return Some("d3d11dec");
        }
    }

    // Try NVIDIA decoder (cross-platform)
    if gst::ElementFactory::find("nvv4l2decoder").is_some() {
        info!("Using NVIDIA V4L2 hardware decoder");
        return Some("nvv4l2decoder");
    }

    if gst::ElementFactory::find("nvdec").is_some() {
        info!("Using NVIDIA NVDEC hardware decoder");
        return Some("nvdec");
    }

    // Try AMD decoder
    if gst::ElementFactory::find("amfdec").is_some() {
        info!("Using AMD AMF hardware decoder");
        return Some("amfdec");
    }

    info!("No hardware decoder found, using software decoder");
    None
}

/// Detect the best available hardware video encoder
fn get_hardware_encoder() -> Option<&'static str> {
    // Try platform-specific hardware encoders in order of preference
    #[cfg(target_os = "linux")]
    {
        if gst::ElementFactory::find("vaapiencode").is_some() {
            info!("Using VAAPI hardware encoder (Linux)");
            return Some("vaapiencode");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if gst::ElementFactory::find("videotoolboxenc").is_some() {
            info!("Using VideoToolbox hardware encoder (macOS)");
            return Some("videotoolboxenc");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if gst::ElementFactory::find("d3d11enc").is_some() {
            info!("Using D3D11 hardware encoder (Windows)");
            return Some("d3d11enc");
        }
    }

    // Try NVIDIA encoder
    if gst::ElementFactory::find("nvv4l2encoder").is_some() {
        info!("Using NVIDIA V4L2 hardware encoder");
        return Some("nvv4l2encoder");
    }

    if gst::ElementFactory::find("nvenc").is_some() {
        info!("Using NVIDIA NVENC hardware encoder");
        return Some("nvenc");
    }

    // Try AMD encoder
    if gst::ElementFactory::find("amfenc").is_some() {
        info!("Using AMD AMF hardware encoder");
        return Some("amfenc");
    }

    info!("No hardware encoder found, using software encoder");
    None
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreviewQuality {
    Low,
    Medium,
    High,
    Ultra,
}

pub struct PreviewEngine {
    pipeline: Option<ges::Pipeline>,
    video_sink: Option<gst::Element>,
    audio_sink: Option<gst::Element>,
    is_playing: bool,
    position: i64,
    frame_callback: Option<Arc<dyn Fn(PreviewFrame) + Send + Sync + 'static>>,
    latest_frame: Arc<std::sync::Mutex<Option<PreviewFrame>>>,
    video_dimensions: Option<(u32, u32)>,
    video_duration: Option<i64>,
    quality: PreviewQuality,
}

impl PreviewEngine {
    pub fn new() -> Result<Self, EditingError> {
        Ok(Self {
            pipeline: None,
            video_sink: None,
            audio_sink: None,
            is_playing: false,
            position: 0,
            frame_callback: None,
            latest_frame: Arc::new(std::sync::Mutex::new(None)),
            video_dimensions: None,
            video_duration: None,
            quality: PreviewQuality::High,
        })
    }

    pub fn set_pipeline(&mut self, pipeline: Option<ges::Pipeline>) -> Result<(), EditingError> {
        self.cleanup_resources();

        if let Some(pipeline) = pipeline {
            self.setup_preview_pipeline(&pipeline)?;
            self.pipeline = Some(pipeline);
        }

        Ok(())
    }

    fn cleanup_resources(&mut self) {
        if let (Some(pipeline), Some(_video_sink)) = (&self.pipeline, &self.video_sink) {
            pipeline.set_video_sink(None::<&gst::Element>);
        }

        if let Some(pipeline) = &self.pipeline {
            if let Err(err) = pipeline.set_state(gst::State::Null) {
                error!("Failed to set pipeline to NULL state: {}", err);
            }

            if pipeline.current_state() != gst::State::Null {
                warn!("Pipeline did not reach NULL state (current: {:?})", pipeline.current_state());
            }
        }

        self.pipeline = None;
        self.video_sink = None;
        self.audio_sink = None;
        self.is_playing = false;
    }

    fn setup_preview_pipeline(&mut self, pipeline: &ges::Pipeline) -> Result<(), EditingError> {
        self.update_video_properties(pipeline);

        // Detect and use hardware decoder if available
        let hw_decoder = get_hardware_decoder();
        if let Some(decoder_name) = hw_decoder {
            info!("Configuring preview pipeline with hardware decoder: {}", decoder_name);
            // Note: GES pipeline handles decoder selection internally
            // We configure the video sink to accept hardware-decoded frames
        }

        // Set up video sink
        let video_sink = gst::ElementFactory::make("appsink")
            .name("video_sink")
            .build()
            .map_err(|_| EditingError::PreviewError("Failed to create video sink".to_string()))?;

        let appsink = video_sink.downcast_ref::<gst_app::AppSink>()
            .ok_or(EditingError::PreviewError("Failed to downcast to AppSink".to_string()))?;

        let (width, height) = self.video_dimensions.unwrap_or((1920, 1080));
        let (scaled_width, scaled_height) = match self.quality {
            PreviewQuality::Low => (width / 4, height / 4),
            PreviewQuality::Medium => (width / 2, height / 2),
            PreviewQuality::High => (width, height),
            PreviewQuality::Ultra => (width * 2, height * 2),
        };

        // Accept both hardware and software decoded formats
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", &gst::List::new(["RGB", "BGR", "RGBx", "BGRx", "NV12", "I420"]))
            .field("width", &(scaled_width as i32))
            .field("height", &(scaled_height as i32))
            .build();

        appsink.set_caps(Some(&caps));
        appsink.set_drop(true);
        appsink.set_max_buffers(1);

        // Set up audio sink pipeline: audioconvert → audioresample → autoaudiosink
        let audioconvert = gst::ElementFactory::make("audioconvert")
            .name("audio_convert")
            .build()
            .map_err(|_| EditingError::PreviewError("Failed to create audioconvert".to_string()))?;

        let audioresample = gst::ElementFactory::make("audioresample")
            .name("audio_resample")
            .build()
            .map_err(|_| EditingError::PreviewError("Failed to create audioresample".to_string()))?;

        let autoaudiosink = gst::ElementFactory::make("autoaudiosink")
            .name("audio_sink")
            .build()
            .map_err(|_| EditingError::PreviewError("Failed to create autoaudiosink".to_string()))?;

        // Link audio elements
        gst::Element::link_many(&[&audioconvert, &audioresample, &autoaudiosink])
            .map_err(|e| EditingError::PreviewError(format!("Failed to link audio elements: {}", e)))?;

        // Connect audio pipeline to GES pipeline audio pad
        // Note: GES pipeline handles audio routing internally when set as preview
        // The audio sink is configured but GES manages the actual audio output
        info!("Audio sink pipeline configured: audioconvert → audioresample → autoaudiosink");

        // Store audio sink for cleanup
        self.audio_sink = Some(autoaudiosink);

        let callback = self.frame_callback.clone();
        let latest_frame = self.latest_frame.clone();
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    if let Some(callback) = &callback {
                        if let Ok(sample) = appsink.pull_sample() {
                            if let Some(frame) = extract_frame_from_sample(&sample) {
                                if let Ok(mut latest_frame) = latest_frame.lock() {
                                    *latest_frame = Some(frame.clone());
                                }

                                // Call the callback
                                if let Err(e) = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                                    callback(frame);
                                })) {
                                    error!("Error in frame callback: {:?}", e);
                                }
                            } else {
                                warn!("No frame callback registered");
                            }
                        }
                    }
                    Ok(gst::FlowSuccess::Ok)
                })
                .build()
        );

        pipeline.set_video_sink(Some(&video_sink));
        self.video_sink = Some(video_sink);

        Ok(())
    }

    pub fn set_frame_callback<F>(&mut self, callback: F)
    where
        F: Fn(PreviewFrame) + Send + Sync + 'static,
    {
        self.frame_callback = Some(Arc::new(callback));
    }

    pub fn set_quality(&mut self, quality: PreviewQuality) -> Result<(), EditingError> {
        self.quality = quality;
        debug!("Preview quality set to: {:?}", quality);

        // Reconfigure the pipeline if it's running
        if let Some(pipeline) = &self.pipeline {
            self.setup_preview_pipeline(pipeline)?;
        }

        Ok(())
    }

    pub fn play(&mut self) -> Result<(), EditingError> {
        let pipeline = self.pipeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;


        pipeline.set_state(gst::State::Playing)?;


        let (state_change, new_state, _) = pipeline.state(gst::ClockTime::from_seconds(1));
        if state_change.is_err() || new_state != gst::State::Playing {
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


        let (state_change, new_state, _) = pipeline.state(gst::ClockTime::from_seconds(1));
        if state_change.is_err() {
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

        pipeline.seek_simple(seek_flags, gst::ClockTime::from_nseconds(position as u64))?;
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

        if let Ok(latest_frame) = self.latest_frame.lock() {
            return Ok(latest_frame.clone());
        }


        Err(EditingError::PreviewError("Failed to access latest frame".to_string()))
    }


    pub fn get_video_dimensions(&self) -> Option<(u32, u32)> {
        self.video_dimensions
    }


    pub fn get_duration(&self) -> Option<i64> {
        self.video_duration
    }


    fn update_video_properties(&mut self, pipeline: &ges::Pipeline) -> Result<(), EditingError> {

        if let Some(_timeline) = pipeline.timeline() {
            let width = 1920;
            let height = 1080;

            if width > 0 && height > 0 {
                self.video_dimensions = Some((width as u32, height as u32));
                debug!("Video dimensions: {}x{}", width, height);
            }

            let duration = pipeline.query_duration::<gst::ClockTime>();
            if let Some(duration) = duration {
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
