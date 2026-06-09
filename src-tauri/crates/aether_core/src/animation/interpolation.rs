use aether_types::animation::{
    TrackValue, KeyframeData, KeyframeCollection
};
use crate::animation::{InterpolationMethod, EasingFunction};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct InterpolationResult {
    pub value: TrackValue,
    pub time: f64,
    pub method: InterpolationMethod,
    pub easing: EasingFunction,
    pub success: bool,
}

impl InterpolationResult {
    pub fn success(value: TrackValue, time: f64, method: InterpolationMethod, easing: EasingFunction) -> Self {
        Self {
            value,
            time,
            method,
            easing,
            success: true,
        }
    }

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

pub struct AnimationInterpolator {
    cache: HashMap<String, InterpolationResult>,
    max_cache_size: usize,
    stats: InterpolationStats,
}

impl AnimationInterpolator {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size: 1000,
            stats: InterpolationStats::default(),
        }
    }

    pub fn with_cache_size(max_cache_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size,
            stats: InterpolationStats::default(),
        }
    }

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

    pub fn interpolate_at_time(
        &mut self,
        keyframes: &[KeyframeData],
        time: f64,
    ) -> InterpolationResult {
        self.stats.total_interpolations += 1;

        let cache_key = format!("{:.6}_{}", time, keyframes.len());
        if let Some(cached) = self.cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            return cached.clone();
        }

        self.stats.cache_misses += 1;

        let result = if keyframes.is_empty() {
            InterpolationResult::failure(time)
        } else if keyframes.len() == 1 {

            let keyframe = &keyframes[0];
            InterpolationResult::success(
                self.keyframe_to_track_value(keyframe),
                time,
                keyframe.interpolation(),
                keyframe.easing(),
            )
        } else {

            let (prev_keyframe, next_keyframe) =
                KeyframeCollection::find_surrounding_keyframes_data(keyframes, time);

            match (prev_keyframe, next_keyframe) {
                (Some(prev), Some(next)) => {
                    if prev.time() == next.time() {

                        InterpolationResult::success(
                            self.keyframe_to_track_value(prev),
                            time,
                            prev.interpolation(),
                            prev.easing(),
                        )
                    } else {

                        self.interpolate_between_keyframes(prev, next, time)
                    }
                }
                (Some(prev), None) => {

                    InterpolationResult::success(
                        self.keyframe_to_track_value(prev),
                        time,
                        prev.interpolation(),
                        prev.easing(),
                    )
                }
                (None, Some(next)) => {

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

        if self.cache.len() >= self.max_cache_size {

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

        let t = (time - prev_time) / (next_time - prev_time);
        let curve_t = self.apply_easing(t, easing_function);

        let mut result = self.linear_interpolate(prev_keyframe, next_keyframe, t).unwrap_or_else(|_| InterpolationResult::failure(time));

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
            TrackValue::Boolean(_) | TrackValue::String(_) => {}
        }

        result
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.stats.cache_cleared += 1;
    }


    pub fn get_stats(&self) -> &InterpolationStats {
        &self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = InterpolationStats::default();
    }

    fn apply_easing(&self, t: f64, _easing: EasingFunction) -> f64 {
        // TODO: implement actual easing functions
        t
    }

    fn linear_interpolate(
        &self,
        prev: &KeyframeData,
        next: &KeyframeData,
        t: f64,
    ) -> Result<InterpolationResult> {
        let value = match (prev, next) {
            (KeyframeData::Float(p), KeyframeData::Float(n)) => {
                TrackValue::Float(p.value + (n.value - p.value) * t)
            }
            (KeyframeData::Vector2(p), KeyframeData::Vector2(n)) => {
                TrackValue::Vector2([
                    p.value[0] + (n.value[0] - p.value[0]) * t,
                    p.value[1] + (n.value[1] - p.value[1]) * t,
                ])
            }
            (KeyframeData::Vector3(p), KeyframeData::Vector3(n)) => {
                TrackValue::Vector3([
                    p.value[0] + (n.value[0] - p.value[0]) * t,
                    p.value[1] + (n.value[1] - p.value[1]) * t,
                    p.value[2] + (n.value[2] - p.value[2]) * t,
                ])
            }
            (KeyframeData::Vector4(p), KeyframeData::Vector4(n)) => {
                TrackValue::Vector4([
                    p.value[0] + (n.value[0] - p.value[0]) * t,
                    p.value[1] + (n.value[1] - p.value[1]) * t,
                    p.value[2] + (n.value[2] - p.value[2]) * t,
                    p.value[3] + (n.value[3] - p.value[3]) * t,
                ])
            }
            (KeyframeData::Color(p), KeyframeData::Color(n)) => {
                TrackValue::Color([
                    p.value[0] + (n.value[0] - p.value[0]) * t,
                    p.value[1] + (n.value[1] - p.value[1]) * t,
                    p.value[2] + (n.value[2] - p.value[2]) * t,
                    p.value[3] + (n.value[3] - p.value[3]) * t,
                ])
            }
            (KeyframeData::Boolean(p), KeyframeData::Boolean(n)) => {
                TrackValue::Boolean(if t < 0.5 { p.value } else { n.value })
            }
            (KeyframeData::String(p), KeyframeData::String(n)) => {
                TrackValue::String(if t < 0.5 { p.value.clone() } else { n.value.clone() })
            }
            _ => {
                return Ok(InterpolationResult::failure(0.0));
            }
        };
        Ok(InterpolationResult::success(
            value,
            0.0,
            InterpolationMethod::Linear,
            EasingFunction::Linear,
        ))
    }
}

impl Default for AnimationInterpolator {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone, Default)]
pub struct InterpolationStats {

    pub total_interpolations: u64,

    pub cache_hits: u64,

    pub cache_misses: u64,

    pub cache_cleared: u64,
}

impl InterpolationStats {

    pub fn cache_hit_ratio(&self) -> f64 {
        if self.total_interpolations == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_interpolations as f64
        }
    }

    pub fn cache_miss_ratio(&self) -> f64 {
        if self.total_interpolations == 0 {
            0.0
        } else {
            self.cache_misses as f64 / self.total_interpolations as f64
        }
    }
}

pub struct BatchInterpolator;

impl BatchInterpolator {

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

    pub fn interpolate_track_at_times(
        interpolator: &mut AnimationInterpolator,
        keyframes: &[KeyframeData],
        times: &[f64],
    ) -> Vec<InterpolationResult> {
        times.iter()
            .map(|&time| interpolator.interpolate_at_time(keyframes, time))
            .collect()
    }

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
    use aether_types::animation::{Keyframe};
    use crate::animation::{InterpolationMethod, EasingFunction};

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
            assert!(v < 5.0);
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
            assert_eq!(v, 0.0);
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


        let result1 = interpolator.interpolate_at_time(&keyframes, 0.5);
        assert!(result1.success);


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
