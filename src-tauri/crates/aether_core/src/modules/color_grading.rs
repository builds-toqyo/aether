use anyhow::{Context, Result};
use gst::{self, prelude::*};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::engine::editing::EditingError;

/// Color space for color grading operations
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

/// Color curve point
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurvePoint {
    /// X coordinate (0.0 to 1.0)
    pub x: f32,
    /// Y coordinate (0.0 to 1.0)
    pub y: f32,
}

/// Color curves for precise color adjustments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorCurves {
    /// RGB composite curve
    pub rgb: Vec<CurvePoint>,
    /// Red channel curve
    pub red: Vec<CurvePoint>,
    /// Green channel curve
    pub green: Vec<CurvePoint>,
    /// Blue channel curve
    pub blue: Vec<CurvePoint>,
    /// Luma (brightness) curve
    pub luma: Vec<CurvePoint>,
}

impl Default for ColorCurves {
    fn default() -> Self {
        // Default curves with just the endpoints (linear)
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

/// LUT (Look-Up Table) format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LutFormat {
    /// CUBE format
    CUBE,
    /// 3DL format
    ThreeDL,
    /// HALD image
    HALD,
    /// PNG image
    PNG,
    /// JPEG image
    JPEG,
}

/// LUT settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LutSettings {
    /// Path to the LUT file
    pub path: PathBuf,
    /// LUT format
    pub format: LutFormat,
    /// Strength of the LUT effect (0.0 to 1.0)
    pub strength: f32,
}

/// Scope type for video analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeType {
    /// Histogram showing color distribution
    Histogram,
    /// Waveform showing luminance distribution
    Waveform,
    /// Vectorscope showing color distribution
    Vectorscope,
    /// RGB parade showing RGB channel distribution
    RGBParade,
}

/// Scope data format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeDataFormat {
    /// Raw binary data
    Raw(Vec<u8>),
    /// Base64 encoded data
    Base64(String),
    /// JSON formatted data
    JSON(String),
}

/// Scope configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {
    /// Scope type
    pub scope_type: ScopeType,
    /// Width of the scope output
    pub width: u32,
    /// Height of the scope output
    pub height: u32,
    /// Whether to update continuously
    pub continuous_update: bool,
    /// Update interval in milliseconds (if continuous_update is true)
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

/// Scope data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeData {
    /// Scope type
    pub scope_type: ScopeType,
    /// Width of the scope data
    pub width: u32,
    /// Height of the scope data
    pub height: u32,
    /// Timestamp when the scope data was captured
    pub timestamp: u64,
    /// The actual scope data
    pub data: ScopeDataFormat,
}

/// Color grading engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGradingConfig {
    /// Working color space
    pub color_space: ColorSpace,
    /// Bit depth for processing
    pub bit_depth: u8,
    /// Whether to use GPU acceleration
    pub use_gpu: bool,
    /// Cache directory for LUTs and temporary files
    pub cache_dir: Option<PathBuf>,
    /// Maximum number of presets to keep in memory
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

/// Main color grading engine
pub struct ColorGradingEngine {
    /// Configuration
    config: ColorGradingConfig,
    /// Color adjustments
    adjustments: ColorAdjustments,
    /// Color curves
    curves: ColorCurves,
    /// LUT settings
    lut: Option<LutSettings>,
    /// Available presets
    presets: HashMap<String, GradingPreset>,
    /// Currently active preset
    active_preset: Option<String>,
    /// GStreamer elements
    elements: HashMap<String, gst::Element>,
    /// GStreamer pipeline
    pipeline: Option<gst::Pipeline>,
    /// Initialization state
    initialized: bool,
    /// Active scopes
    scopes: HashMap<ScopeType, ScopeConfig>,
    /// Scope update timeout ID
    scope_update_timeout_id: Option<glib::SourceId>,
    /// Bus watch for pipeline messages
    bus_watch: Option<glib::SourceId>,
}

impl ColorGradingEngine {
    /// Create a new color grading engine with default settings
    pub fn new() -> Result<Self> {
        // Initialize GStreamer if not already initialized
        if !gst::is_initialized() {
            gst::init()?;
        }
        
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
    
    /// Create a new color grading engine with custom configuration
    pub fn with_config(config: ColorGradingConfig) -> Result<Self> {
        let mut engine = Self::new()?;
        engine.config = config;
        Ok(engine)
    }
    
    /// Initialize the color grading engine
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        debug!("Initializing color grading engine");
        
        // Create pipeline
        let pipeline = gst::Pipeline::new(Some("color-grading-pipeline"));
        self.pipeline = Some(pipeline.clone());
        
        // Create and add basic elements
        self.create_basic_elements(&pipeline)?;
        
        // Link elements
        self.link_elements()?;
        
        // Set up bus watch
        let bus = pipeline.bus().unwrap();
        let weak_pipeline = pipeline.downgrade();
        let bus_watch_id = bus.add_watch(move |_, msg| {
            let pipeline = match weak_pipeline.upgrade() {
                Some(pipeline) => pipeline,
                None => return glib::Continue(false),
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
        
        // Configure elements
        if let Some(src) = self.elements.get("src") {
            src.set_property("format", gst::Format::Time);
            src.set_property("do-timestamp", true);
            src.set_property("is-live", true);
            
            // Set caps for the source
            let src_caps = gst::Caps::builder("video/x-raw")
                .field("format", "RGBA")
                .field("width", 1920)
                .field("height", 1080)
                .field("framerate", gst::Fraction::new(30, 1))
                .build();
            src.set_property("caps", &src_caps);
        }
        
        // Set up caps filter for proper color format
        if let Some(capsfilter) = self.elements.get("capsfilter") {
            let caps = gst::Caps::builder("video/x-raw")
                .field("format", "RGBA")
                .build();
            capsfilter.set_property("caps", &caps);
        }
        
        // Configure queue for better real-time performance
        if let Some(queue) = self.elements.get("queue_main") {
            queue.set_property("leaky", 2); // Downstream leaky queue
            queue.set_property("max-size-buffers", 2);
        }
        
        // Configure sink
        if let Some(sink) = self.elements.get("sink") {
            sink.set_property("emit-signals", true);
            sink.set_property("sync", false);
            
            // Set up sample callback for processed frames
            let appsink = sink.clone().dynamic_cast::<gst_app::AppSink>().expect("Not an appsink");
            appsink.set_callbacks(
                gst_app::AppSinkCallbacks::builder()
                    .new_sample(|appsink| {
                        let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                        
                        // Here we would process the sample and make it available for the application
                        // For now, just log that we received a sample
                        debug!("Received processed frame");
                        
                        Ok(gst::FlowSuccess::Ok)
                    })
                    .build()
            );
        }
        
        // Create LUT element if GPU acceleration is enabled
        if self.config.use_gpu {
            // Try to create glcolorbalance for GPU-accelerated LUT processing
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
        
        // Create scope elements for each active scope
        self.setup_scope_elements(pipeline.clone())?;
        
        // Link elements
        self.link_elements()?;
        
        // Set up bus watch for error handling
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
            glib::Continue(true)
        }).expect("Failed to add bus watch");
        
        self.bus_watch = Some(bus_watch);
        
        // Apply current settings
        self.apply_adjustments()?;
        self.apply_curves()?;
        
        // Set pipeline to ready state
        pipeline.set_state(gst::State::Ready)?;
        
        self.initialized = true;
        
        Ok(())
    }
    
    /// Create CPU-based LUT processing element
    fn create_cpu_lut_element(&mut self, pipeline: &gst::Pipeline) -> Result<()> {
        // Try to create videobalance for CPU-based LUT processing
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
    
    /// Link the GStreamer elements in the pipeline
    fn link_elements(&self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            // Get elements
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
            
            // Create the main processing chain
            let mut elements = vec![src, videoconvert1, capsfilter, gamma, videobalance];
            
            // Add LUT element if present
            if let Some(lut) = self.elements.get("lut") {
                elements.push(lut);
            }
            
            // Add remaining elements in the main chain
            elements.push(saturation);
            elements.push(videoconvert2);
            elements.push(tee);
            
            // Link the main processing chain
            gst::Element::link_many(&elements)?;
            
            // Link tee to main output
            tee.link_pads(Some("src_%u"), queue_main, Some("sink"))?;
            queue_main.link(sink)?;
            
            // Link tee to scope branches if they exist
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
    
    /// Set up elements for video scopes
    fn setup_scope_elements(&mut self, pipeline: gst::Pipeline) -> Result<()> {
        // Create elements for each active scope
        for scope_type in self.get_configured_scopes() {
            let scope_name = format!("{:?}", scope_type).to_lowercase();
            
            // Create queue for this scope branch
            let queue_name = format!("queue_scope_{}", scope_name);
            let queue = gst::ElementFactory::make("queue")
                .name(&queue_name)
                .build()
                .map_err(|_| anyhow::anyhow!("Failed to create queue for scope {}", scope_name))?;
            
            // Configure queue for scope branch
            queue.set_property("leaky", 2); // Downstream leaky queue
            queue.set_property("max-size-buffers", 1);
            queue.set_property("max-size-bytes", 0);
            queue.set_property("max-size-time", gst::ClockTime::from_seconds(0));
            
            // Create appsink for this scope
            let sink_name = format!("scope_sink_{}", scope_name);
            let sink = gst::ElementFactory::make("appsink")
                .name(&sink_name)
                .build()
                .map_err(|_| anyhow::anyhow!("Failed to create sink for scope {}", scope_name))?;
            
            // Configure the sink
            sink.set_property("emit-signals", true);
            sink.set_property("sync", false);
            
            // Set up sample callback for scope data
            let appsink = sink.clone().dynamic_cast::<gst_app::AppSink>().expect("Not an appsink");
            let scope_type_clone = scope_type;
            let weak_self = Arc::downgrade(&Arc::new(Mutex::new(self)));
            
            appsink.set_callbacks(
                gst_app::AppSinkCallbacks::builder()
                    .new_sample(move |appsink| {
                        let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                        
                        // Process the sample for scope data
                        if let Some(arc_self) = weak_self.upgrade() {
                            if let Ok(mut this) = arc_self.lock() {
                                if let Some(config) = this.scopes.get(&scope_type_clone) {
                                    if !config.continuous_update {
                                        // Only process if not in continuous update mode
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
            
            // Add elements to pipeline
            pipeline.add(&queue)?;
            pipeline.add(&sink)?;
            
            // Store elements
            self.elements.insert(queue_name, queue);
            self.elements.insert(sink_name, sink);
        }
        
        Ok(())
    }
    
    /// Process a sample for scope data
    fn process_scope_sample(&self, scope_type: ScopeType, sample: &gst::Sample) -> Result<()> {
        // Get buffer from sample
        let buffer = sample.buffer().ok_or_else(|| anyhow::anyhow!("No buffer in sample"))?;
        
        // Map buffer for reading
        let map = buffer.map_readable().map_err(|_| anyhow::anyhow!("Cannot map buffer"))?;
        
        // Get caps and structure
        let caps = sample.caps().ok_or_else(|| anyhow::anyhow!("No caps in sample"))?;
        let structure = caps.structure(0).ok_or_else(|| anyhow::anyhow!("No structure in caps"))?;
        
        // Get video info
        let width = structure.get::<i32>("width").map_err(|_| anyhow::anyhow!("No width in structure"))?;
        let height = structure.get::<i32>("height").map_err(|_| anyhow::anyhow!("No height in structure"))?;
        let format_str = structure.get::<&str>("format").map_err(|_| anyhow::anyhow!("No format in structure"))?;
        
        debug!("Processing scope sample: {}x{} format={} for {:?}", width, height, format_str, scope_type);
        
        // In a real implementation, we would analyze the frame data here
        // and update the scope data accordingly
        
        Ok(())
    }
    
    /// Shutdown the color grading engine
    pub fn shutdown(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        
        debug!("Shutting down color grading engine");
        
        // Remove bus watch
        if let Some(bus_watch_id) = self.bus_watch.take() {
            bus_watch_id.remove();
        }
        
        // Set pipeline to null state
        if let Some(pipeline) = self.pipeline.take() {
            pipeline.set_state(gst::State::Null)?;
        }
        
        // Clear elements
        self.elements.clear();
        self.initialized = false;
    
    /// Set brightness adjustment
    pub fn set_brightness(&mut self, value: f32) -> Result<()> {
        self.adjustments.brightness = value.clamp(-1.0, 1.0);
        if self.initialized {
            if let Some(videobalance) = self.elements.get("videobalance") {
                videobalance.set_property("brightness", self.adjustments.brightness);
            }
        }
        Ok(())
    }
    
    /// Set contrast adjustment
    pub fn set_contrast(&mut self, value: f32) -> Result<()> {
        self.adjustments.contrast = value.clamp(0.0, 2.0);
        if self.initialized {
            if let Some(videobalance) = self.elements.get("videobalance") {
                videobalance.set_property("contrast", self.adjustments.contrast);
            }
        }
        Ok(())
    }
    
    /// Set saturation adjustment
    pub fn set_saturation(&mut self, value: f32) -> Result<()> {
        self.adjustments.saturation = value.clamp(0.0, 2.0);
        if self.initialized {
            if let Some(saturation) = self.elements.get("saturation") {
                saturation.set_property("saturation", self.adjustments.saturation);
            }
        }
        Ok(())
    }
    
    /// Set gamma adjustment
    pub fn set_gamma(&mut self, value: f32) -> Result<()> {
        self.adjustments.gamma = value.clamp(0.1, 10.0);
        if self.initialized {
            if let Some(gamma) = self.elements.get("gamma") {
                gamma.set_property("gamma", self.adjustments.gamma);
            }
        }
        Ok(())
    }
    
    /// Set hue adjustment
    pub fn set_hue(&mut self, value: f32) -> Result<()> {
        self.adjustments.hue = value.clamp(-180.0, 180.0);
        if self.initialized {
            if let Some(videobalance) = self.elements.get("videobalance") {
                videobalance.set_property("hue", self.adjustments.hue);
            }
        }
        Ok(())
    }
    
    /// Get current color adjustments
    pub fn get_adjustments(&self) -> &ColorAdjustments {
        &self.adjustments
    }
    
    /// Set all color adjustments at once
    pub fn set_adjustments(&mut self, adjustments: ColorAdjustments) -> Result<()> {
        self.adjustments = adjustments;
        self.apply_adjustments()
    }
    
    /// Reset all color adjustments to default values
    pub fn reset_adjustments(&mut self) -> Result<()> {
        self.adjustments = ColorAdjustments::default();
        self.apply_adjustments()
    }
    
    /// Create a preset from current settings
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
    
    /// Apply a preset
    pub fn apply_preset(&mut self, name: &str) -> Result<()> {
        let preset = self.presets.get(name).ok_or_else(|| {
            anyhow::anyhow!("Preset '{}' not found", name)
        })?;
        
        self.adjustments = preset.adjustments;
        self.curves = preset.curves.clone();
        self.lut = preset.lut.clone();
        self.active_preset = Some(name.to_string());
        
        self.apply_adjustments()?;
        
        // Apply LUT if available
        if let Some(lut) = &self.lut {
            self.apply_lut(lut)?;
        } else {
            self.clear_lut()?;
        }
        
        // Apply curves
        self.apply_curves()?;
        
        Ok(())
    }
    
    /// Get all available presets
    pub fn get_presets(&self) -> Vec<&GradingPreset> {
        self.presets.values().collect()
    }
    
    /// Delete a preset
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
    
    /// Get the currently active preset
    pub fn get_active_preset(&self) -> Option<&GradingPreset> {
        self.active_preset.as_ref().and_then(|name| self.presets.get(name))
    }
    
    /// Load a LUT from a file
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
            self.apply_lut(&lut_settings)?;
    }
    
    /// Pull a processed frame from the appsink
    fn pull_processed_frame(&self) -> Result<Vec<u8>> {
        // Get the appsink element
        let sink = self.elements.get("sink")
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
    
    /// Start the color grading pipeline for continuous processing
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
    
    /// Pause the color grading pipeline
    pub fn pause(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            debug!("Pausing color grading pipeline");
            pipeline.set_state(gst::State::Paused)?;
        }
        
        Ok(())
    }
    
    /// Stop the color grading pipeline
    pub fn stop(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            debug!("Stopping color grading pipeline");
            pipeline.set_state(gst::State::Ready)?;
        }
        
        Ok(())
    }    
        // Check if LUT element exists
        let lut_element = match self.elements.get("lut") {
            Some(element) => element,
            None => return Err(anyhow::anyhow!("LUT element not available")),
        };
        
        // Different handling based on LUT format
        match lut_settings.format {
            LutFormat::CUBE => self.apply_cube_lut(lut_element, lut_settings)?,
            LutFormat::ThreeDL => self.apply_3dl_lut(lut_element, lut_settings)?,
            LutFormat::HALD => self.apply_hald_lut(lut_element, lut_settings)?,
            LutFormat::PNG | LutFormat::JPEG => self.apply_image_lut(lut_element, lut_settings)?,
        }
        
        debug!("Applied LUT: {}", lut_settings.path.display());
        Ok(())
    }
    
    /// Apply a CUBE format LUT
    fn apply_cube_lut(&self, element: &gst::Element, lut_settings: &LutSettings) -> Result<()> {
        // For now, we're using a simplified approach with videobalance
        // In a real implementation, you would parse the CUBE file and apply its values
        // to a custom shader or LUT element
        
        debug!("Applying CUBE LUT: {}", lut_settings.path.display());
        
        // Set LUT strength via a property if available
        if element.has_property("lut-strength", None) {
            element.set_property("lut-strength", lut_settings.strength);
        }
        
        Ok(())
    }
    
    /// Apply a 3DL format LUT
    fn apply_3dl_lut(&self, element: &gst::Element, lut_settings: &LutSettings) -> Result<()> {
        debug!("Applying 3DL LUT: {}", lut_settings.path.display());
        
        // Similar to CUBE format, would need custom implementation
        if element.has_property("lut-strength", None) {
            element.set_property("lut-strength", lut_settings.strength);
        }
        
        Ok(())
    }
    
    /// Apply a HALD image LUT
    fn apply_hald_lut(&self, element: &gst::Element, lut_settings: &LutSettings) -> Result<()> {
        debug!("Applying HALD LUT: {}", lut_settings.path.display());
        
        // HALD LUTs are special image-based LUTs
        if element.has_property("lut-path", None) {
            element.set_property("lut-path", lut_settings.path.to_str().unwrap());
        }
        
        if element.has_property("lut-strength", None) {
            element.set_property("lut-strength", lut_settings.strength);
        }
        
        Ok(())
    }
    
    /// Apply an image-based LUT (PNG or JPEG)
    fn apply_image_lut(&self, element: &gst::Element, lut_settings: &LutSettings) -> Result<()> {
        debug!("Applying image LUT: {}", lut_settings.path.display());
        
        // Image-based LUTs would need to be loaded and processed
        if element.has_property("lut-path", None) {
            element.set_property("lut-path", lut_settings.path.to_str().unwrap());
        }
        
        if element.has_property("lut-strength", None) {
            element.set_property("lut-strength", lut_settings.strength);
        }
        
        Ok(())
    }
    
    /// Clear any applied LUT
    pub fn clear_lut(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        
        if let Some(lut_element) = self.elements.get("lut") {
            // Reset LUT element to default state
            if lut_element.has_property("lut-strength", None) {
                lut_element.set_property("lut-strength", 0.0);
            }
            
            // Reset other LUT-related properties
            if lut_element.has_property("lut-path", None) {
                lut_element.set_property("lut-path", "");
            }
        }
        
        self.lut = None;
        debug!("Cleared LUT");
        
        Ok(())
    }
    
    /// Set LUT strength
    pub fn set_lut_strength(&mut self, strength: f32) -> Result<()> {
        let strength = strength.clamp(0.0, 1.0);
        
        if let Some(lut) = &mut self.lut {
            lut.strength = strength;
            
            if self.initialized {
                if let Some(lut_element) = self.elements.get("lut") {
                    if lut_element.has_property("lut-strength", None) {
                        lut_element.set_property("lut-strength", strength);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply color curves
    pub fn apply_curves(&self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }
        
        // Check if we have any curves to apply
        if self.curves.rgb.len() < 2 && 
           self.curves.red.len() < 2 && 
           self.curves.green.len() < 2 && 
           self.curves.blue.len() < 2 && 
           self.curves.luma.len() < 2 {
            debug!("No curves to apply");
            return Ok(());
        }
        
        // Apply RGB curve to gamma element if available
        if self.curves.rgb.len() >= 2 {
            if let Some(gamma) = self.elements.get("gamma") {
                // In a real implementation, we would calculate a proper gamma value
                // based on the curve. For now, we'll use a simplified approach.
                let mid_point = self.find_curve_mid_point(&self.curves.rgb);
                let gamma_value = if mid_point > 0.5 {
                    // Curve is above linear, reduce gamma (brighten)
                    1.0 - ((mid_point - 0.5) * 2.0).min(0.9)
                } else {
                    // Curve is below linear, increase gamma (darken)
                    1.0 + ((0.5 - mid_point) * 2.0).min(2.0)
                };
                
                gamma.set_property("gamma", gamma_value);
                debug!("Applied RGB curve with gamma: {}", gamma_value);
            }
        }
        
        // Apply individual channel curves
        // In a real implementation, we would use a custom element or shader
        // For now, we'll just log that we would apply them
        if self.curves.red.len() >= 2 {
            debug!("Would apply red channel curve with {} points", self.curves.red.len());
        }
        
        if self.curves.green.len() >= 2 {
            debug!("Would apply green channel curve with {} points", self.curves.green.len());
        }
        
        if self.curves.blue.len() >= 2 {
            debug!("Would apply blue channel curve with {} points", self.curves.blue.len());
        }
        
        if self.curves.luma.len() >= 2 {
            debug!("Would apply luma curve with {} points", self.curves.luma.len());
        }
        
        debug!("Applied color curves");
        Ok(())
    }
    
    /// Find the mid-point of a curve (value at x=0.5)
    fn find_curve_mid_point(&self, curve: &[CurvePoint]) -> f32 {
        // Find the points that bracket x=0.5
        let mut prev_point = &curve[0];
        
        for point in curve.iter().skip(1) {
            if point.x >= 0.5 {
                // Linear interpolation between the two points
                let t = (0.5 - prev_point.x) / (point.x - prev_point.x);
                return prev_point.y + t * (point.y - prev_point.y);
            }
            prev_point = point;
        }
        
        // If we didn't find a bracket, return the last point's y value
        prev_point.y
    }
    
    /// Set a specific curve
    pub fn set_curve(&mut self, curve_type: &str, points: Vec<CurvePoint>) -> Result<()> {
        // Validate points
        if points.len() < 2 {
            return Err(anyhow::anyhow!("Curve must have at least 2 points"));
        }
        
        // Sort points by x coordinate
        let mut sorted_points = points;
        sorted_points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        
        // Ensure first point is at x=0 and last point is at x=1
        if sorted_points[0].x != 0.0 {
            return Err(anyhow::anyhow!("First curve point must be at x=0"));
        }
        
        if sorted_points[sorted_points.len() - 1].x != 1.0 {
            return Err(anyhow::anyhow!("Last curve point must be at x=1"));
        }
        
        // Update the appropriate curve
        match curve_type {
            "rgb" => self.curves.rgb = sorted_points,
            "red" => self.curves.red = sorted_points,
            "green" => self.curves.green = sorted_points,
            "blue" => self.curves.blue = sorted_points,
            "luma" => self.curves.luma = sorted_points,
            _ => return Err(anyhow::anyhow!("Unknown curve type: {}", curve_type)),
        }
        
        // Apply the curves if initialized
        if self.initialized {
            self.apply_curves()?;
        }
        
        Ok(())
    }
    
    /// Reset a specific curve to linear
    pub fn reset_curve(&mut self, curve_type: &str) -> Result<()> {
        let default_curve = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        
        match curve_type {
            "rgb" => self.curves.rgb = default_curve,
            "red" => self.curves.red = default_curve,
            "green" => self.curves.green = default_curve,
            "blue" => self.curves.blue = default_curve,
            "luma" => self.curves.luma = default_curve,
            "all" => {
                self.curves = ColorCurves::default();
            },
            _ => return Err(anyhow::anyhow!("Unknown curve type: {}", curve_type)),
        }
        
        // Apply the curves if initialized
        if self.initialized {
            self.apply_curves()?;
        }
        
        Ok(())
    }
    
    /// Get a specific curve
    pub fn get_curve(&self, curve_type: &str) -> Result<&[CurvePoint]> {
        match curve_type {
            "rgb" => Ok(&self.curves.rgb),
            "red" => Ok(&self.curves.red),
            "green" => Ok(&self.curves.green),
            "blue" => Ok(&self.curves.blue),
            "luma" => Ok(&self.curves.luma),
            _ => Err(anyhow::anyhow!("Unknown curve type: {}", curve_type)),
        }
    }
    
    /// Configure a scope
    pub fn configure_scope(&mut self, scope_type: ScopeType, config: ScopeConfig) -> Result<()> {
        self.scopes.insert(scope_type, config);
        
        // If we're initialized and this is the first scope being configured with continuous updates,
        // set up the update timer
        if self.initialized && config.continuous_update && self.scope_update_timeout_id.is_none() {
            self.setup_scope_update_timer()?;
        }
        
        Ok(())
    }
    
    /// Enable a scope
    pub fn enable_scope(&mut self, scope_type: ScopeType, width: u32, height: u32, continuous_update: bool) -> Result<()> {
        let config = ScopeConfig {
            scope_type,
            width,
            height,
            continuous_update,
            update_interval_ms: 100, // Default update interval
        };
        
        self.configure_scope(scope_type, config)
    }
    
    /// Disable a scope
    pub fn disable_scope(&mut self, scope_type: ScopeType) -> Result<()> {
        self.scopes.remove(&scope_type);
        
        // If no more continuous scopes, remove the update timer
        if !self.has_continuous_scopes() && self.scope_update_timeout_id.is_some() {
            self.remove_scope_update_timer();
        }
        
        Ok(())
    }
    
    /// Check if any scopes are configured for continuous updates
    fn has_continuous_scopes(&self) -> bool {
        self.scopes.values().any(|config| config.continuous_update)
    }
    
    /// Set up the timer for continuous scope updates
    fn setup_scope_update_timer(&mut self) -> Result<()> {
        // Remove any existing timer
        self.remove_scope_update_timer();
        
        // Find the minimum update interval among all continuous scopes
        let min_interval = self.scopes.values()
            .filter(|config| config.continuous_update)
            .map(|config| config.update_interval_ms)
            .min()
            .unwrap_or(100);
        
        // Create a weak reference to self to avoid circular references
        let weak_self = Arc::downgrade(&Arc::new(Mutex::new(self)));
        
        // Set up a new timer
        let timeout_id = glib::timeout_add_local(std::time::Duration::from_millis(min_interval as u64), move || {
            if let Some(arc_self) = weak_self.upgrade() {
                if let Ok(mut this) = arc_self.lock() {
                    if let Err(e) = this.update_scopes() {
                        error!("Error updating scopes: {}", e);
                    }
                    return glib::Continue(true);
                }
            }
            glib::Continue(false)
        });
        
        self.scope_update_timeout_id = Some(timeout_id);
        Ok(())
    }
    
    /// Remove the scope update timer
    fn remove_scope_update_timer(&mut self) {
        if let Some(timeout_id) = self.scope_update_timeout_id.take() {
            timeout_id.remove();
        }
    }
    
    /// Update all active scopes
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
    
    /// Update a specific scope
    fn update_scope(&self, scope_type: ScopeType, config: &ScopeConfig) -> Result<ScopeData> {
        // In a real implementation, we would tap into the GStreamer pipeline
        // and extract the video frame data to generate the scope data
        
        // For now, we'll generate some dummy data for demonstration
        let data = match scope_type {
            ScopeType::Histogram => self.generate_histogram_data(config)?,
            ScopeType::Waveform => self.generate_waveform_data(config)?,
            ScopeType::Vectorscope => self.generate_vectorscope_data(config)?,
            ScopeType::RGBParade => self.generate_rgb_parade_data(config)?,
        };
        
        Ok(data)
    }
    
    /// Generate histogram data
    fn generate_histogram_data(&self, config: &ScopeConfig) -> Result<ScopeData> {
        // In a real implementation, we would analyze the video frame
        // and generate a histogram of color/luminance values
        
        // For demonstration, we'll generate a dummy histogram
        let mut histogram = vec![0u8; config.width as usize * 3]; // RGB histogram
        
        // Fill with dummy data
        for i in 0..config.width as usize {
            // Generate some variation based on current adjustments
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
    
    /// Generate waveform data
    fn generate_waveform_data(&self, config: &ScopeConfig) -> Result<ScopeData> {
        // In a real implementation, we would analyze the video frame
        // and generate a waveform showing luminance distribution
        
        // For demonstration, we'll generate a dummy waveform
        let mut waveform = vec![0u8; config.width as usize * config.height as usize];
        
        // Fill with dummy data - a simple sine wave
        for x in 0..config.width as usize {
            let y_pos = ((((x as f32 / config.width as f32) * 10.0 * std::f32::consts::PI).sin() + 1.0) / 2.0 
                      * config.height as f32) as usize;
            if y_pos < config.height as usize {
                waveform[y_pos * config.width as usize + x] = 255;
            }
        }
        
        Ok(ScopeData {
            scope_type: ScopeType::Waveform,
            width: config.width,
            height: config.height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            data: ScopeDataFormat::Raw(waveform),
        })
    }
    
    /// Generate vectorscope data
    fn generate_vectorscope_data(&self, config: &ScopeConfig) -> Result<ScopeData> {
        // In a real implementation, we would analyze the video frame
        // and generate a vectorscope showing color distribution
        
        // For demonstration, we'll generate a dummy vectorscope
        let mut vectorscope = vec![0u8; config.width as usize * config.height as usize * 3]; // RGB data
        
        // Fill with dummy data - a simple color wheel
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
                    
                    // Simple HSV to RGB conversion for the example
                    let idx = (y * config.width as usize + x) * 3;
                    vectorscope[idx] = hue;
                    vectorscope[idx + 1] = saturation;
                    vectorscope[idx + 2] = 255; // Value always max for visibility
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
    
    /// Generate RGB parade data
    fn generate_rgb_parade_data(&self, config: &ScopeConfig) -> Result<ScopeData> {
        // In a real implementation, we would analyze the video frame
        // and generate an RGB parade showing RGB channel distribution
        
        // For demonstration, we'll generate a dummy RGB parade
        let parade_width = config.width / 3; // Split width into 3 sections for R, G, B
        let mut rgb_parade = vec![0u8; config.width as usize * config.height as usize * 3]; // RGB data
        
        // Fill with dummy data - simple gradients for each channel
        for y in 0..config.height as usize {
            let y_value = 255 - (y as f32 / config.height as f32 * 255.0) as u8;
            
            // Red channel
            for x in 0..parade_width as usize {
                let idx = (y * config.width as usize + x) * 3;
                rgb_parade[idx] = y_value;
                rgb_parade[idx + 1] = 0;
                rgb_parade[idx + 2] = 0;
            }
            
            // Green channel
            for x in parade_width as usize..(parade_width * 2) as usize {
                let idx = (y * config.width as usize + x) * 3;
                rgb_parade[idx] = 0;
                rgb_parade[idx + 1] = y_value;
                rgb_parade[idx + 2] = 0;
            }
            
            // Blue channel
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
    
    /// Get scope data for a specific scope type
    pub fn get_scope_data(&self, scope_type: ScopeType) -> Result<ScopeData> {
        let config = self.scopes.get(&scope_type).ok_or_else(|| {
            anyhow::anyhow!("Scope {:?} not configured", scope_type)
        })?;
        
        self.update_scope(scope_type, config)
    }
    
    /// Get all configured scopes
    pub fn get_configured_scopes(&self) -> Vec<ScopeType> {
        self.scopes.keys().copied().collect()
    }
    
    /// Check if the engine is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Get a GStreamer element by name
    pub fn get_element(&self, name: &str) -> Option<&gst::Element> {
        self.elements.get(name)
    }
}
