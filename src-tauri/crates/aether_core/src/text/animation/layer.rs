use serde::{Deserialize, Serialize};
use super::types::*;
use super::typography::*;
use super::path::*;
use super::animator::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLayer {
    pub id: String,
    pub name: String,
    pub text: String,
    pub position: (f64, f64),
    pub typography: TypographyControls,
    pub path: Option<TextPath>,
    pub animator: TextAnimator,
    pub visible: bool,
    pub locked: bool,
    pub blend_mode: BlendMode,
    pub opacity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterTransform {
    pub position: (f64, f64),
    pub rotation: f64,
    pub scale: (f64, f64),
    pub opacity: f64,
    pub color: (f64, f64, f64, f64),
    pub font_size: f64,
    pub baseline_shift: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    ColorDodge,
    ColorBurn,
    Darken,
    Lighten,
    Difference,
    Exclusion,
}

impl TextLayer {
    pub fn new(id: String, name: String, text: String) -> Self {
        Self {
            id,
            name,
            text,
            position: (0.0, 0.0),
            typography: TypographyControls::default(),
            path: None,
            animator: TextAnimator::new(),
            visible: true,
            locked: false,
            blend_mode: BlendMode::Normal,
            opacity: 1.0,
        }
    }

    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.position = (x, y);
        self
    }

    pub fn with_typography(mut self, typography: TypographyControls) -> Self {
        self.typography = typography;
        self
    }

    pub fn with_path(mut self, path: TextPath) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.animator.reset_all();
    }

    pub fn set_typography(&mut self, typography: TypographyControls) {
        self.typography = typography;
    }

    pub fn set_path(&mut self, path: Option<TextPath>) {
        self.path = path;
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        self.position = (x, y);
    }

    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.blend_mode = blend_mode;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }

    pub fn get_character_count(&self) -> usize {
        self.text.chars().count()
    }

    pub fn get_word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    pub fn get_line_count(&self) -> usize {
        self.text.lines().count()
    }

    pub fn get_character_position(&self, char_index: usize) -> Option<(f64, f64)> {
        if let Some(ref path) = self.path {
            let char_distance = char_index as f64 * self.typography.font_size * 0.6;
            path.get_point_at_distance(char_distance)
        } else {
            let char_width = self.typography.font_size * 0.6;
            Some((
                self.position.0 + (char_index as f64 * char_width),
                self.position.1
            ))
        }
    }

    pub fn get_character_transform(&mut self, char_index: usize, _time: f64) -> CharacterTransform {
        let base_position = self.get_character_position(char_index).unwrap_or(self.position);
        let mut transform = CharacterTransform {
            position: base_position,
            rotation: 0.0,
            scale: (1.0, 1.0),
            opacity: self.opacity,
            color: self.typography.color,
            font_size: self.typography.font_size,
            baseline_shift: self.typography.baseline_shift,
        };

        let character_values = self.animator.get_character_values(char_index);

        for (animation_type, value) in character_values {
            match animation_type {
                AnimationType::Position => {
                    if let AnimationValue::Vector2(x, y) = value {
                        transform.position = (base_position.0 + x, base_position.1 + y);
                    }
                }
                AnimationType::Rotation => {
                    if let AnimationValue::Float(angle) = value {
                        transform.rotation = angle;
                    }
                }
                AnimationType::Scale => {
                    if let AnimationValue::Vector2(sx, sy) = value {
                        transform.scale = (sx, sy);
                    }
                }
                AnimationType::Opacity => {
                    if let AnimationValue::Float(opacity) = value {
                        transform.opacity = opacity * self.opacity;
                    }
                }
                AnimationType::Color => {
                    if let AnimationValue::Color(r, g, b, a) = value {
                        transform.color = (r, g, b, a);
                    }
                }
                AnimationType::FontSize => {
                    if let AnimationValue::Float(size) = value {
                        transform.font_size = size;
                    }
                }
                AnimationType::BaselineShift => {
                    if let AnimationValue::Float(shift) = value {
                        transform.baseline_shift = shift;
                    }
                }
                AnimationType::PathPosition => {
                    if let Some(ref path) = self.path {
                        if let AnimationValue::Float(distance) = value {
                            if let Some(pos) = path.get_point_at_distance(distance) {
                                transform.position = pos;
                            }
                        }
                    }
                }
                AnimationType::PathRotation => {
                    if let Some(ref path) = self.path {
                        if let AnimationValue::Float(distance) = value {
                            if let Some((tx, ty)) = path.get_tangent_at_distance(distance) {
                                transform.rotation = ty.atan2(tx);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        transform
    }

    pub fn get_bounds(&self) -> Option<(f64, f64, f64, f64)> {
        if self.text.is_empty() {
            return None;
        }

        let char_count = self.get_character_count();
        let char_width = self.typography.font_size * 0.6;
        let line_height = self.typography.font_size * self.typography.line_height;
        let line_count = self.get_line_count() as f64;

        let width = char_count as f64 * char_width;
        let height = line_count * line_height;

        Some((
            self.position.0,
            self.position.1,
            self.position.0 + width,
            self.position.1 + height,
        ))
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        if let Some((min_x, min_y, max_x, max_y)) = self.get_bounds() {
            x >= min_x && x <= max_x && y >= min_y && y <= max_y
        } else {
            false
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.animator.update(delta_time);
    }

    pub fn play_animations(&mut self) {
        self.animator.play_all();
    }

    pub fn pause_animations(&mut self) {
        self.animator.pause_all();
    }

    pub fn stop_animations(&mut self) {
        self.animator.stop_all();
    }

    pub fn reset_animations(&mut self) {
        self.animator.reset_all();
    }

    pub fn add_animation(&mut self, animation: CharacterAnimation) {
        self.animator.add_animation(animation);
    }

    pub fn remove_animation(&mut self, animation_id: &str) -> bool {
        self.animator.remove_animation(animation_id)
    }

    pub fn get_animation(&self, animation_id: &str) -> Option<&CharacterAnimation> {
        self.animator.get_animation(animation_id)
    }

    pub fn get_animation_mut(&mut self, animation_id: &str) -> Option<&mut CharacterAnimation> {
        self.animator.get_animation_mut(animation_id)
    }

    pub fn get_all_animations(&self) -> Vec<&CharacterAnimation> {
        self.animator.animations.iter().collect()
    }

    pub fn get_animation_count(&self) -> usize {
        self.animator.get_animation_count()
    }

    pub fn create_typography_animation(&mut self, animation_type: AnimationType, target_characters: Vec<usize>) -> String {
        let animation_id = format!("typography_{}_{}", animation_type.as_u8(), uuid::Uuid::new_v4().to_string()[..8].to_string());
        let animation = CharacterAnimation::new(
            animation_id.clone(),
            format!("Typography {:?}", animation_type),
            animation_type,
            target_characters,
        );

        self.add_animation(animation);
        animation_id
    }

    pub fn create_fade_animation(&mut self, target_characters: Vec<usize>, from_opacity: f64, to_opacity: f64, duration: f64) -> String {
        let animation_id = format!("fade_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        let mut animation = CharacterAnimation::new(
            animation_id.clone(),
            "Fade".to_string(),
            AnimationType::Opacity,
            target_characters,
        );

        animation.duration = duration;
        animation.add_keyframe(TextKeyframe {
            time: 0.0,
            value: AnimationValue::Float(from_opacity),
            easing: crate::animation::EasingFunction::Linear,
            interpolation: crate::animation::InterpolationMethod::Linear,
        });

        animation.add_keyframe(TextKeyframe {
            time: duration,
            value: AnimationValue::Float(to_opacity),
            easing: crate::animation::EasingFunction::Linear,
            interpolation: crate::animation::InterpolationMethod::Linear,
        });

        self.add_animation(animation);
        animation_id
    }

    pub fn create_slide_animation(&mut self, target_characters: Vec<usize>, from_offset: (f64, f64), to_offset: (f64, f64), duration: f64) -> String {
        let animation_id = format!("slide_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        let mut animation = CharacterAnimation::new(
            animation_id.clone(),
            "Slide".to_string(),
            AnimationType::Position,
            target_characters,
        );

        animation.duration = duration;
        animation.add_keyframe(TextKeyframe {
            time: 0.0,
            value: AnimationValue::Vector2(from_offset.0, from_offset.1),
            easing: crate::animation::EasingFunction::Linear,
            interpolation: crate::animation::InterpolationMethod::Linear,
        });

        animation.add_keyframe(TextKeyframe {
            time: duration,
            value: AnimationValue::Vector2(to_offset.0, to_offset.1),
            easing: crate::animation::EasingFunction::Linear,
            interpolation: crate::animation::InterpolationMethod::Linear,
        });

        self.add_animation(animation);
        animation_id
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Layer ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Layer name cannot be empty".to_string());
        }

        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err("Opacity must be between 0.0 and 1.0".to_string());
        }

        self.typography.validate()?;

        if let Some(ref path) = self.path {
            path.validate()?;
        }

        self.animator.validate()?;

        Ok(())
    }

    pub fn clone_without_animations(&self) -> Self {
        Self {
            id: format!("{}_clone", self.id),
            name: format!("{} Clone", self.name),
            text: self.text.clone(),
            position: self.position,
            typography: self.typography.clone(),
            path: self.path.clone(),
            animator: TextAnimator::new(),
            visible: self.visible,
            locked: false,
            blend_mode: self.blend_mode,
            opacity: self.opacity,
        }
    }
}

impl Default for TextLayer {
    fn default() -> Self {
        Self::new("default".to_string(), "Default Layer".to_string(), "Text".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{EasingFunction, InterpolationMethod};

    #[test]
    fn test_text_layer_creation() {
        let layer = TextLayer::new("layer1".to_string(), "Test Layer".to_string(), "Hello World".to_string());
        assert_eq!(layer.id, "layer1");
        assert_eq!(layer.name, "Test Layer");
        assert_eq!(layer.text, "Hello World");
        assert_eq!(layer.position, (0.0, 0.0));
        assert!(layer.visible);
        assert!(!layer.locked);
        assert_eq!(layer.blend_mode, BlendMode::Normal);
        assert_eq!(layer.opacity, 1.0);
    }

    #[test]
    fn test_text_layer_builder() {
        let typography = TypographyControls::default().with_font_size(24.0);
        let layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string())
            .with_position(100.0, 200.0)
            .with_typography(typography)
            .with_opacity(0.8)
            .with_blend_mode(BlendMode::Multiply);

        assert_eq!(layer.position, (100.0, 200.0));
        assert_eq!(layer.typography.font_size, 24.0);
        assert_eq!(layer.opacity, 0.8);
        assert_eq!(layer.blend_mode, BlendMode::Multiply);
    }

    #[test]
    fn test_text_layer_character_count() {
        let layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        assert_eq!(layer.get_character_count(), 5);

        let layer = TextLayer::new("layer2".to_string(), "Test".to_string(), "Hello World".to_string());
        assert_eq!(layer.get_character_count(), 11);
    }

    #[test]
    fn test_text_layer_word_count() {
        let layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello World".to_string());
        assert_eq!(layer.get_word_count(), 2);

        let layer = TextLayer::new("layer2".to_string(), "Test".to_string(), "One two three four".to_string());
        assert_eq!(layer.get_word_count(), 4);
    }

    #[test]
    fn test_text_layer_line_count() {
        let layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        assert_eq!(layer.get_line_count(), 1);

        let layer = TextLayer::new("layer2".to_string(), "Test".to_string(), "Line 1\nLine 2\nLine 3".to_string());
        assert_eq!(layer.get_line_count(), 3);
    }

    #[test]
    fn test_text_layer_character_position() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        layer.position = (100.0, 200.0);
        layer.typography.font_size = 20.0;

        let pos = layer.get_character_position(0);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap(), (100.0, 200.0));

        let pos = layer.get_character_position(1);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap(), (112.0, 200.0));
    }

    #[test]
    fn test_text_layer_with_path() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());

        let mut path = TextPath::new("path1".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(200.0, 0.0);

        layer.set_path(Some(path));

        let pos = layer.get_character_position(1);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap(), (10.0, 0.0));
    }

    #[test]
    fn test_text_layer_character_transform() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        layer.position = (100.0, 200.0);

        let transform = layer.get_character_transform(0, 0.0);
        assert_eq!(transform.position, (100.0, 200.0));
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, (1.0, 1.0));
        assert_eq!(transform.opacity, 1.0);
        assert_eq!(transform.font_size, 16.0);
        assert_eq!(transform.baseline_shift, 0.0);
    }

    #[test]
    fn test_text_layer_bounds() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello World".to_string());
        layer.position = (50.0, 100.0);
        layer.typography.font_size = 20.0;

        let bounds = layer.get_bounds();
        assert!(bounds.is_some());
        let (min_x, min_y, max_x, max_y) = bounds.unwrap();
        assert_eq!(min_x, 50.0);
        assert_eq!(min_y, 100.0);

        assert!((max_x - 182.0).abs() < 0.001);

        assert!((max_y - 124.0).abs() < 0.001);
    }

    #[test]
    fn test_text_layer_contains_point() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        layer.position = (0.0, 0.0);
        layer.typography.font_size = 20.0;

        assert!(layer.contains_point(25.0, 10.0));
        assert!(!layer.contains_point(150.0, 10.0));
        assert!(!layer.contains_point(25.0, 50.0));
    }

    #[test]
    fn test_text_layer_animation_integration() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        layer.position = (100.0, 200.0);

        let animation_id = layer.create_fade_animation(vec![0, 1, 2, 3, 4], 0.0, 1.0, 1.0);
        assert!(layer.get_animation(&animation_id).is_some());


        let transform = layer.get_character_transform(0, 0.5);
        assert!((transform.opacity - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_text_layer_slide_animation() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        layer.position = (100.0, 200.0);

        let animation_id = layer.create_slide_animation(vec![0, 1, 2, 3, 4], (0.0, 0.0), (50.0, 25.0), 1.0);
        assert!(layer.get_animation(&animation_id).is_some());


        let transform = layer.get_character_transform(0, 0.5);
        assert_eq!(transform.position, (125.0, 212.5));
    }

    #[test]
    fn test_text_layer_typography_animation() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());

        let animation_id = layer.create_typography_animation(AnimationType::FontSize, vec![0, 1, 2, 3, 4]);
        assert!(layer.get_animation(&animation_id).is_some());

        let animation = layer.get_animation(&animation_id).unwrap();
        assert_eq!(animation.animation_type, AnimationType::FontSize);
    }

    #[test]
    fn test_text_layer_setters() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());

        layer.set_position(150.0, 250.0);
        assert_eq!(layer.position, (150.0, 250.0));

        layer.set_opacity(0.7);
        assert_eq!(layer.opacity, 0.7);

        layer.set_opacity(1.5);
        assert_eq!(layer.opacity, 1.0);

        layer.set_opacity(-0.5);
        assert_eq!(layer.opacity, 0.0);

        layer.set_blend_mode(BlendMode::Screen);
        assert_eq!(layer.blend_mode, BlendMode::Screen);

        layer.set_visible(false);
        assert!(!layer.visible);

        layer.set_locked(true);
        assert!(layer.locked);
    }

    #[test]
    fn test_text_layer_animation_controls() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());

        let animation_id = layer.create_fade_animation(vec![0], 0.0, 1.0, 1.0);

        layer.play_animations();
        assert!(layer.animator.playing);

        layer.pause_animations();
        assert!(!layer.animator.playing);

        layer.stop_animations();
        assert!(!layer.animator.playing);
        assert_eq!(layer.animator.global_time, 0.0);

        layer.reset_animations();
        assert_eq!(layer.animator.global_time, 0.0);
    }

    #[test]
    fn test_text_layer_validation() {
        let valid_layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        assert!(valid_layer.validate().is_ok());

        let mut invalid_layer = TextLayer::new("".to_string(), "Test".to_string(), "Hello".to_string());
        assert!(invalid_layer.validate().is_err());

        invalid_layer.id = "layer1".to_string();
        invalid_layer.name = "".to_string();
        assert!(invalid_layer.validate().is_err());

        invalid_layer.name = "Test".to_string();
        invalid_layer.opacity = 2.0;
        assert!(invalid_layer.validate().is_err());
    }

    #[test]
    fn test_text_layer_clone_without_animations() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        let animation_id = layer.create_fade_animation(vec![0], 0.0, 1.0, 1.0);

        let cloned = layer.clone_without_animations();

        assert_eq!(cloned.id, "layer1_clone");
        assert_eq!(cloned.name, "Test Clone");
        assert_eq!(cloned.text, layer.text);
        assert_eq!(cloned.get_animation_count(), 0);
        assert_ne!(cloned.id, layer.id);
    }

    #[test]
    fn test_blend_modes() {
        let blend_modes = vec![
            BlendMode::Normal,
            BlendMode::Multiply,
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::SoftLight,
            BlendMode::HardLight,
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
            BlendMode::Darken,
            BlendMode::Lighten,
            BlendMode::Difference,
            BlendMode::Exclusion,
        ];

        for blend_mode in blend_modes {
            let mut layer = TextLayer::new("test".to_string(), "Test".to_string(), "Hello".to_string());
            layer.blend_mode = blend_mode;
            assert_eq!(layer.blend_mode, blend_mode);
        }
    }

    #[test]
    fn test_character_transform_default() {
        let transform = CharacterTransform {
            position: (0.0, 0.0),
            rotation: 0.0,
            scale: (1.0, 1.0),
            opacity: 1.0,
            color: (0.0, 0.0, 0.0, 1.0),
            font_size: 16.0,
            baseline_shift: 0.0,
        };

        assert_eq!(transform.position, (0.0, 0.0));
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, (1.0, 1.0));
        assert_eq!(transform.opacity, 1.0);
        assert_eq!(transform.color, (0.0, 0.0, 0.0, 1.0));
        assert_eq!(transform.font_size, 16.0);
        assert_eq!(transform.baseline_shift, 0.0);
    }

    #[test]
    fn test_default_layer() {
        let layer = TextLayer::default();
        assert_eq!(layer.id, "default");
        assert_eq!(layer.name, "Default Layer");
        assert_eq!(layer.text, "Text");
        assert!(layer.visible);
        assert!(!layer.locked);
    }
}
