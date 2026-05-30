

use serde::{Deserialize, Serialize};
use std::fmt;


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EasingFunction {
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    QuintIn,
    QuintOut,
    QuintInOut,
    SineIn,
    SineOut,
    SineInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    CircIn,
    CircOut,
    CircInOut,
    BackIn,
    BackOut,
    BackInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl EasingFunction {

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

    /// Get function description
    pub fn description(&self) -> &'static str {
        match self {
            EasingFunction::Linear => "Constant speed interpolation",
            EasingFunction::QuadIn => "Quadratic acceleration (t²)",
            EasingFunction::QuadOut => "Quadratic deceleration",
            EasingFunction::QuadInOut => "Quadratic acceleration then deceleration",
            EasingFunction::CubicIn => "Cubic acceleration (t³)",
            EasingFunction::CubicOut => "Cubic deceleration",
            EasingFunction::CubicInOut => "Cubic acceleration then deceleration",
            EasingFunction::QuartIn => "Quartic acceleration (t⁴)",
            EasingFunction::QuartOut => "Quartic deceleration",
            EasingFunction::QuartInOut => "Quartic acceleration then deceleration",
            EasingFunction::QuintIn => "Quintic acceleration (t⁵)",
            EasingFunction::QuintOut => "Quintic deceleration",
            EasingFunction::QuintInOut => "Quintic acceleration then deceleration",
            EasingFunction::SineIn => "Sinusoidal acceleration",
            EasingFunction::SineOut => "Sinusoidal deceleration",
            EasingFunction::SineInOut => "Sinusoidal acceleration then deceleration",
            EasingFunction::ExpoIn => "Exponential acceleration",
            EasingFunction::ExpoOut => "Exponential deceleration",
            EasingFunction::ExpoInOut => "Exponential acceleration then deceleration",
            EasingFunction::CircIn => "Circular arc acceleration",
            EasingFunction::CircOut => "Circular arc deceleration",
            EasingFunction::CircInOut => "Circular arc acceleration then deceleration",
            EasingFunction::BackIn => "Overshoot acceleration",
            EasingFunction::BackOut => "Overshoot deceleration",
            EasingFunction::BackInOut => "Overshoot acceleration then deceleration",
            EasingFunction::ElasticIn => "Spring-like acceleration",
            EasingFunction::ElasticOut => "Spring-like deceleration",
            EasingFunction::ElasticInOut => "Spring-like acceleration then deceleration",
            EasingFunction::BounceIn => "Gravity bounce acceleration",
            EasingFunction::BounceOut => "Gravity bounce deceleration",
            EasingFunction::BounceInOut => "Gravity bounce acceleration then deceleration",
        }
    }
}

impl fmt::Display for EasingFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EasingFunction({})", self.name())
    }
}

impl Default for EasingFunction {
    fn default() -> Self {
        EasingFunction::Linear
    }
}

/// Easing function categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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


    pub fn description(&self) -> &'static str {
        match self {
            EasingCategory::Linear => "Linear interpolation",
            EasingCategory::Quadratic => "Quadratic power functions",
            EasingCategory::Cubic => "Cubic power functions",
            EasingCategory::Quartic => "Quartic power functions",
            EasingCategory::Quintic => "Quintic power functions",
            EasingCategory::Sine => "Sine wave based easing",
            EasingCategory::Exponential => "Exponential easing",
            EasingCategory::Circular => "Circular arc based easing",
            EasingCategory::Back => "Overshoot and pullback",
            EasingCategory::Elastic => "Spring-like oscillation",
            EasingCategory::Bounce => "Gravity bounce effect",
        }
    }
}

impl fmt::Display for EasingCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_functions() {

        assert_eq!(EasingFunction::Linear.apply(0.0), 0.0);
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert_eq!(EasingFunction::Linear.apply(1.0), 1.0);


        let quad_in = EasingFunction::QuadIn.apply(0.5);
        let quad_out = EasingFunction::QuadOut.apply(0.5);
        assert!(quad_in < 0.5);
        assert!(quad_out > 0.5);


        for easing in all_easing_functions() {
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
    fn test_complex_easing_functions() {

        let elastic_out = EasingFunction::ElasticOut.apply(0.5);
        assert!(elastic_out > 0.0 && elastic_out <= 1.0);


        let bounce_out = EasingFunction::BounceOut.apply(0.5);
        assert!(bounce_out >= 0.0 && bounce_out <= 1.0);


        let back_out = EasingFunction::BackOut.apply(0.5);
        assert!(back_out >= 0.0 && back_out <= 1.0);
    }

    #[test]
    fn test_easing_function_descriptions() {
        assert!(EasingFunction::Linear.description().contains("Constant"));
        assert!(EasingFunction::QuadIn.description().contains("Quadratic"));
        assert!(EasingFunction::ElasticOut.description().contains("Spring"));
    }

    #[test]
    fn test_easing_category_descriptions() {
        assert!(EasingCategory::Linear.description().contains("Constant"));
        assert!(EasingCategory::Quadratic.description().contains("Quadratic"));
        assert!(EasingCategory::Elastic.description().contains("Spring"));
    }

    fn all_easing_functions() -> Vec<EasingFunction> {
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
}
