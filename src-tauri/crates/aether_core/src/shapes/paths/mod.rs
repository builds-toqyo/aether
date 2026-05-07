//! Path and bezier curve operations
//! 
//! This module provides path building and manipulation capabilities
//! including bezier curves, path segments, and path operations.

pub mod segments;
pub mod builder;
pub mod bezier;
pub mod path;

// Re-export main path types
pub use segments::{PathSegment, PathSegmentType};
pub use builder::PathBuilder;
pub use bezier::{BezierCurve, BezierUtils};
pub use path::Path;
