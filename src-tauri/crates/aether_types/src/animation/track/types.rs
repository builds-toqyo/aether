//! Animation track type definitions
//! 
//! This module contains the core type definitions for animation tracks
//! including track types, values, and binding information.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Animation track types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrackType {
    /// Position animation (x, y, z)
    Position,
    /// Rotation animation (x, y, z)
    Rotation,
    /// Scale animation (x, y, z)
    Scale,
    /// Opacity animation
    Opacity,
    /// Color animation (r, g, b, a)
    Color,
    /// Custom property animation
    Custom,
}

impl TrackType {
    /// Get track type name
    pub fn name(&self) -> &'static str {
        match self {
            TrackType::Position => "Position",
            TrackType::Rotation => "Rotation",
            TrackType::Scale => "Scale",
            TrackType::Opacity => "Opacity",
            TrackType::Color => "Color",
            TrackType::Custom => "Custom",
        }
    }
    
    /// Get default value for track type
    pub fn default_value(&self) -> super::value::TrackValue {
        match self {
            TrackType::Position => super::value::TrackValue::Vector3([0.0, 0.0, 0.0]),
            TrackType::Rotation => super::value::TrackValue::Vector3([0.0, 0.0, 0.0]),
            TrackType::Scale => super::value::TrackValue::Vector3([1.0, 1.0, 1.0]),
            TrackType::Opacity => super::value::TrackValue::Float(1.0),
            TrackType::Color => super::value::TrackValue::Color([1.0, 1.0, 1.0, 1.0]),
            TrackType::Custom => super::value::TrackValue::Float(0.0),
        }
    }
    
    /// Get value dimension for track type
    pub fn dimension(&self) -> usize {
        match self {
            TrackType::Position => 3,
            TrackType::Rotation => 3,
            TrackType::Scale => 3,
            TrackType::Opacity => 1,
            TrackType::Color => 4,
            TrackType::Custom => 1,
        }
    }
}

impl fmt::Display for TrackType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Binding types for parameters
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BindingType {
    /// Direct value binding
    Direct,
    /// Additive binding (value is added to current)
    Additive,
    /// Multiplicative binding (value is multiplied with current)
    Multiplicative,
    /// Override binding (value replaces current)
    Override,
}

impl BindingType {
    /// Get binding type name
    pub fn name(&self) -> &'static str {
        match self {
            BindingType::Direct => "Direct",
            BindingType::Additive => "Additive",
            BindingType::Multiplicative => "Multiplicative",
            BindingType::Override => "Override",
        }
    }
}

impl fmt::Display for BindingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Parameter binding for animation tracks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterBinding {
    /// Target object ID
    pub target_id: String,
    /// Parameter name
    pub parameter_name: String,
    /// Parameter path (for nested properties)
    pub parameter_path: Vec<String>,
    /// Binding type
    pub binding_type: BindingType,
}

impl ParameterBinding {
    /// Create new parameter binding
    pub fn new(target_id: String, parameter_name: String) -> Self {
        Self {
            target_id,
            parameter_name,
            parameter_path: Vec::new(),
            binding_type: BindingType::Direct,
        }
    }
    
    /// Create binding with path
    pub fn with_path(mut self, path: Vec<String>) -> Self {
        self.parameter_path = path;
        self
    }
    
    /// Create binding with type
    pub fn with_binding_type(mut self, binding_type: BindingType) -> Self {
        self.binding_type = binding_type;
        self
    }
    
    /// Get full parameter path
    pub fn full_path(&self) -> String {
        if self.parameter_path.is_empty() {
            self.parameter_name.clone()
        } else {
            format!("{}.{}", self.parameter_path.join("."), self.parameter_name)
        }
    }
    
    /// Validate binding
    pub fn validate(&self) -> Result<(), String> {
        if self.target_id.is_empty() {
            return Err("Target ID cannot be empty".to_string());
        }
        
        if self.parameter_name.is_empty() {
            return Err("Parameter name cannot be empty".to_string());
        }
        
        Ok(())
    }
}
