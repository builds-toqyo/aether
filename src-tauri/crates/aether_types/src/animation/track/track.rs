//! Animation track implementation
//! 
//! This module contains the main AnimationTrack struct and its core
//! functionality for managing keyframes and parameter binding.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;

use super::types::{TrackType, ParameterBinding, BindingType};
use super::value::TrackValue;
use crate::animation::keyframe::{KeyframeData, KeyframeCollection};

/// Animation track containing keyframes and binding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTrack {
    /// Track ID
    pub id: String,
    /// Track name
    pub name: String,
    /// Track type
    pub track_type: TrackType,
    /// Keyframes for this track
    pub keyframes: Vec<KeyframeData>,
    /// Parameter binding
    pub binding: ParameterBinding,
    /// Track enabled state
    pub enabled: bool,
    /// Track muted state
    pub muted: bool,
    /// Track weight (for blending)
    pub weight: f64,
}

impl AnimationTrack {
    /// Create new animation track
    pub fn new(id: String, name: String, track_type: TrackType, binding: ParameterBinding) -> Self {
        Self {
            id,
            name,
            track_type,
            keyframes: Vec::new(),
            binding,
            enabled: true,
            muted: false,
            weight: 1.0,
        }
    }
    
    /// Create track with default binding
    pub fn with_default_binding(id: String, name: String, track_type: TrackType, target_id: String) -> Self {
        let binding = ParameterBinding::new(target_id, track_type.name().to_lowercase());
        Self::new(id, name, track_type, binding)
    }
    
    /// Add keyframe to track
    pub fn add_keyframe(&mut self, keyframe: KeyframeData) {
        self.keyframes.push(keyframe);
        self.sort_keyframes();
    }
    
    /// Remove keyframe at index
    pub fn remove_keyframe(&mut self, index: usize) -> Option<KeyframeData> {
        if index < self.keyframes.len() {
            Some(self.keyframes.remove(index))
        } else {
            None
        }
    }
    
    /// Get keyframe at index
    pub fn get_keyframe(&self, index: usize) -> Option<&KeyframeData> {
        self.keyframes.get(index)
    }
    
    /// Get mutable keyframe at index
    pub fn get_keyframe_mut(&mut self, index: usize) -> Option<&mut KeyframeData> {
        self.keyframes.get_mut(index)
    }
    
    /// Clear all keyframes
    pub fn clear_keyframes(&mut self) {
        self.keyframes.clear();
    }
    
    /// Get number of keyframes
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }
    
    /// Sort keyframes by time
    pub fn sort_keyframes(&mut self) {
        self.keyframes.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Set muted state
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }
    
    /// Set weight
    pub fn set_weight(&mut self, weight: f64) {
        self.weight = weight.clamp(0.0, 1.0);
    }
    
    /// Get value at time
    pub fn get_value_at_time(&self, time: f64) -> Option<TrackValue> {
        if !self.enabled || self.muted || self.keyframes.is_empty() {
            return None;
        }
        
        // Find surrounding keyframes
        let (prev_keyframe, next_keyframe) = KeyframeCollection::find_surrounding_keyframes(&self.keyframes, time);
        
        match (prev_keyframe, next_keyframe) {
            (Some(prev), Some(next)) => {
                if prev.time() == next.time() {
                    // Same time, return prev value
                    self.keyframe_to_track_value(prev)
                } else {
                    // Interpolate between keyframes
                    let t = (time - prev.time()) / (next.time() - prev.time());
                    let prev_value = self.keyframe_to_track_value(prev)?;
                    let next_value = self.keyframe_to_track_value(next)?;
                    
                    prev_value.interpolate(&next_value, t).ok()
                }
            }
            (Some(prev), None) => {
                // After last keyframe, return last value
                self.keyframe_to_track_value(prev)
            }
            (None, Some(next)) => {
                // Before first keyframe, return first value
                self.keyframe_to_track_value(next)
            }
            (None, None) => None,
        }
    }
    
    /// Convert keyframe to track value
    fn keyframe_to_track_value(&self, keyframe: &KeyframeData) -> Option<TrackValue> {
        match keyframe {
            KeyframeData::Float(k) => Some(TrackValue::Float(k.value)),
            KeyframeData::Vector2(k) => Some(TrackValue::Vector2(k.value)),
            KeyframeData::Vector3(k) => Some(TrackValue::Vector3(k.value)),
            KeyframeData::Vector4(k) => Some(TrackValue::Vector4(k.value)),
            KeyframeData::Color(k) => Some(TrackValue::Color(k.value)),
            KeyframeData::Transform(k) => {
                match self.track_type {
                    TrackType::Position => Some(TrackValue::Vector3(k.value.position)),
                    TrackType::Rotation => Some(TrackValue::Vector3(k.value.rotation)),
                    TrackType::Scale => Some(TrackValue::Vector3(k.value.scale)),
                    _ => None,
                }
            }
            KeyframeData::Boolean(k) => Some(TrackValue::Boolean(k.value)),
            KeyframeData::String(k) => Some(TrackValue::String(k.value.clone())),
        }
    }
    
    /// Validate track
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Track ID cannot be empty".to_string());
        }
        
        if self.name.is_empty() {
            return Err("Track name cannot be empty".to_string());
        }
        
        self.binding.validate()?;
        
        // Validate keyframes
        if self.keyframes.is_empty() {
            return Err("Track must have at least one keyframe".to_string());
        }
        
        // Check keyframe types match track type
        for keyframe in &self.keyframes {
            let keyframe_type = match keyframe {
                KeyframeData::Float(_) => TrackType::Opacity,
                KeyframeData::Vector2(_) => TrackType::Custom,
                KeyframeData::Vector3(_) => TrackType::Position,
                KeyframeData::Vector4(_) => TrackType::Custom,
                KeyframeData::Color(_) => TrackType::Color,
                KeyframeData::Transform(_) => TrackType::Position,
                KeyframeData::Boolean(_) => TrackType::Custom,
                KeyframeData::String(_) => TrackType::Custom,
            };
            
            // Allow some flexibility in type matching
            if !self.is_compatible_keyframe_type(keyframe_type) {
                return Err(format!("Keyframe type {:?} is not compatible with track type {:?}", 
                                 keyframe_type, self.track_type));
            }
        }
        
        Ok(())
    }
    
    /// Check if keyframe type is compatible with track type
    fn is_compatible_keyframe_type(&self, keyframe_type: TrackType) -> bool {
        match self.track_type {
            TrackType::Position | TrackType::Rotation | TrackType::Scale => {
                matches!(keyframe_type, TrackType::Position | TrackType::Rotation | TrackType::Scale | TrackType::Custom)
            }
            TrackType::Opacity => {
                matches!(keyframe_type, TrackType::Opacity | TrackType::Custom)
            }
            TrackType::Color => {
                matches!(keyframe_type, TrackType::Color | TrackType::Custom)
            }
            TrackType::Custom => true, // Custom tracks accept any type
        }
    }
    
    /// Get track description
    pub fn description(&self) -> String {
        format!("{} track '{}' with {} keyframes, bound to '{}'", 
                self.track_type, self.name, self.keyframe_count(), self.binding.full_path())
    }
    
    /// Clone track with new ID
    pub fn clone_with_id(&self, new_id: String) -> Self {
        let mut cloned = self.clone();
        cloned.id = new_id;
        cloned
    }
    
    /// Get time range of track
    pub fn time_range(&self) -> Option<(f64, f64)> {
        if self.keyframes.is_empty() {
            return None;
        }
        
        let times: Vec<f64> = self.keyframes.iter().map(|k| k.time()).collect();
        let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_time = times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Some((min_time, max_time))
    }
    
    /// Check if track has keyframes at given time
    pub fn has_keyframe_at_time(&self, time: f64, tolerance: f64) -> bool {
        self.keyframes.iter().any(|k| (k.time() - time).abs() < tolerance)
    }
    
    /// Get keyframe index at given time
    pub fn get_keyframe_index_at_time(&self, time: f64, tolerance: f64) -> Option<usize> {
        self.keyframes.iter().position(|k| (k.time() - time).abs() < tolerance)
    }
}

impl fmt::Display for AnimationTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Default for AnimationTrack {
    fn default() -> Self {
        Self::with_default_binding(
            "default".to_string(),
            "Default Track".to_string(),
            TrackType::Position,
            "default_target".to_string(),
        )
    }
}

/// Animation track builder
pub struct AnimationTrackBuilder {
    track: AnimationTrack,
}

impl AnimationTrackBuilder {
    /// Create new track builder
    pub fn new(id: String, name: String, track_type: TrackType) -> Self {
        let track = AnimationTrack::with_default_binding(id, name, track_type, "default".to_string());
        Self { track }
    }
    
    /// Set target ID
    pub fn target(mut self, target_id: String) -> Self {
        self.track.binding.target_id = target_id;
        self
    }
    
    /// Set parameter name
    pub fn parameter(mut self, parameter_name: String) -> Self {
        self.track.binding.parameter_name = parameter_name;
        self
    }
    
    /// Set binding type
    pub fn binding_type(mut self, binding_type: BindingType) -> Self {
        self.track.binding.binding_type = binding_type;
        self
    }
    
    /// Set weight
    pub fn weight(mut self, weight: f64) -> Self {
        self.track.set_weight(weight);
        self
    }
    
    /// Add keyframe
    pub fn keyframe(mut self, keyframe: KeyframeData) -> Self {
        self.track.add_keyframe(keyframe);
        self
    }
    
    /// Build the track
    pub fn build(self) -> Result<AnimationTrack, String> {
        self.track.validate()?;
        Ok(self.track)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::keyframe::{Keyframe, KeyframeData};
    use crate::animation::interpolation::{InterpolationMethod, EasingFunction};
    
    #[test]
    fn test_track_creation() {
        let binding = ParameterBinding::new("object1".to_string(), "position".to_string());
        let track = AnimationTrack::new(
            "track1".to_string(),
            "Position Track".to_string(),
            TrackType::Position,
            binding,
        );
        
        assert_eq!(track.id, "track1");
        assert_eq!(track.name, "Position Track");
        assert_eq!(track.track_type, TrackType::Position);
        assert!(track.enabled);
        assert!(!track.muted);
        assert_eq!(track.weight, 1.0);
    }
    
    #[test]
    fn test_track_keyframes() {
        let binding = ParameterBinding::new("object1".to_string(), "opacity".to_string());
        let mut track = AnimationTrack::new(
            "track1".to_string(),
            "Opacity Track".to_string(),
            TrackType::Opacity,
            binding,
        );
        
        let keyframe1 = KeyframeData::Float(Keyframe::new(0.0, 0.0));
        let keyframe2 = KeyframeData::Float(Keyframe::new(1.0, 1.0));
        
        track.add_keyframe(keyframe1);
        track.add_keyframe(keyframe2);
        
        assert_eq!(track.keyframe_count(), 2);
        
        // Test keyframe ordering
        assert!(track.get_keyframe(0).unwrap().time() <= track.get_keyframe(1).unwrap().time());
    }
    
    #[test]
    fn test_track_value_at_time() {
        let binding = ParameterBinding::new("object1".to_string(), "opacity".to_string());
        let mut track = AnimationTrack::new(
            "track1".to_string(),
            "Opacity Track".to_string(),
            TrackType::Opacity,
            binding,
        );
        
        let keyframe1 = KeyframeData::Float(Keyframe::new(0.0, 0.0));
        let keyframe2 = KeyframeData::Float(Keyframe::new(1.0, 1.0));
        
        track.add_keyframe(keyframe1);
        track.add_keyframe(keyframe2);
        
        // Test interpolation
        let value = track.get_value_at_time(0.5);
        assert!(value.is_some());
        
        if let Some(TrackValue::Float(v)) = value {
            assert!((v - 0.5).abs() < 0.001);
        } else {
            panic!("Expected float value");
        }
    }
    
    #[test]
    fn test_track_time_range() {
        let binding = ParameterBinding::new("object1".to_string(), "opacity".to_string());
        let mut track = AnimationTrack::new(
            "track1".to_string(),
            "Opacity Track".to_string(),
            TrackType::Opacity,
            binding,
        );
        
        let keyframe1 = KeyframeData::Float(Keyframe::new(0.5, 0.0));
        let keyframe2 = KeyframeData::Float(Keyframe::new(2.5, 1.0));
        
        track.add_keyframe(keyframe1);
        track.add_keyframe(keyframe2);
        
        let time_range = track.time_range();
        assert!(time_range.is_some());
        assert_eq!(time_range.unwrap(), (0.5, 2.5));
    }
    
    #[test]
    fn test_track_builder() {
        let track = AnimationTrackBuilder::new(
            "test".to_string(),
            "Test Track".to_string(),
            TrackType::Position,
        )
        .target("object1".to_string())
        .parameter("position".to_string())
        .binding_type(BindingType::Additive)
        .weight(0.75)
        .build();
        
        assert!(track.is_ok());
        
        let track = track.unwrap();
        assert_eq!(track.id, "test");
        assert_eq!(track.binding.target_id, "object1");
        assert_eq!(track.binding.parameter_name, "position");
        assert_eq!(track.binding.binding_type, BindingType::Additive);
        assert_eq!(track.weight, 0.75);
    }
}
