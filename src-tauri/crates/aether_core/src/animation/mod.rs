//! Animation system for aether_core
//! 
//! This module provides the runtime animation engine that uses the
//! animation data structures from aether_types to evaluate and play
//! animations in real-time.

pub mod interpolation;
pub mod engine;

// Re-export main animation types
pub use interpolation::{AnimationInterpolator, InterpolationResult};
pub use engine::{AnimationEngine, AnimationState, PlaybackState};
