

pub mod methods;
pub mod easing;
pub mod utils;


pub use methods::{InterpolationMethod, BasicInterpolation};
pub use easing::{EasingFunction, EasingCategory};
pub use utils::{InterpolationUtils, EasingFunctionsByCharacteristic, EasingUseCase, EasingComparison};
