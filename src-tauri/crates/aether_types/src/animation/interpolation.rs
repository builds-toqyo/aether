//! Interpolation methods and easing functions
//! 
//! This module provides comprehensive interpolation algorithms and
//! easing functions for smooth animation transitions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Interpolation methods for keyframe animation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InterpolationMethod {
    Linear,
    Bezier,
    CubicSpline,
    Step,
    Hold,
}

impl InterpolationMethod {
    /// Get method name
    pub fn name(&self) -> &'static str {
        match self {
            InterpolationMethod::Linear => "Linear",
            InterpolationMethod::Bezier => "Bezier",
            InterpolationMethod::CubicSpline => "Cubic Spline",
            InterpolationMethod::Step => "Step",
            InterpolationMethod::Hold => "Hold",
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
}

impl fmt::Display for InterpolationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Easing functions for animation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EasingFunction {
    /// Linear easing
    Linear,
    /// Quadratic ease-in
    QuadIn,
    /// Quadratic ease-out
    QuadOut,
    /// Quadratic ease-in-out
    QuadInOut,
    /// Cubic ease-in
    CubicIn,
    /// Cubic ease-out
    CubicOut,
    /// Cubic ease-in-out
    CubicInOut,
    /// Quartic ease-in
    QuartIn,
    /// Quartic ease-out
    QuartOut,
    /// Quartic ease-in-out
    QuartInOut,
    /// Quintic ease-in
    QuintIn,
    /// Quintic ease-out
    QuintOut,
    /// Quintic ease-in-out
    QuintInOut,
    /// Sine ease-in
    SineIn,
    /// Sine ease-out
    SineOut,
    /// Sine ease-in-out
    SineInOut,
    /// Exponential ease-in
    ExpoIn,
    /// Exponential ease-out
    ExpoOut,
    /// Exponential ease-in-out
    ExpoInOut,
    /// Circular ease-in
    CircIn,
    /// Circular ease-out
    CircOut,
    /// Circular ease-in-out
    CircInOut,
    /// Back ease-in
    BackIn,
    /// Back ease-out
    BackOut,
    /// Back ease-in-out
    BackInOut,
    /// Elastic ease-in
    ElasticIn,
    /// Elastic ease-out
    ElasticOut,
    /// Elastic ease-in-out
    ElasticInOut,
    /// Bounce ease-in
    BounceIn,
    /// Bounce ease-out
    BounceOut,
    /// Bounce ease-in-out
    BounceInOut,
}

impl EasingFunction {
    /// Get function name
    pub fn name(&self) -> &'static str {
        match self {
            EasingFunction::Linear => "Linear",
            EasingFunction::QuadIn => "Quad In",
            EasingFunction::QuadOut => "Quad Out",
            EasingFunction::QuadInOut => "Quad In Out",
            EasingFunction::CubicIn => "Cubic In",
            EasingFunction::CubicOut => "Cubic Out",
            EasingFunction::CubicInOut => "Cubic In Out",
            EasingFunction::QuartIn => "Quart In",
            EasingFunction::QuartOut => "Quart Out",
            EasingFunction::QuartInOut => "Quart In Out",
            EasingFunction::QuintIn => "Quint In",
            EasingFunction::QuintOut => "Quint Out",
            EasingFunction::QuintInOut => "Quint In Out",
            EasingFunction::SineIn => "Sine In",
            EasingFunction::SineOut => "Sine Out",
            EasingFunction::SineInOut => "Sine In Out",
            EasingFunction::ExpoIn => "Expo In",
            EasingFunction::ExpoOut => "Expo Out",
            EasingFunction::ExpoInOut => "Expo In Out",
            EasingFunction::CircIn => "Circ In",
            EasingFunction::CircOut => "Circ Out",
            EasingFunction::CircInOut => "Circ In Out",
            EasingFunction::BackIn => "Back In",
            EasingFunction::BackOut => "Back Out",
            EasingFunction::BackInOut => "Back In Out",
            EasingFunction::ElasticIn => "Elastic In",
            EasingFunction::ElasticOut => "Elastic Out",
            EasingFunction::ElasticInOut => "Elastic In Out",
            EasingFunction::BounceIn => "Bounce In",
            EasingFunction::BounceOut => "Bounce Out",
            EasingFunction::BounceInOut => "Bounce In Out",
        }
    }
    
    /// Apply easing function to parameter t (0.0 to 1.0)
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        
        match self {
            EasingFunction::Linear => t,
            
            // Quadratic
            EasingFunction::QuadIn => t * t,
            EasingFunction::QuadOut => t * (2.0 - t),
            EasingFunction::QuadInOut => {
                if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t }
            }
            
            // Cubic
            EasingFunction::CubicIn => t * t * t,
            EasingFunction::CubicOut => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
            EasingFunction::CubicInOut => {
                if t < 0.5 { 4.0 * t * t * t } else {
                    let t = t - 1.0;
                    4.0 * t * t * t + 1.0
                }
            }
            
            // Quartic
            EasingFunction::QuartIn => t * t * t * t,
            EasingFunction::QuartOut => {
                let t = t - 1.0;
                1.0 - t * t * t * t
            }
            EasingFunction::QuartInOut => {
                if t < 0.5 { 8.0 * t * t * t * t } else {
                    let t = t - 1.0;
                    1.0 - 8.0 * t * t * t * t
                }
            }
            
            // Quintic
            EasingFunction::QuintIn => t * t * t * t * t,
            EasingFunction::QuintOut => {
                let t = t - 1.0;
                t * t * t * t * t + 1.0
            }
            EasingFunction::QuintInOut => {
                if t < 0.5 { 16.0 * t * t * t * t * t } else {
                    let t = t - 1.0;
                    16.0 * t * t * t * t * t + 1.0
                }
            }
            
            // Sine
            EasingFunction::SineIn => {
                let t = t - 1.0;
                -t.cos() + 1.0
            }
            EasingFunction::SineOut => t.sin(),
            EasingFunction::SineInOut => {
                -(t.cos() * std::f64::consts::PI) / 2.0 + 0.5
            }
            
            // Exponential
            EasingFunction::ExpoIn => {
                if t == 0.0 { 0.0 } else { 2.0_f64.powf(10.0 * (t - 1.0)) }
            }
            EasingFunction::ExpoOut => {
                if t == 1.0 { 1.0 } else { 1.0 - 2.0_f64.powf(-10.0 * t) }
            }
            EasingFunction::ExpoInOut => {
                if t == 0.0 { 0.0 } else if t == 1.0 { 1.0 } else {
                    if t < 0.5 {
                        2.0_f64.powf(20.0 * t - 10.0) / 2.0
                    } else {
                        (2.0 - 2.0_f64.powf(-20.0 * t + 10.0)) / 2.0
                    }
                }
            }
            
            // Circular
            EasingFunction::CircIn => {
                1.0 - (1.0 - t * t).sqrt()
            }
            EasingFunction::CircOut => {
                let t = t - 1.0;
                (1.0 - t * t).sqrt()
            }
            EasingFunction::CircInOut => {
                if t < 0.5 {
                    (1.0 - (1.0 - 4.0 * t * t).sqrt()) / 2.0
                } else {
                    let t = t - 1.0;
                    (1.0 + (1.0 - 4.0 * t * t).sqrt()) / 2.0
                }
            }
            
            // Back
            EasingFunction::BackIn => {
                const C1: f64 = 1.70158;
                const C3: f64 = C1 + 1.0;
                C3 * t * t * t - C1 * t * t
            }
            EasingFunction::BackOut => {
                const C1: f64 = 1.70158;
                const C3: f64 = C1 + 1.0;
                let t = t - 1.0;
                1.0 + C3 * t * t * t + C1 * t * t
            }
            EasingFunction::BackInOut => {
                const C1: f64 = 1.70158;
                const C2: f64 = C1 * 1.525;
                if t < 0.5 {
                    (2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2) / 2.0
                } else {
                    let t = t - 1.0;
                    (2.0 * t).powi(2) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) / 2.0 + 1.0
                }
            }
            
            // Elastic
            EasingFunction::ElasticIn => {
                const C4: f64 = (2.0 * std::f64::consts::PI) / 3.0;
                if t == 0.0 { 0.0 } else if t == 1.0 { 1.0 } else {
                    -2.0_f64.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * C4).sin()
                }
            }
            EasingFunction::ElasticOut => {
                const C4: f64 = (2.0 * std::f64::consts::PI) / 3.0;
                if t == 0.0 { 0.0 } else if t == 1.0 { 1.0 } else {
                    2.0_f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
                }
            }
            EasingFunction::ElasticInOut => {
                const C5: f64 = (2.0 * std::f64::consts::PI) / 4.5;
                if t == 0.0 { 0.0 } else if t == 1.0 { 1.0 } else {
                    if t < 0.5 {
                        -(2.0_f64.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
                    } else {
                        (2.0_f64.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0 + 1.0
                    }
                }
            }
            
            // Bounce
            EasingFunction::BounceIn => {
                1.0 - EasingFunction::BounceOut.apply(1.0 - t)
            }
            EasingFunction::BounceOut => {
                const N1: f64 = 7.5625;
                const D1: f64 = 2.75;
                
                if t < 1.0 / D1 {
                    N1 * t * t
                } else if t < 2.0 / D1 {
                    let t = t - 1.5 / D1;
                    N1 * t * t + 0.75
                } else if t < 2.5 / D1 {
                    let t = t - 2.25 / D1;
                    N1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / D1;
                    N1 * t * t + 0.984375
                }
            }
            EasingFunction::BounceInOut => {
                if t < 0.5 {
                    EasingFunction::BounceIn.apply(t * 2.0) * 0.5
                } else {
                    EasingFunction::BounceOut.apply(t * 2.0 - 1.0) * 0.5 + 0.5
                }
            }
        }
    }
    
    /// Get easing function category
    pub fn category(&self) -> EasingCategory {
        match self {
            EasingFunction::Linear => EasingCategory::Linear,
            EasingFunction::QuadIn | EasingFunction::QuadOut | EasingFunction::QuadInOut => EasingCategory::Quadratic,
            EasingFunction::CubicIn | EasingFunction::CubicOut | EasingFunction::CubicInOut => EasingCategory::Cubic,
            EasingFunction::QuartIn | EasingFunction::QuartOut | EasingFunction::QuartInOut => EasingCategory::Quartic,
            EasingFunction::QuintIn | EasingFunction::QuintOut | EasingFunction::QuintInOut => EasingCategory::Quintic,
            EasingFunction::SineIn | EasingFunction::SineOut | EasingFunction::SineInOut => EasingCategory::Sine,
            EasingFunction::ExpoIn | EasingFunction::ExpoOut | EasingFunction::ExpoInOut => EasingCategory::Exponential,
            EasingFunction::CircIn | EasingFunction::CircOut | EasingFunction::CircInOut => EasingCategory::Circular,
            EasingFunction::BackIn | EasingFunction::BackOut | EasingFunction::BackInOut => EasingCategory::Back,
            EasingFunction::ElasticIn | EasingFunction::ElasticOut | EasingFunction::ElasticInOut => EasingCategory::Elastic,
            EasingFunction::BounceIn | EasingFunction::BounceOut | EasingFunction::BounceInOut => EasingCategory::Bounce,
        }
    }
    
    /// Check if function is accelerating
    pub fn is_accelerating(&self) -> bool {
        matches!(self, 
            EasingFunction::QuadIn | EasingFunction::CubicIn | EasingFunction::QuartIn | 
            EasingFunction::QuintIn | EasingFunction::SineIn | EasingFunction::ExpoIn |
            EasingFunction::CircIn | EasingFunction::BackIn | EasingFunction::ElasticIn |
            EasingFunction::BounceIn
        )
    }
    
    /// Check if function is decelerating
    pub fn is_decelerating(&self) -> bool {
        matches!(self, 
            EasingFunction::QuadOut | EasingFunction::CubicOut | EasingFunction::QuartOut | 
            EasingFunction::QuintOut | EasingFunction::SineOut | EasingFunction::ExpoOut |
            EasingFunction::CircOut | EasingFunction::BackOut | EasingFunction::ElasticOut |
            EasingFunction::BounceOut
        )
    }
    
    /// Check if function is symmetric (in-out)
    pub fn is_symmetric(&self) -> bool {
        matches!(self, 
            EasingFunction::Linear | EasingFunction::QuadInOut | EasingFunction::CubicInOut |
            EasingFunction::QuartInOut | EasingFunction::QuintInOut | EasingFunction::SineInOut |
            EasingFunction::ExpoInOut | EasingFunction::CircInOut | EasingFunction::BackInOut |
            EasingFunction::ElasticInOut | EasingFunction::BounceInOut
        )
    }
}

impl fmt::Display for EasingFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Default for EasingFunction {
    fn default() -> Self {
        EasingFunction::Linear
    }
}

/// Easing function categories
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EasingCategory {
    Linear,
    Quadratic,
    Cubic,
    Quartic,
    Quintic,
    Sine,
    Exponential,
    Circular,
    Back,
    Elastic,
    Bounce,
}

impl EasingCategory {
    /// Get category name
    pub fn name(&self) -> &'static str {
        match self {
            EasingCategory::Linear => "Linear",
            EasingCategory::Quadratic => "Quadratic",
            EasingCategory::Cubic => "Cubic",
            EasingCategory::Quartic => "Quartic",
            EasingCategory::Quintic => "Quintic",
            EasingCategory::Sine => "Sine",
            EasingCategory::Exponential => "Exponential",
            EasingCategory::Circular => "Circular",
            EasingCategory::Back => "Back",
            EasingCategory::Elastic => "Elastic",
            EasingCategory::Bounce => "Bounce",
        }
    }
    
    /// Get all functions in this category
    pub fn functions(&self) -> Vec<EasingFunction> {
        match self {
            EasingCategory::Linear => vec![EasingFunction::Linear],
            EasingCategory::Quadratic => vec![EasingFunction::QuadIn, EasingFunction::QuadOut, EasingFunction::QuadInOut],
            EasingCategory::Cubic => vec![EasingFunction::CubicIn, EasingFunction::CubicOut, EasingFunction::CubicInOut],
            EasingCategory::Quartic => vec![EasingFunction::QuartIn, EasingFunction::QuartOut, EasingFunction::QuartInOut],
            EasingCategory::Quintic => vec![EasingFunction::QuintIn, EasingFunction::QuintOut, EasingFunction::QuintInOut],
            EasingCategory::Sine => vec![EasingFunction::SineIn, EasingFunction::SineOut, EasingFunction::SineInOut],
            EasingCategory::Exponential => vec![EasingFunction::ExpoIn, EasingFunction::ExpoOut, EasingFunction::ExpoInOut],
            EasingCategory::Circular => vec![EasingFunction::CircIn, EasingFunction::CircOut, EasingFunction::CircInOut],
            EasingCategory::Back => vec![EasingFunction::BackIn, EasingFunction::BackOut, EasingFunction::BackInOut],
            EasingCategory::Elastic => vec![EasingFunction::ElasticIn, EasingFunction::ElasticOut, EasingFunction::ElasticInOut],
            EasingCategory::Bounce => vec![EasingFunction::BounceIn, EasingFunction::BounceOut, EasingFunction::BounceInOut],
        }
    }
}

/// Interpolation utilities
pub struct InterpolationUtils;

impl InterpolationUtils {
    /// Linear interpolation between two values
    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }
    
    /// Linear interpolation between vectors
    pub fn lerp_vec2(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
        [
            Self::lerp(a[0], b[0], t),
            Self::lerp(a[1], b[1], t),
        ]
    }
    
    /// Linear interpolation between 3D vectors
    pub fn lerp_vec3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        [
            Self::lerp(a[0], b[0], t),
            Self::lerp(a[1], b[1], t),
            Self::lerp(a[2], b[2], t),
        ]
    }
    
    /// Linear interpolation between 4D vectors
    pub fn lerp_vec4(a: [f64; 4], b: [f64; 4], t: f64) -> [f64; 4] {
        [
            Self::lerp(a[0], b[0], t),
            Self::lerp(a[1], b[1], t),
            Self::lerp(a[2], b[2], t),
            Self::lerp(a[3], b[3], t),
        ]
    }
    
    /// Bezier interpolation with control points
    pub fn bezier_quadratic(p0: f64, p1: f64, p2: f64, t: f64) -> f64 {
        let mt = 1.0 - t;
        mt * mt * p0 + 2.0 * mt * t * p1 + t * t * p2
    }
    
    /// Cubic bezier interpolation
    pub fn bezier_cubic(p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
        let mt = 1.0 - t;
        mt * mt * mt * p0 + 3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t * p3
    }
    
    /// Apply easing to interpolation
    pub fn eased_lerp(a: f64, b: f64, t: f64, easing: EasingFunction) -> f64 {
        let eased_t = easing.apply(t);
        Self::lerp(a, b, eased_t)
    }
    
    /// Clamp value to range
    pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
        value.max(min).min(max)
    }
    
    /// Remap value from one range to another
    pub fn remap(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
        let t = (value - from_min) / (from_max - from_min);
        Self::lerp(to_min, to_max, t)
    }
    
    /// Smooth step function
    pub fn smooth_step(edge0: f64, edge1: f64, x: f64) -> f64 {
        let t = Self::clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    
    /// Get all easing functions
    pub fn all_easing_functions() -> Vec<EasingFunction> {
        vec![
            EasingFunction::Linear,
            EasingFunction::QuadIn, EasingFunction::QuadOut, EasingFunction::QuadInOut,
            EasingFunction::CubicIn, EasingFunction::CubicOut, EasingFunction::CubicInOut,
            EasingFunction::QuartIn, EasingFunction::QuartOut, EasingFunction::QuartInOut,
            EasingFunction::QuintIn, EasingFunction::QuintOut, EasingFunction::QuintInOut,
            EasingFunction::SineIn, EasingFunction::SineOut, EasingFunction::SineInOut,
            EasingFunction::ExpoIn, EasingFunction::ExpoOut, EasingFunction::ExpoInOut,
            EasingFunction::CircIn, EasingFunction::CircOut, EasingFunction::CircInOut,
            EasingFunction::BackIn, EasingFunction::BackOut, EasingFunction::BackInOut,
            EasingFunction::ElasticIn, EasingFunction::ElasticOut, EasingFunction::ElasticInOut,
            EasingFunction::BounceIn, EasingFunction::BounceOut, EasingFunction::BounceInOut,
        ]
    }
    
    /// Get easing functions by category
    pub fn easing_functions_by_category() -> std::collections::HashMap<EasingCategory, Vec<EasingFunction>> {
        let mut map = std::collections::HashMap::new();
        
        for category in [
            EasingCategory::Linear, EasingCategory::Quadratic, EasingCategory::Cubic,
            EasingCategory::Quartic, EasingCategory::Quintic, EasingCategory::Sine,
            EasingCategory::Exponential, EasingCategory::Circular, EasingCategory::Back,
            EasingCategory::Elastic, EasingCategory::Bounce,
        ] {
            map.insert(category, category.functions());
        }
        
        map
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
    fn test_easing_functions() {
        // Test linear easing
        assert_eq!(EasingFunction::Linear.apply(0.0), 0.0);
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert_eq!(EasingFunction::Linear.apply(1.0), 1.0);
        
        // Test quadratic easing
        let quad_in = EasingFunction::QuadIn.apply(0.5);
        let quad_out = EasingFunction::QuadOut.apply(0.5);
        assert!(quad_in < 0.5); // Accelerating
        assert!(quad_out > 0.5); // Decelerating
        
        // Test bounds
        for easing in InterpolationUtils::all_easing_functions() {
            assert!(easing.apply(0.0) >= 0.0);
            assert!(easing.apply(1.0) <= 1.0);
        }
    }
    
    #[test]
    fn test_easing_categories() {
        let quad_functions = EasingCategory::Quadratic.functions();
        assert_eq!(quad_functions.len(), 3);
        assert!(quad_functions.contains(&EasingFunction::QuadIn));
        assert!(quad_functions.contains(&EasingFunction::QuadOut));
        assert!(quad_functions.contains(&EasingFunction::QuadInOut));
        
        assert!(EasingFunction::QuadIn.is_accelerating());
        assert!(EasingFunction::QuadOut.is_decelerating());
        assert!(EasingFunction::QuadInOut.is_symmetric());
    }
    
    #[test]
    fn test_interpolation_utils() {
        // Test linear interpolation
        assert_eq!(InterpolationUtils::lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(InterpolationUtils::lerp(10.0, 20.0, 0.25), 12.5);
        
        // Test vector interpolation
        let a = [0.0, 1.0, 2.0];
        let b = [10.0, 11.0, 12.0];
        let result = InterpolationUtils::lerp_vec3(a, b, 0.5);
        assert_eq!(result, [5.0, 6.0, 7.0]);
        
        // Test bezier interpolation
        let bezier_result = InterpolationUtils::bezier_quadratic(0.0, 5.0, 10.0, 0.5);
        assert_eq!(bezier_result, 5.0);
        
        // Test clamp
        assert_eq!(InterpolationUtils::clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(InterpolationUtils::clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(InterpolationUtils::clamp(15.0, 0.0, 10.0), 10.0);
        
        // Test remap
        assert_eq!(InterpolationUtils::remap(5.0, 0.0, 10.0, 100.0, 200.0), 150.0);
        
        // Test smooth step
        let smooth = InterpolationUtils::smooth_step(0.0, 1.0, 0.5);
        assert!(smooth > 0.0 && smooth < 1.0);
    }
    
    #[test]
    fn test_eased_lerp() {
        let result = InterpolationUtils::eased_lerp(0.0, 10.0, 0.5, EasingFunction::QuadIn);
        assert!(result < 5.0); // Should be less than linear due to acceleration
        
        let result = InterpolationUtils::eased_lerp(0.0, 10.0, 0.5, EasingFunction::QuadOut);
        assert!(result > 5.0); // Should be more than linear due to deceleration
    }
    
    #[test]
    fn test_complex_easing_functions() {
        // Test elastic easing
        let elastic_out = EasingFunction::ElasticOut.apply(0.5);
        assert!(elastic_out > 0.0 && elastic_out <= 1.0);
        
        // Test bounce easing
        let bounce_out = EasingFunction::BounceOut.apply(0.5);
        assert!(bounce_out >= 0.0 && bounce_out <= 1.0);
        
        // Test back easing
        let back_out = EasingFunction::BackOut.apply(0.5);
        assert!(back_out >= 0.0 && back_out <= 1.0);
    }
    
    #[test]
    fn test_easing_functions_by_category() {
        let categories = InterpolationUtils::easing_functions_by_category();
        
        assert_eq!(categories.get(&EasingCategory::Linear).unwrap().len(), 1);
        assert_eq!(categories.get(&EasingCategory::Quadratic).unwrap().len(), 3);
        assert_eq!(categories.get(&EasingCategory::Cubic).unwrap().len(), 3);
        
        // Test that all functions are categorized
        let total_categorized: usize = categories.values().map(|funcs| funcs.len()).sum();
        let total_functions = InterpolationUtils::all_easing_functions().len();
        assert_eq!(total_categorized, total_functions);
    }
}
