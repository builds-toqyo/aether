

pub mod interpolation;
pub mod engine;


pub use interpolation::{AnimationInterpolator, InterpolationResult};
pub use engine::{AnimationEngine, AnimationState, PlaybackState};

pub use aether_types::animation::{InterpolationMethod, EasingFunction};
