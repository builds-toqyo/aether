

pub mod keyframe;
pub mod curve;
pub mod track;
pub mod interpolation;


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

pub type AnimationValue = TrackValue;
