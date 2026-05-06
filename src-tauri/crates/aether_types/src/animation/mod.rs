//! Animation system data structures
//! 
//! This module provides comprehensive animation data structures for
//! keyframe-based animation, interpolation, and parameter binding
//! in the Aether video editor.

pub mod keyframe;
pub mod curve;
pub mod track;
pub mod interpolation;

// Re-export main animation types
pub use keyframe::{Keyframe, KeyframeData, TransformData, KeyframeCollection};
pub use curve::{AnimationCurve, CurveType, BezierControlPoint, CurveBuilder};
pub use track::{
    AnimationTrack, TrackType, TrackValue, ParameterBinding, BindingType, 
    AnimationTrackCollection, TrackCollectionStats, AnimationTrackBuilder, AnimationTrackCollectionBuilder,
    TrackValueUtils
};
pub use interpolation::{
    InterpolationMethod, EasingFunction, EasingCategory, InterpolationUtils,
    EasingFunctionsByCharacteristic, EasingUseCase, EasingComparison, BasicInterpolation
};
