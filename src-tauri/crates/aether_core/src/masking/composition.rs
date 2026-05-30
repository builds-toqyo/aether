use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;
use super::shapes::*;
use super::gradients::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskCompositionMode {
    Add,
    Subtract,
    Intersect,
    Difference,
    Union,
    Exclude,
    Overlay,
    Mask,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskCompositionOrder {
    TopToBottom,
    BottomToTop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskLayer {
    pub id: String,
    pub name: String,
    pub mask: MaskType,
    pub enabled: bool,
    pub opacity: f64,
    pub blend_mode: MaskBlendMode,
    pub invert: bool,
    pub composition_mode: MaskCompositionMode,
    pub z_index: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskCompositionResult {
    pub value: f64,
    pub position: (f64, f64),
    pub layers_used: Vec<String>,
    pub composition_mode: MaskCompositionMode,
    pub final_blend_mode: MaskBlendMode,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskCompositor {
    pub layers: Vec<MaskLayer>,
    pub composition_order: MaskCompositionOrder,
    pub global_opacity: f64,
    pub global_invert: bool,
    pub cache_enabled: bool,
    pub cache_resolution: (u32, u32),
    pub cache_data: HashMap<String, Vec<f64>>,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaskType {
    Shape(ShapeMask),
    Gradient(GradientMask),
}

impl MaskLayer {
    pub fn new(id: String, name: String, mask: MaskType) -> Self {
        Self {
            id,
            name,
            mask,
            enabled: true,
            opacity: 1.0,
            blend_mode: MaskBlendMode::Normal,
            invert: false,
            composition_mode: MaskCompositionMode::Add,
            z_index: 0,
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

    pub fn with_invert(mut self, invert: bool) -> Self {
        self.invert = invert;
        self
    }

    pub fn with_composition_mode(mut self, mode: MaskCompositionMode) -> Self {
        self.composition_mode = mode;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn evaluate_at(&self, x: f64, y: f64) -> Option<MaskEvaluation> {
        if !self.enabled {
            return None;
        }

        let evaluation = match &self.mask {
            MaskType::Shape(shape_mask) => shape_mask.evaluate_at(x, y),
            MaskType::Gradient(gradient_mask) => gradient_mask.evaluate_at(x, y),
        };

        let mut result = evaluation
            .apply_opacity(self.opacity)
            .apply_invert(if self.invert { MaskInvertMode::Invert } else { MaskInvertMode::None });


        result.value = self.apply_blend_mode(result.value);

        Some(result)
    }

    fn apply_blend_mode(&self, value: f64) -> f64 {
        match self.blend_mode {
            MaskBlendMode::Normal => value,
            MaskBlendMode::Add => (value + 1.0).min(1.0),
            MaskBlendMode::Subtract => (value - 1.0).max(0.0),
            MaskBlendMode::Multiply => value,
            MaskBlendMode::Screen => 1.0 - (1.0 - value) * (1.0 - value),
            MaskBlendMode::Overlay => {
                if value < 0.5 {
                    2.0 * value * value
                } else {
                    1.0 - 2.0 * (1.0 - value) * (1.0 - value)
                }
            }
            MaskBlendMode::SoftLight => {
                if value < 0.5 {
                    2.0 * value * (1.0 - (1.0 - value) * (1.0 - value))
                } else {
                    1.0 - 2.0 * (1.0 - value) * (1.0 - value * (1.0 - value))
                }
            }
            MaskBlendMode::HardLight => {
                if value < 0.5 {
                    2.0 * value * value
                } else {
                    1.0 - 2.0 * (1.0 - value) * (1.0 - value)
                }
            }
            MaskBlendMode::Difference => (value - 1.0).abs(),
            MaskBlendMode::Exclusion => value + (1.0 - value) - 2.0 * value * (1.0 - value),
            MaskBlendMode::In => value * value,
            MaskBlendMode::Out => 1.0 - (1.0 - value) * (1.0 - value),
        }
    }

    pub fn get_bounds(&self) -> Option<(f64, f64, f64, f64)> {
        match &self.mask {
            MaskType::Shape(shape_mask) => shape_mask.get_bounds().map(|b| (b.min_x, b.min_y, b.max_x, b.max_y)),
            MaskType::Gradient(gradient_mask) => gradient_mask.get_bounds(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Mask layer ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Mask layer name cannot be empty".to_string());
        }

        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err("Mask layer opacity must be between 0.0 and 1.0".to_string());
        }

        match &self.mask {
            MaskType::Shape(shape_mask) => shape_mask.validate()?,
            MaskType::Gradient(gradient_mask) => gradient_mask.validate()?,
        }

        Ok(())
    }
}

impl MaskCompositor {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            composition_order: MaskCompositionOrder::TopToBottom,
            global_opacity: 1.0,
            global_invert: false,
            cache_enabled: true,
            cache_resolution: (512, 512),
            cache_data: HashMap::new(),
        }
    }

    pub fn with_composition_order(mut self, order: MaskCompositionOrder) -> Self {
        self.composition_order = order;
        self
    }

    pub fn with_global_opacity(mut self, opacity: f64) -> Self {
        self.global_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_global_invert(mut self, invert: bool) -> Self {
        self.global_invert = invert;
        self
    }

    pub fn with_cache(mut self, enabled: bool, resolution: (u32, u32)) -> Self {
        self.cache_enabled = enabled;
        self.cache_resolution = resolution;
        self
    }

    pub fn add_layer(&mut self, layer: MaskLayer) {
        self.layers.push(layer);
        self.sort_layers();
        self.clear_cache();
    }

    pub fn remove_layer(&mut self, layer_id: &str) -> bool {
        let initial_len = self.layers.len();
        self.layers.retain(|layer| layer.id != layer_id);
        let removed = self.layers.len() < initial_len;
        if removed {
            self.clear_cache();
        }
        removed
    }

    pub fn get_layer(&self, layer_id: &str) -> Option<&MaskLayer> {
        self.layers.iter().find(|layer| layer.id == layer_id)
    }

    pub fn get_layer_mut(&mut self, layer_id: &str) -> Option<&mut MaskLayer> {
        self.layers.iter_mut().find(|layer| layer.id == layer_id)
    }

    fn sort_layers(&mut self) {
        match self.composition_order {
            MaskCompositionOrder::TopToBottom => {
                self.layers.sort_by_key(|layer| -layer.z_index);
            }
            MaskCompositionOrder::BottomToTop => {
                self.layers.sort_by_key(|layer| layer.z_index);
            }
        }
    }

    pub fn evaluate_at(&mut self, x: f64, y: f64) -> MaskCompositionResult {
        let cache_key = format!("{:.2}_{:.2}", x, y);

        if self.cache_enabled {
            if let Some(cached_value) = self.cache_data.get(&cache_key) {
                if !cached_value.is_empty() {
                    return MaskCompositionResult {
                        value: cached_value[0],
                        position: (x, y),
                        layers_used: vec![],
                        composition_mode: MaskCompositionMode::Add,
                        final_blend_mode: MaskBlendMode::Normal,
                    };
                }
            }
        }

        let mut final_value = 0.0;
        let mut layers_used = Vec::new();
        let mut current_composition_mode = MaskCompositionMode::Add;

        for layer in &self.layers {
            if let Some(evaluation) = layer.evaluate_at(x, y) {
                if evaluation.value > 0.0 {
                    layers_used.push(layer.id.clone());

                    if layers_used.len() == 1 {

                        final_value = evaluation.value;
                        current_composition_mode = layer.composition_mode;
                    } else {

                        final_value = self.compose_values(final_value, evaluation.value, layer.composition_mode);
                    }
                }
            }
        }


        final_value *= self.global_opacity;


        if self.global_invert {
            final_value = 1.0 - final_value;
        }

        let result = MaskCompositionResult {
            value: final_value,
            position: (x, y),
            layers_used,
            composition_mode: current_composition_mode,
            final_blend_mode: if let Some(last_layer) = self.layers.last() {
                last_layer.blend_mode
            } else {
                MaskBlendMode::Normal
            },
        };


        if self.cache_enabled {
            self.cache_data.insert(cache_key, vec![final_value]);
        }

        result
    }

    fn compose_values(&self, a: f64, b: f64, mode: MaskCompositionMode) -> f64 {
        match mode {
            MaskCompositionMode::Add => (a + b).min(1.0),
            MaskCompositionMode::Subtract => (a - b).max(0.0),
            MaskCompositionMode::Intersect => a * b,
            MaskCompositionMode::Difference => (a - b).abs(),
            MaskCompositionMode::Union => 1.0 - (1.0 - a) * (1.0 - b),
            MaskCompositionMode::Exclude => a + b - 2.0 * a * b,
            MaskCompositionMode::Overlay => {
                if a < 0.5 {
                    2.0 * a * b
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                }
            }
            MaskCompositionMode::Mask => {
                if a > 0.5 {
                    b
                } else {
                    0.0
                }
            }
        }
    }

    pub fn evaluate_region(&mut self, bounds: (f64, f64, f64, f64), resolution: (u32, u32)) -> Vec<f64> {
        let (min_x, min_y, max_x, max_y) = bounds;
        let (width, height) = resolution;
        let mut result = Vec::with_capacity((width * height) as usize);

        let x_step = (max_x - min_x) / width as f64;
        let y_step = (max_y - min_y) / height as f64;

        for y in 0..height {
            for x in 0..width {
                let px = min_x + x as f64 * x_step;
                let py = min_y + y as f64 * y_step;
                let evaluation = self.evaluate_at(px, py);
                result.push(evaluation.value);
            }
        }

        result
    }

    pub fn get_bounds(&self) -> Option<(f64, f64, f64, f64)> {
        if self.layers.is_empty() {
            return None;
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for layer in &self.layers {
            if let Some((lx, ly, lx2, ly2)) = layer.get_bounds() {
                min_x = min_x.min(lx);
                min_y = min_y.min(ly);
                max_x = max_x.max(lx2);
                max_y = max_y.max(ly2);
            }
        }

        if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
            Some((min_x, min_y, max_x, max_y))
        } else {
            None
        }
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

    pub fn create_simple_composition(
        shape_masks: Vec<ShapeMask>,
        gradient_masks: Vec<GradientMask>,
    ) -> Self {
        let mut compositor = Self::new();
        let shape_masks_len = shape_masks.len();

        for (i, shape_mask) in shape_masks.into_iter().enumerate() {
            let layer = MaskLayer::new(
                format!("shape_{}", i),
                format!("Shape Layer {}", i),
                MaskType::Shape(shape_mask),
            ).with_z_index(i as i32);
            compositor.add_layer(layer);
        }

        for (i, gradient_mask) in gradient_masks.into_iter().enumerate() {
            let layer = MaskLayer::new(
                format!("gradient_{}", i),
                format!("Gradient Layer {}", i),
                MaskType::Gradient(gradient_mask),
            ).with_z_index((shape_masks_len + i) as i32);
            compositor.add_layer(layer);
        }

        compositor
    }

    pub fn validate(&self) -> Result<(), String> {
        for (i, layer) in self.layers.iter().enumerate() {
            layer.validate().map_err(|e| format!("Layer {}: {}", i, e))?;
        }


        let mut ids = std::collections::HashSet::new();
        for layer in &self.layers {
            if !ids.insert(&layer.id) {
                return Err(format!("Duplicate mask layer ID: {}", layer.id));
            }
        }

        Ok(())
    }
}

impl Default for MaskCompositor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{EasingFunction, InterpolationMethod};

    #[test]
    fn test_mask_layer_creation() {
        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test Layer".to_string(), MaskType::Shape(shape_mask));

        assert_eq!(layer.id, "layer1");
        assert_eq!(layer.name, "Test Layer");
        assert!(layer.enabled);
        assert_eq!(layer.opacity, 1.0);
        assert_eq!(layer.blend_mode, MaskBlendMode::Normal);
        assert!(!layer.invert);
        assert_eq!(layer.composition_mode, MaskCompositionMode::Add);
        assert_eq!(layer.z_index, 0);
    }

    #[test]
    fn test_mask_layer_builder() {
        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test".to_string(), MaskType::Shape(shape_mask))
            .with_opacity(0.8)
            .with_blend_mode(MaskBlendMode::Multiply)
            .with_invert(true)
            .with_composition_mode(MaskCompositionMode::Intersect)
            .with_z_index(5);

        assert_eq!(layer.opacity, 0.8);
        assert_eq!(layer.blend_mode, MaskBlendMode::Multiply);
        assert!(layer.invert);
        assert_eq!(layer.composition_mode, MaskCompositionMode::Intersect);
        assert_eq!(layer.z_index, 5);
    }

    #[test]
    fn test_mask_compositor_creation() {
        let compositor = MaskCompositor::new();

        assert!(compositor.layers.is_empty());
        assert_eq!(compositor.composition_order, MaskCompositionOrder::TopToBottom);
        assert_eq!(compositor.global_opacity, 1.0);
        assert!(!compositor.global_invert);
        assert!(compositor.cache_enabled);
        assert_eq!(compositor.cache_resolution, (512, 512));
    }

    #[test]
    fn test_mask_compositor_builder() {
        let compositor = MaskCompositor::new()
            .with_composition_order(MaskCompositionOrder::BottomToTop)
            .with_global_opacity(0.7)
            .with_global_invert(true)
            .with_cache(false, (256, 256));

        assert_eq!(compositor.composition_order, MaskCompositionOrder::BottomToTop);
        assert_eq!(compositor.global_opacity, 0.7);
        assert!(compositor.global_invert);
        assert!(!compositor.cache_enabled);
        assert_eq!(compositor.cache_resolution, (256, 256));
    }

    #[test]
    fn test_add_mask_layer() {
        let mut compositor = MaskCompositor::new();
        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test".to_string(), MaskType::Shape(shape_mask));

        compositor.add_layer(layer);
        assert_eq!(compositor.layers.len(), 1);
        assert_eq!(compositor.layers[0].id, "layer1");
    }

    #[test]
    fn test_remove_mask_layer() {
        let mut compositor = MaskCompositor::new();
        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test".to_string(), MaskType::Shape(shape_mask));

        compositor.add_layer(layer);
        assert_eq!(compositor.layers.len(), 1);

        let removed = compositor.remove_layer("layer1");
        assert!(removed);
        assert_eq!(compositor.layers.len(), 0);

        let not_removed = compositor.remove_layer("nonexistent");
        assert!(!not_removed);
    }

    #[test]
    fn test_mask_composition() {
        let mut compositor = MaskCompositor::new();


        let rect1 = ShapeMask::create_rectangle_mask("rect1".to_string(), "Rect1".to_string(), -25.0, 0.0, 50.0, 50.0);
        let rect2 = ShapeMask::create_rectangle_mask("rect2".to_string(), "Rect2".to_string(), 0.0, 0.0, 50.0, 50.0);

        let layer1 = MaskLayer::new("layer1".to_string(), "Layer1".to_string(), MaskType::Shape(rect1))
            .with_z_index(1);
        let layer2 = MaskLayer::new("layer2".to_string(), "Layer2".to_string(), MaskType::Shape(rect2))
            .with_z_index(2);

        compositor.add_layer(layer1);
        compositor.add_layer(layer2);


        let result = compositor.evaluate_at(0.0, 0.0);
        assert!(result.value > 0.0);
        assert_eq!(result.layers_used.len(), 2);
        assert!(result.layers_used.contains(&"layer1".to_string()));
        assert!(result.layers_used.contains(&"layer2".to_string()));
    }

    #[test]
    fn test_composition_modes() {
        let compositor = MaskCompositor::new();


        assert_eq!(compositor.compose_values(0.5, 0.5, MaskCompositionMode::Add), 1.0);
        assert_eq!(compositor.compose_values(0.7, 0.3, MaskCompositionMode::Subtract), 0.4);
        assert_eq!(compositor.compose_values(0.5, 0.5, MaskCompositionMode::Intersect), 0.25);
        assert_eq!(compositor.compose_values(0.7, 0.3, MaskCompositionMode::Difference), 0.4);
        assert_eq!(compositor.compose_values(0.5, 0.5, MaskCompositionMode::Union), 0.75);
        assert_eq!(compositor.compose_values(0.5, 0.5, MaskCompositionMode::Exclude), 0.5);
    }

    #[test]
    fn test_blend_modes() {
        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test".to_string(), MaskType::Shape(shape_mask));


        let test_values = vec![
            (MaskBlendMode::Normal, 0.5),
            (MaskBlendMode::Add, 1.0),
            (MaskBlendMode::Subtract, 0.0),
            (MaskBlendMode::Multiply, 0.5),
            (MaskBlendMode::Screen, 0.75),
            (MaskBlendMode::Difference, 0.5),
        ];

        for (blend_mode, expected) in test_values {
            let test_layer = layer.clone().with_blend_mode(blend_mode);
            let result = test_layer.apply_blend_mode(0.5);
            assert!((result - expected).abs() < 0.001, "Failed for blend mode: {:?}", blend_mode);
        }
    }

    #[test]
    fn test_global_opacity_and_invert() {
        let mut compositor = MaskCompositor::new()
            .with_global_opacity(0.5)
            .with_global_invert(true);

        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test".to_string(), MaskType::Shape(shape_mask));
        compositor.add_layer(layer);

        let result = compositor.evaluate_at(0.0, 0.0);

        assert!((result.value - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_cache_functionality() {
        let mut compositor = MaskCompositor::new()
            .with_cache(true, (64, 64));

        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test".to_string(), MaskType::Shape(shape_mask));
        compositor.add_layer(layer);


        let result1 = compositor.evaluate_at(10.0, 10.0);
        assert_eq!(compositor.get_cache_size(), 1);


        let result2 = compositor.evaluate_at(10.0, 10.0);
        assert_eq!(result1.value, result2.value);


        compositor.clear_cache();
        assert_eq!(compositor.get_cache_size(), 0);
    }

    #[test]
    fn test_evaluate_region() {
        let mut compositor = MaskCompositor::new();
        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test".to_string(), MaskType::Shape(shape_mask));
        compositor.add_layer(layer);

        let result = compositor.evaluate_region((-50.0, -25.0, 50.0, 25.0), (10, 10));
        assert_eq!(result.len(), 100);


        let inside_count = result.iter().filter(|&&value| value > 0.5).count();
        assert!(inside_count > 0);
    }

    #[test]
    fn test_composition_bounds() {
        let mut compositor = MaskCompositor::new();


        assert!(compositor.get_bounds().is_none());


        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test".to_string(), MaskType::Shape(shape_mask));
        compositor.add_layer(layer);

        let bounds = compositor.get_bounds();
        assert!(bounds.is_some());
        let (min_x, min_y, max_x, max_y) = bounds.unwrap();
        assert_eq!(min_x, -50.0);
        assert_eq!(min_y, -25.0);
        assert_eq!(max_x, 50.0);
        assert_eq!(max_y, 25.0);
    }

    #[test]
    fn test_simple_composition_creation() {
        let shape_masks = vec![
            ShapeMask::create_rectangle_mask("rect1".to_string(), "Rect1".to_string(), 0.0, 0.0, 100.0, 50.0),
        ];
        let gradient_masks = vec![
            GradientMask::create_linear_gradient("grad1".to_string(), "Grad1".to_string(), (-50.0, 0.0), (50.0, 0.0)),
        ];

        let compositor = MaskCompositor::create_simple_composition(shape_masks, gradient_masks);
        assert_eq!(compositor.layers.len(), 2);
        assert_eq!(compositor.layers[0].z_index, 0);
        assert_eq!(compositor.layers[1].z_index, 1);
    }

    #[test]
    fn test_validation() {
        let mut compositor = MaskCompositor::new();


        assert!(compositor.validate().is_ok());


        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);
        let layer = MaskLayer::new("layer1".to_string(), "Test".to_string(), MaskType::Shape(shape_mask));
        compositor.add_layer(layer);
        assert!(compositor.validate().is_ok());


        let layer2 = MaskLayer::new("layer1".to_string(), "Test2".to_string(), MaskType::Shape(shape_mask.clone()));
        compositor.add_layer(layer2);
        assert!(compositor.validate().is_err());
    }

    #[test]
    fn test_layer_sorting() {
        let mut compositor = MaskCompositor::new()
            .with_composition_order(MaskCompositionOrder::TopToBottom);

        let shape_mask = ShapeMask::create_rectangle_mask("rect".to_string(), "Rect".to_string(), 0.0, 0.0, 100.0, 50.0);


        let layer1 = MaskLayer::new("layer1".to_string(), "Layer1".to_string(), MaskType::Shape(shape_mask.clone()))
            .with_z_index(3);
        let layer2 = MaskLayer::new("layer2".to_string(), "Layer2".to_string(), MaskType::Shape(shape_mask.clone()))
            .with_z_index(1);
        let layer3 = MaskLayer::new("layer3".to_string(), "Layer3".to_string(), MaskType::Shape(shape_mask))
            .with_z_index(2);

        compositor.add_layer(layer1);
        compositor.add_layer(layer2);
        compositor.add_layer(layer3);


        assert_eq!(compositor.layers[0].z_index, 3);
        assert_eq!(compositor.layers[1].z_index, 2);
        assert_eq!(compositor.layers[2].z_index, 1);
    }
}
