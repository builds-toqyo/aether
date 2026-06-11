

use anyhow::{Result};

use crate::types::{HistogramData, HistogramChannel};

use crate::scopes::{Statistics, ChannelStatistics};


pub struct HistogramAnalyzer {
    statistics: Statistics,
}

impl HistogramAnalyzer {

    pub fn new() -> Self {
        Self {
            statistics: Statistics,
        }
    }

    pub fn get_statistics(&self, data: &HistogramData) -> Result<HistogramStatistics> {
        let mut stats = HistogramStatistics::new();

        for channel in [HistogramChannel::Red, HistogramChannel::Green, HistogramChannel::Blue, HistogramChannel::Luma] {
            let channel_data = data.channel_data(channel);
            let channel_stats = Statistics::calculate_channel_stats(channel_data);

            match channel {
                HistogramChannel::Red => stats.red = channel_stats,
                HistogramChannel::Green => stats.green = channel_stats,
                HistogramChannel::Blue => stats.blue = channel_stats,
                HistogramChannel::Luma => stats.luma = channel_stats,
            }
        }

        Ok(stats)
    }

    pub fn analyze_exposure(&self, data: &HistogramData) -> Result<ExposureAnalysis> {
        let mut analysis = ExposureAnalysis::new();

        let luma_data = data.channel_data(HistogramChannel::Luma);
        let total_samples: u64 = luma_data.iter().map(|&x| x as u64).sum();

        if total_samples == 0 {
            return Ok(analysis);
        }

        let shadow_samples: u64 = luma_data[0..64].iter().map(|&x| x as u64).sum();
        let midtone_samples: u64 = luma_data[64..192].iter().map(|&x| x as u64).sum();
        let highlight_samples: u64 = luma_data[192..256].iter().map(|&x| x as u64).sum();

        analysis.shadow_percentage = (shadow_samples as f32 / total_samples as f32) * 100.0;
        analysis.midtone_percentage = (midtone_samples as f32 / total_samples as f32) * 100.0;
        analysis.highlight_percentage = (highlight_samples as f32 / total_samples as f32) * 100.0;

        analysis.black_clipped = (luma_data[0] as u64) > total_samples / 1000;
        analysis.white_clipped = (luma_data[255] as u64) > total_samples / 1000;

        let mut min_bin = 255usize;
        let mut max_bin = 0usize;
        for (bin, &count) in luma_data.iter().enumerate() {
            if count > 0 {
                min_bin = min_bin.min(bin);
                max_bin = max_bin.max(bin);
            }
        }
        analysis.dynamic_range = (max_bin - min_bin) as u8;

        Ok(analysis)
    }

    pub fn analyze_color_balance(&self, data: &HistogramData) -> Result<ColorBalanceAnalysis> {
        let mut analysis = ColorBalanceAnalysis::new();

        let red_stats = Statistics::calculate_channel_stats(data.channel_data(HistogramChannel::Red));
        let green_stats = Statistics::calculate_channel_stats(data.channel_data(HistogramChannel::Green));
        let blue_stats = Statistics::calculate_channel_stats(data.channel_data(HistogramChannel::Blue));

        analysis.red_mean = red_stats.mean;
        analysis.green_mean = green_stats.mean;
        analysis.blue_mean = blue_stats.mean;


        let avg_mean = (analysis.red_mean + analysis.green_mean + analysis.blue_mean) as f32 / 3.0;
        analysis.red_cast = analysis.red_mean as f32 - avg_mean;
        analysis.green_cast = analysis.green_mean as f32 - avg_mean;
        analysis.blue_cast = analysis.blue_mean as f32 - avg_mean;


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

    pub fn check_issues(&self, data: &HistogramData) -> Result<Vec<HistogramIssue>> {
        let mut issues = Vec::new();

        let luma_data = data.channel_data(HistogramChannel::Luma);
        let total_samples: u64 = luma_data.iter().map(|&x| x as u64).sum();

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

        let exposure_analysis = self.analyze_exposure(data)?;
        if exposure_analysis.dynamic_range < 200 {
            issues.push(HistogramIssue::LimitedDynamicRange(exposure_analysis.dynamic_range));
        }

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


#[derive(Debug, Clone)]
pub enum HistogramIssue {
    BlackClipping(f32),
    WhiteClipping(f32),
    LimitedDynamicRange(u8),
    ColorCast(ColorCast, f32),
}
