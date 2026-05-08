use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskType {
    Shape,
    Gradient,
    Texture,
    Noise,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskBlendMode {
    Normal,
    Add,
    Subtract,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskChannel {
    Alpha,
    Red,
    Green,
    Blue,
    Luminance,
    Hue,
    Saturation,
    Value,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskInvertMode {
    None,
    Invert,
    InvertAlpha,
    InvertLuma,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskFeatherQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskProperties {
    pub id: String,
    pub name: String,
    pub mask_type: MaskType,
    pub enabled: bool,
    pub opacity: f64,
    pub blend_mode: MaskBlendMode,
    pub channel: MaskChannel,
    pub invert_mode: MaskInvertMode,
    pub feather: f64,
    pub feather_quality: MaskFeatherQuality,
    pub expand: f64,
    pub choke: f64,
    pub position: (f64, f64),
    pub rotation: f64,
    pub scale: (f64, f64),
    pub anchor_point: (f64, f64),
    pub created_at: String,
    pub modified_at: String,
}

impl Default for MaskProperties {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "Untitled Mask".to_string(),
            mask_type: MaskType::Shape,
            enabled: true,
            opacity: 1.0,
            blend_mode: MaskBlendMode::Normal,
            channel: MaskChannel::Alpha,
            invert_mode: MaskInvertMode::None,
            feather: 0.0,
            feather_quality: MaskFeatherQuality::Medium,
            expand: 0.0,
            choke: 0.0,
            position: (0.0, 0.0),
            rotation: 0.0,
            scale: (1.0, 1.0),
            anchor_point: (0.0, 0.0),
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl MaskProperties {
    pub fn new(id: String, name: String, mask_type: MaskType) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            name,
            mask_type,
            created_at: now.clone(),
            modified_at: now,
            ..Default::default()
        }
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_blend_mode(mut self, blend_mode: MaskBlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    pub fn with_channel(mut self, channel: MaskChannel) -> Self {
        self.channel = channel;
        self
    }

    pub fn with_feather(mut self, feather: f64) -> Self {
        self.feather = feather.max(0.0);
        self
    }

    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.position = (x, y);
        self
    }

    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, x: f64, y: f64) -> Self {
        self.scale = (x, y);
        self
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.update_modified_time();
    }

    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.clamp(0.0, 1.0);
        self.update_modified_time();
    }

    pub fn set_blend_mode(&mut self, blend_mode: MaskBlendMode) {
        self.blend_mode = blend_mode;
        self.update_modified_time();
    }

    pub fn set_channel(&mut self, channel: MaskChannel) {
        self.channel = channel;
        self.update_modified_time();
    }

    pub fn set_invert_mode(&mut self, invert_mode: MaskInvertMode) {
        self.invert_mode = invert_mode;
        self.update_modified_time();
    }

    pub fn set_feather(&mut self, feather: f64) {
        self.feather = feather.max(0.0);
        self.update_modified_time();
    }

    pub fn set_feather_quality(&mut self, quality: MaskFeatherQuality) {
        self.feather_quality = quality;
        self.update_modified_time();
    }

    pub fn set_expand(&mut self, expand: f64) {
        self.expand = expand;
        self.update_modified_time();
    }

    pub fn set_choke(&mut self, choke: f64) {
        self.choke = choke;
        self.update_modified_time();
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        self.position = (x, y);
        self.update_modified_time();
    }

    pub fn set_rotation(&mut self, rotation: f64) {
        self.rotation = rotation;
        self.update_modified_time();
    }

    pub fn set_scale(&mut self, x: f64, y: f64) {
        self.scale = (x, y);
        self.update_modified_time();
    }

    pub fn set_anchor_point(&mut self, x: f64, y: f64) {
        self.anchor_point = (x, y);
        self.update_modified_time();
    }

    fn update_modified_time(&mut self) {
        self.modified_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Mask ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Mask name cannot be empty".to_string());
        }

        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err("Opacity must be between 0.0 and 1.0".to_string());
        }

        if self.feather < 0.0 {
            return Err("Feather must be non-negative".to_string());
        }

        Ok(())
    }

    pub fn get_bounds(&self) -> Option<(f64, f64, f64, f64)> {

        Some((
            self.position.0 - 100.0,
            self.position.1 - 100.0,
            self.position.0 + 100.0,
            self.position.1 + 100.0,
        ))
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        if let Some((min_x, min_y, max_x, max_y)) = self.get_bounds() {
            x >= min_x && x <= max_x && y >= min_y && y <= max_y
        } else {
            false
        }
    }

    pub fn clone_with_id(&self, new_id: String) -> Self {
        let mut clone = self.clone();
        clone.id = new_id;
        clone.name = format!("{} Copy", self.name);
        clone.created_at = chrono::Utc::now().to_rfc3339();
        clone.modified_at = chrono::Utc::now().to_rfc3339();
        clone
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskEvaluation {
    pub mask_id: String,
    pub value: f64,
    pub position: (f64, f64),
    pub gradient_value: Option<f64>,
    pub edge_distance: Option<f64>,
    pub is_inside: bool,
}

impl MaskEvaluation {
    pub fn new(mask_id: String, value: f64, position: (f64, f64)) -> Self {
        Self {
            mask_id,
            value,
            position,
            gradient_value: None,
            edge_distance: None,
            is_inside: value > 0.5,
        }
    }

    pub fn with_gradient(mut self, gradient_value: f64) -> Self {
        self.gradient_value = Some(gradient_value);
        self
    }

    pub fn with_edge_distance(mut self, edge_distance: f64) -> Self {
        self.edge_distance = Some(edge_distance);
        self
    }

    pub fn apply_opacity(mut self, opacity: f64) -> Self {
        self.value *= opacity.clamp(0.0, 1.0);
        self
    }

    pub fn apply_invert(mut self, invert_mode: MaskInvertMode) -> Self {
        match invert_mode {
            MaskInvertMode::Invert => {
                self.value = 1.0 - self.value;
            }
            MaskInvertMode::InvertAlpha => {
                self.value = 1.0 - self.value;
            }
            MaskInvertMode::InvertLuma => {


                self.value = 1.0 - self.value;
            }
            MaskInvertMode::None => {}
        }
        self.is_inside = self.value > 0.5;
        self
    }
}


#[derive(Debug, Clone)]
pub struct MaskCache {
    pub cache_enabled: bool,
    pub cache_resolution: (u32, u32),
    pub cache_data: HashMap<String, Vec<f64>>,
    pub cache_timestamp: String,
}

impl Default for MaskCache {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_resolution: (512, 512),
            cache_data: HashMap::new(),
            cache_timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl MaskCache {
    pub fn new(resolution: (u32, u32)) -> Self {
        Self {
            cache_resolution: resolution,
            ..Default::default()
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.clear();
        }
        self.cache_enabled = enabled;
    }

    pub fn set_resolution(&mut self, resolution: (u32, u32)) {
        self.cache_resolution = resolution;
        self.clear();
    }

    pub fn get_cached_data(&self, mask_id: &str) -> Option<&Vec<f64>> {
        self.cache_data.get(mask_id)
    }

    pub fn cache_data(&mut self, mask_id: String, data: Vec<f64>) {
        if self.cache_enabled {
            self.cache_data.insert(mask_id, data);
            self.cache_timestamp = chrono::Utc::now().to_rfc3339();
        }
    }

    pub fn clear(&mut self) {
        self.cache_data.clear();
        self.cache_timestamp = chrono::Utc::now().to_rfc3339();
    }

    pub fn invalidate(&mut self, mask_id: &str) {
        self.cache_data.remove(mask_id);
        self.cache_timestamp = chrono::Utc::now().to_rfc3339();
    }

    pub fn get_cache_size(&self) -> usize {
        self.cache_data.len()
    }

    pub fn get_memory_usage(&self) -> usize {
        self.cache_data.values()
            .map(|data| data.len() * std::mem::size_of::<f64>())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_properties_creation() {
        let mask = MaskProperties::new(
            "mask1".to_string(),
            "Test Mask".to_string(),
            MaskType::Shape,
        );

        assert_eq!(mask.id, "mask1");
        assert_eq!(mask.name, "Test Mask");
        assert_eq!(mask.mask_type, MaskType::Shape);
        assert!(mask.enabled);
        assert_eq!(mask.opacity, 1.0);
        assert_eq!(mask.blend_mode, MaskBlendMode::Normal);
    }

    #[test]
    fn test_mask_properties_builder() {
        let mask = MaskProperties::new("mask1".to_string(), "Test".to_string(), MaskType::Gradient)
            .with_opacity(0.8)
            .with_blend_mode(MaskBlendMode::Multiply)
            .with_channel(MaskChannel::Luminance)
            .with_feather(5.0)
            .with_position(100.0, 200.0)
            .with_rotation(45.0)
            .with_scale(1.5, 0.8);

        assert_eq!(mask.opacity, 0.8);
        assert_eq!(mask.blend_mode, MaskBlendMode::Multiply);
        assert_eq!(mask.channel, MaskChannel::Luminance);
        assert_eq!(mask.feather, 5.0);
        assert_eq!(mask.position, (100.0, 200.0));
        assert_eq!(mask.rotation, 45.0);
        assert_eq!(mask.scale, (1.5, 0.8));
    }

    #[test]
    fn test_mask_properties_setters() {
        let mut mask = MaskProperties::default();

        mask.set_opacity(0.5);
        assert_eq!(mask.opacity, 0.5);

        mask.set_blend_mode(MaskBlendMode::Screen);
        assert_eq!(mask.blend_mode, MaskBlendMode::Screen);

        mask.set_feather(10.0);
        assert_eq!(mask.feather, 10.0);

        mask.set_enabled(false);
        assert!(!mask.enabled);
    }

    #[test]
    fn test_mask_properties_validation() {
        let valid_mask = MaskProperties::default();
        assert!(valid_mask.validate().is_ok());

        let mut invalid_mask = MaskProperties::default();
        invalid_mask.id = "".to_string();
        assert!(invalid_mask.validate().is_err());

        invalid_mask.id = "test".to_string();
        invalid_mask.opacity = 2.0;
        assert!(invalid_mask.validate().is_err());

        invalid_mask.opacity = 0.5;
        invalid_mask.feather = -1.0;
        assert!(invalid_mask.validate().is_err());
    }

    #[test]
    fn test_mask_evaluation() {
        let eval = MaskEvaluation::new("mask1".to_string(), 0.75, (100.0, 200.0));

        assert_eq!(eval.mask_id, "mask1");
        assert_eq!(eval.value, 0.75);
        assert_eq!(eval.position, (100.0, 200.0));
        assert!(eval.is_inside);
        assert!(eval.gradient_value.is_none());
        assert!(eval.edge_distance.is_none());
    }

    #[test]
    fn test_mask_evaluation_modifiers() {
        let eval = MaskEvaluation::new("mask1".to_string(), 0.75, (100.0, 200.0))
            .with_gradient(0.5)
            .with_edge_distance(10.0)
            .apply_opacity(0.8)
            .apply_invert(MaskInvertMode::Invert);

        assert_eq!(eval.value, 0.2);
        assert_eq!(eval.gradient_value, Some(0.5));
        assert_eq!(eval.edge_distance, Some(10.0));
        assert!(!eval.is_inside);
    }

    #[test]
    fn test_mask_cache() {
        let mut cache = MaskCache::new((256, 256));

        assert!(cache.cache_enabled);
        assert_eq!(cache.cache_resolution, (256, 256));
        assert_eq!(cache.get_cache_size(), 0);

        let test_data = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        cache.cache_data("mask1".to_string(), test_data.clone());

        assert_eq!(cache.get_cache_size(), 1);
        assert_eq!(cache.get_cached_data("mask1"), Some(&test_data));
        assert_eq!(cache.get_cached_data("mask2"), None);

        cache.invalidate("mask1");
        assert_eq!(cache.get_cache_size(), 0);

        cache.set_enabled(false);
        cache.cache_data("mask1".to_string(), test_data);
        assert_eq!(cache.get_cache_size(), 0);
    }

    #[test]
    fn test_mask_properties_clone() {
        let original = MaskProperties::new("original".to_string(), "Original".to_string(), MaskType::Shape)
            .with_opacity(0.7)
            .with_position(50.0, 100.0);

        let cloned = original.clone_with_id("cloned".to_string());

        assert_eq!(cloned.id, "cloned");
        assert_eq!(cloned.name, "Original Copy");
        assert_eq!(cloned.opacity, 0.7);
        assert_eq!(cloned.position, (50.0, 100.0));
        assert_ne!(cloned.created_at, original.created_at);
    }

    #[test]
    fn test_mask_blend_modes() {
        let modes = vec![
            MaskBlendMode::Normal,
            MaskBlendMode::Add,
            MaskBlendMode::Subtract,
            MaskBlendMode::Multiply,
            MaskBlendMode::Screen,
            MaskBlendMode::Overlay,
            MaskBlendMode::SoftLight,
            MaskBlendMode::HardLight,
            MaskBlendMode::Difference,
            MaskBlendMode::Exclusion,
            MaskBlendMode::In,
            MaskBlendMode::Out,
        ];

        for mode in modes {
            let mask = MaskProperties::default().with_blend_mode(mode);
            assert_eq!(mask.blend_mode, mode);
        }
    }

    #[test]
    fn test_mask_channels() {
        let channels = vec![
            MaskChannel::Alpha,
            MaskChannel::Red,
            MaskChannel::Green,
            MaskChannel::Blue,
            MaskChannel::Luminance,
            MaskChannel::Hue,
            MaskChannel::Saturation,
            MaskChannel::Value,
        ];

        for channel in channels {
            let mask = MaskProperties::default().with_channel(channel);
            assert_eq!(mask.channel, channel);
        }
    }
}
