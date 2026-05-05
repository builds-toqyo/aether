//! Color management module
//! 
//! This module provides comprehensive color management capabilities including
//! ACES color pipeline, color space conversions, and professional color grading tools.

pub mod aces;

// Re-export color management types
pub use aces::{AcesProcessor, AcesConfig, InputTransform, OutputTransform, LookTransform};
