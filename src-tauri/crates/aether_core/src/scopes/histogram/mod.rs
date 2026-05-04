//! Histogram module for professional color analysis
//! 
//! This module provides a complete histogram system with RGB channel histograms,
//! luma histogram, and real-time analysis capabilities similar to DaVinci Resolve.

pub mod processor;
pub mod rendering;
pub mod analysis;

// Re-export main histogram types
pub use processor::HistogramProcessor;
pub use analysis::{HistogramAnalyzer, HistogramStatistics, ExposureAnalysis, ColorBalanceAnalysis, ColorCast, HistogramIssue};
pub use rendering::HistogramRenderer;
