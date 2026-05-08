use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationType {
    Opacity,
    Position,
    Scale,
    Rotation,
    Color,
    Blur,
    Tracking,
    BaselineShift,
    FontSize,
    FontWeight,
    LineHeight,
    LetterSpacing,
    TextTransform,
    PathPosition,
    PathRotation,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextKeyframe {
    pub time: f64,
    pub value: AnimationValue,
    pub easing: crate::animation::interpolation::EasingFunction,
    pub interpolation: crate::animation::interpolation::InterpolationMethod,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationValue {
    Float(f64),
    Vector2(f64, f64),
    Vector3(f64, f64, f64),
    Color(f64, f64, f64, f64),
    String(String),
    Bool(bool),
}

impl Default for AnimationValue {
    fn default() -> Self {
        AnimationValue::Float(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::interpolation::{EasingFunction, InterpolationMethod};

    #[test]
    fn test_animation_value_creation() {
        let float_val = AnimationValue::Float(1.5);
        let vector2_val = AnimationValue::Vector2(1.0, 2.0);
        let vector3_val = AnimationValue::Vector3(1.0, 2.0, 3.0);
        let color_val = AnimationValue::Color(1.0, 0.0, 0.0, 1.0);
        let string_val = AnimationValue::String("test".to_string());
        let bool_val = AnimationValue::Bool(true);

        assert_eq!(float_val, AnimationValue::Float(1.5));
        assert_eq!(vector2_val, AnimationValue::Vector2(1.0, 2.0));
        assert_eq!(vector3_val, AnimationValue::Vector3(1.0, 2.0, 3.0));
        assert_eq!(color_val, AnimationValue::Color(1.0, 0.0, 0.0, 1.0));
        assert_eq!(string_val, AnimationValue::String("test".to_string()));
        assert_eq!(bool_val, AnimationValue::Bool(true));
    }

    #[test]
    fn test_keyframe_creation() {
        let keyframe = TextKeyframe {
            time: 0.5,
            value: AnimationValue::Float(1.0),
            easing: EasingFunction::Linear,
            interpolation: InterpolationMethod::Linear,
        };

        assert_eq!(keyframe.time, 0.5);
        assert_eq!(keyframe.value, AnimationValue::Float(1.0));
        assert_eq!(keyframe.easing, EasingFunction::Linear);
        assert_eq!(keyframe.interpolation, InterpolationMethod::Linear);
    }

    #[test]
    fn test_animation_types() {
        let types = vec![
            AnimationType::Opacity,
            AnimationType::Position,
            AnimationType::Scale,
            AnimationType::Rotation,
            AnimationType::Color,
            AnimationType::Blur,
            AnimationType::Tracking,
            AnimationType::BaselineShift,
            AnimationType::FontSize,
            AnimationType::FontWeight,
            AnimationType::LineHeight,
            AnimationType::LetterSpacing,
            AnimationType::TextTransform,
            AnimationType::PathPosition,
            AnimationType::PathRotation,
            AnimationType::Custom("custom".to_string()),
        ];

        for anim_type in types {
            // Just verify they can be created and compared
            assert_eq!(anim_type, anim_type);
        }
    }

    #[test]
    fn test_animation_value_default() {
        let default_val = AnimationValue::default();
        assert_eq!(default_val, AnimationValue::Float(0.0));
    }
}
