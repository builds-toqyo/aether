//! Common functionality for all scope processors
//! 
//! This module provides shared components, utilities, and base functionality
//! used across all scope implementations (vectorscope, histogram, waveform).

use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use log::{debug, info};
use image::{Rgb, RgbImage};

use crate::types::{ScopeStats, ScopeResolution, ColorSpace};

/// Base processor with common functionality for all scopes
pub struct BaseScopeProcessor {
    stats: Arc<RwLock<ScopeStats>>,
    processing_buffers: ProcessingBuffers,
    color_converter: ColorConverter,
}

impl BaseScopeProcessor {
    /// Create a new base scope processor
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(ScopeStats::new())),
            processing_buffers: ProcessingBuffers::new(),
            color_converter: ColorConverter::new(ColorSpace::Rec709),
        }
    }
    
    /// Get reference to statistics
    pub fn stats(&self) -> &Arc<RwLock<ScopeStats>> {
        &self.stats
    }
    
    /// Get reference to processing buffers
    pub fn buffers(&mut self) -> &mut ProcessingBuffers {
        &mut self.processing_buffers
    }
    
    /// Get reference to color converter
    pub fn color_converter(&self) -> &ColorConverter {
        &self.color_converter
    }
    
    /// Update processing statistics
    pub fn update_stats(&self, frame_number: u64, timestamp: f64, processing_time: f64, pixels_processed: u64) -> Result<()> {
        if let Ok(mut stats) = self.stats.write() {
            stats.frames_processed += 1;
            stats.total_pixels_processed += pixels_processed;
            
            if processing_time > stats.peak_processing_time_us {
                stats.peak_processing_time_us = processing_time;
            }
            
            // Update average processing time
            stats.avg_processing_time_us = 
                (stats.avg_processing_time_us * (stats.frames_processed - 1) as f64 + processing_time) / 
                stats.frames_processed as f64;
        }
        
        Ok(())
    }
    
    /// Get processing statistics
    pub fn get_stats(&self) -> Result<ScopeStats> {
        let stats = self.stats.read().map_err(|e| anyhow!("Stats lock error: {}", e))?;
        Ok(stats.clone())
    }
}

/// Processing buffers for scope operations
pub struct ProcessingBuffers {
    rgb_samples: Vec<[u8; 3]>,
    luma_samples: Vec<u8>,
    temp_buffer: Vec<u8>,
}

impl ProcessingBuffers {
    /// Create new processing buffers
    pub fn new() -> Self {
        Self {
            rgb_samples: Vec::new(),
            luma_samples: Vec::new(),
            temp_buffer: Vec::new(),
        }
    }
    
    /// Reserve capacity for buffers
    pub fn reserve(&mut self, capacity: usize) {
        self.rgb_samples.reserve(capacity);
        self.luma_samples.reserve(capacity);
        self.temp_buffer.reserve(capacity);
    }
    
    /// Clear all buffers
    pub fn clear(&mut self) {
        self.rgb_samples.clear();
        self.luma_samples.clear();
        self.temp_buffer.clear();
    }
    
    /// Add RGB sample
    pub fn add_rgb_sample(&mut self, r: u8, g: u8, b: u8) {
        self.rgb_samples.push([r, g, b]);
    }
    
    /// Add luma sample
    pub fn add_luma_sample(&mut self, luma: u8) {
        self.luma_samples.push(luma);
    }
    
    /// Get RGB samples
    pub fn rgb_samples(&self) -> &[ [u8; 3] ] {
        &self.rgb_samples
    }
    
    /// Get luma samples
    pub fn luma_samples(&self) -> &[u8] {
        &self.luma_samples
    }
    
    /// Get mutable temp buffer
    pub fn temp_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.temp_buffer
    }
}

/// Color conversion utilities
pub struct ColorConverter {
    color_space: ColorSpace,
    rgb_to_yuv_matrix: [[f32; 3]; 3],
    luma_coefficients: (f32, f32, f32),
}

impl ColorConverter {
    /// Create new color converter
    pub fn new(color_space: ColorSpace) -> Self {
        let rgb_to_yuv_matrix = Self::get_rgb_to_yuv_matrix(color_space);
        let luma_coefficients = Self::get_luma_coefficients(color_space);
        
        Self {
            color_space,
            rgb_to_yuv_matrix,
            luma_coefficients,
        }
    }
    
    /// Convert RGB to UV coordinates
    pub fn rgb_to_uv(&self, r: u8, g: u8, b: u8) -> (f32, f32) {
        let rf = r as f32 / 255.0;
        let gf = g as f32 / 255.0;
        let bf = b as f32 / 255.0;
        
        // Convert RGB to YUV
        let y = self.rgb_to_yuv_matrix[0][0] * rf + 
               self.rgb_to_yuv_matrix[0][1] * gf + 
               self.rgb_to_yuv_matrix[0][2] * bf;
        let u = self.rgb_to_yuv_matrix[1][0] * rf + 
               self.rgb_to_yuv_matrix[1][1] * gf + 
               self.rgb_to_yuv_matrix[1][2] * bf;
        let v = self.rgb_to_yuv_matrix[2][0] * rf + 
               self.rgb_to_yuv_matrix[2][1] * gf + 
               self.rgb_to_yuv_matrix[2][2] * bf;
        
        // Extract UV coordinates
        (u, v)
    }
    
    /// Convert RGB to luma
    pub fn rgb_to_luma(&self, r: u8, g: u8, b: u8) -> u8 {
        let rf = r as f32 / 255.0;
        let gf = g as f32 / 255.0;
        let bf = b as f32 / 255.0;
        
        let luma = self.luma_coefficients.0 * rf + 
                  self.luma_coefficients.1 * gf + 
                  self.luma_coefficients.2 * bf;
        
        (luma * 255.0) as u8
    }
    
    /// Get current color space
    pub fn color_space(&self) -> ColorSpace {
        self.color_space
    }
    
    /// Update color space
    pub fn update_color_space(&mut self, color_space: ColorSpace) {
        self.color_space = color_space;
        self.rgb_to_yuv_matrix = Self::get_rgb_to_yuv_matrix(color_space);
        self.luma_coefficients = Self::get_luma_coefficients(color_space);
    }
    
    /// Get RGB to YUV conversion matrix for color space
    fn get_rgb_to_yuv_matrix(color_space: ColorSpace) -> [[f32; 3]; 3] {
        match color_space {
            ColorSpace::Rec709 => [
                [0.2126, 0.7152, 0.0722],  // Y
                [-0.1146, -0.3854, 0.5000], // U
                [0.5000, -0.4542, -0.0458], // V
            ],
            ColorSpace::Rec601 => [
                [0.299, 0.587, 0.114],    // Y
                [-0.147, -0.289, 0.436],   // U
                [0.615, -0.515, -0.100],   // V
            ],
            ColorSpace::Rec2020 => [
                [0.2627, 0.6780, 0.0593],  // Y
                [-0.1396, -0.3604, 0.5000], // U
                [0.5000, -0.4598, -0.0402], // V
            ],
            _ => Self::get_rgb_to_yuv_matrix(ColorSpace::Rec709),
        }
    }
    
    /// Get luma coefficients for color space
    fn get_luma_coefficients(color_space: ColorSpace) -> (f32, f32, f32) {
        match color_space {
            ColorSpace::Rec709 => (0.2126, 0.7152, 0.0722),
            ColorSpace::Rec601 => (0.299, 0.587, 0.114),
            ColorSpace::Rec2020 => (0.2627, 0.6780, 0.0593),
            _ => (0.2126, 0.7152, 0.0722), // Default to Rec709
        }
    }
}

/// Image rendering utilities
pub struct ImageRenderer;

impl ImageRenderer {
    /// Clear image background
    pub fn clear_background(image: &mut RgbImage, color: Rgb<u8>) {
        for pixel in image.pixels_mut() {
            *pixel = color;
        }
    }
    
    /// Draw grid lines on image
    pub fn draw_grid(image: &mut RgbImage, grid_color: Rgb<u8>) {
        let (width, height) = image.dimensions();
        
        // Draw vertical lines every 32 bins
        for x in (0..width.min(256)).step_by(32) {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    Self::blend_pixel(pixel, grid_color, 0.5);
                }
            }
        }
        
        // Draw horizontal lines at 25%, 50%, 75%
        for y_percent in [25, 50, 75] {
            let y = height * (100 - y_percent) / 100;
            for x in 0..width {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    Self::blend_pixel(pixel, grid_color, 0.5);
                }
            }
        }
    }
    
    /// Draw crosshair at center
    pub fn draw_crosshair(image: &mut RgbImage, cx: u32, cy: u32, color: Rgb<u8>) {
        let (width, height) = image.dimensions();
        
        // Horizontal line
        for x in 0..width {
            if let Some(pixel) = image.get_pixel_mut(x, cy) {
                *pixel = color;
            }
        }
        
        // Vertical line
        for y in 0..height {
            if let Some(pixel) = image.get_pixel_mut(cx, y) {
                *pixel = color;
            }
        }
    }
    
    /// Draw target marker
    pub fn draw_target_marker(image: &mut RgbImage, x: i32, y: i32, color: Rgb<u8>) -> Result<()> {
        let (width, height) = image.dimensions();
        
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            let x = x as u32;
            let y = y as u32;
            
            // Draw cross marker
            for dx in -2..=2 {
                if x as i32 + dx >= 0 && x as i32 + dx < width as i32 {
                    if let Some(pixel) = image.get_pixel_mut((x as i32 + dx) as u32, y) {
                        *pixel = color;
                    }
                }
            }
            
            for dy in -2..=2 {
                if y as i32 + dy >= 0 && y as i32 + dy < height as i32 {
                    if let Some(pixel) = image.get_pixel_mut(x, (y as i32 + dy) as u32) {
                        *pixel = color;
                    }
                }
            }
            
            // Draw circle around target
            for angle in 0..360 {
                let rad = angle as f32 * std::f32::consts::PI / 180.0;
                let cx = x as f32 + rad.cos() * 5.0;
                let cy = y as f32 + rad.sin() * 5.0;
                
                if cx >= 0.0 && cx < width as f32 && cy >= 0.0 && cy < height as f32 {
                    if let Some(pixel) = image.get_pixel_mut(cx as u32, cy as u32) {
                        *pixel = color;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Blend pixels with transparency
    fn blend_pixel(pixel: &mut Rgb<u8>, overlay: Rgb<u8>, alpha: f32) {
        let [r, g, b] = pixel.0;
        let [or, og, ob] = overlay.0;
        
        pixel.0 = [
            (r as f32 * (1.0 - alpha) + or as f32 * alpha) as u8,
            (g as f32 * (1.0 - alpha) + og as f32 * alpha) as u8,
            (b as f32 * (1.0 - alpha) + ob as f32 * alpha) as u8,
        ];
    }
}

/// Frame processing utilities
pub struct FrameProcessor;

impl FrameProcessor {
    /// Process image frame with pixel callback
    pub fn process_frame<F>(&self, image: &RgbImage, mut callback: F) -> (u64, std::time::Duration)
    where
        F: FnMut(u8, u8, u8),
    {
        let start_time = std::time::Instant::now();
        let (width, height) = image.dimensions();
        let mut pixels_processed = 0u64;
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                
                callback(r, g, b);
                pixels_processed += 1;
            }
        }
        
        let processing_time = start_time.elapsed();
        (pixels_processed, processing_time)
    }
    
    /// Process image frame and collect samples
    pub fn collect_samples(&self, image: &RgbImage, buffers: &mut ProcessingBuffers) -> (u64, std::time::Duration) {
        let start_time = std::time::Instant::now();
        
        buffers.clear();
        let (width, height) = image.dimensions();
        buffers.reserve((width * height) as usize);
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                
                buffers.add_rgb_sample(r, g, b);
            }
        }
        
        let processing_time = start_time.elapsed();
        ((width * height) as u64, processing_time)
    }
}

/// Statistical analysis utilities
pub struct Statistics;

impl Statistics {
    /// Calculate channel statistics
    pub fn calculate_channel_stats(data: &[u32; 256]) -> ChannelStatistics {
        let mut stats = ChannelStatistics::new();
        
        // Total samples
        stats.total_samples = data.iter().sum();
        
        if stats.total_samples == 0 {
            return stats;
        }
        
        // Find min and max bins with non-zero values
        for (bin, &count) in data.iter().enumerate() {
            if count > 0 {
                stats.min_bin = stats.min_bin.min(bin as u8);
                stats.max_bin = stats.max_bin.max(bin as u8);
            }
        }
        
        // Calculate mean
        let weighted_sum: u64 = data.iter().enumerate()
            .map(|(bin, &count)| count as u64 * bin as u64)
            .sum();
        stats.mean = (weighted_sum as f32 / stats.total_samples as f32) as u8;
        
        // Calculate median
        let mut cumulative = 0u64;
        for (bin, &count) in data.iter().enumerate() {
            cumulative += count as u64;
            if cumulative >= stats.total_samples / 2 {
                stats.median = bin as u8;
                break;
            }
        }
        
        // Calculate standard deviation
        let variance: f64 = data.iter().enumerate()
            .map(|(bin, &count)| {
                let diff = bin as f32 - stats.mean as f32;
                count as f64 * diff * diff
            })
            .sum();
        stats.standard_deviation = (variance / stats.total_samples as f64).sqrt() as f32;
        
        stats
    }
    
    /// Normalize histogram data to 0-255 range
    pub fn normalize_histogram(data: &[u32; 256]) -> Vec<u8> {
        let max_val = data.iter().copied().max().unwrap_or(0);
        
        if max_val == 0 {
            return vec![0; 256];
        }
        
        data.iter()
            .map(|&val| ((val as f32 / max_val as f32) * 255.0) as u8)
            .collect()
    }
}

/// Statistics for a single channel
#[derive(Debug, Clone)]
pub struct ChannelStatistics {
    pub total_samples: u64,
    pub min_bin: u8,
    pub max_bin: u8,
    pub mean: u8,
    pub median: u8,
    pub standard_deviation: f32,
}

impl ChannelStatistics {
    pub fn new() -> Self {
        Self {
            total_samples: 0,
            min_bin: 255,
            max_bin: 0,
            mean: 0,
            median: 0,
            standard_deviation: 0.0,
        }
    }
    
    pub fn dynamic_range(&self) -> u8 {
        self.max_bin - self.min_bin
    }
}

impl Default for BaseScopeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ProcessingBuffers {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ColorConverter {
    fn default() -> Self {
        Self::new(ColorSpace::Rec709)
    }
}
