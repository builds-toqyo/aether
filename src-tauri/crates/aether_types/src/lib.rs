pub mod node_graph;
pub mod color;
pub mod animation;

pub use node_graph::{
    Node, Graph, Connection, InputPin, OutputPin, Parameter, ParameterValue, PinDataType,
    NodeType, BlendMode, KeyType, GeneratorType, ShapeType, FilterType, AdjustmentType,
};

pub use color::scopes::{
    ColorScopeData, WaveformData, VectorscopeData, HistogramData, ScopeMetadata,
    ScopeResolution, WaveformChannel, HistogramChannel, ColorSpace, VideoRange,
    WaveformConfig, VectorscopeConfig, HistogramConfig, ScopeStats,
};

pub use animation::{
    Keyframe, KeyframeData, TransformData, KeyframeCollection,
    AnimationCurve, CurveType, BezierControlPoint, CurveBuilder,
    AnimationTrack, TrackType, TrackValue, ParameterBinding, BindingType, 
    AnimationTrackCollection, TrackCollectionStats, AnimationTrackBuilder, AnimationTrackCollectionBuilder,
    TrackValueUtils,
    InterpolationMethod, EasingFunction, EasingCategory, InterpolationUtils,
    EasingFunctionsByCharacteristic, EasingUseCase, EasingComparison, BasicInterpolation,
};
