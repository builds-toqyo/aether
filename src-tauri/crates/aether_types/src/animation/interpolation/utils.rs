

use std::collections::HashMap;

use super::methods::{InterpolationMethod, BasicInterpolation};
use super::easing::{EasingFunction, EasingCategory};


pub struct InterpolationUtils;

impl InterpolationUtils {

    pub fn eased_lerp(a: f64, b: f64, t: f64, easing: EasingFunction) -> f64 {
        let eased_t = easing.apply(t);
        BasicInterpolation::lerp(a, b, eased_t)
    }


    pub fn eased_lerp_vec2(a: [f64; 2], b: [f64; 2], t: f64, easing: EasingFunction) -> [f64; 2] {
        let eased_t = easing.apply(t);
        BasicInterpolation::lerp_vec2(a, b, eased_t)
    }


    pub fn eased_lerp_vec3(a: [f64; 3], b: [f64; 3], t: f64, easing: EasingFunction) -> [f64; 3] {
        let eased_t = easing.apply(t);
        BasicInterpolation::lerp_vec3(a, b, eased_t)
    }


    pub fn eased_lerp_vec4(a: [f64; 4], b: [f64; 4], t: f64, easing: EasingFunction) -> [f64; 4] {
        let eased_t = easing.apply(t);
        BasicInterpolation::lerp_vec4(a, b, eased_t)
    }


    pub fn interpolate_with_easing(a: f64, b: f64, t: f64, method: InterpolationMethod, easing: EasingFunction) -> f64 {
        let eased_t = if method.is_smooth() { easing.apply(t) } else { t };
        BasicInterpolation::interpolate(a, b, eased_t, method)
    }


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


    pub fn easing_functions_by_category() -> HashMap<EasingCategory, Vec<EasingFunction>> {
        let mut map = HashMap::new();

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


    pub fn easing_functions_by_characteristic() -> EasingFunctionsByCharacteristic {
        let all_functions = Self::all_easing_functions();

        let accelerating: Vec<EasingFunction> = all_functions.iter()
            .filter(|f| f.is_accelerating())
            .copied()
            .collect();

        let decelerating: Vec<EasingFunction> = all_functions.iter()
            .filter(|f| f.is_decelerating())
            .copied()
            .collect();

        let symmetric: Vec<EasingFunction> = all_functions.iter()
            .filter(|f| f.is_symmetric())
            .copied()
            .collect();

        let smooth: Vec<EasingFunction> = all_functions.iter()
            .filter(|f| **f != EasingFunction::Linear)
            .copied()
            .collect();

        EasingFunctionsByCharacteristic {
            all: all_functions,
            accelerating,
            decelerating,
            symmetric,
            smooth,
        }
    }


    pub fn find_easing_by_name(name: &str) -> Option<EasingFunction> {
        match name.to_lowercase().as_str() {
            "linear" => Some(EasingFunction::Linear),
            "quadin" | "quadratic in" => Some(EasingFunction::QuadIn),
            "quadout" | "quadratic out" => Some(EasingFunction::QuadOut),
            "quadinout" | "quadratic inout" => Some(EasingFunction::QuadInOut),
            "cubicin" | "cubic in" => Some(EasingFunction::CubicIn),
            "cubicout" | "cubic out" => Some(EasingFunction::CubicOut),
            "cubicinout" | "cubic inout" => Some(EasingFunction::CubicInOut),
            "quartin" | "quartic in" => Some(EasingFunction::QuartIn),
            "quartout" | "quartic out" => Some(EasingFunction::QuartOut),
            "quartinout" | "quartic inout" => Some(EasingFunction::QuartInOut),
            "quintin" | "quintic in" => Some(EasingFunction::QuintIn),
            "quintout" | "quintic out" => Some(EasingFunction::QuintOut),
            "quintinout" | "quintic inout" => Some(EasingFunction::QuintInOut),
            "sinein" | "sine in" => Some(EasingFunction::SineIn),
            "sineout" | "sine out" => Some(EasingFunction::SineOut),
            "sineinout" | "sine inout" => Some(EasingFunction::SineInOut),
            "expoin" | "exponential in" => Some(EasingFunction::ExpoIn),
            "expoout" | "exponential out" => Some(EasingFunction::ExpoOut),
            "expoinout" | "exponential inout" => Some(EasingFunction::ExpoInOut),
            "circin" | "circular in" => Some(EasingFunction::CircIn),
            "circout" | "circular out" => Some(EasingFunction::CircOut),
            "circinout" | "circular inout" => Some(EasingFunction::CircInOut),
            "backin" | "back in" => Some(EasingFunction::BackIn),
            "backout" | "back out" => Some(EasingFunction::BackOut),
            "backinout" | "back inout" => Some(EasingFunction::BackInOut),
            "elasticin" | "elastic in" => Some(EasingFunction::ElasticIn),
            "elasticout" | "elastic out" => Some(EasingFunction::ElasticOut),
            "elasticinout" | "elastic inout" => Some(EasingFunction::ElasticInOut),
            "bouncein" | "bounce in" => Some(EasingFunction::BounceIn),
            "bounceout" | "bounce out" => Some(EasingFunction::BounceOut),
            "bounceinout" | "bounce inout" => Some(EasingFunction::BounceInOut),
            _ => None,
        }
    }


    pub fn recommended_easing_for_use_case(use_case: EasingUseCase) -> Vec<EasingFunction> {
        match use_case {
            EasingUseCase::General => vec![
                EasingFunction::Linear,
                EasingFunction::QuadInOut,
                EasingFunction::CubicInOut,
            ],
            EasingUseCase::UIAnimation => vec![
                EasingFunction::QuadOut,
                EasingFunction::CubicOut,
                EasingFunction::BackOut,
            ],
            EasingUseCase::NaturalMotion => vec![
                EasingFunction::SineInOut,
                EasingFunction::CubicInOut,
                EasingFunction::ExpoInOut,
            ],
            EasingUseCase::BouncyEffect => vec![
                EasingFunction::BounceOut,
                EasingFunction::ElasticOut,
                EasingFunction::BackOut,
            ],
            EasingUseCase::DramaticEffect => vec![
                EasingFunction::ExpoIn,
                EasingFunction::BackIn,
                EasingFunction::ElasticIn,
            ],
        }
    }


    pub fn compare_easing_functions(a: EasingFunction, b: EasingFunction) -> EasingComparison {
        let similarity_score = Self::calculate_similarity(a, b);

        EasingComparison {
            function_a: a,
            function_b: b,
            similarity_score,
            shared_category: a.category() == b.category(),
            shared_characteristics: EasingCharacteristics {
                both_accelerating: a.is_accelerating() && b.is_accelerating(),
                both_decelerating: a.is_decelerating() && b.is_decelerating(),
                both_symmetric: a.is_symmetric() && b.is_symmetric(),
                both_smooth: a != EasingFunction::Linear && b != EasingFunction::Linear,
            },
        }
    }


    fn calculate_similarity(a: EasingFunction, b: EasingFunction) -> f64 {
        let mut score = 0.0;
        let max_score = 4.0;


        if a.category() == b.category() {
            score += 1.0;
        }


        if a.is_accelerating() == b.is_accelerating() {
            score += 0.5;
        }
        if a.is_decelerating() == b.is_decelerating() {
            score += 0.5;
        }
        if a.is_symmetric() == b.is_symmetric() {
            score += 0.5;
        }


        let mut visual_diff = 0.0;
        for i in 0..11 {
            let t = i as f64 / 10.0;
            let diff = (a.apply(t) - b.apply(t)).abs();
            visual_diff += diff;
        }
        visual_diff /= 11.0;


        let visual_similarity = 1.0 - visual_diff.min(1.0);
        score += visual_similarity * 1.5;

        score / max_score
    }
}


#[derive(Debug, Clone)]
pub struct EasingFunctionsByCharacteristic {

    pub all: Vec<EasingFunction>,

    pub accelerating: Vec<EasingFunction>,

    pub decelerating: Vec<EasingFunction>,

    pub symmetric: Vec<EasingFunction>,

    pub smooth: Vec<EasingFunction>,
}

impl EasingFunctionsByCharacteristic {

    pub fn get_by_characteristics(&self, accelerating: bool, decelerating: bool, symmetric: bool) -> Vec<EasingFunction> {
        self.all.iter()
            .filter(|f| {
                let matches_accelerating = !accelerating || f.is_accelerating();
                let matches_decelerating = !decelerating || f.is_decelerating();
                let matches_symmetric = !symmetric || f.is_symmetric();
                matches_accelerating && matches_decelerating && matches_symmetric
            })
            .copied()
            .collect()
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingUseCase {

    General,

    UIAnimation,

    NaturalMotion,

    BouncyEffect,

    DramaticEffect,
}

impl EasingUseCase {

    pub fn name(&self) -> &'static str {
        match self {
            EasingUseCase::General => __STRING_61__,
            EasingUseCase::UIAnimation => __STRING_62__,
            EasingUseCase::NaturalMotion => __STRING_63__,
            EasingUseCase::BouncyEffect => __STRING_64__,
            EasingUseCase::DramaticEffect => __STRING_65__,
        }
    }

    /// Get use case description
    pub fn description(&self) -> &'static str {
        match self {
            EasingUseCase::General => "General purpose animations",
            EasingUseCase::UIAnimation => "User interface animations and transitions",
            EasingUseCase::NaturalMotion => "Natural motion simulation",
            EasingUseCase::BouncyEffect => "Bouncy and playful effects",
            EasingUseCase::DramaticEffect => "Dramatic and impactful effects",
        }
    }
}


#[derive(Debug, Clone)]
pub struct EasingComparison {

    pub function_a: EasingFunction,

    pub function_b: EasingFunction,

    pub similarity_score: f64,

    pub shared_category: bool,

    pub shared_characteristics: EasingCharacteristics,
}


#[derive(Debug, Clone, Copy)]
pub struct EasingCharacteristics {

    pub both_accelerating: bool,

    pub both_decelerating: bool,

    pub both_symmetric: bool,

    pub both_smooth: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eased_lerp() {
        let result = InterpolationUtils::eased_lerp(0.0, 10.0, 0.5, EasingFunction::QuadIn);
        assert!(result < 5.0);

        let result = InterpolationUtils::eased_lerp(0.0, 10.0, 0.5, EasingFunction::QuadOut);
        assert!(result > 5.0);
    }

    #[test]
    fn test_all_easing_functions() {
        let functions = InterpolationUtils::all_easing_functions();
        assert_eq!(functions.len(), 31);


        let mut unique_functions = functions.clone();
        unique_functions.sort();
        unique_functions.dedup();
        assert_eq!(unique_functions.len(), functions.len());
    }

    #[test]
    fn test_easing_functions_by_category() {
        let categories = InterpolationUtils::easing_functions_by_category();

        assert_eq!(categories.get(&EasingCategory::Linear).unwrap().len(), 1);
        assert_eq!(categories.get(&EasingCategory::Quadratic).unwrap().len(), 3);
        assert_eq!(categories.get(&EasingCategory::Cubic).unwrap().len(), 3);


        let total_categorized: usize = categories.values().map(|funcs| funcs.len()).sum();
        let total_functions = InterpolationUtils::all_easing_functions().len();
        assert_eq!(total_categorized, total_functions);
    }

    #[test]
    fn test_find_easing_by_name() {
        assert_eq!(InterpolationUtils::find_easing_by_name("linear"), Some(EasingFunction::Linear));
        assert_eq!(InterpolationUtils::find_easing_by_name("quadin"), Some(EasingFunction::QuadIn));
        assert_eq!(InterpolationUtils::find_easing_by_name("cubic out"), Some(EasingFunction::CubicOut));
        assert_eq!(InterpolationUtils::find_easing_by_name("bounceinout"), Some(EasingFunction::BounceInOut));
        assert_eq!(InterpolationUtils::find_easing_by_name("nonexistent"), None);
    }

    #[test]
    fn test_recommended_easing_for_use_case() {
        let ui_animations = InterpolationUtils::recommended_easing_for_use_case(EasingUseCase::UIAnimation);
        assert!(ui_animations.contains(&EasingFunction::QuadOut));
        assert!(ui_animations.contains(&EasingFunction::CubicOut));

        let bouncy_effects = InterpolationUtils::recommended_easing_for_use_case(EasingUseCase::BouncyEffect);
        assert!(bouncy_effects.contains(&EasingFunction::BounceOut));
        assert!(bouncy_effects.contains(&EasingFunction::ElasticOut));
    }

    #[test]
    fn test_easing_functions_by_characteristic() {
        let by_char = InterpolationUtils::easing_functions_by_characteristic();

        assert!(!by_char.accelerating.is_empty());
        assert!(!by_char.decelerating.is_empty());
        assert!(!by_char.symmetric.is_empty());
        assert!(!by_char.smooth.is_empty());


        let symmetric_accelerating = by_char.get_by_characteristics(true, false, true);
        assert!(!symmetric_accelerating.is_empty());


        assert!(!by_char.smooth.contains(&EasingFunction::Linear));
    }

    #[test]
    fn test_easing_comparison() {
        let comparison = InterpolationUtils::compare_easing_functions(
            EasingFunction::QuadIn,
            EasingFunction::QuadOut
        );

        assert_eq!(comparison.function_a, EasingFunction::QuadIn);
        assert_eq!(comparison.function_b, EasingFunction::QuadOut);
        assert!(comparison.shared_category);
        assert!(!comparison.shared_characteristics.both_accelerating);
        assert!(!comparison.shared_characteristics.both_decelerating);
        assert!(!comparison.shared_characteristics.both_symmetric);


        let identical = InterpolationUtils::compare_easing_functions(
            EasingFunction::Linear,
            EasingFunction::Linear
        );
        assert!(identical.similarity_score > 0.9);
    }

    #[test]
    fn test_use_cases() {
        assert_eq!(EasingUseCase::UIAnimation.name(), "UI Animation");
        assert!(EasingUseCase::UIAnimation.description().contains("User interface"));

        assert_eq!(EasingUseCase::BouncyEffect.name(), "Bouncy Effect");
        assert!(EasingUseCase::BouncyEffect.description().contains("playful"));
    }
}
