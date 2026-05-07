

use serde::{Deserialize, Serialize};
use std::fmt;


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InterpolationMethod {

    Linear,

    Bezier,

    CubicSpline,

    Step,

    Hold,
}

impl InterpolationMethod {

    pub fn name(&self) -> &'static str {
        match self {
            InterpolationMethod::Linear => __STRING_0__,
            InterpolationMethod::Bezier => __STRING_1__,
            InterpolationMethod::CubicSpline => __STRING_2__,
            InterpolationMethod::Step => __STRING_3__,
            InterpolationMethod::Hold => __STRING_4__,
        }
    }

    /// Check if method is smooth
    pub fn is_smooth(&self) -> bool {
        !matches!(self, InterpolationMethod::Step | InterpolationMethod::Hold)
    }

    /// Check if method requires control points
    pub fn requires_control_points(&self) -> bool {
        matches!(self, InterpolationMethod::Bezier | InterpolationMethod::CubicSpline)
    }

    /// Get method description
    pub fn description(&self) -> &'static str {
        match self {
            InterpolationMethod::Linear => "Linear interpolation between keyframes",
            InterpolationMethod::Bezier => "Bezier curve interpolation with control points",
            InterpolationMethod::CubicSpline => "Cubic spline interpolation for smooth curves",
            InterpolationMethod::Step => "Step function with no interpolation",
            InterpolationMethod::Hold => "Hold previous value until next keyframe",
        }
    }
}

impl fmt::Display for InterpolationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Default for InterpolationMethod {
    fn default() -> Self {
        InterpolationMethod::Linear
    }
}


pub struct BasicInterpolation;

impl BasicInterpolation {

    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }


    pub fn lerp_vec2(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
        [
            Self::lerp(a[0], b[0], t),
            Self::lerp(a[1], b[1], t),
        ]
    }


    pub fn lerp_vec3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        [
            Self::lerp(a[0], b[0], t),
            Self::lerp(a[1], b[1], t),
            Self::lerp(a[2], b[2], t),
        ]
    }


    pub fn lerp_vec4(a: [f64; 4], b: [f64; 4], t: f64) -> [f64; 4] {
        [
            Self::lerp(a[0], b[0], t),
            Self::lerp(a[1], b[1], t),
            Self::lerp(a[2], b[2], t),
            Self::lerp(a[3], b[3], t),
        ]
    }


    pub fn bezier_quadratic(p0: f64, p1: f64, p2: f64, t: f64) -> f64 {
        let mt = 1.0 - t;
        mt * mt * p0 + 2.0 * mt * t * p1 + t * t * p2
    }


    pub fn bezier_cubic(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
        let mt = 1.0 - t;
        mt * mt * mt * p0 + 3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t * p3
    }


    pub fn step(a: f64, b: f64, t: f64) -> f64 {
        if t < 0.5 { a } else { b }
    }


    pub fn hold(a: f64, _b: f64, _t: f64) -> f64 {
        a
    }


    pub fn interpolate(a: f64, b: f64, t: f64, method: InterpolationMethod) -> f64 {
        match method {
            InterpolationMethod::Linear => Self::lerp(a, b, t),
            InterpolationMethod::Bezier => Self::lerp(a, b, t),
            InterpolationMethod::CubicSpline => Self::lerp(a, b, t),
            InterpolationMethod::Step => Self::step(a, b, t),
            InterpolationMethod::Hold => Self::hold(a, b, t),
        }
    }


    pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
        value.max(min).min(max)
    }


    pub fn remap(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
        let t = (value - from_min) / (from_max - from_min);
        Self::lerp(to_min, to_max, t)
    }


    pub fn smooth_step(edge0: f64, edge1: f64, x: f64) -> f64 {
        let t = Self::clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolation_methods() {
        assert_eq!(InterpolationMethod::Linear.name(), "Linear");
        assert!(InterpolationMethod::Linear.is_smooth());
        assert!(!InterpolationMethod::Step.is_smooth());
        assert!(InterpolationMethod::Bezier.requires_control_points());
        assert!(!InterpolationMethod::Linear.requires_control_points());
    }

    #[test]
    fn test_basic_interpolation() {

        assert_eq!(BasicInterpolation::lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(BasicInterpolation::lerp(10.0, 20.0, 0.25), 12.5);


        let a = [0.0, 1.0, 2.0];
        let b = [10.0, 11.0, 12.0];
        let result = BasicInterpolation::lerp_vec3(a, b, 0.5);
        assert_eq!(result, [5.0, 6.0, 7.0]);


        let bezier_result = BasicInterpolation::bezier_quadratic(0.0, 5.0, 10.0, 0.5);
        assert_eq!(bezier_result, 5.0);


        assert_eq!(BasicInterpolation::clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(BasicInterpolation::clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(BasicInterpolation::clamp(15.0, 0.0, 10.0), 10.0);


        assert_eq!(BasicInterpolation::remap(5.0, 0.0, 10.0, 100.0, 200.0), 150.0);


        let smooth = BasicInterpolation::smooth_step(0.0, 1.0, 0.5);
        assert!(smooth > 0.0 && smooth < 1.0);
    }

    #[test]
    fn test_interpolation_method_descriptions() {
        assert!(InterpolationMethod::Linear.description().contains("Linear"));
        assert!(InterpolationMethod::Bezier.description().contains("Bezier"));
        assert!(InterpolationMethod::Step.description().contains("Step"));
    }

    #[test]
    fn test_step_and_hold() {
        let step_result = BasicInterpolation::step(0.0, 10.0, 0.25);
        assert_eq!(step_result, 0.0);

        let step_result = BasicInterpolation::step(0.0, 10.0, 0.75);
        assert_eq!(step_result, 10.0);

        let hold_result = BasicInterpolation::hold(5.0, 10.0, 0.75);
        assert_eq!(hold_result, 5.0);
    }
}
