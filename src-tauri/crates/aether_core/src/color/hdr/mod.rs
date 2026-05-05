//! HDR Support Module
//! 
//! This module provides comprehensive HDR (High Dynamic Range) support including
//! HDR display management, tone mapping algorithms, and gamut mapping for professional
//! video production workflows.

pub mod processor;
pub mod types;
pub mod config;
pub mod display;
pub mod tone;
pub mod gamut;
pub mod analysis;

// Re-export main HDR types
pub use processor::HdrProcessor;
pub use config::HdrConfig;
pub use types::{HdrImage, HdrPixel, HdrDisplayType, ColorPrimaries, TransferFunction};
pub use tone::{ToneMapper, ToneMappingAlgorithm};
pub use gamut::{GamutMapper, GamutMappingAlgorithm};
pub use analysis::{HdrAnalyzer, HdrAnalysis, HdrContentType, HdrQualityMetrics, HdrIssue};
