//! Histogram analysis and metrics
//! 
//! This module provides advanced analysis capabilities for histogram data,
//! including exposure analysis, color balance checking, and issue detection.

use anyhow::{Result};
use log::debug;

use crate::types::{HistogramData, HistogramChannel};

use crate::scopes::{Statistics, ChannelStatistics};

/// Analyzer for advanced histogram analysis
pub struct HistogramAnalyzer {
    statistics: Statistics,
}

impl HistogramAnalyzer {
    /// Create new histogram analyzer
    pub fn new() -> Self {
        Self {
            statistics: Statistics,
        }
    }
    
    /// Get comprehensive histogram statistics
    pub fn get_statistics(&self, data: &HistogramData) -> Result<HistogramStatistics> {
        let mut stats = HistogramStatistics::new();
        
        // Calculate statistics for each channel
        for channel in [HistogramChannel::Red, HistogramChannel::Green, HistogramChannel::Blue, HistogramChannel::Luma] {
            let channel_data = data.channel_data(channel);
            let channel_stats = self.statistics.calculate_channel_stats(channel_data);
            
            match channel {
                HistogramChannel::Red => stats.red = channel_stats,
                HistogramChannel::Green => stats.green = channel_stats,
                HistogramChannel::Blue => stats.blue = channel_stats,
                HistogramChannel::Luma => stats.luma = channel_stats,
            }
        }
        
        Ok(stats)
    }
    
    /// Analyze exposure levels
    pub fn analyze_exposure(&self, data: &HistogramData) -> Result<ExposureAnalysis> {
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
    pub fn analyze_color_balance(&self, data: &HistogramData) -> Result<ColorBalanceAnalysis> {
        let mut analysis = ColorBalanceAnalysis::new();
        
        // Get statistics for each channel
        let red_stats = self.statistics.calculate_channel_stats(data.channel_data(HistogramChannel::Red));
        let green_stats = self.statistics.calculate_channel_stats(data.channel_data(HistogramChannel::Green));
        let blue_stats = self.statistics.calculate_channel_stats(data.channel_data(HistogramChannel::Blue));
        
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
    pub fn check_issues(&self, data: &HistogramData) -> Result<Vec<HistogramIssue>> {
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
        let exposure_analysis = self.analyze_exposure(data)?;
        if exposure_analysis.dynamic_range < 200 {
            issues.push(HistogramIssue::LimitedDynamicRange(exposure_analysis.dynamic_range));
        }
        
        // Check for color cast
        let color_analysis = self.analyze_color_balance(data)?;
        if color_analysis.dominant_cast != ColorCast::Neutral {
            let cast_strength = color_analysis.red_cast.abs().max(color_analysis.green_cast.abs().max(color_analysis.blue_cast.abs()));
            if cast_strength > 10.0 {
                issues.push(HistogramIssue::ColorCast(color_analysis.dominant_cast, cast_strength));
            }
        }
        
        Ok(issues)
    }
}

impl Default for HistogramAnalyzer {
    fn default() -> Self {
        Self::new()
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
