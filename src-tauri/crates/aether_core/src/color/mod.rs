//! Color management module
//! 
//! This module provides comprehensive color management capabilities including
//! ACES color pipeline, HDR support, color space conversions, and professional color grading tools.

pub mod aces;
pub mod hdr;

// Re-export color management types
pub use aces::{AcesProcessor, AcesConfig, InputTransform, OutputTransform, LookTransform};
pub use hdr::{HdrProcessor, HdrConfig, HdrImage, HdrPixel, HdrDisplayType, ToneMappingAlgorithm, GamutMappingAlgorithm};
