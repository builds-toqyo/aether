//! Histogram implementation for professional color analysis
//! 
//! This module provides a complete histogram system with RGB channel histograms,
//! luma histogram, and real-time analysis capabilities similar to DaVinci Resolve.

use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};
use image::{Rgb, RgbImage};

use crate::types::{
    HistogramData, HistogramChannel, HistogramConfig, HistogramMode,
    ScopeResolution, ColorSpace, VideoRange, ScopeStats,
};

/// Histogram processor for real-time color analysis
pub struct HistogramProcessor {
    config: HistogramConfig,
    data: Arc<RwLock<HistogramData>>,
    stats: Arc<RwLock<ScopeStats>>,
    
    // Processing buffers
    rgb_buffer: Vec<[u8; 3]>,
    luma_buffer: Vec<u8>,
    
    // Color conversion coefficients
    luma_coefficients: (f32, f32, f32), // R, G, B weights for luma calculation
}

impl HistogramProcessor {
    /// Create a new histogram processor
    pub fn new(config: HistogramConfig) -> Result<Self> {
        info!("Creating histogram processor with config: {:?}", config);
        
        let processor = Self {
            config: config.clone(),
            data: Arc::new(RwLock::new(HistogramData::new(config.resolution))),
            stats: Arc::new(RwLock::new(ScopeStats::new())),
            rgb_buffer: Vec::new(),
            luma_buffer: Vec::new(),
            luma_coefficients: Self::get_luma_coefficients(ColorSpace::Rec709),
        };
        
        // Initialize buffers
        processor.rgb_buffer.reserve(1000);
        processor.luma_buffer.reserve(1000);
        
        info!("Histogram processor created successfully");
        
        Ok(processor)
    }
    
    /// Process an image frame and update histogram data
    pub fn process_frame(&mut self, image: &RgbImage, frame_number: u64, timestamp: f64) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        debug!("Processing histogram frame {} at timestamp {:.3}", frame_number, timestamp);
        
        // Clear previous data
        self.clear_data();
        
        // Process each pixel
        let (width, height) = image.dimensions();
        let mut pixels_processed = 0u64;
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                
                // Add sample to histogram
                self.add_sample(r, g, b);
                pixels_processed += 1;
            }
        }
        
        // Update statistics
        let processing_time = start_time.elapsed().as_micros() as f64;
        self.update_stats(frame_number, timestamp, processing_time, pixels_processed);
        
        debug!("Histogram frame {} processed in {:.2}μs, {} pixels", 
               frame_number, processing_time, pixels_processed);
        
        Ok(())
    }
    
    /// Add a color sample to the histogram
    pub fn add_sample(&mut self, r: u8, g: u8, b: u8) {
        let mut data = self.data.write().map_err(|e| anyhow!("Data lock error: {}", e)).unwrap();
        data.add_sample(r, g, b);
    }
    
    /// Get current histogram data
    pub fn get_data(&self) -> Result<HistogramData> {
        let data = self.data.read().map_err(|e| anyhow!("Data lock error: {}", e))?;
        Ok(data.clone())
    }
    
    /// Get histogram data for specific channel
    pub fn get_channel_data(&self, channel: HistogramChannel) -> Result<[u32; 256]> {
        let data = self.data.read().map_err(|e| anyhow!("Data lock error: {}", e))?;
        Ok(*data.channel_data(channel))
    }
    
    /// Get normalized histogram data (0-255 range)
    pub fn get_normalized_data(&self, channel: HistogramChannel) -> Result<Vec<u8>> {
        let data = self.data.read().map_err(|e| anyhow!("Data lock error: {}", e))?;
        Ok(data.normalize(channel))
    }
    
    /// Generate histogram image
    pub fn generate_image(&self, width: u32, height: u32) -> Result<RgbImage> {
        let data = self.data.read().map_err(|e| anyhow!("Data lock error: {}", e))?;
        
        let mut image = RgbImage::new(width, height);
        
        // Clear background
        for pixel in image.pixels_mut() {
            *pixel = Rgb([16, 16, 16]); // Dark background
        }
        
        match self.config.mode {
            HistogramMode::RGB => self.draw_rgb_histogram(&mut image, &data)?,
            HistogramMode::Luma => self.draw_luma_histogram(&mut image, &data)?,
            HistogramMode::Individual => self.draw_individual_histograms(&mut image, &data)?,
            HistogramMode::Parade => self.draw_parade_histograms(&mut image, &data)?,
        }
        
        // Draw grid lines
        self.draw_grid(&mut image)?;
        
        Ok(image)
    }
    
    /// Draw RGB histogram (all channels overlaid)
    fn draw_rgb_histogram(&self, image: &mut RgbImage, data: &HistogramData) -> Result<()> {
        let (width, height) = image.dimensions();
        
        // Get normalized data for each channel
        let red_norm = data.normalize(HistogramChannel::Red);
        let green_norm = data.normalize(HistogramChannel::Green);
        let blue_norm = data.normalize(HistogramChannel::Blue);
        
        // Draw each channel with transparency
        for x in 0..width.min(256) {
            let bin = x as usize;
            
            // Calculate heights
            let red_height = (red_norm[bin] as u32 * height / 256) as u32;
            let green_height = (green_norm[bin] as u32 * height / 256) as u32;
            let blue_height = (blue_norm[bin] as u32 * height / 256) as u32;
            
            // Draw red channel
            for y in (height - red_height)..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([255.min(r + 128), g, b]); // Add red with transparency
                }
            }
            
            // Draw green channel
            for y in (height - green_height)..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([r, 255.min(g + 128), b]); // Add green with transparency
                }
            }
            
            // Draw blue channel
            for y in (height - blue_height)..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([r, g, 255.min(b + 128)]); // Add blue with transparency
                }
            }
        }
        
        Ok(())
    }
    
    /// Draw luma histogram
    fn draw_luma_histogram(&self, image: &mut RgbImage, data: &HistogramData) -> Result<()> {
        let (width, height) = image.dimensions();
        
        let luma_norm = data.normalize(HistogramChannel::Luma);
        
        for x in 0..width.min(256) {
            let bin = x as usize;
            let luma_height = (luma_norm[bin] as u32 * height / 256) as u32;
            
            for y in (height - luma_height)..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    *pixel = Rgb([200, 200, 200]); // Gray for luma
                }
            }
        }
        
        Ok(())
    }
    
    /// Draw individual histograms (side by side)
    fn draw_individual_histograms(&self, image: &mut RgbImage, data: &HistogramData) -> Result<()> {
        let (width, height) = image.dimensions();
        let channel_width = width / 3;
        
        let channels = [
            (HistogramChannel::Red, Rgb([255, 0, 0]), 0),
            (HistogramChannel::Green, Rgb([0, 255, 0]), 1),
            (HistogramChannel::Blue, Rgb([0, 0, 255]), 2),
        ];
        
        for (channel, color, index) in channels {
            let x_offset = (index as u32 * channel_width) as u32;
            let channel_data = data.normalize(channel);
            
            for x in 0..channel_width.min(256) {
                let bin = x as usize;
                let bin_height = (channel_data[bin] as u32 * height / 256) as u32;
                
                for y in (height - bin_height)..height {
                    let img_x = x_offset + x;
                    if img_x < width && let Some(pixel) = image.get_pixel_mut(img_x, y) {
                        *pixel = color;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Draw parade histograms (stacked)
    fn draw_parade_histograms(&self, image: &mut RgbImage, data: &HistogramData) -> Result<()> {
        let (width, height) = image.dimensions();
        let channel_height = height / 3;
        
        let channels = [
            (HistogramChannel::Red, Rgb([255, 0, 0]), 0),
            (HistogramChannel::Green, Rgb([0, 255, 0]), 1),
            (HistogramChannel::Blue, Rgb([0, 0, 255]), 2),
        ];
        
        for (channel, color, index) in channels {
            let y_offset = (index as u32 * channel_height) as u32;
            let channel_data = data.normalize(channel);
            
            for x in 0..width.min(256) {
                let bin = x as usize;
                let bin_height = (channel_data[bin] as u32 * channel_height / 256) as u32;
                
                for y in 0..bin_height {
                    let img_y = y_offset + (channel_height - y);
                    if img_y < height && let Some(pixel) = image.get_pixel_mut(x, img_y) {
                        *pixel = color;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Draw grid lines on histogram
    fn draw_grid(&self, image: &mut RgbImage) -> Result<()> {
        let (width, height) = image.dimensions();
        
        // Draw vertical lines every 32 bins
        for x in (0..width.min(256)).step_by(32) {
            for y in 0..height {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([r/2, g/2, b/2]); // Darken for grid
                }
            }
        }
        
        // Draw horizontal lines at 25%, 50%, 75%
        for y_percent in [25, 50, 75] {
            let y = height * (100 - y_percent) / 100;
            for x in 0..width {
                if let Some(pixel) = image.get_pixel_mut(x, y) {
                    let [r, g, b] = pixel.0;
                    *pixel = Rgb([r/2, g/2, b/2]); // Darken for grid
                }
            }
        }
        
        Ok(())
    }
    
    /// Get histogram statistics
    pub fn get_statistics(&self) -> Result<HistogramStatistics> {
        let data = self.data.read().map_err(|e| anyhow!("Data lock error: {}", e))?;
        
        let mut stats = HistogramStatistics::new();
        
        // Calculate statistics for each channel
        for channel in [HistogramChannel::Red, HistogramChannel::Green, HistogramChannel::Blue, HistogramChannel::Luma] {
            let channel_data = data.channel_data(channel);
            let channel_stats = self.calculate_channel_stats(channel_data);
            
            match channel {
                HistogramChannel::Red => stats.red = channel_stats,
                HistogramChannel::Green => stats.green = channel_stats,
                HistogramChannel::Blue => stats.blue = channel_stats,
                HistogramChannel::Luma => stats.luma = channel_stats,
            }
        }
        
        Ok(stats)
    }
    
    /// Calculate statistics for a single channel
    fn calculate_channel_stats(&self, data: &[u32; 256]) -> ChannelStatistics {
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
    
    /// Get luma coefficients for color space
    fn get_luma_coefficients(color_space: ColorSpace) -> (f32, f32, f32) {
        match color_space {
            ColorSpace::Rec709 => (0.2126, 0.7152, 0.0722),
            ColorSpace::Rec601 => (0.299, 0.587, 0.114),
            ColorSpace::Rec2020 => (0.2627, 0.6780, 0.0593),
            _ => (0.2126, 0.7152, 0.0722), // Default to Rec709
        }
    }
    
    /// Calculate luma from RGB
    fn calculate_luma(&self, r: u8, g: u8, b: u8) -> u8 {
        let rf = r as f32 / 255.0;
        let gf = g as f32 / 255.0;
        let bf = b as f32 / 255.0;
        
        let luma = self.luma_coefficients.0 * rf + 
                  self.luma_coefficients.1 * gf + 
                  self.luma_coefficients.2 * bf;
        
        (luma * 255.0) as u8
    }
    
    /// Clear histogram data
    fn clear_data(&mut self) {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
        self.rgb_buffer.clear();
        self.luma_buffer.clear();
    }
    
    /// Update processing statistics
    fn update_stats(&mut self, frame_number: u64, timestamp: f64, processing_time: f64, pixels_processed: u64) {
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
        
        // Update frame metadata
        if let Ok(mut data) = self.data.write() {
            data.metadata.update_frame(frame_number, timestamp);
        }
    }
    
    /// Get processing statistics
    pub fn get_stats(&self) -> Result<ScopeStats> {
        let stats = self.stats.read().map_err(|e| anyhow!("Stats lock error: {}", e))?;
        Ok(stats.clone())
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: HistogramConfig) -> Result<()> {
        debug!("Updating histogram configuration");
        
        self.config = config.clone();
        
        // Reinitialize data with new resolution
        let new_data = HistogramData::new(config.resolution);
        if let Ok(mut data) = self.data.write() {
            *data = new_data;
        }
        
        // Update luma coefficients
        self.luma_coefficients = Self::get_luma_coefficients(ColorSpace::Rec709);
        
        info!("Histogram configuration updated");
        
        Ok(())
    }
}

impl Default for HistogramProcessor {
    fn default() -> Self {
        Self::new(HistogramConfig::default()).unwrap()
    }
}

/// Histogram analyzer for advanced color analysis
pub struct HistogramAnalyzer {
    processor: HistogramProcessor,
}

impl HistogramAnalyzer {
    /// Create new histogram analyzer
    pub fn new(config: HistogramConfig) -> Result<Self> {
        Ok(Self {
            processor: HistogramProcessor::new(config)?,
        })
    }
    
    /// Analyze exposure levels
    pub fn analyze_exposure(&self) -> Result<ExposureAnalysis> {
        let data = self.processor.get_data()?;
        let mut analysis = ExposureAnalysis::new();
        
        // Analyze luma histogram
        let luma_data = data.channel_data(HistogramChannel::Luma);
        let total_samples: u64 = luma_data.iter().sum();
        
        if total_samples == 0 {
            return Ok(analysis);
        }
        
        // Calculate shadow/midtone/highlight distribution
        let shadow_samples: u64 = luma_data[0..64].iter().sum(); // 0-25%
        let midtone_samples: u64 = luma_data[64..192].iter().sum(); // 25-75%
        let highlight_samples: u64 = luma_data[192..256].iter().sum(); // 75-100%
        
        analysis.shadow_percentage = (shadow_samples as f32 / total_samples as f32) * 100.0;
        analysis.midtone_percentage = (midtone_samples as f32 / total_samples as f32) * 100.0;
        analysis.highlight_percentage = (highlight_samples as f32 / total_samples as f32) * 100.0;
        
        // Check for clipping
        analysis.black_clipped = luma_data[0] > total_samples / 1000; // More than 0.1% in pure black
        analysis.white_clipped = luma_data[255] > total_samples / 1000; // More than 0.1% in pure white
        
        // Calculate dynamic range
        let mut min_bin = 255;
        let mut max_bin = 0;
        for (bin, &count) in luma_data.iter().enumerate() {
            if count > 0 {
                min_bin = min_bin.min(bin);
                max_bin = max_bin.max(bin);
            }
        }
        analysis.dynamic_range = max_bin - min_bin;
        
        Ok(analysis)
    }
    
    /// Analyze color balance
    pub fn analyze_color_balance(&self) -> Result<ColorBalanceAnalysis> {
        let data = self.processor.get_data()?;
        let mut analysis = ColorBalanceAnalysis::new();
        
        // Get statistics for each channel
        let red_stats = self.processor.calculate_channel_stats(data.channel_data(HistogramChannel::Red));
        let green_stats = self.processor.calculate_channel_stats(data.channel_data(HistogramChannel::Green));
        let blue_stats = self.processor.calculate_channel_stats(data.channel_data(HistogramChannel::Blue));
        
        analysis.red_mean = red_stats.mean;
        analysis.green_mean = green_stats.mean;
        analysis.blue_mean = blue_stats.mean;
        
        // Calculate color cast
        let avg_mean = (analysis.red_mean + analysis.green_mean + analysis.blue_mean) as f32 / 3.0;
        analysis.red_cast = analysis.red_mean as f32 - avg_mean;
        analysis.green_cast = analysis.green_mean as f32 - avg_mean;
        analysis.blue_cast = analysis.blue_mean as f32 - avg_mean;
        
        // Determine dominant color
        let max_cast = analysis.red_cast.abs().max(analysis.green_cast.abs().max(analysis.blue_cast.abs()));
        if max_cast < 5.0 {
            analysis.dominant_cast = ColorCast::Neutral;
        } else if analysis.red_cast.abs() == max_cast {
            analysis.dominant_cast = if analysis.red_cast > 0.0 { ColorCast::Red } else { ColorCast::Cyan };
        } else if analysis.green_cast.abs() == max_cast {
            analysis.dominant_cast = if analysis.green_cast > 0.0 { ColorCast::Green } else { ColorCast::Magenta };
        } else {
            analysis.dominant_cast = if analysis.blue_cast > 0.0 { ColorCast::Blue } else { ColorCast::Yellow };
        }
        
        Ok(analysis)
    }
    
    /// Check for histogram issues
    pub fn check_issues(&self) -> Result<Vec<HistogramIssue>> {
        let data = self.processor.get_data()?;
        let mut issues = Vec::new();
        
        // Check for clipping
        let luma_data = data.channel_data(HistogramChannel::Luma);
        let total_samples: u64 = luma_data.iter().sum();
        
        if total_samples > 0 {
            let black_percentage = (luma_data[0] as f32 / total_samples as f32) * 100.0;
            let white_percentage = (luma_data[255] as f32 / total_samples as f32) * 100.0;
            
            if black_percentage > 0.1 {
                issues.push(HistogramIssue::BlackClipping(black_percentage));
            }
            
            if white_percentage > 0.1 {
                issues.push(HistogramIssue::WhiteClipping(white_percentage));
            }
        }
        
        // Check for limited dynamic range
        let exposure_analysis = self.analyze_exposure()?;
        if exposure_analysis.dynamic_range < 200 {
            issues.push(HistogramIssue::LimitedDynamicRange(exposure_analysis.dynamic_range));
        }
        
        // Check for color cast
        let color_analysis = self.analyze_color_balance()?;
        if color_analysis.dominant_cast != ColorCast::Neutral {
            let cast_strength = color_analysis.red_cast.abs().max(color_analysis.green_cast.abs().max(color_analysis.blue_cast.abs()));
            if cast_strength > 10.0 {
                issues.push(HistogramIssue::ColorCast(color_analysis.dominant_cast, cast_strength));
            }
        }
        
        Ok(issues)
    }
}

/// Histogram statistics for all channels
#[derive(Debug, Clone)]
pub struct HistogramStatistics {
    pub red: ChannelStatistics,
    pub green: ChannelStatistics,
    pub blue: ChannelStatistics,
    pub luma: ChannelStatistics,
}

impl HistogramStatistics {
    pub fn new() -> Self {
        Self {
            red: ChannelStatistics::new(),
            green: ChannelStatistics::new(),
            blue: ChannelStatistics::new(),
            luma: ChannelStatistics::new(),
        }
    }
}

/// Statistics for a single histogram channel
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

/// Exposure analysis results
#[derive(Debug, Clone)]
pub struct ExposureAnalysis {
    pub shadow_percentage: f32,
    pub midtone_percentage: f32,
    pub highlight_percentage: f32,
    pub black_clipped: bool,
    pub white_clipped: bool,
    pub dynamic_range: u8,
}

impl ExposureAnalysis {
    pub fn new() -> Self {
        Self {
            shadow_percentage: 0.0,
            midtone_percentage: 0.0,
            highlight_percentage: 0.0,
            black_clipped: false,
            white_clipped: false,
            dynamic_range: 0,
        }
    }
    
    pub fn is_well_exposed(&self) -> bool {
        !self.black_clipped && 
        !self.white_clipped && 
        self.dynamic_range > 200 &&
        self.shadow_percentage > 5.0 &&
        self.highlight_percentage > 5.0
    }
}

/// Color balance analysis results
#[derive(Debug, Clone)]
pub struct ColorBalanceAnalysis {
    pub red_mean: u8,
    pub green_mean: u8,
    pub blue_mean: u8,
    pub red_cast: f32,
    pub green_cast: f32,
    pub blue_cast: f32,
    pub dominant_cast: ColorCast,
}

impl ColorBalanceAnalysis {
    pub fn new() -> Self {
        Self {
            red_mean: 0,
            green_mean: 0,
            blue_mean: 0,
            red_cast: 0.0,
            green_cast: 0.0,
            blue_cast: 0.0,
            dominant_cast: ColorCast::Neutral,
        }
    }
    
    pub fn is_balanced(&self, tolerance: f32) -> bool {
        self.red_cast.abs() < tolerance && 
        self.green_cast.abs() < tolerance && 
        self.blue_cast.abs() < tolerance
    }
}

/// Color cast types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCast {
    Neutral,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
}

/// Histogram issue types
#[derive(Debug, Clone)]
pub enum HistogramIssue {
    BlackClipping(f32),    // Percentage of pixels clipped
    WhiteClipping(f32),    // Percentage of pixels clipped
    LimitedDynamicRange(u8), // Dynamic range in bins
    ColorCast(ColorCast, f32), // Color cast and strength
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_histogram_processor_creation() {
        let config = HistogramConfig::default();
        let processor = HistogramProcessor::new(config);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_channel_statistics() {
        let mut stats = ChannelStatistics::new();
        stats.total_samples = 1000;
        stats.mean = 128;
        stats.min_bin = 10;
        stats.max_bin = 245;
        
        assert_eq!(stats.dynamic_range(), 235);
    }
    
    #[test]
    fn test_exposure_analysis() {
        let mut analysis = ExposureAnalysis::new();
        analysis.shadow_percentage = 20.0;
        analysis.midtone_percentage = 60.0;
        analysis.highlight_percentage = 20.0;
        analysis.dynamic_range = 230;
        
        assert!(analysis.is_well_exposed());
        
        analysis.black_clipped = true;
        assert!(!analysis.is_well_exposed());
    }
    
    #[test]
    fn test_color_balance() {
        let mut analysis = ColorBalanceAnalysis::new();
        analysis.red_mean = 130;
        analysis.green_mean = 128;
        analysis.blue_mean = 126;
        
        analysis.red_cast = 2.0;
        analysis.green_cast = 0.0;
        analysis.blue_cast = -2.0;
        
        assert!(analysis.is_balanced(5.0));
        assert!(!analysis.is_balanced(1.0));
    }
    
    #[test]
    fn test_luma_calculation() {
        let processor = HistogramProcessor::new(HistogramConfig::default()).unwrap();
        
        // Test pure white
        let luma = processor.calculate_luma(255, 255, 255);
        assert_eq!(luma, 255);
        
        // Test pure black
        let luma = processor.calculate_luma(0, 0, 0);
        assert_eq!(luma, 0);
        
        // Test middle gray
        let luma = processor.calculate_luma(128, 128, 128);
        assert!((luma as i32 - 128).abs() < 5); // Allow small rounding error
    }
}
