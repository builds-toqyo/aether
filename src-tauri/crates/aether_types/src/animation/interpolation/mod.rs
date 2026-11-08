pub mod easing;
pub mod methods;
pub mod utils;

pub use easing::{EasingCategory, EasingFunction};
pub use methods::{BasicInterpolation, InterpolationMethod};
pub use utils::{
    EasingComparison, EasingFunctionsByCharacteristic, EasingUseCase, InterpolationUtils,
};
