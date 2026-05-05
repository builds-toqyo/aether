//! Professional LUT (Look-Up Table) System
//! 
//! This module provides comprehensive LUT support including 3D LUT import/export,
//! Cube file format support, and real-time LUT application for professional
//! color grading workflows.

use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Result, anyhow};
use log::{debug, info, warn, error};
use image::{Rgb, RgbImage};

use crate::types::ColorSpace;

/// Professional LUT processor
pub struct LutProcessor {
    luts: std::collections::HashMap<String, LutData>,
    active_lut: Option<String>,
    config: LutConfig,
}

impl LutProcessor {
    /// Create new LUT processor
    pub fn new(config: LutConfig) -> Result<Self> {
        info!("Creating LUT processor with config: {:?}", config);
        
        let processor = Self {
            luts: std::collections::HashMap::new(),
            active_lut: None,
            config: config.clone(),
        };
        
        info!("LUT processor created successfully");
        
        Ok(processor)
    }
    
    /// Load LUT from file
    pub fn load_lut(&mut self, file_path: &Path) -> Result<String> {
        debug!("Loading LUT from: {:?}", file_path);
        
        let file_extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("Invalid file extension"))?;
        
        let lut_data = match file_extension {
            "cube" => self.load_cube_file(file_path)?,
            "3dl" => self.load_3dl_file(file_path)?,
            "look" => self.load_look_file(file_path)?,
            _ => return Err(anyhow!("Unsupported LUT format: {}", file_extension)),
        };
        
        let lut_name = file_path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("unnamed_lut")
            .to_string();
        
        self.luts.insert(lut_name.clone(), lut_data);
        
        info!("LUT loaded successfully: {}", lut_name);
        
        Ok(lut_name)
    }
    
    /// Load Cube format LUT
    fn load_cube_file(&self, file_path: &Path) -> Result<LutData> {
        debug!("Loading Cube LUT file: {:?}", file_path);
        
        let mut file = std::fs::File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let mut lut_data = LutData::default();
        let mut size = 33; // Default size
        let mut title = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if line.starts_with("TITLE") {
                title = line.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
            } else if line.starts_with("LUT_3D_SIZE") {
                size = line.split_whitespace()
                    .nth(1)
                    .ok_or_else(|| anyhow!("Invalid LUT_3D_SIZE line"))?
                    .parse()
                    .map_err(|e| anyhow!("Invalid LUT size: {}", e))?;
            } else if line.contains(' ') {
                // RGB data line
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let r = parts[0].parse::<f32>()?;
                    let g = parts[1].parse::<f32>()?;
                    let b = parts[2].parse::<f32>()?;
                    
                    lut_data.data.push([r, g, b]);
                }
            }
        }
        
        lut_data.name = title;
        lut_data.size = size;
        lut_data.format = LutFormat::Cube;
        
        // Validate data size
        let expected_size = size * size * size;
        if lut_data.data.len() != expected_size {
            warn!("LUT data size mismatch: expected {}, got {}", expected_size, lut_data.data.len());
        }
        
        Ok(lut_data)
    }
    
    /// Load 3DL format LUT
    fn load_3dl_file(&self, file_path: &Path) -> Result<LutData> {
        debug!("Loading 3DL LUT file: {:?}", file_path);
        
        let mut file = std::fs::File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let mut lut_data = LutData::default();
        let mut size = 33;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if line.contains(' ') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    // 3DL format: r_in g_in b_in r_out g_out b_out
                    let r_out = parts[3].parse::<f32>()?;
                    let g_out = parts[4].parse::<f32>()?;
                    let b_out = parts[5].parse::<f32>()?;
                    
                    lut_data.data.push([r_out, g_out, b_out]);
                }
            }
        }
        
        // Calculate size from data length
        let data_len = lut_data.data.len();
        size = (data_len as f32).cbrt() as u32;
        
        lut_data.name = file_path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("3dl_lut")
            .to_string();
        lut_data.size = size;
        lut_data.format = LutFormat::ThreeDL;
        
        Ok(lut_data)
    }
    
    /// Load LOOK format LUT
    fn load_look_file(&self, file_path: &Path) -> Result<LutData> {
        debug!("Loading LOOK LUT file: {:?}", file_path);
        
        // LOOK format parsing (simplified)
        let mut lut_data = LutData::default();
        lut_data.name = file_path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("look_lut")
            .to_string();
        lut_data.size = 33;
        lut_data.format = LutFormat::Look;
        
        // Generate identity LUT as placeholder
        lut_data.generate_identity_lut(33);
        
        Ok(lut_data)
    }
    
    /// Save LUT to file
    pub fn save_lut(&self, lut_name: &str, file_path: &Path, format: LutFormat) -> Result<()> {
        debug!("Saving LUT '{}' to: {:?}", lut_name, file_path);
        
        let lut_data = self.luts.get(lut_name)
            .ok_or_else(|| anyhow!("LUT not found: {}", lut_name))?;
        
        match format {
            LutFormat::Cube => self.save_cube_file(lut_data, file_path)?,
            LutFormat::ThreeDL => self.save_3dl_file(lut_data, file_path)?,
            LutFormat::Look => self.save_look_file(lut_data, file_path)?,
        }
        
        info!("LUT saved successfully: {} -> {:?}", lut_name, file_path);
        
        Ok(())
    }
    
    /// Save as Cube format
    fn save_cube_file(&self, lut_data: &LutData, file_path: &Path) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        
        writeln!(file, "TITLE {}", lut_data.name)?;
        writeln!(file, "LUT_3D_SIZE {}", lut_data.size)?;
        writeln!(file)?;
        
        for rgb in &lut_data.data {
            writeln!(file, "{} {} {}", rgb[0], rgb[1], rgb[2])?;
        }
        
        Ok(())
    }
    
    /// Save as 3DL format
    fn save_3dl_file(&self, lut_data: &LutData, file_path: &Path) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        
        let mut index = 0;
        for b in 0..lut_data.size {
            for g in 0..lut_data.size {
                for r in 0..lut_data.size {
                    if index < lut_data.data.len() {
                        let rgb = &lut_data.data[index];
                        let r_in = r as f32 / (lut_data.size - 1) as f32;
                        let g_in = g as f32 / (lut_data.size - 1) as f32;
                        let b_in = b as f32 / (lut_data.size - 1) as f32;
                        
                        writeln!(file, "{} {} {} {} {} {}", 
                            r_in, g_in, b_in, rgb[0], rgb[1], rgb[2])?;
                        index += 1;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Save as LOOK format
    fn save_look_file(&self, lut_data: &LutData, file_path: &Path) -> Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        
        writeln!(file, "<Look>")?;
        writeln!(file, "  <Name>{}</Name>", lut_data.name)?;
        writeln!(file, "  <Size>{}</Size>", lut_data.size)?;
        writeln!(file, "  <Data>")?;
        
        for rgb in &lut_data.data {
            writeln!(file, "    {} {} {}", rgb[0], rgb[1], rgb[2])?;
        }
        
        writeln!(file, "  </Data>")?;
        writeln!(file, "</Look>")?;
        
        Ok(())
    }
    
    /// Apply LUT to image
    pub fn apply_lut(&self, image: &mut RgbImage, lut_name: &str) -> Result<()> {
        debug!("Applying LUT '{}' to image", lut_name);
        
        let lut_data = self.luts.get(lut_name)
            .ok_or_else(|| anyhow!("LUT not found: {}", lut_name))?;
        
        let (width, height) = image.dimensions();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let [r, g, b] = pixel.0;
                
                // Convert to 0-1 range
                let rf = r as f32 / 255.0;
                let gf = g as f32 / 255.0;
                let bf = b as f32 / 255.0;
                
                // Apply LUT
                let transformed = self.interpolate_lut(lut_data, [rf, gf, bf]);
                
                // Convert back to 0-255 range
                let new_r = (transformed[0] * 255.0).clamp(0.0, 255.0) as u8;
                let new_g = (transformed[1] * 255.0).clamp(0.0, 255.0) as u8;
                let new_b = (transformed[2] * 255.0).clamp(0.0, 255.0) as u8;
                
                image.put_pixel(x, y, Rgb([new_r, new_g, new_b]));
            }
        }
        
        debug!("LUT applied successfully");
        
        Ok(())
    }
    
    /// Interpolate LUT values
    fn interpolate_lut(&self, lut_data: &LutData, input: [f32; 3]) -> [f32; 3] {
        let size = lut_data.size as f32;
        let size_minus_1 = lut_data.size - 1;
        
        // Clamp input to valid range
        let r = input[0].clamp(0.0, 1.0);
        let g = input[1].clamp(0.0, 1.0);
        let b = input[2].clamp(0.0, 1.0);
        
        // Calculate indices
        let r_index = (r * size_minus_1 as f32) as u32;
        let g_index = (g * size_minus_1 as f32) as u32;
        let b_index = (b * size_minus_1 as f32) as u32;
        
        // Calculate fractional parts
        let r_frac = (r * size_minus_1 as f32) - r_index as f32;
        let g_frac = (g * size_minus_1 as f32) - g_index as f32;
        let b_frac = (b * size_minus_1 as f32) - b_index as f32;
        
        // Get corner values
        let idx000 = self.lut_index(r_index, g_index, b_index, lut_data.size);
        let idx100 = self.lut_index(r_index + 1, g_index, b_index, lut_data.size);
        let idx010 = self.lut_index(r_index, g_index + 1, b_index, lut_data.size);
        let idx110 = self.lut_index(r_index + 1, g_index + 1, b_index, lut_data.size);
        let idx001 = self.lut_index(r_index, g_index, b_index + 1, lut_data.size);
        let idx101 = self.lut_index(r_index + 1, g_index, b_index + 1, lut_data.size);
        let idx011 = self.lut_index(r_index, g_index + 1, b_index + 1, lut_data.size);
        let idx111 = self.lut_index(r_index + 1, g_index + 1, b_index + 1, lut_data.size);
        
        let c000 = lut_data.data.get(idx000).unwrap_or(&[0.0, 0.0, 0.0]);
        let c100 = lut_data.data.get(idx100).unwrap_or(&[0.0, 0.0, 0.0]);
        let c010 = lut_data.data.get(idx010).unwrap_or(&[0.0, 0.0, 0.0]);
        let c110 = lut_data.data.get(idx110).unwrap_or(&[0.0, 0.0, 0.0]);
        let c001 = lut_data.data.get(idx001).unwrap_or(&[0.0, 0.0, 0.0]);
        let c101 = lut_data.data.get(idx101).unwrap_or(&[0.0, 0.0, 0.0]);
        let c011 = lut_data.data.get(idx011).unwrap_or(&[0.0, 0.0, 0.0]);
        let c111 = lut_data.data.get(idx111).unwrap_or(&[0.0, 0.0, 0.0]);
        
        // Trilinear interpolation
        let c00 = self.lerp(c000, c100, r_frac);
        let c01 = self.lerp(c001, c101, r_frac);
        let c10 = self.lerp(c010, c110, r_frac);
        let c11 = self.lerp(c011, c111, r_frac);
        
        let c0 = self.lerp(&c00, &c10, g_frac);
        let c1 = self.lerp(&c01, &c11, g_frac);
        
        self.lerp(&c0, &c1, b_frac)
    }
    
    /// Calculate LUT index
    fn lut_index(&self, r: u32, g: u32, b: u32, size: u32) -> usize {
        ((b * size + g) * size + r) as usize
    }
    
    /// Linear interpolation
    fn lerp(&self, a: &[f32; 3], b: &[f32; 3], t: f32) -> [f32; 3] {
        [
            a[0] + t * (b[0] - a[0]),
            a[1] + t * (b[1] - a[1]),
            a[2] + t * (b[2] - a[2]),
        ]
    }
    
    /// Set active LUT
    pub fn set_active_lut(&mut self, lut_name: Option<&str>) -> Result<()> {
        if let Some(name) = lut_name {
            if !self.luts.contains_key(name) {
                return Err(anyhow!("LUT not found: {}", name));
            }
        }
        
        self.active_lut = lut_name.map(String::from);
        
        info!("Active LUT set to: {:?}", self.active_lut);
        
        Ok(())
    }
    
    /// Apply active LUT to image
    pub fn apply_active_lut(&self, image: &mut RgbImage) -> Result<()> {
        if let Some(ref lut_name) = self.active_lut {
            self.apply_lut(image, lut_name)
        } else {
            Err(anyhow!("No active LUT set"))
        }
    }
    
    /// Create identity LUT
    pub fn create_identity_lut(&mut self, name: &str, size: u32) -> Result<String> {
        debug!("Creating identity LUT: {} (size: {})", name, size);
        
        let mut lut_data = LutData::default();
        lut_data.name = name.to_string();
        lut_data.size = size;
        lut_data.format = LutFormat::Cube;
        lut_data.generate_identity_lut(size);
        
        let lut_name = name.to_string();
        self.luts.insert(lut_name.clone(), lut_data);
        
        info!("Identity LUT created: {}", lut_name);
        
        Ok(lut_name)
    }
    
    /// Create color correction LUT
    pub fn create_color_correction_lut(&mut self, name: &str, size: u32, correction: &ColorCorrection) -> Result<String> {
        debug!("Creating color correction LUT: {}", name);
        
        let mut lut_data = LutData::default();
        lut_data.name = name.to_string();
        lut_data.size = size;
        lut_data.format = LutFormat::Cube;
        
        for b in 0..size {
            for g in 0..size {
                for r in 0..size {
                    let rf = r as f32 / (size - 1) as f32;
                    let gf = g as f32 / (size - 1) as f32;
                    let bf = b as f32 / (size - 1) as f32;
                    
                    // Apply color correction
                    let corrected = self.apply_color_correction([rf, gf, bf], correction);
                    lut_data.data.push(corrected);
                }
            }
        }
        
        let lut_name = name.to_string();
        self.luts.insert(lut_name.clone(), lut_data);
        
        info!("Color correction LUT created: {}", lut_name);
        
        Ok(lut_name)
    }
    
    /// Apply color correction to RGB values
    fn apply_color_correction(&self, rgb: [f32; 3], correction: &ColorCorrection) -> [f32; 3] {
        let [r, g, b] = rgb;
        
        // Apply gamma correction
        let r_gamma = r.powf(correction.gamma);
        let g_gamma = g.powf(correction.gamma);
        let b_gamma = b.powf(correction.gamma);
        
        // Apply contrast
        let r_contrast = ((r_gamma - 0.5) * correction.contrast + 0.5).clamp(0.0, 1.0);
        let g_contrast = ((g_gamma - 0.5) * correction.contrast + 0.5).clamp(0.0, 1.0);
        let b_contrast = ((b_gamma - 0.5) * correction.contrast + 0.5).clamp(0.0, 1.0);
        
        // Apply saturation
        let luma = 0.2126 * r_contrast + 0.7152 * g_contrast + 0.0722 * b_contrast;
        let r_saturated = luma + (r_contrast - luma) * correction.saturation;
        let g_saturated = luma + (g_contrast - luma) * correction.saturation;
        let b_saturated = luma + (b_contrast - luma) * correction.saturation;
        
        // Apply color balance
        let r_balanced = r_saturated * correction.color_balance[0];
        let g_balanced = g_saturated * correction.color_balance[1];
        let b_balanced = b_saturated * correction.color_balance[2];
        
        [r_balanced, g_balanced, b_balanced]
    }
    
    /// Get loaded LUTs
    pub fn get_loaded_luts(&self) -> Vec<&str> {
        self.luts.keys().map(|s| s.as_str()).collect()
    }
    
    /// Get LUT information
    pub fn get_lut_info(&self, lut_name: &str) -> Option<LutInfo> {
        self.luts.get(lut_name).map(|lut_data| LutInfo {
            name: lut_data.name.clone(),
            size: lut_data.size,
            format: lut_data.format,
            data_points: lut_data.data.len(),
        })
    }
    
    /// Remove LUT
    pub fn remove_lut(&mut self, lut_name: &str) -> Result<()> {
        if self.luts.remove(lut_name).is_none() {
            return Err(anyhow!("LUT not found: {}", lut_name));
        }
        
        // Clear active LUT if it was removed
        if let Some(ref active) = self.active_lut {
            if active == lut_name {
                self.active_lut = None;
            }
        }
        
        info!("LUT removed: {}", lut_name);
        
        Ok(())
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: LutConfig) {
        self.config = config;
    }
}

impl Default for LutProcessor {
    fn default() -> Self {
        Self::new(LutConfig::default()).unwrap()
    }
}

/// LUT data structure
#[derive(Debug, Clone)]
pub struct LutData {
    pub name: String,
    pub size: u32,
    pub format: LutFormat,
    pub data: Vec<[f32; 3]>,
}

impl LutData {
    /// Generate identity LUT
    pub fn generate_identity_lut(&mut self, size: u32) {
        self.data.clear();
        
        for b in 0..size {
            for g in 0..size {
                for r in 0..size {
                    let rf = r as f32 / (size - 1) as f32;
                    let gf = g as f32 / (size - 1) as f32;
                    let bf = b as f32 / (size - 1) as f32;
                    
                    self.data.push([rf, gf, bf]);
                }
            }
        }
    }
}

impl Default for LutData {
    fn default() -> Self {
        Self {
            name: String::new(),
            size: 33,
            format: LutFormat::Cube,
            data: Vec::new(),
        }
    }
}

/// LUT format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LutFormat {
    Cube,
    ThreeDL,
    Look,
}

/// LUT configuration
#[derive(Debug, Clone)]
pub struct LutConfig {
    pub interpolation_quality: InterpolationQuality,
    pub color_space: ColorSpace,
    pub clamp_output: bool,
}

impl Default for LutConfig {
    fn default() -> Self {
        Self {
            interpolation_quality: InterpolationQuality::Trilinear,
            color_space: ColorSpace::Rec709,
            clamp_output: true,
        }
    }
}

/// Interpolation quality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationQuality {
    Nearest,
    Linear,
    Trilinear,
}

/// Color correction parameters
#[derive(Debug, Clone)]
pub struct ColorCorrection {
    pub gamma: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub color_balance: [f32; 3],
}

impl Default for ColorCorrection {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            color_balance: [1.0, 1.0, 1.0],
        }
    }
}

/// LUT information
#[derive(Debug, Clone)]
pub struct LutInfo {
    pub name: String,
    pub size: u32,
    pub format: LutFormat,
    pub data_points: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_lut_processor_creation() {
        let config = LutConfig::default();
        let processor = LutProcessor::new(config);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_identity_lut_creation() {
        let mut processor = LutProcessor::default();
        let lut_name = processor.create_identity_lut("test_identity", 33).unwrap();
        
        let luts = processor.get_loaded_luts();
        assert!(luts.contains(&"test_identity"));
        
        let info = processor.get_lut_info("test_identity").unwrap();
        assert_eq!(info.name, "test_identity");
        assert_eq!(info.size, 33);
        assert_eq!(info.data_points, 33 * 33 * 33);
    }
    
    #[test]
    fn test_color_correction_lut() {
        let mut processor = LutProcessor::default();
        
        let correction = ColorCorrection {
            gamma: 2.2,
            contrast: 1.2,
            saturation: 1.1,
            color_balance: [1.0, 0.9, 0.8],
        };
        
        let lut_name = processor.create_color_correction_lut("test_correction", 17, &correction).unwrap();
        
        let luts = processor.get_loaded_luts();
        assert!(luts.contains(&"test_correction"));
        
        let info = processor.get_lut_info("test_correction").unwrap();
        assert_eq!(info.size, 17);
    }
    
    #[test]
    fn test_lut_application() {
        let mut processor = LutProcessor::default();
        
        // Create identity LUT
        processor.create_identity_lut("identity", 33).unwrap();
        
        // Create test image
        let mut image = RgbImage::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                let r = (x * 255 / 100) as u8;
                let g = (y * 255 / 100) as u8;
                let b = ((x + y) * 255 / 200) as u8;
                image.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        
        // Apply identity LUT (should not change image)
        let original_data = image.clone();
        processor.apply_lut(&mut image, "identity").unwrap();
        
        // Identity LUT should preserve original values (within interpolation tolerance)
        for y in 0..100 {
            for x in 0..100 {
                let original = original_data.get_pixel(x, y);
                let processed = image.get_pixel(x, y);
                
                // Allow small differences due to interpolation
                for c in 0..3 {
                    let diff = (original.0[c] as f32 - processed.0[c] as f32).abs();
                    assert!(diff <= 2.0, "Pixel ({}, {}) channel {} changed too much", x, y, c);
                }
            }
        }
    }
    
    #[test]
    fn test_cube_file_io() {
        let mut processor = LutProcessor::default();
        
        // Create test Cube file
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.cube");
        
        let cube_content = r#"TITLE Test LUT
LUT_3D_SIZE 3

0.0 0.0 0.0
0.5 0.5 0.5
1.0 1.0 1.0
0.0 0.0 0.5
0.5 0.5 1.0
1.0 1.0 0.0
0.0 0.5 0.0
0.5 1.0 0.5
1.0 0.0 1.0
0.5 0.0 0.0
1.0 0.5 0.5
0.0 1.0 1.0
0.0 0.5 1.0
0.5 1.0 1.0
1.0 0.0 0.5
0.5 0.0 1.0
1.0 1.0 0.0
0.0 1.0 0.0
0.5 0.0 0.5
1.0 0.5 1.0
0.0 1.0 0.5
0.5 0.5 0.0
1.0 1.0 1.0
0.0 0.0 0.0
0.5 0.5 0.5
1.0 1.0 1.0
0.0 0.0 0.5
0.5 0.5 1.0
1.0 1.0 0.0
"#;
        
        fs::write(&file_path, cube_content).unwrap();
        
        // Load LUT
        let lut_name = processor.load_lut(&file_path).unwrap();
        assert_eq!(lut_name, "test");
        
        // Save LUT
        let save_path = dir.path().join("saved.cube");
        processor.save_lut("test", &save_path, LutFormat::Cube).unwrap();
        
        // Verify saved file exists
        assert!(save_path.exists());
    }
    
    #[test]
    fn test_active_lut() {
        let mut processor = LutProcessor::default();
        
        // Create and set active LUT
        processor.create_identity_lut("active_test", 33).unwrap();
        processor.set_active_lut(Some("active_test")).unwrap();
        
        // Create test image
        let mut image = RgbImage::new(10, 10);
        image.put_pixel(0, 0, Rgb([128, 128, 128]));
        
        // Apply active LUT
        processor.apply_active_lut(&mut image).unwrap();
        
        // Clear active LUT
        processor.set_active_lut(None).unwrap();
        let result = processor.apply_active_lut(&mut image);
        assert!(result.is_err());
    }
}
