//! ACES Color Pipeline Module
//! 
//! This module provides a complete ACES (Academy Color Encoding System) implementation
//! for professional color grading and color management, supporting input transforms,
//! output transforms, and look modifications.

pub mod processor;
pub mod transforms;
pub mod looks;
pub mod config;
pub mod gamma;

// Re-export main ACES types
pub use processor::AcesProcessor;
pub use config::AcesConfig;
pub use transforms::{InputTransform, OutputTransform};
pub use looks::LookTransform;
