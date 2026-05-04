//! Color grading and analysis types for Aether
//! 
//! This module provides comprehensive data structures for color grading,
//! color analysis, and professional color tools implementation.

pub mod scopes;

// Re-export scope types for convenience
pub use scopes::{
    ColorScopeData, WaveformData, VectorscopeData, HistogramData, ScopeMetadata,
    ScopeResolution, WaveformChannel, HistogramChannel, ColorSpace, VideoRange,
    WaveformConfig, VectorscopeConfig, HistogramConfig, ScopeStats,
    VectorscopePoint, WaveformMode, WaveformScale, VectorscopeTarget, 
    VectorscopeScale, HistogramMode,
};
