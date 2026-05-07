

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::animation::interpolation::{InterpolationMethod, EasingFunction};


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Keyframe<T> {
    pub time: f64,
    pub value: T,

    pub interpolation: InterpolationMethod,

    pub easing: EasingFunction,
}

impl<T> Keyframe<T> {

    pub fn new(time: f64, value: T) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationMethod::Linear,
            easing: EasingFunction::Linear,
        }
    }


    pub fn with_interpolation(
        time: f64,
        value: T,
        interpolation: InterpolationMethod,
    ) -> Self {
        Self {
            time,
            value,
            interpolation,
            easing: EasingFunction::Linear,
        }
    }


    pub fn with_easing(
        time: f64,
        value: T,
        interpolation: InterpolationMethod,
        easing: EasingFunction,
    ) -> Self {
        Self {
            time,
            value,
            interpolation,
            easing,
        }
    }


    pub fn set_interpolation(&mut self, interpolation: InterpolationMethod) {
        self.interpolation = interpolation;
    }


    pub fn set_easing(&mut self, easing: EasingFunction) {
        self.easing = easing;
    }


    pub fn time_string(&self) -> String {
        format!("{:.3}s", self.time)
    }
}

impl<T: fmt::Display> fmt::Display for Keyframe<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, __STRING_1__,
               self.time, self.value, self.interpolation, self.easing)
    }
}

/// Generic keyframe data container for different value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyframeData {
    /// Float value keyframe
    Float(Keyframe<f64>),
    /// Vector2 keyframe
    Vector2(Keyframe<[f64; 2]>),
    /// Vector3 keyframe
    Vector3(Keyframe<[f64; 3]>),
    /// Vector4 keyframe
    Vector4(Keyframe<[f64; 4]>),
    /// Color keyframe (RGBA)
    Color(Keyframe<[f64; 4]>),
    /// Transform keyframe (position, rotation, scale)
    Transform(Keyframe<TransformData>),
    /// Boolean keyframe
    Boolean(Keyframe<bool>),
    /// String keyframe
    String(Keyframe<String>),
}

impl KeyframeData {
    /// Get the time value
    pub fn time(&self) -> f64 {
        match self {
            KeyframeData::Float(k) => k.time,
            KeyframeData::Vector2(k) => k.time,
            KeyframeData::Vector3(k) => k.time,
            KeyframeData::Vector4(k) => k.time,
            KeyframeData::Color(k) => k.time,
            KeyframeData::Transform(k) => k.time,
            KeyframeData::Boolean(k) => k.time,
            KeyframeData::String(k) => k.time,
        }
    }

    /// Get the interpolation method
    pub fn interpolation(&self) -> InterpolationMethod {
        match self {
            KeyframeData::Float(k) => k.interpolation,
            KeyframeData::Vector2(k) => k.interpolation,
            KeyframeData::Vector3(k) => k.interpolation,
            KeyframeData::Vector4(k) => k.interpolation,
            KeyframeData::Color(k) => k.interpolation,
            KeyframeData::Transform(k) => k.interpolation,
            KeyframeData::Boolean(k) => k.interpolation,
            KeyframeData::String(k) => k.interpolation,
        }
    }

    /// Get the easing function
    pub fn easing(&self) -> EasingFunction {
        match self {
            KeyframeData::Float(k) => k.easing,
            KeyframeData::Vector2(k) => k.easing,
            KeyframeData::Vector3(k) => k.easing,
            KeyframeData::Vector4(k) => k.easing,
            KeyframeData::Color(k) => k.easing,
            KeyframeData::Transform(k) => k.easing,
            KeyframeData::Boolean(k) => k.easing,
            KeyframeData::String(k) => k.easing,
        }
    }

    /// Set interpolation method
    pub fn set_interpolation(&mut self, interpolation: InterpolationMethod) {
        match self {
            KeyframeData::Float(k) => k.set_interpolation(interpolation),
            KeyframeData::Vector2(k) => k.set_interpolation(interpolation),
            KeyframeData::Vector3(k) => k.set_interpolation(interpolation),
            KeyframeData::Vector4(k) => k.set_interpolation(interpolation),
            KeyframeData::Color(k) => k.set_interpolation(interpolation),
            KeyframeData::Transform(k) => k.set_interpolation(interpolation),
            KeyframeData::Boolean(k) => k.set_interpolation(interpolation),
            KeyframeData::String(k) => k.set_interpolation(interpolation),
        }
    }

    /// Set easing function
    pub fn set_easing(&mut self, easing: EasingFunction) {
        match self {
            KeyframeData::Float(k) => k.set_easing(easing),
            KeyframeData::Vector2(k) => k.set_easing(easing),
            KeyframeData::Vector3(k) => k.set_easing(easing),
            KeyframeData::Vector4(k) => k.set_easing(easing),
            KeyframeData::Color(k) => k.set_easing(easing),
            KeyframeData::Transform(k) => k.set_easing(easing),
            KeyframeData::Boolean(k) => k.set_easing(easing),
            KeyframeData::String(k) => k.set_easing(easing),
        }
    }
}

/// Transform data for animation keyframes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransformData {
    /// Position (x, y, z)
    pub position: [f64; 3],
    /// Rotation in degrees (x, y, z)
    pub rotation: [f64; 3],
    /// Scale (x, y, z)
    pub scale: [f64; 3],
}

impl TransformData {
    /// Create identity transform
    pub fn identity() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    /// Create transform with position only
    pub fn from_position(x: f64, y: f64, z: f64) -> Self {
        Self {
            position: [x, y, z],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    /// Create transform with rotation only
    pub fn from_rotation(x: f64, y: f64, z: f64) -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [x, y, z],
            scale: [1.0, 1.0, 1.0],
        }
    }

    /// Create transform with scale only
    pub fn from_scale(x: f64, y: f64, z: f64) -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [x, y, z],
        }
    }

    /// Create complete transform
    pub fn new(position: [f64; 3], rotation: [f64; 3], scale: [f64; 3]) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}

impl Default for TransformData {
    fn default() -> Self {
        Self::identity()
    }
}

impl fmt::Display for TransformData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transform(pos: [{:.1}, {:.1}, {:.1}], rot: [{:.1}, {:.1}, {:.1}], scale: [{:.1}, {:.1}, {:.1}])",
               self.position[0], self.position[1], self.position[2],
               self.rotation[0], self.rotation[1], self.rotation[2],
               self.scale[0], self.scale[1], self.scale[2])
    }
}


pub struct KeyframeCollection;

impl KeyframeCollection {

    pub fn sort_by_time<T>(keyframes: &mut Vec<Keyframe<T>>) {
        keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    }


    pub fn find_keyframe_at_time<T>(keyframes: &[Keyframe<T>], time: f64) -> Option<usize> {
        keyframes.iter().position(|k| k.time >= time).map(|i| {
            if i > 0 { i - 1 } else { i }
        })
    }


    pub fn find_surrounding_keyframes<T>(keyframes: &[Keyframe<T>], time: f64) -> (Option<&Keyframe<T>>, Option<&Keyframe<T>>) {
        let pos = keyframes.iter().position(|k| k.time >= time);

        match pos {
            Some(0) => (None, keyframes.get(0)),
            Some(i) => (keyframes.get(i - 1), keyframes.get(i)),
            None => {
                if keyframes.is_empty() {
                    (None, None)
                } else {
                    (keyframes.last(), None)
                }
            }
        }
    }


    pub fn validate_keyframes<T>(keyframes: &[Keyframe<T>]) -> Result<(), String> {
        if keyframes.is_empty() {
            return Err("No keyframes provided".to_string());
        }


        let mut times = Vec::new();
        for keyframe in keyframes {
            if times.contains(&keyframe.time) {
                return Err(format!("Duplicate keyframe time: {}", keyframe.time));
            }
            times.push(keyframe.time);
        }


        for i in 1..keyframes.len() {
            if keyframes[i].time < keyframes[i - 1].time {
                return Err(format!("Keyframe time not monotonic: {} < {}",
                                 keyframes[i].time, keyframes[i - 1].time));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::interpolation::{InterpolationMethod, EasingFunction};

    #[test]
    fn test_keyframe_creation() {
        let keyframe = Keyframe::new(1.5, 42.0);

        assert_eq!(keyframe.time, 1.5);
        assert_eq!(keyframe.value, 42.0);
        assert_eq!(keyframe.interpolation, InterpolationMethod::Linear);
        assert_eq!(keyframe.easing, EasingFunction::Linear);
    }

    #[test]
    fn test_keyframe_with_interpolation() {
        let keyframe = Keyframe::with_interpolation(
            2.0,
            100.0,
            InterpolationMethod::Bezier
        );

        assert_eq!(keyframe.interpolation, InterpolationMethod::Bezier);
        assert_eq!(keyframe.easing, EasingFunction::Linear);
    }

    #[test]
    fn test_keyframe_display() {
        let keyframe = Keyframe::new(1.234, 42.0);
        let display = format!("{}", keyframe);

        assert!(display.contains("1.234"));
        assert!(display.contains("42"));
        assert!(display.contains("Linear"));
    }

    #[test]
    fn test_transform_data() {
        let transform = TransformData::identity();

        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_transform_data_creation() {
        let transform = TransformData::from_position(10.0, 20.0, 30.0);

        assert_eq!(transform.position, [10.0, 20.0, 30.0]);
        assert_eq!(transform.rotation, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_keyframe_data() {
        let float_keyframe = KeyframeData::Float(Keyframe::new(1.0, 42.0));
        let vector_keyframe = KeyframeData::Vector3(Keyframe::new(2.0, [1.0, 2.0, 3.0]));

        assert_eq!(float_keyframe.time(), 1.0);
        assert_eq!(vector_keyframe.time(), 2.0);
    }

    #[test]
    fn test_keyframe_collection_sort() {
        let mut keyframes = vec![
            Keyframe::new(3.0, 3.0),
            Keyframe::new(1.0, 1.0),
            Keyframe::new(2.0, 2.0),
        ];

        KeyframeCollection::sort_by_time(&mut keyframes);

        assert_eq!(keyframes[0].time, 1.0);
        assert_eq!(keyframes[1].time, 2.0);
        assert_eq!(keyframes[2].time, 3.0);
    }

    #[test]
    fn test_keyframe_validation() {
        let valid_keyframes = vec![
            Keyframe::new(1.0, 1.0),
            Keyframe::new(2.0, 2.0),
            Keyframe::new(3.0, 3.0),
        ];

        assert!(KeyframeCollection::validate_keyframes(&valid_keyframes).is_ok());

        let invalid_keyframes = vec![
            Keyframe::new(1.0, 1.0),
            Keyframe::new(1.0, 2.0),
        ];

        assert!(KeyframeCollection::validate_keyframes(&invalid_keyframes).is_err());
    }

    #[test]
    fn test_surrounding_keyframes() {
        let keyframes = vec![
            Keyframe::new(1.0, 1.0),
            Keyframe::new(3.0, 3.0),
            Keyframe::new(5.0, 5.0),
        ];


        let (prev, next) = KeyframeCollection::find_surrounding_keyframes(&keyframes, 0.5);
        assert!(prev.is_none());
        assert!(next.is_some());
        assert_eq!(next.unwrap().time, 1.0);


        let (prev, next) = KeyframeCollection::find_surrounding_keyframes(&keyframes, 2.0);
        assert!(prev.is_some());
        assert!(next.is_some());
        assert_eq!(prev.unwrap().time, 1.0);
        assert_eq!(next.unwrap().time, 3.0);


        let (prev, next) = KeyframeCollection::find_surrounding_keyframes(&keyframes, 6.0);
        assert!(prev.is_some());
        assert!(next.is_none());
        assert_eq!(prev.unwrap().time, 5.0);
    }
}
