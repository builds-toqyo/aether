

use serde::{Deserialize, Serialize};
use std::fmt;


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrackType {

    Position,

    Rotation,

    Scale,

    Opacity,

    Color,

    Custom,
}

impl TrackType {

    pub fn name(&self) -> &'static str {
        match self {
            TrackType::Position => "position",
            TrackType::Rotation => "rotation",
            TrackType::Scale => "scale",
            TrackType::Opacity => "opacity",
            TrackType::Color => "color",
            TrackType::Custom => "custom",
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


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BindingType {

    Direct,

    Additive,

    Multiplicative,

    Override,
}

impl BindingType {

    pub fn name(&self) -> &'static str {
        match self {
            BindingType::Direct => "direct",
            BindingType::Additive => "additive",
            BindingType::Multiplicative => "multiplicative",
            BindingType::Override => "override",
        }
    }
}

impl fmt::Display for BindingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterBinding {

    pub target_id: String,

    pub parameter_name: String,

    pub parameter_path: Vec<String>,

    pub binding_type: BindingType,
}

impl ParameterBinding {

    pub fn new(target_id: String, parameter_name: String) -> Self {
        Self {
            target_id,
            parameter_name,
            parameter_path: Vec::new(),
            binding_type: BindingType::Direct,
        }
    }


    pub fn with_path(mut self, path: Vec<String>) -> Self {
        self.parameter_path = path;
        self
    }


    pub fn with_binding_type(mut self, binding_type: BindingType) -> Self {
        self.binding_type = binding_type;
        self
    }


    pub fn full_path(&self) -> String {
        if self.parameter_path.is_empty() {
            self.parameter_name.clone()
        } else {
            format!("{}.{}", self.parameter_path.join("."), self.parameter_name)
        }
    }


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
