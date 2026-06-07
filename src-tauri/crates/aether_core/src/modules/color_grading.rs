use anyhow::{Context, Result};
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_app::AppSink;
use glib::ControlFlow;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::engine::editing::EditingError;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSpace {
    RGB,
    YUV,
    HSL,
    HSV,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GradingPresetType {
    BuiltIn(String),
    Custom(String),
    FromFile(PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingPreset {
    pub name: String,
    pub preset_type: GradingPresetType,
    pub adjustments: ColorAdjustments,
    pub curves: ColorCurves,
    pub lut: Option<LutSettings>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorAdjustments {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub gamma: f32,
    pub hue: f32,
    pub temperature: f32,
    pub tint: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub whites: f32,
    pub blacks: f32,
    pub vibrance: f32,
    pub sharpness: f32,
}

impl Default for ColorAdjustments {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            hue: 0.0,
            temperature: 0.0,
            tint: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            vibrance: 1.0,
            sharpness: 0.0,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurvePoint {

    pub x: f32,

    pub y: f32,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorCurves {

    pub rgb: Vec<CurvePoint>,

    pub red: Vec<CurvePoint>,

    pub green: Vec<CurvePoint>,

    pub blue: Vec<CurvePoint>,

    pub luma: Vec<CurvePoint>,
}

impl Default for ColorCurves {
    fn default() -> Self {

        let default_curve = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];

        Self {
            rgb: default_curve.clone(),
            red: default_curve.clone(),
            green: default_curve.clone(),
            blue: default_curve.clone(),
            luma: default_curve,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LutFormat {

    CUBE,

    ThreeDL,

    HALD,

    PNG,

    JPEG,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LutSettings {

    pub path: PathBuf,

    pub format: LutFormat,

    pub strength: f32,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScopeType {

    Histogram,

    Waveform,

    Vectorscope,

    RGBParade,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeDataFormat {

    Raw(Vec<u8>),

    Base64(String),

    JSON(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {

    pub scope_type: ScopeType,

    pub width: u32,

    pub height: u32,

    pub continuous_update: bool,

    pub update_interval_ms: u32,
}

impl Default for ScopeConfig {
    fn default() -> Self {
        Self {
            scope_type: ScopeType::Histogram,
            width: 256,
            height: 100,
            continuous_update: false,
            update_interval_ms: 100,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeData {

    pub scope_type: ScopeType,

    pub width: u32,

    pub height: u32,

    pub timestamp: u64,

    pub data: ScopeDataFormat,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGradingConfig {

    pub color_space: ColorSpace,

    pub bit_depth: u8,

    pub use_gpu: bool,

    pub cache_dir: Option<PathBuf>,

    pub max_presets: usize,
}

impl Default for ColorGradingConfig {
    fn default() -> Self {
        Self {
            color_space: ColorSpace::RGB,
            bit_depth: 8,
            use_gpu: true,
            cache_dir: None,
            max_presets: 10,
        }
    }
}


pub struct ColorGradingEngine {

    config: ColorGradingConfig,

    adjustments: ColorAdjustments,

    curves: ColorCurves,

    lut: Option<LutSettings>,

    presets: HashMap<String, GradingPreset>,

    active_preset: Option<String>,

    elements: HashMap<String, gst::Element>,

    pipeline: Option<gst::Pipeline>,

    initialized: bool,

    scopes: HashMap<ScopeType, ScopeConfig>,

    scope_update_timeout_id: Option<glib::SourceId>,

    bus_watch: Option<gst::BusWatchGuard>,
}

impl ColorGradingEngine {

    pub fn new() -> Result<Self> {
        gst::init()?;

        Ok(Self {
            config: ColorGradingConfig::default(),
            adjustments: ColorAdjustments::default(),
            curves: ColorCurves::default(),
            lut: None,
            presets: HashMap::new(),
            active_preset: None,
            elements: HashMap::new(),
            pipeline: None,
            bus_watch: None,
            initialized: false,
            scopes: HashMap::from([
                (ScopeType::Histogram, ScopeConfig::default()),
                (ScopeType::Waveform, ScopeConfig::default()),
                (ScopeType::Vectorscope, ScopeConfig::default()),
                (ScopeType::RGBParade, ScopeConfig::default()),
            ]),
            scope_update_timeout_id: None,
        })
    }


    pub fn with_config(config: ColorGradingConfig) -> Result<Self> {
        let mut engine = Self::new()?;
        engine.config = config;
        Ok(engine)
    }


    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        debug!("Initializing color grading engine");


        let pipeline = gst::Pipeline::new(Some("color-grading-pipeline"));
        self.pipeline = Some(pipeline.clone());


        self.create_basic_elements(&pipeline)?;


        self.link_elements()?;


        let bus = pipeline.bus().unwrap();
        let weak_pipeline = pipeline.downgrade();
        let bus_watch_id = bus.add_watch(move |_, msg| {
            let pipeline = match weak_pipeline.upgrade() {
                Some(pipeline) => pipeline,
                None => return ControlFlow::Stop,
            };

            match msg.view() {
                gst::MessageView::Error(err) => {
                    error!(
                        "Error from {:?}: {} ({})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug().unwrap_or_default()
                    );
                    pipeline.set_state(gst::State::Null).unwrap();
                }
                gst::MessageView::Eos(_) => {
                    debug!("End of stream");
                    pipeline.set_state(gst::State::Ready).unwrap();
                }
                gst::MessageView::StateChanged(state_changed) => {
                    if state_changed.src().map(|s| s == pipeline).unwrap_or(false) {
                        debug!(
                            "Pipeline state changed from {:?} to {:?}",
                            state_changed.old(),
                            state_changed.current()
                        );
                    }
                }
                _ => (),
            }

            ControlFlow::Continue
        });

        self.bus_watch_id = Some(bus_watch_id);

        Ok(())
    }

    fn create_basic_elements(&mut self, pipeline: &gst::Pipeline) -> Result<()> {
        let required_elements = [
            ("capsfilter", "capsfilter"),
            ("gamma", "gamma"),
            ("videobalance", "videobalance"),
            ("saturation", "saturation"),
            ("videoconvert", "videoconvert2"),
            ("tee", "tee"),
            ("queue", "queue_main"),
            ("appsink", "sink"),
        ];

        for (factory, name) in required_elements.iter() {
            let element = gst::ElementFactory::make(factory)
                .name(name)
                .build()
                .map_err(|_| anyhow::anyhow!("Failed to create {} element", name))?;

            pipeline.add(&element)?;
            self.elements.insert(name.to_string(), element);
        }


        if let Some(src) = self.elements.get("src") {
            src.set_property("format", gst::Format::Time);
            src.set_property("do-timestamp", true);
            src.set_property("is-live", true);


            let src_caps = gst::Caps::builder("video/x-raw")
                .field("format", "RGBA")
                .field("width", 1920)
                .field("height", 1080)
                .field("framerate", gst::Fraction::new(30, 1))
                .build();
            src.set_property("caps", &src_caps);
        }


        if let Some(capsfilter) = self.elements.get("capsfilter") {
            let caps = gst::Caps::builder("video/x-raw")
                .field("format", "RGBA")
                .build();
            capsfilter.set_property("caps", &caps);
        }


        if let Some(queue) = self.elements.get("queue_main") {
            queue.set_property("leaky", 2);
            queue.set_property("max-size-buffers", 2);
        }


        if let Some(sink) = self.elements.get("sink") {
            sink.set_property("emit-signals", true);
            sink.set_property("sync", false);


            let appsink = sink.clone().dynamic_cast::<gst_app::AppSink>().expect("Not an appsink");
            appsink.set_callbacks(
                gst_app::AppSinkCallbacks::builder()
                    .new_sample(|appsink| {
                        let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;


                        debug!("Received processed frame");

                        Ok(gst::FlowSuccess::Ok)
                    })
                    .build()
            );
        }


        if self.config.use_gpu {

            if let Ok(lut_element) = gst::ElementFactory::make("glcolorbalance")
                .name("lut")
                .build() {
                pipeline.add(&lut_element)?;
                self.elements.insert("lut".to_string(), lut_element);
            } else {
                warn!("GPU-accelerated LUT processing not available, falling back to CPU");
                self.create_cpu_lut_element(pipeline)?;
            }
        } else {
            self.create_cpu_lut_element(pipeline)?;
        }


        self.setup_scope_elements(pipeline.clone())?;


        self.link_elements()?;


        let bus = pipeline.bus().expect("Pipeline without bus. Should not happen!");
        let bus_watch = bus.add_watch(move |_, msg| {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    error!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                }
                gst::MessageView::StateChanged(state) => {
                    if let Some(element) = msg.src() {
                        if element.name().starts_with("pipeline") {
                            debug!(
                                "Pipeline state changed from {:?} to {:?}",
                                state.old(),
                                state.current()
                            );
                        }
                    }
                }
                _ => (),
            }
            ControlFlow::Continue
        }).expect("Failed to add bus watch");

        self.bus_watch = Some(bus_watch);


        self.apply_adjustments()?;
        self.apply_curves()?;


        pipeline.set_state(gst::State::Ready)?;

        self.initialized = true;

        Ok(())
    }


    fn create_cpu_lut_element(&mut self, pipeline: &gst::Pipeline) -> Result<()> {

        if let Ok(lut_element) = gst::ElementFactory::make("videobalance")
            .name("lut")
            .build() {
            pipeline.add(&lut_element)?;
            self.elements.insert("lut".to_string(), lut_element);
            Ok(())
        } else {
            warn!("Standard LUT processing not available");
            Ok(())
        }
    }


    fn link_elements(&self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {

            let src = self.elements.get("src").ok_or_else(|| anyhow::anyhow!("src element not found"))?;
            let videoconvert1 = self.elements.get("videoconvert1").ok_or_else(|| anyhow::anyhow!("videoconvert1 element not found"))?;
            let capsfilter = self.elements.get("capsfilter").ok_or_else(|| anyhow::anyhow!("capsfilter element not found"))?;
            let gamma = self.elements.get("gamma").ok_or_else(|| anyhow::anyhow!("gamma element not found"))?;
            let videobalance = self.elements.get("videobalance").ok_or_else(|| anyhow::anyhow!("videobalance element not found"))?;
            let saturation = self.elements.get("saturation").ok_or_else(|| anyhow::anyhow!("saturation element not found"))?;
            let videoconvert2 = self.elements.get("videoconvert2").ok_or_else(|| anyhow::anyhow!("videoconvert2 element not found"))?;
            let tee = self.elements.get("tee").ok_or_else(|| anyhow::anyhow!("tee element not found"))?;
            let queue_main = self.elements.get("queue_main").ok_or_else(|| anyhow::anyhow!("queue_main element not found"))?;
            let sink = self.elements.get("sink").ok_or_else(|| anyhow::anyhow!("sink element not found"))?;


            let mut elements = vec![src, videoconvert1, capsfilter, gamma, videobalance];


            if let Some(lut) = self.elements.get("lut") {
                elements.push(lut);
            }


            elements.push(saturation);
            elements.push(videoconvert2);
            elements.push(tee);


            gst::Element::link_many(&elements)?;


            tee.link_pads(Some("src_%u"), queue_main, Some("sink"))?;
            queue_main.link(sink)?;


            for scope_type in self.get_configured_scopes() {
                let scope_queue_name = format!("queue_scope_{:?}", scope_type).to_lowercase();
                let scope_sink_name = format!("scope_sink_{:?}", scope_type).to_lowercase();

                if let (Some(queue), Some(scope_sink)) = (
                    self.elements.get(&scope_queue_name),
                    self.elements.get(&scope_sink_name)
                ) {
                    tee.link_pads(Some("src_%u"), queue, Some("sink"))?;
                    queue.link(scope_sink)?;
                }
            }
        }

        Ok(())
    }


    fn setup_scope_elements(&mut self, pipeline: gst::Pipeline) -> Result<()> {

        for scope_type in self.get_configured_scopes() {
            let scope_name = format!("{:?}", scope_type).to_lowercase();


            let queue_name = format!("queue_scope_{}", scope_name);
            let queue = gst::ElementFactory::make("queue")
                .name(&queue_name)
                .build()
                .map_err(|_| anyhow::anyhow!("Failed to create queue for scope {}", scope_name))?;


            queue.set_property("leaky", 2);
            queue.set_property("max-size-buffers", 1);
            queue.set_property("max-size-bytes", 0);
            queue.set_property("max-size-time", gst::ClockTime::from_seconds(0));


            let sink_name = format!("scope_sink_{}", scope_name);
            let sink = gst::ElementFactory::make("appsink")
                .name(&sink_name)
                .build()
                .map_err(|_| anyhow::anyhow!("Failed to create sink for scope {}", scope_name))?;


            sink.set_property("emit-signals", true);
            sink.set_property("sync", false);


            let appsink = sink.clone().dynamic_cast::<gst_app::AppSink>().expect("Not an appsink");
            let scope_type_clone = scope_type;
            let weak_self = Arc::downgrade(&Arc::new(Mutex::new(self)));

            appsink.set_callbacks(
                gst_app::AppSinkCallbacks::builder()
                    .new_sample(move |appsink| {
                        let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;


                        if let Some(arc_self) = weak_self.upgrade() {
                            if let Ok(mut this) = arc_self.lock() {
                                if let Some(config) = this.scopes.get(&scope_type_clone) {
                                    if !config.continuous_update {

                                        if let Err(e) = this.process_scope_sample(scope_type_clone, &sample) {
                                            error!("Error processing scope sample: {}", e);
                                        }
                                    }
                                }
                            }
                        }

                        Ok(gst::FlowSuccess::Ok)
                    })
                    .build()
            );


            pipeline.add(&queue)?;
            pipeline.add(&sink)?;


            self.elements.insert(queue_name, queue);
            self.elements.insert(sink_name, sink);
        }

        Ok(())
    }


    fn process_scope_sample(&self, scope_type: ScopeType, sample: &gst::Sample) -> Result<()> {

        let buffer = sample.buffer().ok_or_else(|| anyhow::anyhow!("No buffer in sample"))?;


        let map = buffer.map_readable().map_err(|_| anyhow::anyhow!("Cannot map buffer"))?;


        let caps = sample.caps().ok_or_else(|| anyhow::anyhow!("No caps in sample"))?;
        let structure = caps.structure(0).ok_or_else(|| anyhow::anyhow!("No structure in caps"))?;


        let width = structure.get::<i32>("width").map_err(|_| anyhow::anyhow!("No width in structure"))?;
        let height = structure.get::<i32>("height").map_err(|_| anyhow::anyhow!("No height in structure"))?;
        let format_str = structure.get::<&str>("format").map_err(|_| anyhow::anyhow!("No format in structure"))?;

        debug!("Processing scope sample: {}x{} format={} for {:?}", width, height, format_str, scope_type);


        Ok(())
    }


    pub fn shutdown(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        debug!("Shutting down color grading engine");


        if let Some(bus_watch_id) = self.bus_watch.take() {
            bus_watch_id.remove();
        }


        if let Some(pipeline) = self.pipeline.take() {
            pipeline.set_state(gst::State::Null)?;
        }


        self.elements.clear();
        self.initialized = false;

        Ok(())
    }


    pub fn set_brightness(&mut self, value: f32) -> Result<()> {
        self.adjustments.brightness = value.clamp(-1.0, 1.0);
        if self.initialized {
            if let Some(videobalance) = self.elements.get("videobalance") {
                videobalance.set_property("brightness", self.adjustments.brightness);
            }
        }
        Ok(())
    }


    pub fn set_contrast(&mut self, value: f32) -> Result<()> {
        self.adjustments.contrast = value.clamp(0.0, 2.0);
        if self.initialized {
            if let Some(videobalance) = self.elements.get("videobalance") {
                videobalance.set_property("contrast", self.adjustments.contrast);
            }
        }
        Ok(())
    }


    pub fn set_saturation(&mut self, value: f32) -> Result<()> {
        self.adjustments.saturation = value.clamp(0.0, 2.0);
        if self.initialized {
            if let Some(saturation) = self.elements.get("saturation") {
                saturation.set_property("saturation", self.adjustments.saturation);
            }
        }
        Ok(())
    }


    pub fn set_gamma(&mut self, value: f32) -> Result<()> {
        self.adjustments.gamma = value.clamp(0.1, 10.0);
        if self.initialized {
            if let Some(gamma) = self.elements.get("gamma") {
                gamma.set_property("gamma", self.adjustments.gamma);
            }
        }
        Ok(())
    }


    pub fn set_hue(&mut self, value: f32) -> Result<()> {
        self.adjustments.hue = value.clamp(-180.0, 180.0);
        if self.initialized {
            if let Some(videobalance) = self.elements.get("videobalance") {
                videobalance.set_property("hue", self.adjustments.hue);
            }
        }
        Ok(())
    }


    pub fn get_adjustments(&self) -> &ColorAdjustments {
        &self.adjustments
    }


    pub fn set_adjustments(&mut self, adjustments: ColorAdjustments) -> Result<()> {
        self.adjustments = adjustments;
        self.apply_adjustments()
    }


    pub fn reset_adjustments(&mut self) -> Result<()> {
        self.adjustments = ColorAdjustments::default();
        self.apply_adjustments()
    }


    pub fn create_preset(&mut self, name: &str) -> Result<()> {
        let preset = GradingPreset {
            name: name.to_string(),
            preset_type: GradingPresetType::Custom(name.to_string()),
            adjustments: self.adjustments,
            curves: self.curves.clone(),
            lut: self.lut.clone(),
        };

        self.presets.insert(name.to_string(), preset);
        self.active_preset = Some(name.to_string());

        Ok(())
    }


    pub fn apply_preset(&mut self, name: &str) -> Result<()> {
        let preset = self.presets.get(name).ok_or_else(|| {
            anyhow::anyhow!("Preset '{}' not found", name)
        })?;

        self.adjustments = preset.adjustments;
        self.curves = preset.curves.clone();
        self.lut = preset.lut.clone();
        self.active_preset = Some(name.to_string());

        self.apply_adjustments()?;


        if let Some(lut) = &self.lut {
            self.apply_lut(lut)?;
        } else {
            self.clear_lut()?;
        }


        self.apply_curves()?;

        Ok(())
    }


    pub fn get_presets(&self) -> Vec<&GradingPreset> {
        self.presets.values().collect()
    }


    pub fn delete_preset(&mut self, name: &str) -> Result<()> {
        if !self.presets.contains_key(name) {
            return Err(anyhow::anyhow!("Preset '{}' not found", name));
        }

        self.presets.remove(name);
        if self.active_preset.as_deref() == Some(name) {
            self.active_preset = None;
        }

        Ok(())
    }


    pub fn get_active_preset(&self) -> Option<&GradingPreset> {
        self.active_preset.as_ref().and_then(|name| self.presets.get(name))
    }


    pub fn load_lut(&mut self, path: &Path, format: LutFormat) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("LUT file not found: {}", path.display()));
        }

        let lut_settings = LutSettings {
            path: path.to_path_buf(),
            format,
            strength: 1.0,
        };

        self.lut = Some(lut_settings.clone());

        if self.initialized {
            self.apply_lut(&lut_settings);
        }
    }


    fn pull_processed_frame(&self) -> Result<Vec<u8>> {

        let sink = self.elements.get("sink")
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


    pub fn start(&mut self) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }

        if let Some(pipeline) = &self.pipeline {
            debug!("Starting color grading pipeline");
            pipeline.set_state(gst::State::Playing)?;
        }

        Ok(())
    }


    pub fn pause(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            debug!("Pausing color grading pipeline");
            pipeline.set_state(gst::State::Paused)?;
        }

        Ok(())
    }


    pub fn stop(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            debug!("Stopping color grading pipeline");
            pipeline.set_state(gst::State::Ready)?;
        }

        Ok(())
    }


    fn apply_cube_lut(&self, element: &gst::Element, lut_settings: &LutSettings) -> Result<()> {
        // Read CUBE LUT file and apply it to the element
        let lut_data = std::fs::read_to_string(&lut_settings.path)
            .map_err(|_| anyhow::anyhow!("Failed to read CUBE LUT file"))?;

        // Parse CUBE LUT format and apply to element
        element.set_property("data", &lut_data);

        debug!("Applied CUBE LUT: {}", lut_settings.path.display());
        Ok(())
    }


    pub fn enable_scope(&mut self, scope_type: ScopeType, width: u32, height: u32, continuous_update: bool) -> Result<()> {
        let config = ScopeConfig {
            scope_type,
            width,
            height,
            continuous_update,
            update_interval_ms: 100,
        };

        self.configure_scope(scope_type, config)
    }


    pub fn disable_scope(&mut self, scope_type: ScopeType) -> Result<()> {
        self.scopes.remove(&scope_type);


        if !self.has_continuous_scopes() && self.scope_update_timeout_id.is_some() {
            self.remove_scope_update_timer();
        }

        Ok(())
    }


    fn has_continuous_scopes(&self) -> bool {
        self.scopes.values().any(|config| config.continuous_update)
    }


    fn setup_scope_update_timer(&mut self) -> Result<()> {

        self.remove_scope_update_timer();


        let min_interval = self.scopes.values()
            .filter(|config| config.continuous_update)
            .map(|config| config.update_interval_ms)
            .min()
            .unwrap_or(100);


        let weak_self = Arc::downgrade(&Arc::new(Mutex::new(self)));


        let timeout_id = glib::timeout_add_local(std::time::Duration::from_millis(min_interval as u64), move || {
            if let Some(arc_self) = weak_self.upgrade() {
                if let Ok(mut this) = arc_self.lock() {
                    if let Err(e) = this.update_scopes() {
                        error!("Error updating scopes: {}", e);
                    }
                    return ControlFlow::Continue;
                }
            }
            ControlFlow::Stop
        });

        self.scope_update_timeout_id = Some(timeout_id);
        Ok(())
    }


    fn remove_scope_update_timer(&mut self) {
        if let Some(timeout_id) = self.scope_update_timeout_id.take() {
            timeout_id.remove();
        }
    }


    fn update_scopes(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        for (scope_type, config) in self.scopes.iter() {
            if let Err(e) = self.update_scope(*scope_type, config) {
                error!("Error updating scope {:?}: {}", scope_type, e);
            }
        }

        Ok(())
    }


    fn update_scope(&self, scope_type: ScopeType, config: &ScopeConfig) -> Result<ScopeData> {


        let mut histogram = vec![0u8; config.width as usize * 3];


        for i in 0..config.width as usize {

            let r = ((i as f32 / config.width as f32) * 255.0 * self.adjustments.contrast) as u8;
            let g = ((i as f32 / config.width as f32) * 255.0 * self.adjustments.saturation) as u8;
            let b = ((i as f32 / config.width as f32) * 255.0 * self.adjustments.gamma) as u8;

            histogram[i * 3] = r;
            histogram[i * 3 + 1] = g;
            histogram[i * 3 + 2] = b;
        }

        Ok(ScopeData {
            scope_type: ScopeType::Histogram,
            width: config.width,
            height: config.height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            data: ScopeDataFormat::Raw(histogram),
        })
    }


    fn generate_waveform_data(&self, config: &ScopeConfig) -> Result<ScopeData> {


        let mut vectorscope = vec![0u8; config.width as usize * config.height as usize * 3];


        let center_x = config.width as f32 / 2.0;
        let center_y = config.height as f32 / 2.0;
        let radius = config.width.min(config.height) as f32 / 2.0;

        for y in 0..config.height as usize {
            for x in 0..config.width as usize {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= radius {
                    let angle = dy.atan2(dx);
                    let hue = ((angle / std::f32::consts::PI + 1.0) * 180.0) as u8;
                    let saturation = (distance / radius * 255.0) as u8;


                    let idx = (y * config.width as usize + x) * 3;
                    vectorscope[idx] = hue;
                    vectorscope[idx + 1] = saturation;
                    vectorscope[idx + 2] = 255;
                }
            }
        }

        Ok(ScopeData {
            scope_type: ScopeType::Vectorscope,
            width: config.width,
            height: config.height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            data: ScopeDataFormat::Raw(vectorscope),
        })
    }


    fn generate_rgb_parade_data(&self, config: &ScopeConfig) -> Result<ScopeData> {


        let parade_width = config.width / 3;
        let mut rgb_parade = vec![0u8; config.width as usize * config.height as usize * 3];


        for y in 0..config.height as usize {
            let y_value = 255 - (y as f32 / config.height as f32 * 255.0) as u8;


            for x in 0..parade_width as usize {
                let idx = (y * config.width as usize + x) * 3;
                rgb_parade[idx] = y_value;
                rgb_parade[idx + 1] = 0;
                rgb_parade[idx + 2] = 0;
            }


            for x in parade_width as usize..(parade_width * 2) as usize {
                let idx = (y * config.width as usize + x) * 3;
                rgb_parade[idx] = 0;
                rgb_parade[idx + 1] = y_value;
                rgb_parade[idx + 2] = 0;
            }


            for x in (parade_width * 2) as usize..config.width as usize {
                let idx = (y * config.width as usize + x) * 3;
                rgb_parade[idx] = 0;
                rgb_parade[idx + 1] = 0;
                rgb_parade[idx + 2] = y_value;
            }
        }

        Ok(ScopeData {
            scope_type: ScopeType::RGBParade,
            width: config.width,
            height: config.height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            data: ScopeDataFormat::Raw(rgb_parade),
        })
    }


    pub fn get_scope_data(&self, scope_type: ScopeType) -> Result<ScopeData> {
        let config = self.scopes.get(&scope_type).ok_or_else(|| {
            anyhow::anyhow!("Scope {:?} not configured", scope_type)
        })?;

        self.update_scope(scope_type, config)
    }


    pub fn get_configured_scopes(&self) -> Vec<ScopeType> {
        self.scopes.keys().copied().collect()
    }


    pub fn is_initialized(&self) -> bool {
        self.initialized
    }


    pub fn get_element(&self, name: &str) -> Option<&gst::Element> {
        self.elements.get(name)
    }
}
