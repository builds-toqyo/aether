use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskEffectType {
    Blur,
    Sharpen,
    Emboss,
    EdgeDetect,
    Noise,
    Displace,
    Turbulence,
    FractalNoise,
    MotionBlur,
    GaussianBlur,
    BoxBlur,
    MedianFilter,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskEffectParameters {
    pub intensity: f64,
    pub radius: f64,
    pub angle: f64,
    pub amount: f64,
    pub threshold: f64,
    pub quality: EffectQuality,
    pub custom_params: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EffectQuality {
    Low,
    Medium,
    High,
    Ultra,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskEffect {
    pub id: String,
    pub name: String,
    pub effect_type: MaskEffectType,
    pub parameters: MaskEffectParameters,
    pub enabled: bool,
    pub blend_mode: MaskBlendMode,
    pub opacity: f64,
    pub mask_channel: MaskChannel,
}


#[derive(Debug, Clone)]
pub struct MaskEffectProcessor {
    pub effects: Vec<MaskEffect>,
    pub cache_enabled: bool,
    pub cache_resolution: (u32, u32),
    pub cache_data: HashMap<String, Vec<f64>>,
}

impl MaskEffectParameters {
    pub fn new() -> Self {
        Self {
            intensity: 1.0,
            radius: 1.0,
            angle: 0.0,
            amount: 1.0,
            threshold: 0.5,
            quality: EffectQuality::Medium,
            custom_params: HashMap::new(),
        }
    }

    pub fn with_intensity(mut self, intensity: f64) -> Self {
        self.intensity = intensity.clamp(0.0, 2.0);
        self
    }

    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    pub fn with_angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    pub fn with_amount(mut self, amount: f64) -> Self {
        self.amount = amount.clamp(0.0, 2.0);
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    pub fn with_quality(mut self, quality: EffectQuality) -> Self {
        self.quality = quality;
        self
    }

    pub fn with_custom_param(mut self, key: String, value: f64) -> Self {
        self.custom_params.insert(key, value);
        self
    }

    pub fn get_custom_param(&self, key: &str) -> Option<f64> {
        self.custom_params.get(key).copied()
    }

    pub fn set_custom_param(&mut self, key: String, value: f64) {
        self.custom_params.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.intensity < 0.0 || self.intensity > 2.0 {
            return Err("Intensity must be between 0.0 and 2.0".to_string());
        }

        if self.radius < 0.0 {
            return Err("Radius must be non-negative".to_string());
        }

        if self.amount < 0.0 || self.amount > 2.0 {
            return Err("Amount must be between 0.0 and 2.0".to_string());
        }

        if self.threshold < 0.0 || self.threshold > 1.0 {
            return Err("Threshold must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }
}

impl Default for MaskEffectParameters {
    fn default() -> Self {
        Self::new()
    }
}

impl MaskEffect {
    pub fn new(id: String, name: String, effect_type: MaskEffectType) -> Self {
        Self {
            id,
            name,
            effect_type,
            parameters: MaskEffectParameters::new(),
            enabled: true,
            blend_mode: MaskBlendMode::Normal,
            opacity: 1.0,
            mask_channel: MaskChannel::Alpha,
        }
    }

    pub fn with_parameters(mut self, parameters: MaskEffectParameters) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn with_blend_mode(mut self, blend_mode: MaskBlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_mask_channel(mut self, channel: MaskChannel) -> Self {
        self.mask_channel = channel;
        self
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn set_blend_mode(&mut self, blend_mode: MaskBlendMode) {
        self.blend_mode = blend_mode;
    }

    pub fn set_mask_channel(&mut self, channel: MaskChannel) {
        self.mask_channel = channel;
    }

    pub fn apply_to_value(&self, value: f64, x: f64, y: f64) -> f64 {
        if !self.enabled {
            return value;
        }

        let mut result = value;

        match self.effect_type {
            MaskEffectType::Blur => result = self.apply_blur(result, x, y),
            MaskEffectType::Sharpen => result = self.apply_sharpen(result, x, y),
            MaskEffectType::Emboss => result = self.apply_emboss(result, x, y),
            MaskEffectType::EdgeDetect => result = self.apply_edge_detect(result, x, y),
            MaskEffectType::Noise => result = self.apply_noise(result, x, y),
            MaskEffectType::Displace => result = self.apply_displace(result, x, y),
            MaskEffectType::Turbulence => result = self.apply_turbulence(result, x, y),
            MaskEffectType::FractalNoise => result = self.apply_fractal_noise(result, x, y),
            MaskEffectType::MotionBlur => result = self.apply_motion_blur(result, x, y),
            MaskEffectType::GaussianBlur => result = self.apply_gaussian_blur(result, x, y),
            MaskEffectType::BoxBlur => result = self.apply_box_blur(result, x, y),
            MaskEffectType::MedianFilter => result = self.apply_median_filter(result, x, y),
            MaskEffectType::Custom(_) => result = self.apply_custom_effect(result, x, y),
        }


        result = result * self.opacity;


        result = self.apply_blend_mode(result, value);

        result.clamp(0.0, 1.0)
    }

    fn apply_blur(&self, value: f64, x: f64, y: f64) -> f64 {
        let blur_amount = self.parameters.radius * self.parameters.intensity;
        let noise = (x * y).sin().abs() * 0.1;
        let blur_factor = 1.0 - (blur_amount / 10.0).min(1.0);
        value * blur_factor + noise * (1.0 - blur_factor)
    }

    fn apply_sharpen(&self, value: f64, x: f64, y: f64) -> f64 {
        let sharpen_amount = self.parameters.intensity;
        let edge_factor = ((x * 0.1).sin() * (y * 0.1).cos()).abs();
        value + (value - 0.5) * sharpen_amount * edge_factor
    }

    fn apply_emboss(&self, value: f64, x: f64, y: f64) -> f64 {
        let angle = self.parameters.angle.to_radians();
        let emboss_factor = (x * angle.cos() + y * angle.sin()).sin() * 0.5 + 0.5;
        value * emboss_factor * self.parameters.intensity
    }

    fn apply_edge_detect(&self, value: f64, x: f64, y: f64) -> f64 {
        let threshold = self.parameters.threshold;
        let edge_factor = ((x * 0.2).sin() * (y * 0.2).cos()).abs();
        if edge_factor > threshold {
            1.0
        } else {
            0.0
        }
    }

    fn apply_noise(&self, value: f64, x: f64, y: f64) -> f64 {
        let noise_amount = self.parameters.intensity;
        let noise = (x * 123.456 + y * 789.012).sin().abs() * 0.5;
        value + (noise - 0.25) * noise_amount
    }

    fn apply_displace(&self, value: f64, x: f64, y: f64) -> f64 {
        let amount = self.parameters.amount;
        let displacement = (x * 0.1).sin() * (y * 0.1).cos() * amount;
        (value + displacement).clamp(0.0, 1.0)
    }

    fn apply_turbulence(&self, value: f64, x: f64, y: f64) -> f64 {
        let scale = self.parameters.radius;
        let turbulence = self.turbulence_noise(x * scale, y * scale, 4);
        value * (0.5 + turbulence * 0.5) * self.parameters.intensity
    }

    fn apply_fractal_noise(&self, value: f64, x: f64, y: f64) -> f64 {
        let scale = self.parameters.radius;
        let depth = self.parameters.intensity as u32;
        let noise = self.fractal_noise(x * scale, y * scale, depth);
        value * (0.5 + noise * 0.5)
    }

    fn apply_motion_blur(&self, value: f64, x: f64, y: f64) -> f64 {
        let angle = self.parameters.angle.to_radians();
        let distance = self.parameters.radius;
        let blur_samples = 5;
        let mut sum = 0.0;

        for i in 0..blur_samples {
            let t = (i as f64 - blur_samples as f64 / 2.0) / (blur_samples as f64 / 2.0);
            let sample_x = x + t * distance * angle.cos();
            let sample_y = y + t * distance * angle.sin();
            sum += (sample_x * sample_y).sin().abs() * 0.1 + value;
        }

        sum / blur_samples as f64
    }

    fn apply_gaussian_blur(&self, value: f64, x: f64, y: f64) -> f64 {
        let sigma = self.parameters.radius;
        let blur_factor = (-((x * x + y * y) / (2.0 * sigma * sigma))).exp();
        value * blur_factor * self.parameters.intensity
    }

    fn apply_box_blur(&self, value: f64, x: f64, y: f64) -> f64 {
        let radius = self.parameters.radius as i32;
        let mut sum = 0.0;
        let mut count = 0;

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let sample_x = x + dx as f64;
                let sample_y = y + dy as f64;
                sum += (sample_x * sample_y).sin().abs() * 0.1 + value;
                count += 1;
            }
        }

        sum / count as f64
    }

    fn apply_median_filter(&self, value: f64, x: f64, y: f64) -> f64 {
        let radius = self.parameters.radius as i32;
        let mut values = Vec::new();

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let sample_x = x + dx as f64;
                let sample_y = y + dy as f64;
                let sample_value = (sample_x * sample_y).sin().abs() * 0.1 + value;
                values.push(sample_value);
            }
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        values[values.len() / 2]
    }

    fn apply_custom_effect(&self, value: f64, x: f64, y: f64) -> f64 {

        if let Some(custom_value) = self.parameters.get_custom_param("multiplier") {
            value * custom_value
        } else {
            value
        }
    }

    fn apply_blend_mode(&self, modified_value: f64, original_value: f64) -> f64 {
        match self.blend_mode {
            MaskBlendMode::Normal => modified_value,
            MaskBlendMode::Add => (modified_value + original_value).min(1.0),
            MaskBlendMode::Subtract => (modified_value - original_value).max(0.0),
            MaskBlendMode::Multiply => modified_value * original_value,
            MaskBlendMode::Screen => 1.0 - (1.0 - modified_value) * (1.0 - original_value),
            MaskBlendMode::Overlay => {
                if original_value < 0.5 {
                    2.0 * modified_value * original_value
                } else {
                    1.0 - 2.0 * (1.0 - modified_value) * (1.0 - original_value)
                }
            }
            MaskBlendMode::SoftLight => {
                if original_value < 0.5 {
                    2.0 * modified_value * original_value + original_value * original_value - 2.0 * original_value * original_value * modified_value
                } else {
                    modified_value + original_value - 2.0 * modified_value * original_value
                }
            }
            MaskBlendMode::HardLight => {
                if modified_value < 0.5 {
                    2.0 * modified_value * original_value
                } else {
                    1.0 - 2.0 * (1.0 - modified_value) * (1.0 - original_value)
                }
            }
            MaskBlendMode::Difference => (modified_value - original_value).abs(),
            MaskBlendMode::Exclusion => modified_value + original_value - 2.0 * modified_value * original_value,
            MaskBlendMode::In => modified_value * original_value,
            MaskBlendMode::Out => 1.0 - (1.0 - modified_value) * (1.0 - original_value),
        }
    }

    fn turbulence_noise(&self, x: f64, y: f64, octaves: u32) -> f64 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            value += (x * frequency).sin() * (y * frequency).cos() * amplitude;
            max_value += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        (value / max_value + 1.0) / 2.0
    }

    fn fractal_noise(&self, x: f64, y: f64, depth: u32) -> f64 {
        if depth == 0 {
            0.0
        } else {
            let noise = (x * 123.456).sin() * (y * 789.012).cos();
            noise * 0.5 + self.fractal_noise(x * 2.0, y * 2.0, depth - 1) * 0.5
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Effect ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Effect name cannot be empty".to_string());
        }

        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err("Effect opacity must be between 0.0 and 1.0".to_string());
        }

        self.parameters.validate()?;

        Ok(())
    }
}

impl MaskEffectProcessor {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            cache_enabled: true,
            cache_resolution: (512, 512),
            cache_data: HashMap::new(),
        }
    }

    pub fn with_cache(mut self, enabled: bool, resolution: (u32, u32)) -> Self {
        self.cache_enabled = enabled;
        self.cache_resolution = resolution;
        self
    }

    pub fn add_effect(&mut self, effect: MaskEffect) {
        self.effects.push(effect);
        self.clear_cache();
    }

    pub fn remove_effect(&mut self, effect_id: &str) -> bool {
        let initial_len = self.effects.len();
        self.effects.retain(|effect| effect.id != effect_id);
        let removed = self.effects.len() < initial_len;
        if removed {
            self.clear_cache();
        }
        removed
    }

    pub fn get_effect(&self, effect_id: &str) -> Option<&MaskEffect> {
        self.effects.iter().find(|effect| effect.id == effect_id)
    }

    pub fn get_effect_mut(&mut self, effect_id: &str) -> Option<&mut MaskEffect> {
        self.effects.iter_mut().find(|effect| effect.id == effect_id)
    }

    pub fn process_value(&mut self, value: f64, x: f64, y: f64) -> f64 {
        let cache_key = format!("{:.2}_{:.2}_{:.4}", x, y, value);

        if self.cache_enabled {
            if let Some(cached_value) = self.cache_data.get(&cache_key) {
                if !cached_value.is_empty() {
                    return cached_value[0];
                }
            }
        }

        let mut result = value;

        for effect in &self.effects {
            result = effect.apply_to_value(result, x, y);
        }


        if self.cache_enabled {
            self.cache_data.insert(cache_key, vec![result]);
        }

        result
    }

    pub fn process_region(&mut self, values: &[f64], bounds: (f64, f64, f64, f64), resolution: (u32, u32)) -> Vec<f64> {
        let (min_x, min_y, max_x, max_y) = bounds;
        let (width, height) = resolution;
        let mut result = Vec::with_capacity(values.len());

        let x_step = (max_x - min_x) / width as f64;
        let y_step = (max_y - min_y) / height as f64;

        for (i, &value) in values.iter().enumerate() {
            let x = min_x + (i % width as usize) as f64 * x_step;
            let y = min_y + (i / width as usize) as f64 * y_step;
            let processed_value = self.process_value(value, x, y);
            result.push(processed_value);
        }

        result
    }

    pub fn clear_cache(&mut self) {
        self.cache_data.clear();
    }

    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
        if !enabled {
            self.clear_cache();
        }
    }

    pub fn set_cache_resolution(&mut self, resolution: (u32, u32)) {
        self.cache_resolution = resolution;
        self.clear_cache();
    }

    pub fn get_cache_size(&self) -> usize {
        self.cache_data.len()
    }

    pub fn get_memory_usage(&self) -> usize {
        self.cache_data.values()
            .map(|data| data.len() * std::mem::size_of::<f64>())
            .sum()
    }

    pub fn create_blur_effect(id: String, name: String, radius: f64, intensity: f64) -> MaskEffect {
        let params = MaskEffectParameters::new()
            .with_radius(radius)
            .with_intensity(intensity);

        MaskEffect::new(id, name, MaskEffectType::Blur)
            .with_parameters(params)
    }

    pub fn create_sharpen_effect(id: String, name: String, intensity: f64) -> MaskEffect {
        let params = MaskEffectParameters::new()
            .with_intensity(intensity);

        MaskEffect::new(id, name, MaskEffectType::Sharpen)
            .with_parameters(params)
    }

    pub fn create_noise_effect(id: String, name: String, intensity: f64) -> MaskEffect {
        let params = MaskEffectParameters::new()
            .with_intensity(intensity);

        MaskEffect::new(id, name, MaskEffectType::Noise)
            .with_parameters(params)
    }

    pub fn validate(&self) -> Result<(), String> {
        for (i, effect) in self.effects.iter().enumerate() {
            effect.validate().map_err(|e| format!("Effect {}: {}", i, e))?;
        }


        let mut ids = std::collections::HashSet::new();
        for effect in &self.effects {
            if !ids.insert(&effect.id) {
                return Err(format!("Duplicate effect ID: {}", effect.id));
            }
        }

        Ok(())
    }
}

impl Default for MaskEffectProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_effect_creation() {
        let effect = MaskEffect::new("effect1".to_string(), "Blur Effect".to_string(), MaskEffectType::Blur);

        assert_eq!(effect.id, "effect1");
        assert_eq!(effect.name, "Blur Effect");
        assert_eq!(effect.effect_type, MaskEffectType::Blur);
        assert!(effect.enabled);
        assert_eq!(effect.blend_mode, MaskBlendMode::Normal);
        assert_eq!(effect.opacity, 1.0);
        assert_eq!(effect.mask_channel, MaskChannel::Alpha);
    }

    #[test]
    fn test_mask_effect_builder() {
        let params = MaskEffectParameters::new()
            .with_intensity(1.5)
            .with_radius(5.0)
            .with_angle(45.0)
            .with_quality(EffectQuality::High);

        let effect = MaskEffect::new("effect1".to_string(), "Test".to_string(), MaskEffectType::Sharpen)
            .with_parameters(params)
            .with_blend_mode(MaskBlendMode::Multiply)
            .with_opacity(0.8)
            .with_mask_channel(MaskChannel::Luminance);

        assert_eq!(effect.parameters.intensity, 1.5);
        assert_eq!(effect.parameters.radius, 5.0);
        assert_eq!(effect.parameters.angle, 45.0);
        assert_eq!(effect.parameters.quality, EffectQuality::High);
        assert_eq!(effect.blend_mode, MaskBlendMode::Multiply);
        assert_eq!(effect.opacity, 0.8);
        assert_eq!(effect.mask_channel, MaskChannel::Luminance);
    }

    #[test]
    fn test_mask_effect_parameters() {
        let params = MaskEffectParameters::new()
            .with_intensity(0.8)
            .with_radius(10.0)
            .with_angle(90.0)
            .with_amount(1.2)
            .with_threshold(0.7)
            .with_quality(EffectQuality::Ultra)
            .with_custom_param("custom".to_string(), 42.0);

        assert_eq!(params.intensity, 0.8);
        assert_eq!(params.radius, 10.0);
        assert_eq!(params.angle, 90.0);
        assert_eq!(params.amount, 1.2);
        assert_eq!(params.threshold, 0.7);
        assert_eq!(params.quality, EffectQuality::Ultra);
        assert_eq!(params.get_custom_param("custom"), Some(42.0));

        params.set_custom_param("test".to_string(), 123.0);
        assert_eq!(params.get_custom_param("test"), Some(123.0));
    }

    #[test]
    fn test_effect_application() {
        let effect = MaskEffect::new("blur".to_string(), "Blur".to_string(), MaskEffectType::Blur)
            .with_parameters(MaskEffectParameters::new().with_intensity(1.0).with_radius(5.0));

        let result = effect.apply_to_value(0.8, 10.0, 15.0);
        assert!(result >= 0.0 && result <= 1.0);
        assert!(result != 0.8);
    }

    #[test]
    fn test_effect_processor() {
        let processor = MaskEffectProcessor::new();

        assert!(processor.effects.is_empty());
        assert!(processor.cache_enabled);
        assert_eq!(processor.cache_resolution, (512, 512));
    }

    #[test]
    fn test_add_remove_effects() {
        let mut processor = MaskEffectProcessor::new();
        let effect = MaskEffect::new("effect1".to_string(), "Test".to_string(), MaskEffectType::Blur);

        processor.add_effect(effect);
        assert_eq!(processor.effects.len(), 1);

        let removed = processor.remove_effect("effect1");
        assert!(removed);
        assert_eq!(processor.effects.len(), 0);

        let not_removed = processor.remove_effect("nonexistent");
        assert!(!not_removed);
    }

    #[test]
    fn test_effect_processing() {
        let mut processor = MaskEffectProcessor::new();
        let effect = MaskEffect::new("blur".to_string(), "Blur".to_string(), MaskEffectType::Blur)
            .with_parameters(MaskEffectParameters::new().with_intensity(0.5));

        processor.add_effect(effect);

        let result = processor.process_value(0.8, 10.0, 15.0);
        assert!(result >= 0.0 && result <= 1.0);
        assert!(result != 0.8);
    }

    #[test]
    fn test_effect_convenience_methods() {
        let blur_effect = MaskEffectProcessor::create_blur_effect(
            "blur".to_string(),
            "Blur".to_string(),
            5.0,
            1.0
        );

        assert_eq!(blur_effect.effect_type, MaskEffectType::Blur);
        assert_eq!(blur_effect.parameters.radius, 5.0);
        assert_eq!(blur_effect.parameters.intensity, 1.0);

        let sharpen_effect = MaskEffectProcessor::create_sharpen_effect(
            "sharpen".to_string(),
            "Sharpen".to_string(),
            1.5
        );

        assert_eq!(sharpen_effect.effect_type, MaskEffectType::Sharpen);
        assert_eq!(sharpen_effect.parameters.intensity, 1.5);

        let noise_effect = MaskEffectProcessor::create_noise_effect(
            "noise".to_string(),
            "Noise".to_string(),
            0.3
        );

        assert_eq!(noise_effect.effect_type, MaskEffectType::Noise);
        assert_eq!(noise_effect.parameters.intensity, 0.3);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = MaskEffectProcessor::new()
            .with_cache(true, (64, 64));

        let effect = MaskEffect::new("effect1".to_string(), "Test".to_string(), MaskEffectType::Blur);
        processor.add_effect(effect);


        let result1 = processor.process_value(0.8, 10.0, 15.0);
        assert_eq!(processor.get_cache_size(), 1);


        let result2 = processor.process_value(0.8, 10.0, 15.0);
        assert_eq!(result1, result2);


        processor.clear_cache();
        assert_eq!(processor.get_cache_size(), 0);
    }

    #[test]
    fn test_region_processing() {
        let mut processor = MaskEffectProcessor::new();
        let effect = MaskEffect::new("effect1".to_string(), "Test".to_string(), MaskEffectType::Blur);
        processor.add_effect(effect);

        let values = vec![0.5, 0.7, 0.3, 0.9];
        let bounds = (0.0, 0.0, 100.0, 100.0);
        let resolution = (2, 2);

        let result = processor.process_region(&values, bounds, resolution);
        assert_eq!(result.len(), 4);


        for (i, &input_value) in values.iter().enumerate() {
            assert!(result[i] != input_value);
        }
    }

    #[test]
    fn test_effect_validation() {
        let valid_effect = MaskEffect::new("effect1".to_string(), "Test".to_string(), MaskEffectType::Blur);
        assert!(valid_effect.validate().is_ok());

        let mut invalid_effect = valid_effect.clone();
        invalid_effect.id = "".to_string();
        assert!(invalid_effect.validate().is_err());

        invalid_effect.id = "test".to_string();
        invalid_effect.opacity = 2.0;
        assert!(invalid_effect.validate().is_err());

        invalid_effect.opacity = 0.5;
        invalid_effect.parameters.intensity = 3.0;
        assert!(invalid_effect.validate().is_err());
    }

    #[test]
    fn test_processor_validation() {
        let mut processor = MaskEffectProcessor::new();


        assert!(processor.validate().is_ok());


        let effect = MaskEffect::new("effect1".to_string(), "Test".to_string(), MaskEffectType::Blur);
        processor.add_effect(effect);
        assert!(processor.validate().is_ok());


        let effect2 = MaskEffect::new("effect1".to_string(), "Test2".to_string(), MaskEffectType::Sharpen);
        processor.add_effect(effect2);
        assert!(processor.validate().is_err());
    }

    #[test]
    fn test_disabled_effects() {
        let effect = MaskEffect::new("effect1".to_string(), "Test".to_string(), MaskEffectType::Blur);
        let mut disabled_effect = effect.clone();
        disabled_effect.set_enabled(false);

        let result = disabled_effect.apply_to_value(0.8, 10.0, 15.0);
        assert_eq!(result, 0.8);
    }

    #[test]
    fn test_effect_types() {
        let effect_types = vec![
            MaskEffectType::Blur,
            MaskEffectType::Sharpen,
            MaskEffectType::Emboss,
            MaskEffectType::EdgeDetect,
            MaskEffectType::Noise,
            MaskEffectType::Displace,
            MaskEffectType::Turbulence,
            MaskEffectType::FractalNoise,
            MaskEffectType::MotionBlur,
            MaskEffectType::GaussianBlur,
            MaskEffectType::BoxBlur,
            MaskEffectType::MedianFilter,
        ];

        for effect_type in effect_types {
            let effect = MaskEffect::new("test".to_string(), "Test".to_string(), effect_type);
            assert_eq!(effect.effect_type, effect_type);
        }
    }

    #[test]
    fn test_quality_levels() {
        let quality_levels = vec![
            EffectQuality::Low,
            EffectQuality::Medium,
            EffectQuality::High,
            EffectQuality::Ultra,
        ];

        for quality in quality_levels {
            let params = MaskEffectParameters::new().with_quality(quality);
            assert_eq!(params.quality, quality);
        }
    }

    #[test]
    fn test_custom_effects() {
        let params = MaskEffectParameters::new()
            .with_custom_param("multiplier".to_string(), 2.0);

        let effect = MaskEffect::new("custom".to_string(), "Custom".to_string(), MaskEffectType::Custom("test".to_string()))
            .with_parameters(params);

        let result = effect.apply_to_value(0.5, 10.0, 15.0);
        assert_eq!(result, 1.0);
    }
}
