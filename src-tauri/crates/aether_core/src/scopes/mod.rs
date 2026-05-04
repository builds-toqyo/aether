//! Color scopes module for professional color analysis
//! 
//! This module provides comprehensive color scope implementations including
//! waveform monitors, vectorscopes, and histograms for professional color grading.

pub mod common;
pub mod vectorscope;
pub mod waveform;
pub mod histogram;

// Re-export common types
pub use common::{BaseScopeProcessor, ColorConverter, ImageRenderer, FrameProcessor, Statistics, ChannelStatistics};

// Re-export main scope types
pub use vectorscope::{VectorscopeProcessor, VectorscopeAnalyzer, ColorDistribution, TargetCompliance};
pub use waveform::WaveformProcessor;
pub use histogram::{HistogramProcessor, HistogramAnalyzer, HistogramStatistics, ExposureAnalysis, ColorBalanceAnalysis};
