//! Color scope data structures for professional color grading
//! 
//! This module provides comprehensive data structures for implementing
//! professional color scopes similar to DaVinci Resolve, including
//! waveform monitors, vectorscopes, and histograms.

use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

/// Color scope data container for all scope types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScopeData {
    /// Waveform data (luma and RGB parade)
    pub waveform: WaveformData,
    /// Vectorscope data (UV coordinates)
    pub vectorscope: VectorscopeData,
    /// Histogram data (RGB channels)
    pub histogram: HistogramData,
    /// Scope metadata and configuration
    pub metadata: ScopeMetadata,
}

impl ColorScopeData {
    /// Create new empty scope data
    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            waveform: WaveformData::new(resolution),
            vectorscope: VectorscopeData::new(resolution),
            histogram: HistogramData::new(resolution),
            metadata: ScopeMetadata::new(),
        }
    }
    
    /// Clear all scope data
    pub fn clear(&mut self) {
        self.waveform.clear();
        self.vectorscope.clear();
        self.histogram.clear();
        self.metadata.reset();
    }
    
    /// Get scope data size in bytes
    pub fn size_bytes(&self) -> usize {
        self.waveform.size_bytes() + self.vectorscope.size_bytes() + self.histogram.size_bytes()
    }
}

/// Waveform data for luma and RGB parade display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformData {
    /// Luma waveform data (0-255 values for each horizontal position)
    pub luma: Vec<u8>,
    /// Red channel parade data
    pub red: Vec<u8>,
    /// Green channel parade data
    pub green: Vec<u8>,
    /// Blue channel parade data
    pub blue: Vec<u8>,
    /// Waveform configuration
    pub config: WaveformConfig,
}

impl WaveformData {
    /// Create new waveform data
    pub fn new(resolution: ScopeResolution) -> Self {
        let width = resolution.width();
        let height = resolution.height();
        
        Self {
            luma: vec![0; width * height],
            red: vec![0; width * height],
            green: vec![0; width * height],
            blue: vec![0; width * height],
            config: WaveformConfig::new(resolution),
        }
    }
    
    /// Clear waveform data
    pub fn clear(&mut self) {
        self.luma.fill(0);
        self.red.fill(0);
        self.green.fill(0);
        self.blue.fill(0);
    }
    
    /// Get waveform data for specific channel
    pub fn channel_data(&self, channel: WaveformChannel) -> &[u8] {
        match channel {
            WaveformChannel::Luma => &self.luma,
            WaveformChannel::Red => &self.red,
            WaveformChannel::Green => &self.green,
            WaveformChannel::Blue => &self.blue,
        }
    }
    
    /// Get mutable waveform data for specific channel
    pub fn channel_data_mut(&mut self, channel: WaveformChannel) -> &mut [u8] {
        match channel {
            WaveformChannel::Luma => &mut self.luma,
            WaveformChannel::Red => &mut self.red,
            WaveformChannel::Green => &mut self.green,
            WaveformChannel::Blue => &mut self.blue,
        }
    }
    
    /// Set waveform data for specific position
    pub fn set_pixel(&mut self, x: usize, y: usize, channel: WaveformChannel, value: u8) {
        let width = self.config.resolution.width();
        if x < width && y < self.config.resolution.height() {
            let index = y * width + x;
            self.channel_data_mut(channel)[index] = value;
        }
    }
    
    /// Get waveform data size in bytes
    pub fn size_bytes(&self) -> usize {
        self.luma.len() + self.red.len() + self.green.len() + self.blue.len()
    }
}

/// Vectorscope data for UV coordinate plotting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorscopeData {
    /// UV coordinate data points
    pub points: Vec<VectorscopePoint>,
    /// Intensity grid for color density visualization
    pub intensity_grid: Vec<u16>,
    /// Vectorscope configuration
    pub config: VectorscopeConfig,
}

impl VectorscopeData {
    /// Create new vectorscope data
    pub fn new(resolution: ScopeResolution) -> Self {
        let grid_size = resolution.vectorscope_grid_size();
        
        Self {
            points: Vec::new(),
            intensity_grid: vec![0; grid_size * grid_size],
            config: VectorscopeConfig::new(resolution),
        }
    }
    
    /// Clear vectorscope data
    pub fn clear(&mut self) {
        self.points.clear();
        self.intensity_grid.fill(0);
    }
    
    /// Add a color point to the vectorscope
    pub fn add_point(&mut self, u: f32, v: f32, intensity: u16) {
        let point = VectorscopePoint::new(u, v, intensity);
        self.points.push(point);
        
        // Update intensity grid
        let grid_x = ((u + 0.5) * self.config.resolution.vectorscope_grid_size() as f32) as usize;
        let grid_y = ((v + 0.5) * self.config.resolution.vectorscope_grid_size() as f32) as usize;
        
        if grid_x < self.config.resolution.vectorscope_grid_size() && 
           grid_y < self.config.resolution.vectorscope_grid_size() {
            let index = grid_y * self.config.resolution.vectorscope_grid_size() + grid_x;
            self.intensity_grid[index] = self.intensity_grid[index].saturating_add(intensity);
        }
    }
    
    /// Get intensity at specific UV coordinate
    pub fn get_intensity(&self, u: f32, v: f32) -> u16 {
        let grid_x = ((u + 0.5) * self.config.resolution.vectorscope_grid_size() as f32) as usize;
        let grid_y = ((v + 0.5) * self.config.resolution.vectorscope_grid_size() as f32) as usize;
        
        if grid_x < self.config.resolution.vectorscope_grid_size() && 
           grid_y < self.config.resolution.vectorscope_grid_size() {
            let index = grid_y * self.config.resolution.vectorscope_grid_size() + grid_x;
            self.intensity_grid[index]
        } else {
            0
        }
    }
    
    /// Get vectorscope data size in bytes
    pub fn size_bytes(&self) -> usize {
        self.points.len() * std::mem::size_of::<VectorscopePoint>() + 
        self.intensity_grid.len() * std::mem::size_of::<u16>()
    }
}

/// Individual vectorscope point
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VectorscopePoint {
    /// U coordinate (-0.5 to 0.5)
    pub u: f32,
    /// V coordinate (-0.5 to 0.5)
    pub v: f32,
    /// Point intensity/value
    pub intensity: u16,
}

impl VectorscopePoint {
    /// Create new vectorscope point
    pub fn new(u: f32, v: f32, intensity: u16) -> Self {
        Self {
            u: u.clamp(-0.5, 0.5),
            v: v.clamp(-0.5, 0.5),
            intensity,
        }
    }
    
    /// Convert RGB to UV coordinates
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        // YUV conversion
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        let u = (b - y) / 1.772;
        let v = (r - y) / 1.402;
        
        Self::new(u, v, 1)
    }
}

/// Histogram data for RGB channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramData {
    /// Red channel histogram (256 bins)
    pub red: [u32; 256],
    /// Green channel histogram (256 bins)
    pub green: [u32; 256],
    /// Blue channel histogram (256 bins)
    pub blue: [u32; 256],
    /// Luma histogram (256 bins)
    pub luma: [u32; 256],
    /// Histogram configuration
    pub config: HistogramConfig,
}

impl HistogramData {
    /// Create new histogram data
    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            red: [0; 256],
            green: [0; 256],
            blue: [0; 256],
            luma: [0; 256],
            config: HistogramConfig::new(resolution),
        }
    }
    
    /// Clear histogram data
    pub fn clear(&mut self) {
        self.red.fill(0);
        self.green.fill(0);
        self.blue.fill(0);
        self.luma.fill(0);
    }
    
    /// Add color sample to histogram
    pub fn add_sample(&mut self, r: u8, g: u8, b: u8) {
        self.red[r as usize] += 1;
        self.green[g as usize] += 1;
        self.blue[b as usize] += 1;
        
        // Calculate luma
        let y = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
        self.luma[y as usize] += 1;
    }
    
    /// Get histogram data for specific channel
    pub fn channel_data(&self, channel: HistogramChannel) -> &[u32; 256] {
        match channel {
            HistogramChannel::Red => &self.red,
            HistogramChannel::Green => &self.green,
            HistogramChannel::Blue => &self.blue,
            HistogramChannel::Luma => &self.luma,
        }
    }
    
    /// Get maximum histogram value
    pub fn max_value(&self, channel: HistogramChannel) -> u32 {
        self.channel_data(channel).iter().copied().max().unwrap_or(0)
    }
    
    /// Normalize histogram values to 0-255 range
    pub fn normalize(&self, channel: HistogramChannel) -> Vec<u8> {
        let data = self.channel_data(channel);
        let max_val = self.max_value(channel);
        
        if max_val == 0 {
            return vec![0; 256];
        }
        
        data.iter()
            .map(|&val| ((val as f32 / max_val as f32) * 255.0) as u8)
            .collect()
    }
    
    /// Get histogram data size in bytes
    pub fn size_bytes(&self) -> usize {
        4 * 256 * std::mem::size_of::<u32>() // 4 channels * 256 bins * 4 bytes
    }
}

/// Scope metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeMetadata {
    /// Frame number this data represents
    pub frame_number: u64,
    /// Timestamp in seconds
    pub timestamp: f64,
    /// Resolution of the source video
    pub source_resolution: (u32, u32),
    /// Color space information
    pub color_space: ColorSpace,
    /// Range information (legal/extended)
    pub range: VideoRange,
    /// Processing statistics
    pub stats: ScopeStats,
}

impl ScopeMetadata {
    /// Create new scope metadata
    pub fn new() -> Self {
        Self {
            frame_number: 0,
            timestamp: 0.0,
            source_resolution: (1920, 1080),
            color_space: ColorSpace::Rec709,
            range: VideoRange::Limited,
            stats: ScopeStats::new(),
        }
    }
    
    /// Reset metadata
    pub fn reset(&mut self) {
        self.frame_number = 0;
        self.timestamp = 0.0;
        self.stats = ScopeStats::new();
    }
    
    /// Update frame information
    pub fn update_frame(&mut self, frame_number: u64, timestamp: f64) {
        self.frame_number = frame_number;
        self.timestamp = timestamp;
        self.stats.frames_processed += 1;
    }
}

/// Scope processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeStats {
    /// Number of frames processed
    pub frames_processed: u64,
    /// Average processing time per frame (microseconds)
    pub avg_processing_time_us: f64,
    /// Peak processing time (microseconds)
    pub peak_processing_time_us: f64,
    /// Total pixels processed
    pub total_pixels_processed: u64,
}

impl ScopeStats {
    /// Create new scope statistics
    pub fn new() -> Self {
        Self {
            frames_processed: 0,
            avg_processing_time_us: 0.0,
            peak_processing_time_us: 0.0,
            total_pixels_processed: 0,
        }
    }
    
    /// Update processing statistics
    pub fn update_processing_time(&mut self, processing_time_us: f64, pixels_processed: u64) {
        self.total_pixels_processed += pixels_processed;
        
        if processing_time_us > self.peak_processing_time_us {
            self.peak_processing_time_us = processing_time_us;
        }
        
        // Update average processing time
        if self.frames_processed > 0 {
            self.avg_processing_time_us = 
                (self.avg_processing_time_us * (self.frames_processed - 1) as f64 + processing_time_us) / 
                self.frames_processed as f64;
        } else {
            self.avg_processing_time_us = processing_time_us;
        }
    }
}

// Configuration enums and structs

/// Scope resolution settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeResolution {
    Low,    // 640x480
    Medium, // 1280x720
    High,   // 1920x1080
    Ultra,  // 3840x2160
}

impl ScopeResolution {
    /// Get width for this resolution
    pub fn width(&self) -> usize {
        match self {
            ScopeResolution::Low => 640,
            ScopeResolution::Medium => 1280,
            ScopeResolution::High => 1920,
            ScopeResolution::Ultra => 3840,
        }
    }
    
    /// Get height for this resolution
    pub fn height(&self) -> usize {
        match self {
            ScopeResolution::Low => 480,
            ScopeResolution::Medium => 720,
            ScopeResolution::High => 1080,
            ScopeResolution::Ultra => 2160,
        }
    }
    
    /// Get vectorscope grid size
    pub fn vectorscope_grid_size(&self) -> usize {
        match self {
            ScopeResolution::Low => 256,
            ScopeResolution::Medium => 512,
            ScopeResolution::High => 1024,
            ScopeResolution::Ultra => 2048,
        }
    }
}

/// Waveform channel selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveformChannel {
    Luma,
    Red,
    Green,
    Blue,
}

/// Histogram channel selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistogramChannel {
    Red,
    Green,
    Blue,
    Luma,
}

/// Color space specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSpace {
    Rec601,
    Rec709,
    Rec2020,
    DCIP3,
    SRGB,
    ProPhotoRGB,
}

/// Video range (legal vs extended)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoRange {
    Limited, // 16-235 (legal)
    Full,    // 0-255 (full/extended)
}

/// Waveform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformConfig {
    pub resolution: ScopeResolution,
    pub mode: WaveformMode,
    pub scale: WaveformScale,
    pub filtering: bool,
}

impl WaveformConfig {
    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            resolution,
            mode: WaveformMode::Parade,
            scale: WaveformScale::IRE,
            filtering: true,
        }
    }
}

/// Waveform display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveformMode {
    Luma,
    Parade,
    Overlay,
}

/// Waveform scale type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveformScale {
    IRE,    // 0-100 IRE units
    Millivolts, // 0-700mV
    Bits,   // 0-255 or 0-1023
}

/// Vectorscope configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorscopeConfig {
    pub resolution: ScopeResolution,
    pub targets: Vec<VectorscopeTarget>,
    pub scale: VectorscopeScale,
    pub filtering: bool,
}

impl VectorscopeConfig {
    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            resolution,
            targets: vec![
                VectorscopeTarget::Primary,
                VectorscopeTarget::SkinTones,
                VectorscopeTarget::Blue,
                VectorscopeTarget::Yellow,
                VectorscopeTarget::Cyan,
                VectorscopeTarget::Green,
                VectorscopeTarget::Magenta,
                VectorscopeTarget::Red,
            ],
            scale: VectorscopeScale::UV,
            filtering: true,
        }
    }
}

/// Vectorscope target colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorscopeTarget {
    Primary,    // White point
    SkinTones,  // Skin tone line
    Blue,       // Blue target
    Yellow,     // Yellow target
    Cyan,       // Cyan target
    Green,      // Green target
    Magenta,    // Magenta target
    Red,        // Red target
}

/// Vectorscope scale type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorscopeScale {
    UV,         // UV coordinates
    YPbPr,      // YPbPr coordinates
    XY,         // CIE xy coordinates
}

/// Histogram configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramConfig {
    pub resolution: ScopeResolution,
    pub mode: HistogramMode,
    pub log_scale: bool,
    pub show_peaks: bool,
}

impl HistogramConfig {
    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            resolution,
            mode: HistogramMode::RGB,
            log_scale: false,
            show_peaks: true,
        }
    }
}

/// Histogram display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistogramMode {
    RGB,
    Luma,
    Individual,
    Parade,
}
