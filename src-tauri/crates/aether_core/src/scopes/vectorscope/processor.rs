//! Vectorscope processor implementation
//! 
//! This module contains the main VectorscopeProcessor struct and its core functionality.

use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use log::{debug, info};
use image::{Rgb, RgbImage};

use crate::types::{
    VectorscopeData, VectorscopeConfig, ScopeStats,
};

use super::{targets::TargetRenderer, rendering::VectorscopeRenderer};
use crate::scopes::{BaseScopeProcessor, ColorConverter, FrameProcessor};

/// Vectorscope processor for real-time color analysis
pub struct VectorscopeProcessor {
    config: VectorscopeConfig,
    data: Arc<RwLock<VectorscopeData>>,
    base: BaseScopeProcessor,
    
    // Vectorscope-specific processing
    uv_buffer: Vec<(f32, f32)>,
    intensity_cache: Vec<u16>,
    
    // Specialized components
    target_renderer: TargetRenderer,
    vectorscope_renderer: VectorscopeRenderer,
}

impl VectorscopeProcessor {
    /// Create a new vectorscope processor
    pub fn new(config: VectorscopeConfig) -> Result<Self> {
        info!("Creating vectorscope processor with config: {:?}", config);
        
        let base = BaseScopeProcessor::new();
        let grid_size = config.resolution.vectorscope_grid_size();
        
        let processor = Self {
            config: config.clone(),
            data: Arc::new(RwLock::new(VectorscopeData::new(config.resolution))),
            base,
            uv_buffer: Vec::with_capacity(grid_size * grid_size),
            intensity_cache: vec![0; grid_size * grid_size],
            target_renderer: TargetRenderer::new(),
            vectorscope_renderer: VectorscopeRenderer::new(),
        };
        
        info!("Vectorscope processor created successfully");
        
        Ok(processor)
    }
    
    /// Process an image frame and update vectorscope data
    pub fn process_frame(&mut self, image: &RgbImage, frame_number: u64, timestamp: f64) -> Result<()> {
        debug!("Processing vectorscope frame {} at timestamp {:.3}", frame_number, timestamp);
        
        // Clear previous data
        self.clear_data();
        
        // Process frame using common frame processor
        let frame_processor = FrameProcessor;
        let (pixels_processed, processing_time) = frame_processor.process_frame(image, |r, g, b| {
            // Convert RGB to UV coordinates
            let (u, v) = self.base.color_converter().rgb_to_uv(r, g, b);
            
            // Add point to vectorscope
            self.add_uv_point(u, v, 1);
        });
        
        // Update statistics
        let processing_time_us = processing_time.as_micros() as f64;
        self.base.update_stats(frame_number, timestamp, processing_time_us, pixels_processed)?;
        
        debug!("Vectorscope frame {} processed in {:.2}μs, {} pixels", 
               frame_number, processing_time_us, pixels_processed);
        
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
        
        // Use specialized renderer
        self.vectorscope_renderer.generate_image(width, height, &data, &self.config)
    }
    
    /// Get processing statistics
    pub fn get_stats(&self) -> Result<ScopeStats> {
        self.base.get_stats()
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
        
        // Resize buffers
        let grid_size = config.resolution.vectorscope_grid_size();
        self.intensity_cache.resize(grid_size * grid_size, 0);
        
        info!("Vectorscope configuration updated");
        
        Ok(())
    }
    
    /// Clear vectorscope data
    fn clear_data(&mut self) {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
        self.uv_buffer.clear();
        self.intensity_cache.fill(0);
    }
}

impl Default for VectorscopeProcessor {
    fn default() -> Self {
        Self::new(VectorscopeConfig::default()).unwrap()
    }
}
