//! LUT processor implementation
//! 
//! This module contains the main LutProcessor struct and its core functionality.

use anyhow::{Result, anyhow};
use log::{debug, info};
use image::RgbImage;

use super::{loader::LutLoader, saver::LutSaver, applicator::LutApplicator, config::LutConfig, types::LutData};

/// Professional LUT processor
pub struct LutProcessor {
    luts: std::collections::HashMap<String, LutData>,
    active_lut: Option<String>,
    config: LutConfig,
    loader: LutLoader,
    saver: LutSaver,
    applicator: LutApplicator,
}

impl LutProcessor {
    /// Create new LUT processor
    pub fn new(config: LutConfig) -> Result<Self> {
        info!("Creating LUT processor with config: {:?}", config);
        
        let processor = Self {
            luts: std::collections::HashMap::new(),
            active_lut: None,
            config: config.clone(),
            loader: LutLoader::new(),
            saver: LutSaver::new(),
            applicator: LutApplicator::new(config.clone()),
        };
        
        info!("LUT processor created successfully");
        
        Ok(processor)
    }
    
    /// Load LUT from file
    pub fn load_lut(&mut self, file_path: &std::path::Path) -> Result<String> {
        debug!("Loading LUT from: {:?}", file_path);
        
        let lut_data = self.loader.load_lut(file_path)?;
        let lut_name = lut_data.name.clone();
        
        self.luts.insert(lut_name.clone(), lut_data);
        
        info!("LUT loaded successfully: {}", lut_name);
        
        Ok(lut_name)
    }
    
    /// Save LUT to file
    pub fn save_lut(&self, lut_name: &str, file_path: &std::path::Path, format: super::types::LutFormat) -> Result<()> {
        debug!("Saving LUT '{}' to: {:?}", lut_name, file_path);
        
        let lut_data = self.luts.get(lut_name)
            .ok_or_else(|| anyhow!("LUT not found: {}", lut_name))?;
        
        self.saver.save_lut(lut_data, file_path, format)?;
        
        info!("LUT saved successfully: {} -> {:?}", lut_name, file_path);
        
        Ok(())
    }
    
    /// Apply LUT to image
    pub fn apply_lut(&self, image: &mut RgbImage, lut_name: &str) -> Result<()> {
        debug!("Applying LUT '{}' to image", lut_name);
        
        let lut_data = self.luts.get(lut_name)
            .ok_or_else(|| anyhow!("LUT not found: {}", lut_name))?;
        
        self.applicator.apply_lut(image, lut_data)?;
        
        debug!("LUT applied successfully");
        
        Ok(())
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
        lut_data.format = super::types::LutFormat::Cube;
        lut_data.generate_identity_lut(size);
        
        let lut_name = name.to_string();
        self.luts.insert(lut_name.clone(), lut_data);
        
        info!("Identity LUT created: {}", lut_name);
        
        Ok(lut_name)
    }
    
    /// Create color correction LUT
    pub fn create_color_correction_lut(&mut self, name: &str, size: u32, correction: &super::types::ColorCorrection) -> Result<String> {
        debug!("Creating color correction LUT: {}", name);
        
        let mut lut_data = LutData::default();
        lut_data.name = name.to_string();
        lut_data.size = size;
        lut_data.format = super::types::LutFormat::Cube;
        
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
    fn apply_color_correction(&self, rgb: [f32; 3], correction: &super::types::ColorCorrection) -> [f32; 3] {
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
    pub fn get_lut_info(&self, lut_name: &str) -> Option<super::types::LutInfo> {
        self.luts.get(lut_name).map(|lut_data| super::types::LutInfo {
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
        self.config = config.clone();
        self.applicator.update_config(config);
    }
}

impl Default for LutProcessor {
    fn default() -> Self {
        Self::new(LutConfig::default()).unwrap()
    }
}
