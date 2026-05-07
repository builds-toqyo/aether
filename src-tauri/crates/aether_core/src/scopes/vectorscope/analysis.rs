

use anyhow::{Result};
use log::debug;

use crate::types::{
    VectorscopeData, VectorscopeTarget, ColorSpace,
};

use crate::scopes::ColorConverter;


pub struct VectorscopeAnalyzer {
    color_converter: ColorConverter,
}

impl VectorscopeAnalyzer {

    pub fn new() -> Self {
        Self {
            color_converter: ColorConverter::new(ColorSpace::Rec709),
        }
    }


    pub fn analyze_color_distribution(&self, data: &VectorscopeData) -> Result<ColorDistribution> {
        let mut distribution = ColorDistribution::new();


        let max_intensity = data.intensity_grid.iter().copied().max().unwrap_or(0);
        let total_intensity: u64 = data.intensity_grid.iter().map(|&v| v as u64).sum();

        distribution.max_intensity = max_intensity;
        distribution.total_intensity = total_intensity;
        distribution.average_intensity = if total_intensity > 0 {
            total_intensity as f32 / data.intensity_grid.len() as f32
        } else {
            0.0
        };


        let mut red_sum = 0.0;
        let mut green_sum = 0.0;
        let mut blue_sum = 0.0;
        let mut sample_count = 0;

        for point in &data.points {

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


    pub fn check_target_compliance(&self, data: &VectorscopeData) -> Result<TargetCompliance> {
        let mut compliance = TargetCompliance::new();

        for target in &data.config.targets {
            let (target_u, target_v) = self.get_target_uv(*target);
            let intensity = data.get_intensity(target_u, target_v);

            let target_result = TargetResult {
                target: *target,
                intensity,
                within_range: intensity > 10,
                deviation: self.calculate_uv_deviation(target_u, target_v, data),
            };

            compliance.targets.push(target_result);
        }

        Ok(compliance)
    }


    fn uv_to_rgb(&self, u: f32, v: f32) -> (f32, f32, f32) {

        let r = (v + 0.5).clamp(0.0, 1.0);
        let g = (0.5 - u.abs() * 0.5 - v.abs() * 0.5).clamp(0.0, 1.0);
        let b = (u + 0.5).clamp(0.0, 1.0);

        (r, g, b)
    }


    fn get_target_uv(&self, target: VectorscopeTarget) -> (f32, f32) {
        match target {
            VectorscopeTarget::Primary => (0.0, 0.0),
            VectorscopeTarget::SkinTones => (0.1, 0.2),
            VectorscopeTarget::Blue => (-0.2, -0.4),
            VectorscopeTarget::Yellow => (0.3, 0.4),
            VectorscopeTarget::Cyan => (-0.3, 0.2),
            VectorscopeTarget::Green => (-0.4, -0.2),
            VectorscopeTarget::Magenta => (0.4, -0.2),
            VectorscopeTarget::Red => (0.3, -0.4),
        }
    }


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

impl Default for VectorscopeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}


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


#[derive(Debug, Clone)]
pub struct TargetResult {
    pub target: VectorscopeTarget,
    pub intensity: u16,
    pub within_range: bool,
    pub deviation: f32,
}
