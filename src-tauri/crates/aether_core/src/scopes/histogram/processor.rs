//! Histogram processor implementation
//! 
//! This module contains the main HistogramProcessor struct and its core functionality.

use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use log::{debug, info};
use image::{Rgb, RgbImage};

use crate::types::{
    HistogramData, HistogramChannel, HistogramConfig, HistogramMode,
    ScopeStats,
};

use super::{rendering::HistogramRenderer, analysis::HistogramAnalyzer};
use crate::scopes::{BaseScopeProcessor, FrameProcessor};

/// Histogram processor for real-time color analysis
pub struct HistogramProcessor {
    config: HistogramConfig,
    data: Arc<RwLock<HistogramData>>,
    base: BaseScopeProcessor,
    
    // Specialized components
    renderer: HistogramRenderer,
    analyzer: HistogramAnalyzer,
}

impl HistogramProcessor {
    /// Create a new histogram processor
    pub fn new(config: HistogramConfig) -> Result<Self> {
        info!("Creating histogram processor with config: {:?}", config);
        
        let base = BaseScopeProcessor::new();
        
        let processor = Self {
            config: config.clone(),
            data: Arc::new(RwLock::new(HistogramData::new(config.resolution))),
            base,
            renderer: HistogramRenderer::new(),
            analyzer: HistogramAnalyzer::new(),
        };
        
        info!("Histogram processor created successfully");
        
        Ok(processor)
    }
    
    /// Process an image frame and update histogram data
    pub fn process_frame(&mut self, image: &RgbImage, frame_number: u64, timestamp: f64) -> Result<()> {
        debug!("Processing histogram frame {} at timestamp {:.3}", frame_number, timestamp);
        
        // Clear previous data
        self.clear_data();
        
        // Process frame using common frame processor
        let frame_processor = FrameProcessor;
        let (pixels_processed, processing_time) = frame_processor.process_frame(image, |r, g, b| {
            // Add sample to histogram
            self.add_sample(r, g, b);
        });
        
        // Update statistics
        let processing_time_us = processing_time.as_micros() as f64;
        self.base.update_stats(frame_number, timestamp, processing_time_us, pixels_processed)?;
        
        debug!("Histogram frame {} processed in {:.2}μs, {} pixels", 
               frame_number, processing_time_us, pixels_processed);
        
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
        
        // Use specialized renderer
        self.renderer.generate_image(width, height, &data, &self.config)
    }
    
    /// Get histogram statistics
    pub fn get_statistics(&self) -> Result<HistogramStatistics> {
        let data = self.data.read().map_err(|e| anyhow!("Data lock error: {}", e))?;
        self.analyzer.get_statistics(&data)
    }
    
    /// Get processing statistics
    pub fn get_stats(&self) -> Result<ScopeStats> {
        self.base.get_stats()
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
        
        info!("Histogram configuration updated");
        
        Ok(())
    }
    
    /// Clear histogram data
    fn clear_data(&mut self) {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
        self.base.buffers().clear();
    }
}

impl Default for HistogramProcessor {
    fn default() -> Self {
        Self::new(HistogramConfig::default()).unwrap()
    }
}
