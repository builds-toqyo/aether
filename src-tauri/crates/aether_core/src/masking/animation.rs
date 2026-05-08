use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MaskAnimationTarget {
    Position,
    Rotation,
    Scale,
    Opacity,
    Feather,
    Expand,
    Choke,
    CornerRadius,
    GradientStart,
    GradientEnd,
    GradientCenter,
    GradientRadius,
    GradientAngle,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskKeyframe {
    pub time: f64,
    pub target: MaskAnimationTarget,
    pub value: AnimationValue,
    pub easing: crate::animation::interpolation::EasingFunction,
    pub interpolation: crate::animation::interpolation::InterpolationMethod,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationValue {
    Float(f64),
    Vector2(f64, f64),
    Vector3(f64, f64, f64),
    Color(f64, f64, f64, f64),
    Boolean(bool),
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskAnimationTrack {
    pub id: String,
    pub name: String,
    pub target: MaskAnimationTarget,
    pub keyframes: Vec<MaskKeyframe>,
    pub enabled: bool,
    pub loop_animation: bool,
    pub loop_count: usize,
    pub pre_roll: f64,
    pub post_roll: f64,
}

impl MaskAnimationTrack {
    pub fn new(id: String, name: String, target: MaskAnimationTarget) -> Self {
        Self {
            id,
            name,
            target,
            keyframes: Vec::new(),
            enabled: true,
            loop_animation: false,
            loop_count: 1,
            pre_roll: 0.0,
            post_roll: 0.0,
        }
    }

    pub fn with_keyframes(mut self, keyframes: Vec<MaskKeyframe>) -> Self {
        self.keyframes = keyframes;
        self.sort_keyframes();
        self
    }

    pub fn with_loop(mut self, loop_animation: bool, count: usize) -> Self {
        self.loop_animation = loop_animation;
        self.loop_count = count;
        self
    }

    pub fn with_pre_post_roll(mut self, pre_roll: f64, post_roll: f64) -> Self {
        self.pre_roll = pre_roll;
        self.post_roll = post_roll;
        self
    }

    pub fn add_keyframe(&mut self, keyframe: MaskKeyframe) {
        self.keyframes.push(keyframe);
        self.sort_keyframes();
    }

    pub fn remove_keyframe(&mut self, time: f64) -> bool {
        let initial_len = self.keyframes.len();
        self.keyframes.retain(|kf| (kf.time - time).abs() > f64::EPSILON);
        self.keyframes.len() < initial_len
    }

    pub fn clear_keyframes(&mut self) {
        self.keyframes.clear();
    }

    fn sort_keyframes(&mut self) {
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn evaluate(&self, time: f64) -> Option<AnimationValue> {
        if !self.enabled || self.keyframes.is_empty() {
            return None;
        }

        let adjusted_time = time - self.pre_roll;
        if adjusted_time < 0.0 {
            return None;
        }

        let total_duration = self.get_duration();
        if total_duration == 0.0 {
            return None;
        }

        let mut eval_time = adjusted_time;


        if self.loop_animation {
            if self.loop_count == 0 {
                eval_time = adjusted_time % total_duration;
            } else {
                let loop_duration = total_duration * self.loop_count as f64;
                if adjusted_time < loop_duration {
                    eval_time = adjusted_time % total_duration;
                } else {

                    if let Some(last_keyframe) = self.keyframes.last() {
                        return Some(last_keyframe.value.clone());
                    }
                    return None;
                }
            }
        } else if adjusted_time > total_duration {

            if let Some(last_keyframe) = self.keyframes.last() {
                return Some(last_keyframe.value.clone());
            }
            return None;
        }


        let prev_keyframe = self.get_keyframe_at_or_before(eval_time);
        let next_keyframe = self.get_keyframe_at_or_after(eval_time);

        match (prev_keyframe, next_keyframe) {
            (Some(prev), Some(next)) => {
                if prev.time == next.time {
                    return Some(prev.value.clone());
                }

                let t = (eval_time - prev.time) / (next.time - prev.time);
                let interpolated_value = self.interpolate_values(&prev.value, &next.value, t, &prev.easing);
                Some(interpolated_value)
            }
            (Some(prev), None) => Some(prev.value.clone()),
            (None, Some(next)) => {
                if eval_time >= next.time {
                    Some(next.value.clone())
                } else {
                    None
                }
            }
            (None, None) => None,
        }
    }

    fn get_keyframe_at_or_before(&self, time: f64) -> Option<&MaskKeyframe> {
        self.keyframes.iter()
            .rev()
            .find(|kf| kf.time <= time)
    }

    fn get_keyframe_at_or_after(&self, time: f64) -> Option<&MaskKeyframe> {
        self.keyframes.iter()
            .find(|kf| kf.time >= time)
    }

    fn get_duration(&self) -> f64 {
        if let Some(last_keyframe) = self.keyframes.last() {
            last_keyframe.time + self.post_roll
        } else {
            0.0
        }
    }

    fn interpolate_values(
        &self,
        from: &AnimationValue,
        to: &AnimationValue,
        t: f64,
        easing: &crate::animation::interpolation::EasingFunction,
    ) -> AnimationValue {
        let eased_t = easing.apply(t);

        match (from, to) {
            (AnimationValue::Float(from_f), AnimationValue::Float(to_f)) => {
                AnimationValue::Float(from_f + (to_f - from_f) * eased_t)
            }
            (AnimationValue::Vector2(from_x, from_y), AnimationValue::Vector2(to_x, to_y)) => {
                AnimationValue::Vector2(
                    from_x + (to_x - from_x) * eased_t,
                    from_y + (to_y - from_y) * eased_t,
                )
            }
            (AnimationValue::Vector3(from_x, from_y, from_z), AnimationValue::Vector3(to_x, to_y, to_z)) => {
                AnimationValue::Vector3(
                    from_x + (to_x - from_x) * eased_t,
                    from_y + (to_y - from_y) * eased_t,
                    from_z + (to_z - from_z) * eased_t,
                )
            }
            (AnimationValue::Color(from_r, from_g, from_b, from_a), AnimationValue::Color(to_r, to_g, to_b, to_a)) => {
                AnimationValue::Color(
                    from_r + (to_r - from_r) * eased_t,
                    from_g + (to_g - from_g) * eased_t,
                    from_b + (to_b - from_b) * eased_t,
                    from_a + (to_a - from_a) * eased_t,
                )
            }
            (AnimationValue::Boolean(_), AnimationValue::Boolean(to_b)) => {
                if eased_t >= 0.5 {
                    AnimationValue::Boolean(*to_b)
                } else {
                    from.clone()
                }
            }
            _ => from.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Animation track ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Animation track name cannot be empty".to_string());
        }

        if self.keyframes.is_empty() {
            return Err("Animation track must have at least one keyframe".to_string());
        }

        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time < 0.0 {
                return Err(format!("Keyframe {} has negative time", i));
            }
            if keyframe.time < self.pre_roll {
                return Err(format!("Keyframe {} time is before pre-roll", i));
            }
        }

        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskAnimationSystem {
    pub tracks: Vec<MaskAnimationTrack>,
    pub global_time: f64,
    pub playing: bool,
    pub playback_speed: f64,
    pub start_time: f64,
    pub current_values: HashMap<MaskAnimationTarget, AnimationValue>,
}

impl MaskAnimationSystem {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            global_time: 0.0,
            playing: false,
            playback_speed: 1.0,
            start_time: 0.0,
            current_values: HashMap::new(),
        }
    }

    pub fn add_track(&mut self, track: MaskAnimationTrack) {
        self.tracks.push(track);
    }

    pub fn remove_track(&mut self, track_id: &str) -> bool {
        let initial_len = self.tracks.len();
        self.tracks.retain(|track| track.id != track_id);
        self.tracks.len() < initial_len
    }

    pub fn get_track(&self, track_id: &str) -> Option<&MaskAnimationTrack> {
        self.tracks.iter().find(|track| track.id == track_id)
    }

    pub fn get_track_mut(&mut self, track_id: &str) -> Option<&mut MaskAnimationTrack> {
        self.tracks.iter_mut().find(|track| track.id == track_id)
    }

    pub fn update(&mut self, delta_time: f64) {
        if self.playing {
            self.global_time += delta_time * self.playback_speed;
            self.evaluate_all_tracks();
        }
    }

    fn evaluate_all_tracks(&mut self) {
        self.current_values.clear();

        for track in &self.tracks {
            if track.enabled {
                if let Some(value) = track.evaluate(self.global_time) {
                    self.current_values.insert(track.target, value);
                }
            }
        }
    }

    pub fn get_value(&self, target: MaskAnimationTarget) -> Option<&AnimationValue> {
        self.current_values.get(&target)
    }

    pub fn get_float_value(&self, target: MaskAnimationTarget) -> Option<f64> {
        self.get_value(target).and_then(|value| {
            if let AnimationValue::Float(f) = value {
                Some(*f)
            } else {
                None
            }
        })
    }

    pub fn get_vector2_value(&self, target: MaskAnimationTarget) -> Option<(f64, f64)> {
        self.get_value(target).and_then(|value| {
            if let AnimationValue::Vector2(x, y) = value {
                Some((*x, *y))
            } else {
                None
            }
        })
    }

    pub fn get_color_value(&self, target: MaskAnimationTarget) -> Option<(f64, f64, f64, f64)> {
        self.get_value(target).and_then(|value| {
            if let AnimationValue::Color(r, g, b, a) = value {
                Some((*r, *g, *b, *a))
            } else {
                None
            }
        })
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.global_time = self.start_time;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.global_time = self.start_time;
        self.current_values.clear();
    }

    pub fn reset(&mut self) {
        self.global_time = self.start_time;
        self.current_values.clear();
    }

    pub fn seek(&mut self, time: f64) {
        self.global_time = time;
        self.evaluate_all_tracks();
    }

    pub fn set_playback_speed(&mut self, speed: f64) {
        if speed > 0.0 {
            self.playback_speed = speed;
        }
    }

    pub fn set_start_time(&mut self, time: f64) {
        self.start_time = time;
        if !self.playing {
            self.global_time = time;
        }
    }

    pub fn get_duration(&self) -> f64 {
        self.tracks.iter()
            .filter_map(|track| {
                if track.enabled {
                    Some(track.get_duration())
                } else {
                    None
                }
            })
            .fold(0.0, f64::max)
    }

    pub fn create_position_animation(
        &mut self,
        track_id: String,
        name: String,
        keyframes: Vec<(f64, f64, f64)>,
    ) -> String {
        let animation_keyframes: Vec<MaskKeyframe> = keyframes
            .into_iter()
            .map(|(time, x, y)| MaskKeyframe {
                time,
                target: MaskAnimationTarget::Position,
                value: AnimationValue::Vector2(x, y),
                easing: crate::animation::interpolation::EasingFunction::Linear,
                interpolation: crate::animation::interpolation::InterpolationMethod::Linear,
            })
            .collect();

        let track = MaskAnimationTrack::new(track_id.clone(), name, MaskAnimationTarget::Position)
            .with_keyframes(animation_keyframes);

        self.add_track(track);
        track_id
    }

    pub fn create_opacity_animation(
        &mut self,
        track_id: String,
        name: String,
        keyframes: Vec<(f64, f64)>,
    ) -> String {
        let animation_keyframes: Vec<MaskKeyframe> = keyframes
            .into_iter()
            .map(|(time, opacity)| MaskKeyframe {
                time,
                target: MaskAnimationTarget::Opacity,
                value: AnimationValue::Float(opacity),
                easing: crate::animation::interpolation::EasingFunction::Linear,
                interpolation: crate::animation::interpolation::InterpolationMethod::Linear,
            })
            .collect();

        let track = MaskAnimationTrack::new(track_id.clone(), name, MaskAnimationTarget::Opacity)
            .with_keyframes(animation_keyframes);

        self.add_track(track);
        track_id
    }

    pub fn create_scale_animation(
        &mut self,
        track_id: String,
        name: String,
        keyframes: Vec<(f64, f64, f64)>,
    ) -> String {
        let animation_keyframes: Vec<MaskKeyframe> = keyframes
            .into_iter()
            .map(|(time, x, y)| MaskKeyframe {
                time,
                target: MaskAnimationTarget::Scale,
                value: AnimationValue::Vector2(x, y),
                easing: crate::animation::interpolation::EasingFunction::Linear,
                interpolation: crate::animation::interpolation::InterpolationMethod::Linear,
            })
            .collect();

        let track = MaskAnimationTrack::new(track_id.clone(), name, MaskAnimationTarget::Scale)
            .with_keyframes(animation_keyframes);

        self.add_track(track);
        track_id
    }

    pub fn validate(&self) -> Result<(), String> {
        for (i, track) in self.tracks.iter().enumerate() {
            track.validate().map_err(|e| format!("Track {}: {}", i, e))?;
        }
        Ok(())
    }
}

impl Default for MaskAnimationSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::interpolation::{EasingFunction, InterpolationMethod};

    #[test]
    fn test_mask_animation_track_creation() {
        let track = MaskAnimationTrack::new(
            "track1".to_string(),
            "Position Track".to_string(),
            MaskAnimationTarget::Position,
        );

        assert_eq!(track.id, "track1");
        assert_eq!(track.name, "Position Track");
        assert_eq!(track.target, MaskAnimationTarget::Position);
        assert!(track.enabled);
        assert!(!track.loop_animation);
        assert_eq!(track.loop_count, 1);
    }

    #[test]
    fn test_mask_keyframe_creation() {
        let keyframe = MaskKeyframe {
            time: 1.0,
            target: MaskAnimationTarget::Position,
            value: AnimationValue::Vector2(100.0, 200.0),
            easing: EasingFunction::Linear,
            interpolation: InterpolationMethod::Linear,
        };

        assert_eq!(keyframe.time, 1.0);
        assert_eq!(keyframe.target, MaskAnimationTarget::Position);
        assert_eq!(keyframe.value, AnimationValue::Vector2(100.0, 200.0));
    }

    #[test]
    fn test_animation_track_with_keyframes() {
        let keyframes = vec![
            MaskKeyframe {
                time: 0.0,
                target: MaskAnimationTarget::Opacity,
                value: AnimationValue::Float(0.0),
                easing: EasingFunction::Linear,
                interpolation: InterpolationMethod::Linear,
            },
            MaskKeyframe {
                time: 2.0,
                target: MaskAnimationTarget::Opacity,
                value: AnimationValue::Float(1.0),
                easing: EasingFunction::Linear,
                interpolation: InterpolationMethod::Linear,
            },
        ];

        let track = MaskAnimationTrack::new("track1".to_string(), "Opacity".to_string(), MaskAnimationTarget::Opacity)
            .with_keyframes(keyframes);

        assert_eq!(track.keyframes.len(), 2);
        assert_eq!(track.keyframes[0].time, 0.0);
        assert_eq!(track.keyframes[1].time, 2.0);
    }

    #[test]
    fn test_animation_track_evaluation() {
        let keyframes = vec![
            MaskKeyframe {
                time: 0.0,
                target: MaskAnimationTarget::Opacity,
                value: AnimationValue::Float(0.0),
                easing: EasingFunction::Linear,
                interpolation: InterpolationMethod::Linear,
            },
            MaskKeyframe {
                time: 2.0,
                target: MaskAnimationTarget::Opacity,
                value: AnimationValue::Float(1.0),
                easing: EasingFunction::Linear,
                interpolation: InterpolationMethod::Linear,
            },
        ];

        let track = MaskAnimationTrack::new("track1".to_string(), "Opacity".to_string(), MaskAnimationTarget::Opacity)
            .with_keyframes(keyframes);


        let result = track.evaluate(0.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), AnimationValue::Float(0.0));


        let result = track.evaluate(2.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), AnimationValue::Float(1.0));


        let result = track.evaluate(1.0);
        assert!(result.is_some());
        if let AnimationValue::Float(value) = result.unwrap() {
            assert!((value - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_animation_track_looping() {
        let keyframes = vec![
            MaskKeyframe {
                time: 0.0,
                target: MaskAnimationTarget::Opacity,
                value: AnimationValue::Float(0.0),
                easing: EasingFunction::Linear,
                interpolation: InterpolationMethod::Linear,
            },
            MaskKeyframe {
                time: 1.0,
                target: MaskAnimationTarget::Opacity,
                value: AnimationValue::Float(1.0),
                easing: EasingFunction::Linear,
                interpolation: InterpolationMethod::Linear,
            },
        ];

        let track = MaskAnimationTrack::new("track1".to_string(), "Loop".to_string(), MaskAnimationTarget::Opacity)
            .with_keyframes(keyframes)
            .with_loop(true, 0);


        let result = track.evaluate(1.5);
        assert!(result.is_some());
        if let AnimationValue::Float(value) = result.unwrap() {
            assert!((value - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_mask_animation_system() {
        let mut system = MaskAnimationSystem::new();

        assert!(!system.playing);
        assert_eq!(system.global_time, 0.0);
        assert_eq!(system.playback_speed, 1.0);
        assert!(system.tracks.is_empty());
        assert!(system.current_values.is_empty());
    }

    #[test]
    fn test_animation_system_add_track() {
        let mut system = MaskAnimationSystem::new();
        let track = MaskAnimationTrack::new("track1".to_string(), "Test".to_string(), MaskAnimationTarget::Position);

        system.add_track(track);
        assert_eq!(system.tracks.len(), 1);
    }

    #[test]
    fn test_animation_system_playback() {
        let mut system = MaskAnimationSystem::new();

        system.play();
        assert!(system.playing);
        assert_eq!(system.global_time, 0.0);

        system.pause();
        assert!(!system.playing);

        system.stop();
        assert!(!system.playing);
        assert_eq!(system.global_time, 0.0);
    }

    #[test]
    fn test_animation_system_seek() {
        let mut system = MaskAnimationSystem::new();

        system.seek(2.5);
        assert_eq!(system.global_time, 2.5);
    }

    #[test]
    fn test_animation_system_convenience_methods() {
        let mut system = MaskAnimationSystem::new();

        let track_id = system.create_position_animation(
            "pos_track".to_string(),
            "Position".to_string(),
            vec![(0.0, 0.0, 0.0), (2.0, 100.0, 200.0)],
        );

        assert_eq!(track_id, "pos_track");
        assert_eq!(system.tracks.len(), 1);

        let opacity_id = system.create_opacity_animation(
            "opacity_track".to_string(),
            "Opacity".to_string(),
            vec![(0.0, 0.0), (1.0, 1.0)],
        );

        assert_eq!(opacity_id, "opacity_track");
        assert_eq!(system.tracks.len(), 2);
    }

    #[test]
    fn test_animation_value_getters() {
        let mut system = MaskAnimationSystem::new();

        system.create_opacity_animation(
            "opacity".to_string(),
            "Opacity".to_string(),
            vec![(0.0, 0.5), (1.0, 0.8)],
        );

        system.seek(0.5);
        system.evaluate_all_tracks();

        let float_value = system.get_float_value(MaskAnimationTarget::Opacity);
        assert!(float_value.is_some());
        assert!(float_value.unwrap() > 0.0 && float_value.unwrap() < 1.0);

        let vector_value = system.get_vector2_value(MaskAnimationTarget::Position);
        assert!(vector_value.is_none());
    }

    #[test]
    fn test_animation_track_validation() {
        let valid_track = MaskAnimationTrack::new("track1".to_string(), "Test".to_string(), MaskAnimationTarget::Opacity);
        assert!(valid_track.validate().is_ok());

        let mut invalid_track = valid_track.clone();
        invalid_track.id = "".to_string();
        assert!(invalid_track.validate().is_err());

        invalid_track.id = "test".to_string();
        invalid_track.keyframes.clear();
        assert!(invalid_track.validate().is_err());
    }

    #[test]
    fn test_animation_interpolation() {
        let keyframes = vec![
            MaskKeyframe {
                time: 0.0,
                target: MaskAnimationTarget::Position,
                value: AnimationValue::Vector2(0.0, 0.0),
                easing: EasingFunction::Linear,
                interpolation: InterpolationMethod::Linear,
            },
            MaskKeyframe {
                time: 2.0,
                target: MaskAnimationTarget::Position,
                value: AnimationValue::Vector2(100.0, 200.0),
                easing: EasingFunction::Linear,
                interpolation: InterpolationMethod::Linear,
            },
        ];

        let track = MaskAnimationTrack::new("track1".to_string(), "Position".to_string(), MaskAnimationTarget::Position)
            .with_keyframes(keyframes);

        let result = track.evaluate(1.0);
        assert!(result.is_some());
        if let AnimationValue::Vector2(x, y) = result.unwrap() {
            assert!((x - 50.0).abs() < 0.001);
            assert!((y - 100.0).abs() < 0.001);
        } else {
            panic!("Expected Vector2 value");
        }
    }

    #[test]
    fn test_pre_post_roll() {
        let keyframes = vec![
            MaskKeyframe {
                time: 1.0,
                target: MaskAnimationTarget::Opacity,
                value: AnimationValue::Float(0.5),
                easing: EasingFunction::Linear,
                interpolation: InterpolationMethod::Linear,
            },
        ];

        let track = MaskAnimationTrack::new("track1".to_string(), "Test".to_string(), MaskAnimationTarget::Opacity)
            .with_keyframes(keyframes)
            .with_pre_post_roll(0.5, 0.25);

        assert_eq!(track.pre_roll, 0.5);
        assert_eq!(track.post_roll, 0.25);
        assert_eq!(track.get_duration(), 1.75);


        assert!(track.evaluate(0.25).is_none());


        assert!(track.evaluate(1.0).is_some());


        assert!(track.evaluate(2.0).is_none());
    }
}
