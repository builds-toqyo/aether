pub mod curve;
pub mod interpolation;
pub mod keyframe;
pub mod track;

pub use curve::{AnimationCurve, BezierControlPoint, CurveBuilder, CurveType};
pub use interpolation::{
    BasicInterpolation, EasingCategory, EasingComparison, EasingFunction,
    EasingFunctionsByCharacteristic, EasingUseCase, InterpolationMethod, InterpolationUtils,
};
pub use keyframe::{Keyframe, KeyframeCollection, KeyframeData, TransformData};
pub use track::{
    AnimationTrack, AnimationTrackBuilder, AnimationTrackCollection,
    AnimationTrackCollectionBuilder, BindingType, ParameterBinding, TrackCollectionStats,
    TrackType, TrackValue, TrackValueUtils,
};

pub type AnimationValue = TrackValue;
