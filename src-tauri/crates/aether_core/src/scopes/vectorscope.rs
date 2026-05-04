//! Vectorscope implementation for professional color analysis
//! 
//! This module provides a complete vectorscope system with UV coordinate plotting,
//! color target overlays, and real-time analysis capabilities similar to DaVinci Resolve.

use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};
use image::{Rgb, RgbImage};

use crate::types::{
    VectorscopeData, VectorscopePoint, VectorscopeConfig, VectorscopeTarget,
    ScopeResolution, ColorSpace, VideoRange, ScopeStats,
};

/// Vectorscope processor for real-time color analysis
pub struct VectorscopeProcessor {
    config: VectorscopeConfig,
    data: Arc<RwLock<VectorscopeData>>,
    stats: Arc<RwLock<ScopeStats>>,
    
    // Processing buffers
    uv_buffer: Vec<(f32, f32)>,
    intensity_cache: Vec<u16>,
    
    // Color conversion matrices
    rgb_to_yuv_matrix: [[f32; 3]; 3],
    yuv_to_uv_matrix: [[f32; 2]; 3],
}

impl VectorscopeProcessor {
    /// Create a new vectorscope processor
    pub fn new(config: VectorscopeConfig) -> Result<Self> {
        info!("Creating vectorscope processor with config: {:?}", config);
        
        let processor = Self {
            config: config.clone(),
            data: Arc::new(RwLock::new(VectorscopeData::new(config.resolution))),
            stats: Arc::new(RwLock::new(ScopeStats::new())),
            uv_buffer: Vec::new(),
            intensity_cache: Vec::new(),
            rgb_to_yuv_matrix: Self::get_rgb_to_yuv_matrix(ColorSpace::Rec709),
            yuv_to_uv_matrix: Self::get_yuv_to_uv_matrix(ColorSpace::Rec709),
        };
        
        // Initialize buffers
        let grid_size = config.resolution.vectorscope_grid_size();
        processor.uv_buffer.reserve(grid_size * grid_size);
        processor.intensity_cache.resize(grid_size * grid_size, 0);
        
        info!("Vectorscope processor created successfully");
        
        Ok(processor)
    }
    
    /// Process an image frame and update vectorscope data
    pub fn process_frame(&mut self, image: &RgbImage, frame_number: u64, timestamp: f64) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        debug!("Processing vectorscope frame {} at timestamp {:.3}", frame_number, timestamp);
        
        // Clear previous data
        self.clear_data();
        
        // Process each pixel
        let (width, height) = image.dimensions();
        let mut pixels_processed = 0u64;
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                
                // Convert RGB to UV coordinates
                let (u, v) = self.rgb_to_uv(r, g, b);
                
                // Add point to vectorscope
                self.add_uv_point(u, v, 1);
                pixels_processed += 1;
            }
        }
        
        // Update statistics
        let processing_time = start_time.elapsed().as_micros() as f64;
        self.update_stats(frame_number, timestamp, processing_time, pixels_processed);
        
        debug!("Vectorscope frame {} processed in {:.2}μs, {} pixels", 
               frame_number, processing_time, pixels_processed);
        
        Ok(())
    }
    
    /// Add a UV point to the vectorscope
    pub fn add_uv_point(&mut self, u: f32, v: f32, intensity: u16) {
        let mut data = self.data.write().map_err(|e| anyhow!("Data lock error: {}", e))?;
        data.add_point(u, v, intensity);
    }
    
    /// Get current vectorscope data
    pub fn get_data(&self) -> Result<VectorscopeData> {
        let data = self.data.read().map_err(|e| anyhow!("Data lock error: {}", e))?;
        Ok(data.clone())
    }
    
    /// Get intensity at specific UV coordinates
    pub fn get_intensity(&self, u: f32, v: f32) -> Result<u16> {
        let data = self.data.read().map_err(|e| anyhow!("Data lock error: {}", e))?;
        Ok(data.get_intensity(u, v))
    }
    
    /// Generate vectorscope image
    pub fn generate_image(&self, width: u32, height: u32) -> Result<RgbImage> {
        let data = self.data.read().map_err(|e| anyhow!("Data lock error: {}", e))?;
        
        let mut image = RgbImage::new(width, height);
        
        // Clear background
        for pixel in image.pixels_mut() {
            *pixel = Rgb([16, 16, 16]); // Dark background
        }
        
        // Draw intensity grid
        let grid_size = data.config.resolution.vectorscope_grid_size();
        for y in 0..grid_size {
            for x in 0..grid_size {
                let index = y * grid_size + x;
                let intensity = data.intensity_grid[index];
                
                if intensity > 0 {
                    // Map grid coordinates to image coordinates
                    let img_x = (x * width as usize / grid_size) as u32;
                    let img_y = (y * height as usize / grid_size) as u32;
                    
                    // Convert intensity to color
                    let color = self.intensity_to_color(intensity);
                    
                    if let Some(pixel) = image.get_pixel_mut(img_x, img_y) {
                        *pixel = color;
                    }
                }
            }
        }
        
        // Draw target overlays
        self.draw_target_overlays(&mut image)?;
        
        Ok(image)
    }
    
    /// Draw color target overlays on the vectorscope
    fn draw_target_overlays(&self, image: &mut RgbImage) -> Result<()> {
        let (width, height) = image.dimensions();
        let center_x = width / 2;
        let center_y = height / 2;
        let radius = width.min(height) / 2 - 10;
        
        for target in &self.config.targets {
            let color = self.get_target_color(*target);
            let (u, v) = self.get_target_uv(*target);
            
            // Convert UV to image coordinates
            let img_x = center_x + (u * radius as f32) as i32;
            let img_y = center_y - (v * radius as f32) as i32; // Invert Y for image coordinates
            
            // Draw target marker
            self.draw_target_marker(image, img_x, img_y, color)?;
        }
        
        // Draw center crosshair
        self.draw_crosshair(image, center_x, center_y, Rgb([128, 128, 128]))?;
        
        Ok(())
    }
    
    /// Draw a target marker at the specified position
    fn draw_target_marker(&self, image: &mut RgbImage, x: i32, y: i32, color: Rgb<u8>) -> Result<()> {
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
    
    /// Draw crosshair at center
    fn draw_crosshair(&self, image: &mut RgbImage, cx: u32, cy: u32, color: Rgb<u8>) -> Result<()> {
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
        
        Ok(())
    }
    
    /// Convert RGB to UV coordinates
    fn rgb_to_uv(&self, r: u8, g: u8, b: u8) -> (f32, f32) {
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
        let uv_u = self.yuv_to_uv_matrix[0][0] * u + self.yuv_to_uv_matrix[0][1] * v;
        let uv_v = self.yuv_to_uv_matrix[1][0] * u + self.yuv_to_uv_matrix[1][1] * v;
        
        (uv_u, uv_v)
    }
    
    /// Convert intensity to display color
    fn intensity_to_color(&self, intensity: u16) -> Rgb<u8> {
        let normalized = (intensity as f32 / 1000.0).min(1.0);
        
        // Use a color gradient from blue to green to yellow to red
        let r = (normalized * 255.0) as u8;
        let g = ((1.0 - (normalized - 0.5).abs() * 2.0) * 255.0) as u8;
        let b = ((1.0 - normalized) * 255.0) as u8;
        
        Rgb([r, g, b])
    }
    
    /// Get target color for display
    fn get_target_color(&self, target: VectorscopeTarget) -> Rgb<u8> {
        match target {
            VectorscopeTarget::Primary => Rgb([255, 255, 255]),
            VectorscopeTarget::SkinTones => Rgb([255, 200, 150]),
            VectorscopeTarget::Blue => Rgb([0, 100, 255]),
            VectorscopeTarget::Yellow => Rgb([255, 255, 0]),
            VectorscopeTarget::Cyan => Rgb([0, 255, 255]),
            VectorscopeTarget::Green => Rgb([0, 255, 0]),
            VectorscopeTarget::Magenta => Rgb([255, 0, 255]),
            VectorscopeTarget::Red => Rgb([255, 0, 0]),
        }
    }
    
    /// Get target UV coordinates
    fn get_target_uv(&self, target: VectorscopeTarget) -> (f32, f32) {
        match target {
            VectorscopeTarget::Primary => (0.0, 0.0), // Center (white)
            VectorscopeTarget::SkinTones => (0.1, 0.2), // Skin tone line
            VectorscopeTarget::Blue => (-0.2, -0.4),
            VectorscopeTarget::Yellow => (0.3, 0.4),
            VectorscopeTarget::Cyan => (-0.3, 0.2),
            VectorscopeTarget::Green => (-0.4, -0.2),
            VectorscopeTarget::Magenta => (0.4, -0.2),
            VectorscopeTarget::Red => (0.3, -0.4),
        }
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
    
    /// Get YUV to UV conversion matrix
    fn get_yuv_to_uv_matrix(color_space: ColorSpace) -> [[f32; 2]; 3] {
        match color_space {
            ColorSpace::Rec709 => [
                [1.0, 0.0], // U to UV-U
                [0.0, 1.0], // V to UV-V
            ],
            _ => Self::get_yuv_to_uv_matrix(ColorSpace::Rec709),
        }
    }
    
    /// Clear vectorscope data
    fn clear_data(&mut self) {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
        self.uv_buffer.clear();
        self.intensity_cache.fill(0);
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
    pub fn update_config(&mut self, config: VectorscopeConfig) -> Result<()> {
        debug!("Updating vectorscope configuration");
        
        self.config = config.clone();
        
        // Reinitialize data with new resolution
        let new_data = VectorscopeData::new(config.resolution);
        if let Ok(mut data) = self.data.write() {
            *data = new_data;
        }
        
        // Update conversion matrices
        self.rgb_to_yuv_matrix = Self::get_rgb_to_yuv_matrix(ColorSpace::Rec709);
        self.yuv_to_uv_matrix = Self::get_yuv_to_uv_matrix(ColorSpace::Rec709);
        
        // Resize buffers
        let grid_size = config.resolution.vectorscope_grid_size();
        self.intensity_cache.resize(grid_size * grid_size, 0);
        
        info!("Vectorscope configuration updated");
        
        Ok(())
    }
}

impl Default for VectorscopeProcessor {
    fn default() -> Self {
        Self::new(VectorscopeConfig::default()).unwrap()
    }
}

/// Vectorscope analyzer for advanced color analysis
pub struct VectorscopeAnalyzer {
    processor: VectorscopeProcessor,
}

impl VectorscopeAnalyzer {
    /// Create new vectorscope analyzer
    pub fn new(config: VectorscopeConfig) -> Result<Self> {
        Ok(Self {
            processor: VectorscopeProcessor::new(config)?,
        })
    }
    
    /// Analyze color distribution
    pub fn analyze_color_distribution(&self) -> Result<ColorDistribution> {
        let data = self.processor.get_data()?;
        
        let mut distribution = ColorDistribution::new();
        
        // Analyze intensity distribution
        let max_intensity = data.intensity_grid.iter().copied().max().unwrap_or(0);
        let total_intensity: u64 = data.intensity_grid.iter().map(|&v| v as u64).sum();
        
        distribution.max_intensity = max_intensity;
        distribution.total_intensity = total_intensity;
        distribution.average_intensity = if total_intensity > 0 {
            total_intensity as f32 / data.intensity_grid.len() as f32
        } else {
            0.0
        };
        
        // Analyze color balance
        let mut red_sum = 0.0;
        let mut green_sum = 0.0;
        let mut blue_sum = 0.0;
        let mut sample_count = 0;
        
        for point in &data.points {
            // Convert UV back to RGB for analysis (simplified)
            let (r, g, b) = self.uv_to_rgb(point.u, point.v);
            red_sum += r;
            green_sum += g;
            blue_sum += b;
            sample_count += 1;
        }
        
        if sample_count > 0 {
            distribution.color_balance.red = red_sum / sample_count as f32;
            distribution.color_balance.green = green_sum / sample_count as f32;
            distribution.color_balance.blue = blue_sum / sample_count as f32;
        }
        
        Ok(distribution)
    }
    
    /// Check if colors are within target ranges
    pub fn check_target_compliance(&self) -> Result<TargetCompliance> {
        let data = self.processor.get_data()?;
        let mut compliance = TargetCompliance::new();
        
        for target in &data.config.targets {
            let (target_u, target_v) = self.processor.get_target_uv(*target);
            let intensity = data.get_intensity(target_u, target_v);
            
            let target_result = TargetResult {
                target: *target,
                intensity,
                within_range: intensity > 10, // Threshold for "detected"
                deviation: self.calculate_uv_deviation(target_u, target_v, &data),
            };
            
            compliance.targets.push(target_result);
        }
        
        Ok(compliance)
    }
    
    /// Convert UV back to RGB (simplified inverse)
    fn uv_to_rgb(&self, u: f32, v: f32) -> (f32, f32, f32) {
        // Simplified conversion - in practice this would use proper inverse matrices
        let r = (v + 0.5).clamp(0.0, 1.0);
        let g = (0.5 - u.abs() * 0.5 - v.abs() * 0.5).clamp(0.0, 1.0);
        let b = (u + 0.5).clamp(0.0, 1.0);
        
        (r, g, b)
    }
    
    /// Calculate UV deviation from target
    fn calculate_uv_deviation(&self, target_u: f32, target_v: f32, data: &VectorscopeData) -> f32 {
        let mut total_deviation = 0.0;
        let mut sample_count = 0;
        
        for point in &data.points {
            let du = point.u - target_u;
            let dv = point.v - target_v;
            let distance = (du * du + dv * dv).sqrt();
            total_deviation += distance;
            sample_count += 1;
        }
        
        if sample_count > 0 {
            total_deviation / sample_count as f32
        } else {
            0.0
        }
    }
}

/// Color distribution analysis results
#[derive(Debug, Clone)]
pub struct ColorDistribution {
    pub max_intensity: u16,
    pub total_intensity: u64,
    pub average_intensity: f32,
    pub color_balance: ColorBalance,
}

impl ColorDistribution {
    pub fn new() -> Self {
        Self {
            max_intensity: 0,
            total_intensity: 0,
            average_intensity: 0.0,
            color_balance: ColorBalance::new(),
        }
    }
}

/// Color balance information
#[derive(Debug, Clone)]
pub struct ColorBalance {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl ColorBalance {
    pub fn new() -> Self {
        Self {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        }
    }
    
    pub fn is_balanced(&self, tolerance: f32) -> bool {
        let avg = (self.red + self.green + self.blue) / 3.0;
        (self.red - avg).abs() < tolerance &&
        (self.green - avg).abs() < tolerance &&
        (self.blue - avg).abs() < tolerance
    }
}

/// Target compliance analysis results
#[derive(Debug, Clone)]
pub struct TargetCompliance {
    pub targets: Vec<TargetResult>,
}

impl TargetCompliance {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
        }
    }
    
    pub fn compliance_rate(&self) -> f32 {
        if self.targets.is_empty() {
            0.0
        } else {
            let compliant = self.targets.iter().filter(|t| t.within_range).count();
            compliant as f32 / self.targets.len() as f32
        }
    }
}

/// Individual target analysis result
#[derive(Debug, Clone)]
pub struct TargetResult {
    pub target: VectorscopeTarget,
    pub intensity: u16,
    pub within_range: bool,
    pub deviation: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vectorscope_processor_creation() {
        let config = VectorscopeConfig::default();
        let processor = VectorscopeProcessor::new(config);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_uv_conversion() {
        let processor = VectorscopeProcessor::new(VectorscopeConfig::default()).unwrap();
        
        // Test pure red
        let (u, v) = processor.rgb_to_uv(255, 0, 0);
        assert!(u > 0.0); // Red should be on the right side
        assert!(v < 0.0); // Red should be on the bottom
        
        // Test pure blue
        let (u, v) = processor.rgb_to_uv(0, 0, 255);
        assert!(u < 0.0); // Blue should be on the left side
        assert!(v < 0.0); // Blue should be on the bottom
        
        // Test pure green
        let (u, v) = processor.rgb_to_uv(0, 255, 0);
        assert!(u < 0.0); // Green should be on the left side
        assert!(v > 0.0); // Green should be on the top
    }
    
    #[test]
    fn test_target_colors() {
        let processor = VectorscopeProcessor::new(VectorscopeConfig::default()).unwrap();
        
        let red_color = processor.get_target_color(VectorscopeTarget::Red);
        assert_eq!(red_color, Rgb([255, 0, 0]));
        
        let blue_color = processor.get_target_color(VectorscopeTarget::Blue);
        assert_eq!(blue_color, Rgb([0, 100, 255]));
        
        let skin_color = processor.get_target_color(VectorscopeTarget::SkinTones);
        assert_eq!(skin_color, Rgb([255, 200, 150]));
    }
    
    #[test]
    fn test_color_balance() {
        let mut balance = ColorBalance::new();
        balance.red = 0.8;
        balance.green = 0.8;
        balance.blue = 0.8;
        
        assert!(balance.is_balanced(0.1));
        assert!(!balance.is_balanced(0.05));
    }
    
    #[test]
    fn test_target_compliance() {
        let mut compliance = TargetCompliance::new();
        compliance.targets.push(TargetResult {
            target: VectorscopeTarget::Red,
            intensity: 100,
            within_range: true,
            deviation: 0.1,
        });
        compliance.targets.push(TargetResult {
            target: VectorscopeTarget::Blue,
            intensity: 5,
            within_range: false,
            deviation: 0.5,
        });
        
        assert_eq!(compliance.compliance_rate(), 0.5);
    }
}
