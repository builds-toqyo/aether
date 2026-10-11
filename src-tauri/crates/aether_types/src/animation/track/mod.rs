#[allow(clippy::module_inception)]
pub mod track;

pub mod collection;
pub mod types;
pub mod value;

pub use collection::{
    AnimationTrackCollection, AnimationTrackCollectionBuilder, TrackCollectionStats,
};
pub use track::{AnimationTrack, AnimationTrackBuilder};
pub use types::{BindingType, ParameterBinding, TrackType};
pub use value::{TrackValue, TrackValueUtils};
