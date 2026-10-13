pub mod engine;
pub mod interpolation;

pub use engine::{AnimationEngine, AnimationState, PlaybackState};
pub use interpolation::{AnimationInterpolator, InterpolationResult};

pub use aether_types::animation::{EasingFunction, InterpolationMethod};
