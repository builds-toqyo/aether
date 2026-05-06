//! Interpolation module
//! 
//! This module provides comprehensive interpolation methods and easing
//! functions for smooth animation transitions.

pub mod methods;
pub mod easing;
pub mod utils;

// Re-export main interpolation types
pub use methods::{InterpolationMethod, BasicInterpolation};
pub use easing::{EasingFunction, EasingCategory};
pub use utils::{InterpolationUtils, EasingFunctionsByCharacteristic, EasingUseCase, EasingComparison};
