//! Vectorscope module for professional color analysis
//! 
//! This module provides a complete vectorscope system with UV coordinate plotting,
//! color target overlays, and real-time analysis capabilities similar to DaVinci Resolve.

pub mod processor;
pub mod targets;
pub mod rendering;
pub mod analysis;

// Re-export main vectorscope types
pub use processor::VectorscopeProcessor;
pub use analysis::{VectorscopeAnalyzer, ColorDistribution, TargetCompliance, ColorBalance, TargetResult};
pub use targets::TargetRenderer;
pub use rendering::VectorscopeRenderer;
