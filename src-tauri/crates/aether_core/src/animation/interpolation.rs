//! Animation interpolation algorithms
//! 
//! This module provides runtime interpolation algorithms that use the
//! animation data structures from aether_types to evaluate animations
//! at specific time points with various interpolation methods.

use aether_types::animation::{
    InterpolationMethod, EasingFunction, AnimationCurve, TrackValue,
    KeyframeData, KeyframeCollection
};
use std::collections::HashMap;

/// Result of interpolation operation
#[derive(Debug, Clone)]
pub struct InterpolationResult {
    pub value: TrackValue,
    pub time: f64,
    pub method: InterpolationMethod,
    pub easing: EasingFunction,
    pub success: bool,
}

impl InterpolationResult {
    /// Create successful interpolation result
    pub fn success(value: TrackValue, time: f64, method: InterpolationMethod, easing: EasingFunction) -> Self {
        Self {
            value,
            time,
            method,
            easing,
            success: true,
        }
    }
    
    /// Create failed interpolation result
    pub fn failure(time: f64) -> Self {
        Self {
            value: TrackValue::Float(0.0),
            time,
            method: InterpolationMethod::Linear,
            easing: EasingFunction::Linear,
            success: false,
        }
    }
}

/// High-performance animation interpolator
pub struct AnimationInterpolator {
    cache: HashMap<String, InterpolationResult>,
    max_cache_size: usize,
    stats: InterpolationStats,
}

impl AnimationInterpolator {
    /// Create new interpolator
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size: 1000,
            stats: InterpolationStats::default(),
        }
    }
    
    /// Create interpolator with custom cache size
    pub fn with_cache_size(max_cache_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size,
            stats: InterpolationStats::default(),
        }
    }
    
    /// Interpolate value at given time using keyframes
    pub fn interpolate_at_time(
        &mut self,
        keyframes: &[KeyframeData],
        time: f64,
    ) -> InterpolationResult {
        self.stats.total_interpolations += 1;
        
        // Check cache first
        let cache_key = format!("{:.6}_{}", time, keyframes.len());
        if let Some(cached) = self.cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            return cached.clone();
        }
        
        self.stats.cache_misses += 1;
        
        // Perform interpolation
        let result = if keyframes.is_empty() {
            InterpolationResult::failure(time)
        } else if keyframes.len() == 1 {
            // Single keyframe - return its value
            let keyframe = &keyframes[0];
            InterpolationResult::success(
                self.keyframe_to_track_value(keyframe),
                time,
                keyframe.interpolation(),
                keyframe.easing(),
            )
        } else {
            // Multiple keyframes - interpolate between surrounding keyframes
            let (prev_keyframe, next_keyframe) = 
                KeyframeCollection::find_surrounding_keyframes(keyframes, time);
            
            match (prev_keyframe, next_keyframe) {
                (Some(prev), Some(next)) => {
                    if prev.time() == next.time() {
                        // Same time, return prev value
                        InterpolationResult::success(
                            self.keyframe_to_track_value(prev),
                            time,
                            prev.interpolation(),
                            prev.easing(),
                        )
                    } else {
                        // Interpolate between keyframes
                        self.interpolate_between_keyframes(prev, next, time)
                    }
                }
                (Some(prev), None) => {
                    // After last keyframe, return last value
                    InterpolationResult::success(
                        self.keyframe_to_track_value(prev),
                        time,
                        prev.interpolation(),
                        prev.easing(),
                    )
                }
                (None, Some(next)) => {
                    // Before first keyframe, return first value
                    InterpolationResult::success(
                        self.keyframe_to_track_value(next),
                        time,
                        next.interpolation(),
                        next.easing(),
                    )
                }
                (None, None) => InterpolationResult::failure(time),
            }
        };
        
        // Cache result
        if self.cache.len() >= self.max_cache_size {
            // Remove oldest entries (simple FIFO)
            let keys_to_remove: Vec<String> = self.cache.keys()
                .take(self.max_cache_size / 10)
                .cloned()
                .collect();
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }
        self.cache.insert(cache_key, result.clone());
        
        result
    }
    
    /// Interpolate between two keyframes
    fn interpolate_between_keyframes(
        &self,
        prev_keyframe: &KeyframeData,
        next_keyframe: &KeyframeData,
        time: f64,
    ) -> InterpolationResult {
        let prev_time = prev_keyframe.time();
        let next_time = next_keyframe.time();
        let interpolation_method = prev_keyframe.interpolation();
        let easing_function = prev_keyframe.easing();
        
        // Calculate interpolation parameter t (0.0 to 1.0)
        let t = (time - prev_time) / (next_time - prev_time);
        
        // Apply easing if it's a smooth interpolation
        let eased_t = if interpolation_method.is_smooth() {
            easing_function.apply(t)
        } else {
            t
        };
        
        // Interpolate values based on method
        let interpolated_value = match interpolation_method {
            InterpolationMethod::Linear => self.linear_interpolate(prev_keyframe, next_keyframe, eased_t),
            InterpolationMethod::Bezier => self.bezier_interpolate(prev_keyframe, next_keyframe, eased_t),
            InterpolationMethod::CubicSpline => self.cubic_spline_interpolate(prev_keyframe, next_keyframe, eased_t),
            InterpolationMethod::Step => self.step_interpolate(prev_keyframe, next_keyframe, eased_t),
            InterpolationMethod::Hold => self.hold_interpolate(prev_keyframe, next_keyframe, eased_t),
        };
        
        InterpolationResult::success(
            interpolated_value,
            time,
            interpolation_method,
            easing_function,
        )
    }
    
    /// Linear interpolation between keyframes
    fn linear_interpolate(
        &self,
        prev_keyframe: &KeyframeData,
        next_keyframe: &KeyframeData,
        t: f64,
    ) -> TrackValue {
        let prev_value = self.keyframe_to_track_value(prev_keyframe);
        let next_value = self.keyframe_to_track_value(next_keyframe);
        
        prev_value.interpolate(&next_value, t).unwrap_or(prev_value)
    }
    
    /// Bezier interpolation between keyframes
    fn bezier_interpolate(
        &self,
        prev_keyframe: &KeyframeData,
        next_keyframe: &KeyframeData,
        t: f64,
    ) -> TrackValue {
        // For bezier interpolation, we use cubic bezier with control points
        // derived from the easing function
        let prev_value = self.keyframe_to_track_value(prev_keyframe);
        let next_value = self.keyframe_to_track_value(next_keyframe);
        
        // Apply bezier easing to the interpolation parameter
        let bezier_t = self.apply_bezier_easing(t, prev_keyframe.easing());
        
        prev_value.interpolate(&next_value, bezier_t).unwrap_or(prev_value)
    }
    
    /// Cubic spline interpolation between keyframes
    fn cubic_spline_interpolate(
        &self,
        prev_keyframe: &KeyframeData,
        next_keyframe: &KeyframeData,
        t: f64,
    ) -> TrackValue {
        // For cubic spline, we use smoothstep function
        let smooth_t = t * t * (3.0 - 2.0 * t);
        
        let prev_value = self.keyframe_to_track_value(prev_keyframe);
        let next_value = self.keyframe_to_track_value(next_keyframe);
        
        prev_value.interpolate(&next_value, smooth_t).unwrap_or(prev_value)
    }
    
    /// Step interpolation between keyframes
    fn step_interpolate(
        &self,
        prev_keyframe: &KeyframeData,
        next_keyframe: &KeyframeData,
        t: f64,
    ) -> TrackValue {
        if t < 0.5 {
            self.keyframe_to_track_value(prev_keyframe)
        } else {
            self.keyframe_to_track_value(next_keyframe)
        }
    }
    
    /// Hold interpolation (return previous value)
    fn hold_interpolate(
        &self,
        prev_keyframe: &KeyframeData,
        _next_keyframe: &KeyframeData,
        _t: f64,
    ) -> TrackValue {
        self.keyframe_to_track_value(prev_keyframe)
    }
    
    /// Apply bezier easing to parameter
    fn apply_bezier_easing(&self, t: f64, easing: EasingFunction) -> f64 {
        // Use the easing function to apply bezier-like behavior
        easing.apply(t)
    }
    
    /// Convert keyframe to track value
    fn keyframe_to_track_value(&self, keyframe: &KeyframeData) -> TrackValue {
        match keyframe {
            KeyframeData::Float(k) => TrackValue::Float(k.value),
            KeyframeData::Vector2(k) => TrackValue::Vector2(k.value),
            KeyframeData::Vector3(k) => TrackValue::Vector3(k.value),
            KeyframeData::Vector4(k) => TrackValue::Vector4(k.value),
            KeyframeData::Color(k) => TrackValue::Color(k.value),
            KeyframeData::Transform(k) => TrackValue::Vector3(k.value.position),
            KeyframeData::Boolean(k) => TrackValue::Boolean(k.value),
            KeyframeData::String(k) => TrackValue::String(k.value.clone()),
        }
    }
    
    /// Interpolate using animation curve
    pub fn interpolate_with_curve(
        &mut self,
        keyframes: &[KeyframeData],
        time: f64,
        curve: &AnimationCurve,
    ) -> InterpolationResult {
        let mut result = self.interpolate_at_time(keyframes, time);
        
        if result.success {
            // Apply curve to the interpolated value
            let curve_t = curve.sample(time);
            
            // For now, we'll apply the curve as a multiplier to float values
            // In a more sophisticated implementation, we'd handle different value types
            match &mut result.value {
                TrackValue::Float(v) => *v *= curve_t,
                TrackValue::Vector2(v) => {
                    v[0] *= curve_t;
                    v[1] *= curve_t;
                }
                TrackValue::Vector3(v) => {
                    v[0] *= curve_t;
                    v[1] *= curve_t;
                    v[2] *= curve_t;
                }
                TrackValue::Vector4(v) => {
                    v[0] *= curve_t;
                    v[1] *= curve_t;
                    v[2] *= curve_t;
                    v[3] *= curve_t;
                }
                TrackValue::Color(v) => {
                    v[0] *= curve_t;
                    v[1] *= curve_t;
                    v[2] *= curve_t;
                    v[3] *= curve_t;
                }
                // Boolean and String values are not affected by curves
                TrackValue::Boolean(_) | TrackValue::String(_) => {}
            }
        }
        
        result
    }
    
    /// Clear interpolation cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.stats.cache_cleared += 1;
    }
    
    /// Get interpolation statistics
    pub fn get_stats(&self) -> &InterpolationStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = InterpolationStats::default();
    }
}

impl Default for AnimationInterpolator {
    fn default() -> Self {
        Self::new()
    }
}

/// Interpolation performance statistics
#[derive(Debug, Clone, Default)]
pub struct InterpolationStats {
    /// Total number of interpolations performed
    pub total_interpolations: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Number of times cache was cleared
    pub cache_cleared: u64,
}

impl InterpolationStats {
    /// Get cache hit ratio (0.0 to 1.0)
    pub fn cache_hit_ratio(&self) -> f64 {
        if self.total_interpolations == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_interpolations as f64
        }
    }
    
    /// Get cache miss ratio (0.0 to 1.0)
    pub fn cache_miss_ratio(&self) -> f64 {
        if self.total_interpolations == 0 {
            0.0
        } else {
            self.cache_misses as f64 / self.total_interpolations as f64
        }
    }
}

/// Batch interpolation utilities
pub struct BatchInterpolator;

impl BatchInterpolator {
    /// Interpolate multiple tracks at the same time
    pub fn interpolate_tracks_at_time(
        interpolator: &mut AnimationInterpolator,
        tracks: &[(String, &[KeyframeData])],
        time: f64,
    ) -> HashMap<String, InterpolationResult> {
        let mut results = HashMap::new();
        
        for (track_id, keyframes) in tracks {
            let result = interpolator.interpolate_at_time(keyframes, time);
            results.insert(track_id.clone(), result);
        }
        
        results
    }
    
    /// Interpolate multiple time points for a single track
    pub fn interpolate_track_at_times(
        interpolator: &mut AnimationInterpolator,
        keyframes: &[KeyframeData],
        times: &[f64],
    ) -> Vec<InterpolationResult> {
        times.iter()
            .map(|&time| interpolator.interpolate_at_time(keyframes, time))
            .collect()
    }
    
    /// Interpolate multiple tracks at multiple time points
    pub fn interpolate_tracks_at_times(
        interpolator: &mut AnimationInterpolator,
        tracks: &[(String, &[KeyframeData])],
        times: &[f64],
    ) -> HashMap<String, Vec<InterpolationResult>> {
        let mut results = HashMap::new();
        
        for (track_id, keyframes) in tracks {
            let track_results = times.iter()
                .map(|&time| interpolator.interpolate_at_time(keyframes, time))
                .collect();
            results.insert(track_id.clone(), track_results);
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::animation::{Keyframe, InterpolationMethod, EasingFunction};
    
    #[test]
    fn test_linear_interpolation() {
        let mut interpolator = AnimationInterpolator::new();
        
        let keyframes = vec![
            KeyframeData::Float(Keyframe::new(0.0, 0.0)),
            KeyframeData::Float(Keyframe::new(1.0, 10.0)),
        ];
        
        let result = interpolator.interpolate_at_time(&keyframes, 0.5);
        
        assert!(result.success);
        assert_eq!(result.time, 0.5);
        assert_eq!(result.method, InterpolationMethod::Linear);
        
        if let TrackValue::Float(v) = result.value {
            assert_eq!(v, 5.0);
        } else {
            panic!("Expected float value");
        }
    }
    
    #[test]
    fn test_eased_interpolation() {
        let mut interpolator = AnimationInterpolator::new();
        
        let keyframes = vec![
            KeyframeData::Float(Keyframe::with_easing(
                0.0, 
                0.0, 
                InterpolationMethod::Linear, 
                EasingFunction::QuadIn
            )),
            KeyframeData::Float(Keyframe::new(1.0, 10.0)),
        ];
        
        let result = interpolator.interpolate_at_time(&keyframes, 0.5);
        
        assert!(result.success);
        assert_eq!(result.easing, EasingFunction::QuadIn);
        
        if let TrackValue::Float(v) = result.value {
            assert!(v < 5.0); // Should be less than linear due to quad-in easing
        } else {
            panic!("Expected float value");
        }
    }
    
    #[test]
    fn test_step_interpolation() {
        let mut interpolator = AnimationInterpolator::new();
        
        let keyframes = vec![
            KeyframeData::Float(Keyframe::with_interpolation(
                0.0, 
                0.0, 
                InterpolationMethod::Step
            )),
            KeyframeData::Float(Keyframe::new(1.0, 10.0)),
        ];
        
        let result = interpolator.interpolate_at_time(&keyframes, 0.25);
        
        assert!(result.success);
        assert_eq!(result.method, InterpolationMethod::Step);
        
        if let TrackValue::Float(v) = result.value {
            assert_eq!(v, 0.0); // Should be first value for t < 0.5
        } else {
            panic!("Expected float value");
        }
    }
    
    #[test]
    fn test_interpolation_cache() {
        let mut interpolator = AnimationInterpolator::with_cache_size(10);
        
        let keyframes = vec![
            KeyframeData::Float(Keyframe::new(0.0, 0.0)),
            KeyframeData::Float(Keyframe::new(1.0, 10.0)),
        ];
        
        // First interpolation
        let result1 = interpolator.interpolate_at_time(&keyframes, 0.5);
        assert!(result1.success);
        
        // Second interpolation (should hit cache)
        let result2 = interpolator.interpolate_at_time(&keyframes, 0.5);
        assert!(result2.success);
        
        let stats = interpolator.get_stats();
        assert_eq!(stats.total_interpolations, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }
    
    #[test]
    fn test_batch_interpolation() {
        let mut interpolator = AnimationInterpolator::new();
        
        let tracks = vec![
            ("track1".to_string(), &[
                KeyframeData::Float(Keyframe::new(0.0, 0.0)),
                KeyframeData::Float(Keyframe::new(1.0, 10.0)),
            ][..]),
            ("track2".to_string(), &[
                KeyframeData::Vector3(Keyframe::new(0.0, [0.0, 0.0, 0.0])),
                KeyframeData::Vector3(Keyframe::new(1.0, [1.0, 1.0, 1.0])),
            ][..]),
        ];
        
        let results = BatchInterpolator::interpolate_tracks_at_time(&mut interpolator, &tracks, 0.5);
        
        assert_eq!(results.len(), 2);
        assert!(results.contains_key("track1"));
        assert!(results.contains_key("track2"));
        
        let track1_result = results.get("track1").unwrap();
        assert!(track1_result.success);
        
        let track2_result = results.get("track2").unwrap();
        assert!(track2_result.success);
    }
}
