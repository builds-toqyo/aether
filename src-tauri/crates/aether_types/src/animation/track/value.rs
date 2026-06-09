

use serde::{Deserialize, Serialize};
use std::fmt;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackValue {

    Float(f64),

    Vector2([f64; 2]),

    Vector3([f64; 3]),

    Vector4([f64; 4]),

    Color([f64; 4]),

    Boolean(bool),

    String(String),
}

impl TrackValue {

    pub fn as_float(&self) -> Option<f64> {
        match self {
            TrackValue::Float(v) => Some(*v),
            _ => None,
        }
    }


    pub fn as_vector2(&self) -> Option<[f64; 2]> {
        match self {
            TrackValue::Vector2(v) => Some(*v),
            _ => None,
        }
    }


    pub fn as_vector3(&self) -> Option<[f64; 3]> {
        match self {
            TrackValue::Vector3(v) => Some(*v),
            _ => None,
        }
    }


    pub fn as_vector4(&self) -> Option<[f64; 4]> {
        match self {
            TrackValue::Vector4(v) => Some(*v),
            TrackValue::Color(v) => Some(*v),
            _ => None,
        }
    }


    pub fn as_color(&self) -> Option<[f64; 4]> {
        match self {
            TrackValue::Color(v) => Some(*v),
            TrackValue::Vector4(v) => Some(*v),
            _ => None,
        }
    }


    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            TrackValue::Boolean(v) => Some(*v),
            _ => None,
        }
    }


    pub fn as_string(&self) -> Option<&str> {
        match self {
            TrackValue::String(v) => Some(v),
            _ => None,
        }
    }


    pub fn dimension(&self) -> usize {
        match self {
            TrackValue::Float(_) => 1,
            TrackValue::Vector2(_) => 2,
            TrackValue::Vector3(_) => 3,
            TrackValue::Vector4(_) => 4,
            TrackValue::Color(_) => 4,
            TrackValue::Boolean(_) => 1,
            TrackValue::String(_) => 1,
        }
    }


    pub fn interpolate(&self, other: &TrackValue, t: f64) -> Result<TrackValue, String> {
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return Err("Cannot interpolate different value types".to_string());
        }

        match (self, other) {
            (TrackValue::Float(a), TrackValue::Float(b)) => {
                Ok(TrackValue::Float(a + (b - a) * t))
            }
            (TrackValue::Vector2(a), TrackValue::Vector2(b)) => {
                Ok(TrackValue::Vector2([
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                ]))
            }
            (TrackValue::Vector3(a), TrackValue::Vector3(b)) => {
                Ok(TrackValue::Vector3([
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                    a[2] + (b[2] - a[2]) * t,
                ]))
            }
            (TrackValue::Vector4(a), TrackValue::Vector4(b)) => {
                Ok(TrackValue::Vector4([
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                    a[2] + (b[2] - a[2]) * t,
                    a[3] + (b[3] - a[3]) * t,
                ]))
            }
            (TrackValue::Color(a), TrackValue::Color(b)) => {
                Ok(TrackValue::Color([
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                    a[2] + (b[2] - a[2]) * t,
                    a[3] + (b[3] - a[3]) * t,
                ]))
            }
            (TrackValue::Boolean(a), TrackValue::Boolean(b)) => {
                Ok(TrackValue::Boolean(if t < 0.5 { *a } else { *b }))
            }
            (TrackValue::String(a), TrackValue::String(b)) => {
                Ok(TrackValue::String(if t < 0.5 { a.clone() } else { b.clone() }))
            }
            _ => Err("Unsupported interpolation for this type".to_string()),
        }
    }


    pub fn default_for_type(track_type: super::types::TrackType) -> Self {
        match track_type {
            super::types::TrackType::Position => TrackValue::Vector3([0.0, 0.0, 0.0]),
            super::types::TrackType::Rotation => TrackValue::Vector3([0.0, 0.0, 0.0]),
            super::types::TrackType::Scale => TrackValue::Vector3([1.0, 1.0, 1.0]),
            super::types::TrackType::Opacity => TrackValue::Float(1.0),
            super::types::TrackType::Color => TrackValue::Color([1.0, 1.0, 1.0, 1.0]),
            super::types::TrackType::Custom => TrackValue::Float(0.0),
        }
    }


    pub fn is_compatible_with_track(&self, track_type: super::types::TrackType) -> bool {
        match track_type {
            super::types::TrackType::Position | super::types::TrackType::Rotation | super::types::TrackType::Scale => {
                matches!(self, TrackValue::Vector3(_))
            }
            super::types::TrackType::Opacity => {
                matches!(self, TrackValue::Float(_))
            }
            super::types::TrackType::Color => {
                matches!(self, TrackValue::Color(_) | TrackValue::Vector4(_))
            }
            super::types::TrackType::Custom => true,
        }
    }
}

impl fmt::Display for TrackValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackValue::Float(v) => write!(f, "{:.3}", v),
            TrackValue::Vector2(v) => write!(f, "[{:.3}, {:.3}]", v[0], v[1]),
            TrackValue::Vector3(v) => write!(f, "[{:.3}, {:.3}, {:.3}]", v[0], v[1], v[2]),
            TrackValue::Vector4(v) => write!(f, "[{:.3}, {:.3}, {:.3}, {:.3}]", v[0], v[1], v[2], v[3]),
            TrackValue::Color(v) => write!(f, "rgba({:.3}, {:.3}, {:.3}, {:.3})", v[0], v[1], v[2], v[3]),
            TrackValue::Boolean(v) => write!(f, "{}", v),
            TrackValue::String(v) => write!(f, "\"{}\"", v),
        }
    }
}

impl Default for TrackValue {
    fn default() -> Self {
        TrackValue::Float(0.0)
    }
}


pub struct TrackValueUtils;

impl TrackValueUtils {

    pub fn lerp(a: &TrackValue, b: &TrackValue, t: f64) -> Result<TrackValue, String> {
        a.interpolate(b, t)
    }


    pub fn zero_for_type(track_type: super::types::TrackType) -> TrackValue {
        match track_type {
            super::types::TrackType::Position => TrackValue::Vector3([0.0, 0.0, 0.0]),
            super::types::TrackType::Rotation => TrackValue::Vector3([0.0, 0.0, 0.0]),
            super::types::TrackType::Scale => TrackValue::Vector3([0.0, 0.0, 0.0]),
            super::types::TrackType::Opacity => TrackValue::Float(0.0),
            super::types::TrackType::Color => TrackValue::Color([0.0, 0.0, 0.0, 0.0]),
            super::types::TrackType::Custom => TrackValue::Float(0.0),
        }
    }


    pub fn identity_for_type(track_type: super::types::TrackType) -> TrackValue {
        match track_type {
            super::types::TrackType::Position => TrackValue::Vector3([0.0, 0.0, 0.0]),
            super::types::TrackType::Rotation => TrackValue::Vector3([0.0, 0.0, 0.0]),
            super::types::TrackType::Scale => TrackValue::Vector3([1.0, 1.0, 1.0]),
            super::types::TrackType::Opacity => TrackValue::Float(1.0),
            super::types::TrackType::Color => TrackValue::Color([1.0, 1.0, 1.0, 1.0]),
            super::types::TrackType::Custom => TrackValue::Float(0.0),
        }
    }


    pub fn to_float_array(value: &TrackValue) -> Vec<f64> {
        match value {
            TrackValue::Float(v) => vec![*v],
            TrackValue::Vector2(v) => v.to_vec(),
            TrackValue::Vector3(v) => v.to_vec(),
            TrackValue::Vector4(v) => v.to_vec(),
            TrackValue::Color(v) => v.to_vec(),
            TrackValue::Boolean(v) => vec![if *v { 1.0 } else { 0.0 }],
            TrackValue::String(_) => vec![0.0],
        }
    }


    pub fn from_float_array(values: &[f64], track_type: super::types::TrackType) -> Result<TrackValue, String> {
        match track_type {
            super::types::TrackType::Position | super::types::TrackType::Rotation | super::types::TrackType::Scale => {
                if values.len() >= 3 {
                    Ok(TrackValue::Vector3([values[0], values[1], values[2]]))
                } else {
                    Err("Insufficient values for 3D vector".to_string())
                }
            }
            super::types::TrackType::Opacity => {
                if values.len() >= 1 {
                    Ok(TrackValue::Float(values[0]))
                } else {
                    Err("Insufficient values for float".to_string())
                }
            }
            super::types::TrackType::Color => {
                if values.len() >= 4 {
                    Ok(TrackValue::Color([values[0], values[1], values[2], values[3]]))
                } else {
                    Err("Insufficient values for color".to_string())
                }
            }
            super::types::TrackType::Custom => {
                if values.len() >= 1 {
                    Ok(TrackValue::Float(values[0]))
                } else {
                    Err("Insufficient values for custom type".to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::track::types::TrackType;

    #[test]
    fn test_track_value_creation() {
        let float_val = TrackValue::Float(42.0);
        let vector_val = TrackValue::Vector3([1.0, 2.0, 3.0]);
        let color_val = TrackValue::Color([1.0, 0.0, 0.0, 1.0]);

        assert_eq!(float_val.as_float(), Some(42.0));
        assert_eq!(vector_val.as_vector3(), Some([1.0, 2.0, 3.0]));
        assert_eq!(color_val.as_color(), Some([1.0, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn test_track_value_interpolation() {
        let a = TrackValue::Float(0.0);
        let b = TrackValue::Float(10.0);

        let result = a.interpolate(&b, 0.5).unwrap();

        if let TrackValue::Float(v) = result {
            assert_eq!(v, 5.0);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_track_value_compatibility() {
        let position_val = TrackValue::Vector3([1.0, 2.0, 3.0]);
        let color_val = TrackValue::Color([1.0, 0.0, 0.0, 1.0]);

        assert!(position_val.is_compatible_with_track(TrackType::Position));
        assert!(!position_val.is_compatible_with_track(TrackType::Color));
        assert!(color_val.is_compatible_with_track(TrackType::Color));
        assert!(color_val.is_compatible_with_track(TrackType::Color));
    }

    #[test]
    fn test_track_value_utils() {
        let zero_pos = TrackValueUtils::zero_for_type(TrackType::Position);
        let identity_scale = TrackValueUtils::identity_for_type(TrackType::Scale);

        assert_eq!(zero_pos.as_vector3(), Some([0.0, 0.0, 0.0]));
        assert_eq!(identity_scale.as_vector3(), Some([1.0, 1.0, 1.0]));
    }

    #[test]
    fn test_float_array_conversion() {
        let vector_val = TrackValue::Vector3([1.0, 2.0, 3.0]);
        let array = TrackValueUtils::to_float_array(&vector_val);

        assert_eq!(array, vec![1.0, 2.0, 3.0]);

        let reconstructed = TrackValueUtils::from_float_array(&array, TrackType::Position).unwrap();
        assert_eq!(reconstructed, vector_val);
    }
}
