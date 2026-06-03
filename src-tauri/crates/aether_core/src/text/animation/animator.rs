use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterAnimation {
    pub id: String,
    pub name: String,
    pub animation_type: AnimationType,
    pub keyframes: Vec<TextKeyframe>,
    pub target_characters: Vec<usize>,
    pub duration: f64,
    pub delay: f64,
    pub loops: bool,
    pub loop_count: usize,
    pub playing: bool,
    pub current_time: f64,
    pub current_value: Option<AnimationValue>,
    pub easing: Option<aether_types::animation::EasingFunction>,
    pub character_delays: HashMap<usize, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextAnimator {
    pub animations: Vec<CharacterAnimation>,
    pub global_time: f64,
    pub playing: bool,
    pub playback_speed: f64,
    pub start_time: f64,
}

impl CharacterAnimation {
    pub fn new(
        id: String,
        name: String,
        animation_type: AnimationType,
        target_characters: Vec<usize>,
    ) -> Self {
        Self {
            id,
            name,
            animation_type,
            keyframes: Vec::new(),
            target_characters,
            duration: 1.0,
            delay: 0.0,
            loops: false,
            loop_count: 1,
            playing: false,
            current_time: 0.0,
            current_value: None,
            easing: None,
            character_delays: HashMap::new(),
        }
    }

    pub fn add_keyframe(&mut self, keyframe: TextKeyframe) {
        self.keyframes.push(keyframe);
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn remove_keyframe(&mut self, time: f64) -> bool {
        let initial_len = self.keyframes.len();
        self.keyframes.retain(|kf| (kf.time - time).abs() > f64::EPSILON);
        self.keyframes.len() < initial_len
    }

    pub fn get_keyframe(&self, time: f64) -> Option<&TextKeyframe> {
        self.keyframes.iter().find(|kf| (kf.time - time).abs() < f64::EPSILON)
    }

    pub fn get_keyframe_at_or_before(&self, time: f64) -> Option<&TextKeyframe> {
        self.keyframes.iter()
            .rev()
            .find(|kf| kf.time <= time)
    }

    pub fn get_keyframe_at_or_after(&self, time: f64) -> Option<&TextKeyframe> {
        self.keyframes.iter()
            .find(|kf| kf.time >= time)
    }

    pub fn evaluate(&mut self, time: f64, char_index: usize) -> Option<AnimationValue> {
        let char_delay = self.character_delays.get(&char_index).copied().unwrap_or(0.0);
        let adjusted_time = time - self.delay - char_delay;

        if adjusted_time < 0.0 {
            return None;
        }

        let mut time = adjusted_time;
        if self.loops && self.loop_count > 0 {
            let loop_duration = self.duration;
            time = adjusted_time % loop_duration;
        } else if self.loops {
            let loop_duration = self.duration;
            time = adjusted_time % loop_duration;
        } else if adjusted_time > self.duration {
            if let Some(last_keyframe) = self.keyframes.last() {
                return Some(last_keyframe.value.clone());
            }
            return None;
        }

        self.current_time = time;

        // Extract keyframes first to avoid borrowing issues
        let prev_keyframe = self.get_keyframe_at_or_before(time);
        let next_keyframe = self.get_keyframe_at_or_after(time);

        match (prev_keyframe, next_keyframe) {
            (Some(prev), Some(next)) => {
                if prev.time == next.time {
                    let result = prev.value.clone();
                    self.current_value = Some(result.clone());
                    return Some(result);
                }

                let t = (time - prev.time) / (next.time - prev.time);
                let interpolated_value = self.interpolate_values(&prev.value, &next.value, t, &prev.easing);

                self.current_value = Some(interpolated_value.clone());
                Some(interpolated_value)
            }
            (Some(prev), None) => {
                let result = prev.value.clone();
                self.current_value = Some(result.clone());
                Some(result)
            }
            (None, Some(next)) => {
                if time >= next.time {
                    let result = next.value.clone();
                    self.current_value = Some(result.clone());
                    Some(result)
                } else {
                    None
                }
            }
            (None, None) => None,
        }
    }

    fn interpolate_values(
        &self,
        from: &AnimationValue,
        to: &AnimationValue,
        t: f64,
        easing: &aether_types::animation::EasingFunction,
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
            (AnimationValue::String(_), AnimationValue::String(to_s)) => {
                if eased_t >= 0.5 {
                    AnimationValue::String(to_s.clone())
                } else {
                    from.clone()
                }
            }
            (AnimationValue::Bool(_), AnimationValue::Bool(to_b)) => {
                if eased_t >= 0.5 {
                    AnimationValue::Bool(*to_b)
                } else {
                    from.clone()
                }
            }
            _ => from.clone(),
        }
    }

    pub fn set_character_delay(&mut self, char_index: usize, delay: f64) {
        self.character_delays.insert(char_index, delay);
    }

    pub fn remove_character_delay(&mut self, char_index: usize) -> bool {
        self.character_delays.remove(&char_index).is_some()
    }

    pub fn get_character_delay(&self, char_index: usize) -> Option<f64> {
        self.character_delays.get(&char_index).copied()
    }

    pub fn affects_character(&self, char_index: usize) -> bool {
        self.target_characters.contains(&char_index)
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.current_time = 0.0;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn resume(&mut self) {
        self.playing = true;
    }

    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.current_value = None;
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Animation ID cannot be empty".to_string());
        }

        if self.target_characters.is_empty() {
            return Err("Animation must target at least one character".to_string());
        }

        if self.duration <= 0.0 {
            return Err("Animation duration must be positive".to_string());
        }

        if self.keyframes.is_empty() {
            return Err("Animation must have at least one keyframe".to_string());
        }

        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time < 0.0 || keyframe.time > self.duration {
                return Err(format!("Keyframe {} time is outside animation duration", i));
            }
        }

        Ok(())
    }

    pub fn clone_without_id(&self, new_id: String) -> Self {
        Self {
            id: new_id,
            name: self.name.clone(),
            animation_type: self.animation_type.clone(),
            keyframes: self.keyframes.clone(),
            target_characters: self.target_characters.clone(),
            duration: self.duration,
            delay: self.delay,
            loops: self.loops,
            loop_count: self.loop_count,
            playing: self.playing,
            current_time: self.current_time,
            current_value: self.current_value.clone(),
            easing: self.easing,
            character_delays: self.character_delays.clone(),
        }
    }
}

impl TextAnimator {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            global_time: 0.0,
            playing: false,
            playback_speed: 1.0,
            start_time: 0.0,
        }
    }

    pub fn add_animation(&mut self, animation: CharacterAnimation) {
        self.animations.push(animation);
    }

    pub fn remove_animation(&mut self, animation_id: &str) -> bool {
        let initial_len = self.animations.len();
        self.animations.retain(|anim| anim.id != animation_id);
        self.animations.len() < initial_len
    }

    pub fn get_animation(&self, animation_id: &str) -> Option<&CharacterAnimation> {
        self.animations.iter().find(|anim| anim.id == animation_id)
    }

    pub fn get_animation_mut(&mut self, animation_id: &str) -> Option<&mut CharacterAnimation> {
        self.animations.iter_mut().find(|anim| anim.id == animation_id)
    }

    pub fn update(&mut self, delta_time: f64) {
        if self.playing {
            self.global_time += delta_time * self.playback_speed;

            for animation in &mut self.animations {
                if animation.playing {

                }
            }
        }
    }

    pub fn get_character_animations(&self, char_index: usize) -> Vec<&CharacterAnimation> {
        self.animations
            .iter()
            .filter(|anim| anim.affects_character(char_index))
            .collect()
    }

    pub fn get_character_values(&self, char_index: usize) -> HashMap<AnimationType, AnimationValue> {
        let mut values = HashMap::new();

        for animation in &self.animations {
            if animation.affects_character(char_index) {
                if let Some(value) = animation.evaluate(self.global_time, char_index) {
                    values.insert(animation.animation_type.clone(), value);
                }
            }
        }

        values
    }

    pub fn play_all(&mut self) {
        self.playing = true;
        self.global_time = 0.0;
        for animation in &mut self.animations {
            animation.play();
        }
    }

    pub fn stop_all(&mut self) {
        self.playing = false;
        self.global_time = 0.0;
        for animation in &mut self.animations {
            animation.stop();
        }
    }

    pub fn pause_all(&mut self) {
        self.playing = false;
        for animation in &mut self.animations {
            animation.pause();
        }
    }

    pub fn resume_all(&mut self) {
        self.playing = true;
        for animation in &mut self.animations {
            animation.resume();
        }
    }

    pub fn reset_all(&mut self) {
        self.global_time = 0.0;
        for animation in &mut self.animations {
            animation.reset();
        }
    }

    pub fn create_staggered_animation(
        &mut self,
        base_animation: CharacterAnimation,
        stagger_delay: f64,
    ) -> String {
        let mut staggered_animations = Vec::new();

        for (i, &char_index) in base_animation.target_characters.iter().enumerate() {
            let mut anim = base_animation.clone_without_id(format!("{}_stagger_{}", base_animation.id, i));
            anim.target_characters = vec![char_index];
            anim.delay = base_animation.delay + (i as f64 * stagger_delay);
            staggered_animations.push(anim);
        }

        for anim in staggered_animations {
            self.add_animation(anim);
        }

        format!("{}_staggered", base_animation.id)
    }

    pub fn validate(&self) -> Result<(), String> {
        for (i, animation) in self.animations.iter().enumerate() {
            animation.validate().map_err(|e| format!("Animation {}: {}", i, e))?;
        }
        Ok(())
    }

    pub fn get_animation_count(&self) -> usize {
        self.animations.len()
    }

    pub fn clear_animations(&mut self) {
        self.animations.clear();
    }

    pub fn set_playback_speed(&mut self, speed: f64) {
        if speed > 0.0 {
            self.playback_speed = speed;
        }
    }
}

impl Default for TextAnimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{EasingFunction, InterpolationMethod};

    #[test]
    fn test_character_animation_creation() {
        let animation = CharacterAnimation::new(
            "fade_in".to_string(),
            "Fade In".to_string(),
            AnimationType::Opacity,
            vec![0, 1, 2, 3, 4],
        );

        assert_eq!(animation.id, "fade_in");
        assert_eq!(animation.name, "Fade In");
        assert_eq!(animation.animation_type, AnimationType::Opacity);
        assert_eq!(animation.target_characters, vec![0, 1, 2, 3, 4]);
        assert_eq!(animation.duration, 1.0);
        assert!(!animation.playing);
    }

    #[test]
    fn test_keyframe_management() {
        let mut animation = CharacterAnimation::new(
            "test".to_string(),
            "Test".to_string(),
            AnimationType::Opacity,
            vec![0],
        );

        let keyframe1 = TextKeyframe {
            time: 0.0,
            value: AnimationValue::Float(0.0),
            easing: EasingFunction::Linear,
            interpolation: InterpolationMethod::Linear,
        };

        let keyframe2 = TextKeyframe {
            time: 1.0,
            value: AnimationValue::Float(1.0),
            easing: EasingFunction::Linear,
            interpolation: InterpolationMethod::Linear,
        };

        animation.add_keyframe(keyframe1);
        animation.add_keyframe(keyframe2);

        assert_eq!(animation.keyframes.len(), 2);
        assert_eq!(animation.keyframes[0].time, 0.0);
        assert_eq!(animation.keyframes[1].time, 1.0);

        assert!(animation.remove_keyframe(0.0));
        assert_eq!(animation.keyframes.len(), 1);
        assert!(!animation.remove_keyframe(0.0));
    }

    #[test]
    fn test_animation_evaluation() {
        let mut animation = CharacterAnimation::new(
            "test".to_string(),
            "Test".to_string(),
            AnimationType::Opacity,
            vec![0],
        );

        animation.add_keyframe(TextKeyframe {
            time: 0.0,
            value: AnimationValue::Float(0.0),
            easing: EasingFunction::Linear,
            interpolation: InterpolationMethod::Linear,
        });

        animation.add_keyframe(TextKeyframe {
            time: 1.0,
            value: AnimationValue::Float(1.0),
            easing: EasingFunction::Linear,
            interpolation: InterpolationMethod::Linear,
        });

        let result = animation.evaluate(0.0, 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), AnimationValue::Float(0.0));

        let result = animation.evaluate(1.0, 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), AnimationValue::Float(1.0));

        let result = animation.evaluate(0.5, 0);
        assert!(result.is_some());
        if let AnimationValue::Float(value) = result.unwrap() {
            assert!((value - 0.5).abs() < 0.001);
        }
    }

    #[test]
    fn test_character_delays() {
        let mut animation = CharacterAnimation::new(
            "test".to_string(),
            "Test".to_string(),
            AnimationType::Opacity,
            vec![0, 1],
        );

        animation.set_character_delay(0, 0.5);
        animation.set_character_delay(1, 1.0);

        assert_eq!(animation.get_character_delay(0), Some(0.5));
        assert_eq!(animation.get_character_delay(1), Some(1.0));
        assert_eq!(animation.get_character_delay(2), None);

        assert!(animation.remove_character_delay(0));
        assert_eq!(animation.get_character_delay(0), None);
        assert!(!animation.remove_character_delay(2));
    }

    #[test]
    fn test_text_animator() {
        let mut animator = TextAnimator::new();

        let animation = CharacterAnimation::new(
            "fade_in".to_string(),
            "Fade In".to_string(),
            AnimationType::Opacity,
            vec![0, 1],
        );

        animator.add_animation(animation);
        assert_eq!(animator.animations.len(), 1);

        let retrieved = animator.get_animation("fade_in");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "fade_in");

        assert!(animator.remove_animation("fade_in"));
        assert_eq!(animator.animations.len(), 0);
        assert!(!animator.remove_animation("nonexistent"));
    }

    #[test]
    fn test_animation_playback() {
        let mut animation = CharacterAnimation::new(
            "test".to_string(),
            "Test".to_string(),
            AnimationType::Opacity,
            vec![0],
        );

        assert!(!animation.playing);

        animation.play();
        assert!(animation.playing);
        assert_eq!(animation.current_time, 0.0);

        animation.pause();
        assert!(!animation.playing);

        animation.resume();
        assert!(animation.playing);

        animation.stop();
        assert!(!animation.playing);
        assert_eq!(animation.current_time, 0.0);
    }

    #[test]
    fn test_staggered_animation() {
        let mut animator = TextAnimator::new();

        let base_animation = CharacterAnimation::new(
            "base".to_string(),
            "Base".to_string(),
            AnimationType::Opacity,
            vec![0, 1, 2],
        );

        let staggered_id = animator.create_staggered_animation(base_animation, 0.1);

        assert_eq!(staggered_id, "base_staggered");
        assert_eq!(animator.animations.len(), 3);

        for i in 0..3 {
            let anim = animator.get_animation(&format!("base_stagger_{}", i));
            assert!(anim.is_some());
            assert_eq!(anim.unwrap().target_characters, vec![i]);
            assert_eq!(anim.unwrap().delay, i as f64 * 0.1);
        }
    }

    #[test]
    fn test_animation_validation() {
        let animation = CharacterAnimation::new(
            "test".to_string(),
            "Test".to_string(),
            AnimationType::Opacity,
            vec![0],
        );

        assert!(animation.validate().is_ok());

        let mut invalid_animation = animation.clone();
        invalid_animation.id = "".to_string();
        assert!(invalid_animation.validate().is_err());

        invalid_animation.id = "test".to_string();
        invalid_animation.target_characters = vec![];
        assert!(invalid_animation.validate().is_err());

        invalid_animation.target_characters = vec![0];
        invalid_animation.duration = -1.0;
        assert!(invalid_animation.validate().is_err());
    }

    #[test]
    fn test_value_interpolation() {
        let animation = CharacterAnimation::new(
            "test".to_string(),
            "Test".to_string(),
            AnimationType::Opacity,
            vec![0],
        );

        let from = AnimationValue::Float(0.0);
        let to = AnimationValue::Float(1.0);
        let easing = EasingFunction::Linear;

        let result = animation.interpolate_values(&from, &to, 0.5, &easing);

        if let AnimationValue::Float(value) = result {
            assert!((value - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_animator_playback_controls() {
        let mut animator = TextAnimator::new();

        assert!(!animator.playing);
        assert_eq!(animator.global_time, 0.0);
        assert_eq!(animator.playback_speed, 1.0);

        animator.play_all();
        assert!(animator.playing);
        assert_eq!(animator.global_time, 0.0);

        animator.pause_all();
        assert!(!animator.playing);

        animator.resume_all();
        assert!(animator.playing);

        animator.stop_all();
        assert!(!animator.playing);
        assert_eq!(animator.global_time, 0.0);
    }

    #[test]
    fn test_animator_playback_speed() {
        let mut animator = TextAnimator::new();

        animator.set_playback_speed(2.0);
        assert_eq!(animator.playback_speed, 2.0);

        animator.set_playback_speed(0.5);
        assert_eq!(animator.playback_speed, 0.5);

        animator.set_playback_speed(-1.0);
        assert_eq!(animator.playback_speed, 0.5);
    }

    #[test]
    fn test_character_animation_clone() {
        let original = CharacterAnimation::new(
            "original".to_string(),
            "Original".to_string(),
            AnimationType::Opacity,
            vec![0, 1],
        );

        let cloned = original.clone_without_id("cloned".to_string());

        assert_eq!(cloned.id, "cloned");
        assert_eq!(cloned.name, original.name);
        assert_eq!(cloned.animation_type, original.animation_type);
        assert_eq!(cloned.target_characters, original.target_characters);
        assert_ne!(cloned.id, original.id);
    }

    #[test]
    fn test_animator_clear() {
        let mut animator = TextAnimator::new();

        let animation = CharacterAnimation::new(
            "test".to_string(),
            "Test".to_string(),
            AnimationType::Opacity,
            vec![0],
        );

        animator.add_animation(animation);
        assert_eq!(animator.get_animation_count(), 1);

        animator.clear_animations();
        assert_eq!(animator.get_animation_count(), 0);
    }
}
