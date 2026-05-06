//! Animation track module
//! 
//! This module provides comprehensive animation track functionality
//! including track types, values, management, and collections.

pub mod types;
pub mod value;
pub mod track;
pub mod collection;

// Re-export main track types
pub use types::{TrackType, BindingType, ParameterBinding};
pub use value::{TrackValue, TrackValueUtils};
pub use track::{AnimationTrack, AnimationTrackBuilder};
pub use collection::{AnimationTrackCollection, TrackCollectionStats, AnimationTrackCollectionBuilder};
