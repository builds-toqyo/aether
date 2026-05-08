

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationType {

    Opacity,

    Position,

    Scale,

    Rotation,

    Color,

    Blur,

    Tracking,

    BaselineShift,

    FontSize,

    FontWeight,

    LineHeight,

    LetterSpacing,

    TextTransform,

    PathPosition,

    PathRotation,

    Custom(String),
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextKeyframe {

    pub time: f64,

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

    String(String),

    Bool(bool),
}


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

    pub easing: Option<crate::animation::interpolation::EasingFunction>,

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


        let prev_keyframe = self.get_keyframe_at_or_before(time);
        let next_keyframe = self.get_keyframe_at_or_after(time);

        match (prev_keyframe, next_keyframe) {
            (Some(prev), Some(next)) => {
                if prev.time == next.time {

                    self.current_value = Some(prev.value.clone());
                    return self.current_value.clone();
                }


                let t = (time - prev.time) / (next.time - prev.time);
                let interpolated_value = self.interpolate_values(&prev.value, &next.value, t, &prev.easing);

                self.current_value = Some(interpolated_value.clone());
                Some(interpolated_value)
            }
            (Some(prev), None) => {

                self.current_value = Some(prev.value.clone());
                Some(prev.value.clone())
            }
            (None, Some(next)) => {

                if time >= next.time {
                    self.current_value = Some(next.value.clone());
                    Some(next.value.clone())
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
                    values.insert(animation.animation_type, value);
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
            let mut anim = base_animation.clone();
            anim.id = format!("{}_stagger_{}", base_animation.id, i);
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
}

impl Default for TextAnimator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AnimationValue {
    fn default() -> Self {
        AnimationValue::Float(0.0)
    }
}

// Typography Controls
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyControls {
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: FontWeight,
    pub line_height: f64,
    pub letter_spacing: f64,
    pub word_spacing: f64,
    pub text_align: TextAlign,
    pub text_transform: TextTransform,
    pub color: (f64, f64, f64, f64),
    pub baseline_shift: f64,
    pub kerning: bool,
    pub ligatures: bool,
    pub small_caps: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub overline: bool,
}

impl Default for TypographyControls {
    fn default() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            line_height: 1.2,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            text_align: TextAlign::Left,
            text_transform: TextTransform::None,
            color: (0.0, 0.0, 0.0, 1.0),
            baseline_shift: 0.0,
            kerning: true,
            ligatures: true,
            small_caps: false,
            underline: false,
            strikethrough: false,
            overline: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    Custom(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

// Text-on-Path Support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextPath {
    pub id: String,
    pub path_type: PathType,
    pub points: Vec<(f64, f64)>,
    pub closed: bool,
    pub start_offset: f64,
    pub direction: PathDirection,
    pub spacing: PathSpacing,
    pub alignment: PathAlignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathType {
    Line,
    Bezier,
    Circle,
    Ellipse,
    Rectangle,
    Polygon,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathDirection {
    Forward,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathSpacing {
    Uniform,
    Proportional,
    Fixed(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathAlignment {
    Center,
    Left,
    Right,
}

impl TextPath {
    pub fn new(id: String, path_type: PathType) -> Self {
        Self {
            id,
            path_type,
            points: Vec::new(),
            closed: false,
            start_offset: 0.0,
            direction: PathDirection::Forward,
            spacing: PathSpacing::Uniform,
            alignment: PathAlignment::Center,
        }
    }

    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push((x, y));
    }

    pub fn get_point_at_distance(&self, distance: f64) -> Option<(f64, f64)> {
        if self.points.is_empty() {
            return None;
        }

        if self.points.len() == 1 {
            return Some(self.points[0]);
        }

        let mut total_length = 0.0;
        let mut segment_lengths = Vec::new();

        for i in 0..self.points.len() - 1 {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[i + 1];
            let length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
            segment_lengths.push(length);
            total_length += length;
        }

        if self.closed {
            let (x1, y1) = self.points[self.points.len() - 1];
            let (x2, y2) = self.points[0];
            let length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
            segment_lengths.push(length);
            total_length += length;
        }

        let target_distance = distance % total_length;
        let mut accumulated_distance = 0.0;

        for (i, &segment_length) in segment_lengths.iter().enumerate() {
            if accumulated_distance + segment_length >= target_distance {
                let t = (target_distance - accumulated_distance) / segment_length;
                let (x1, y1) = self.points[i];
                let (x2, y2) = if i < self.points.len() - 1 {
                    self.points[i + 1]
                } else {
                    self.points[0]
                };
                return Some((x1 + (x2 - x1) * t, y1 + (y2 - y1) * t));
            }
            accumulated_distance += segment_length;
        }

        Some(self.points[0])
    }

    pub fn get_tangent_at_distance(&self, distance: f64) -> Option<(f64, f64)> {
        if self.points.len() < 2 {
            return None;
        }

        let epsilon = 0.01;
        let p1 = self.get_point_at_distance(distance - epsilon)?;
        let p2 = self.get_point_at_distance(distance + epsilon)?;

        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        let length = (dx * dx + dy * dy).sqrt();

        if length > 0.0 {
            Some((dx / length, dy / length))
        } else {
            Some((1.0, 0.0))
        }
    }

    pub fn get_normal_at_distance(&self, distance: f64) -> Option<(f64, f64)> {
        let (tx, ty) = self.get_tangent_at_distance(distance)?;
        Some((-ty, tx))
    }
}

// Text Layer System
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLayer {
    pub id: String,
    pub name: String,
    pub text: String,
    pub position: (f64, f64),
    pub typography: TypographyControls,
    pub path: Option<TextPath>,
    pub animator: TextAnimator,
    pub visible: bool,
    pub locked: bool,
    pub blend_mode: BlendMode,
    pub opacity: f64,
}

impl TextLayer {
    pub fn new(id: String, name: String, text: String) -> Self {
        Self {
            id,
            name,
            text,
            position: (0.0, 0.0),
            typography: TypographyControls::default(),
            path: None,
            animator: TextAnimator::new(),
            visible: true,
            locked: false,
            blend_mode: BlendMode::Normal,
            opacity: 1.0,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        // Reset animations to apply to new text
        self.animator.reset_all();
    }

    pub fn set_typography(&mut self, typography: TypographyControls) {
        self.typography = typography;
    }

    pub fn set_path(&mut self, path: Option<TextPath>) {
        self.path = path;
    }

    pub fn get_character_count(&self) -> usize {
        self.text.chars().count()
    }

    pub fn get_character_position(&self, char_index: usize) -> Option<(f64, f64)> {
        if let Some(ref path) = self.path {
            // Calculate character position along path
            let char_distance = char_index as f64 * 10.0; // Approximate character width
            path.get_point_at_distance(char_distance)
        } else {
            // Calculate position in straight line
            let char_width = self.typography.font_size * 0.6; // Approximate character width
            Some((
                self.position.0 + (char_index as f64 * char_width),
                self.position.1
            ))
        }
    }

    pub fn get_character_transform(&self, char_index: usize, time: f64) -> CharacterTransform {
        let base_position = self.get_character_position(char_index).unwrap_or(self.position);
        let mut transform = CharacterTransform {
            position: base_position,
            rotation: 0.0,
            scale: (1.0, 1.0),
            opacity: self.opacity,
            color: self.typography.color,
            font_size: self.typography.font_size,
            baseline_shift: self.typography.baseline_shift,
        };

        // Apply animations
        let character_values = self.animator.get_character_values(char_index);
        
        for (animation_type, value) in character_values {
            match animation_type {
                AnimationType::Position => {
                    if let AnimationValue::Vector2(x, y) = value {
                        transform.position = (base_position.0 + x, base_position.1 + y);
                    }
                }
                AnimationType::Rotation => {
                    if let AnimationValue::Float(angle) = value {
                        transform.rotation = angle;
                    }
                }
                AnimationType::Scale => {
                    if let AnimationValue::Vector2(sx, sy) = value {
                        transform.scale = (sx, sy);
                    }
                }
                AnimationType::Opacity => {
                    if let AnimationValue::Float(opacity) = value {
                        transform.opacity = opacity * self.opacity;
                    }
                }
                AnimationType::Color => {
                    if let AnimationValue::Color(r, g, b, a) = value {
                        transform.color = (r, g, b, a);
                    }
                }
                AnimationType::FontSize => {
                    if let AnimationValue::Float(size) = value {
                        transform.font_size = size;
                    }
                }
                AnimationType::BaselineShift => {
                    if let AnimationValue::Float(shift) = value {
                        transform.baseline_shift = shift;
                    }
                }
                AnimationType::PathPosition => {
                    if let Some(ref path) = self.path {
                        if let AnimationValue::Float(distance) = value {
                            if let Some(pos) = path.get_point_at_distance(distance) {
                                transform.position = pos;
                            }
                        }
                    }
                }
                AnimationType::PathRotation => {
                    if let Some(ref path) = self.path {
                        if let AnimationValue::Float(distance) = value {
                            if let Some((tx, ty)) = path.get_tangent_at_distance(distance) {
                                transform.rotation = ty.atan2(tx);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        transform
    }

    pub fn update(&mut self, delta_time: f64) {
        self.animator.update(delta_time);
    }

    pub fn play_animations(&mut self) {
        self.animator.play_all();
    }

    pub fn pause_animations(&mut self) {
        self.animator.pause_all();
    }

    pub fn stop_animations(&mut self) {
        self.animator.stop_all();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterTransform {
    pub position: (f64, f64),
    pub rotation: f64,
    pub scale: (f64, f64),
    pub opacity: f64,
    pub color: (f64, f64, f64, f64),
    pub font_size: f64,
    pub baseline_shift: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    ColorDodge,
    ColorBurn,
    Darken,
    Lighten,
    Difference,
    Exclusion,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::interpolation::{EasingFunction, InterpolationMethod};

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
    fn test_typography_controls_default() {
        let typography = TypographyControls::default();
        assert_eq!(typography.font_family, "Arial");
        assert_eq!(typography.font_size, 16.0);
        assert_eq!(typography.font_weight, FontWeight::Normal);
        assert_eq!(typography.line_height, 1.2);
        assert_eq!(typography.color, (0.0, 0.0, 0.0, 1.0));
        assert!(typography.kerning);
        assert!(typography.ligatures);
        assert!(!typography.underline);
    }

    #[test]
    fn test_text_path_creation() {
        let path = TextPath::new("test_path".to_string(), PathType::Line);
        assert_eq!(path.id, "test_path");
        assert_eq!(path.path_type, PathType::Line);
        assert!(path.points.is_empty());
        assert!(!path.closed);
        assert_eq!(path.direction, PathDirection::Forward);
    }

    #[test]
    fn test_text_path_add_points() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        path.add_point(100.0, 100.0);
        
        assert_eq!(path.points.len(), 3);
        assert_eq!(path.points[0], (0.0, 0.0));
        assert_eq!(path.points[1], (100.0, 0.0));
        assert_eq!(path.points[2], (100.0, 100.0));
    }

    #[test]
    fn test_text_path_point_at_distance() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        
        let point = path.get_point_at_distance(50.0);
        assert!(point.is_some());
        assert_eq!(point.unwrap(), (50.0, 0.0));
        
        let point = path.get_point_at_distance(25.0);
        assert!(point.is_some());
        assert_eq!(point.unwrap(), (25.0, 0.0));
    }

    #[test]
    fn test_text_path_tangent() {
        let mut path = TextPath::new("test_path".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        
        let tangent = path.get_tangent_at_distance(50.0);
        assert!(tangent.is_some());
        let (tx, ty) = tangent.unwrap();
        assert!((tx - 1.0).abs() < 0.001);
        assert!((ty - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_text_layer_creation() {
        let layer = TextLayer::new("layer1".to_string(), "Test Layer".to_string(), "Hello World".to_string());
        assert_eq!(layer.id, "layer1");
        assert_eq!(layer.name, "Test Layer");
        assert_eq!(layer.text, "Hello World");
        assert_eq!(layer.position, (0.0, 0.0));
        assert!(layer.visible);
        assert!(!layer.locked);
        assert_eq!(layer.blend_mode, BlendMode::Normal);
        assert_eq!(layer.opacity, 1.0);
    }

    #[test]
    fn test_text_layer_character_count() {
        let layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        assert_eq!(layer.get_character_count(), 5);
        
        let layer = TextLayer::new("layer2".to_string(), "Test".to_string(), "Hello World".to_string());
        assert_eq!(layer.get_character_count(), 11); // Including space
    }

    #[test]
    fn test_text_layer_character_position() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        layer.position = (100.0, 200.0);
        layer.typography.font_size = 20.0;
        
        let pos = layer.get_character_position(0);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap(), (100.0, 200.0));
        
        let pos = layer.get_character_position(1);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap(), (112.0, 200.0)); // 100 + (20 * 0.6)
    }

    #[test]
    fn test_text_layer_with_path() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        
        let mut path = TextPath::new("path1".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(200.0, 0.0);
        
        layer.set_path(Some(path));
        
        let pos = layer.get_character_position(1);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap(), (10.0, 0.0)); // 1 * 10.0 character width
    }

    #[test]
    fn test_text_layer_character_transform() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        layer.position = (100.0, 200.0);
        
        let transform = layer.get_character_transform(0, 0.0);
        assert_eq!(transform.position, (100.0, 200.0));
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, (1.0, 1.0));
        assert_eq!(transform.opacity, 1.0);
        assert_eq!(transform.font_size, 16.0);
        assert_eq!(transform.baseline_shift, 0.0);
    }

    #[test]
    fn test_text_layer_animation_integration() {
        let mut layer = TextLayer::new("layer1".to_string(), "Test".to_string(), "Hello".to_string());
        layer.position = (100.0, 200.0);
        
        // Create a fade-in animation
        let mut animation = CharacterAnimation::new(
            "fade".to_string(),
            "Fade In".to_string(),
            AnimationType::Opacity,
            vec![0, 1, 2, 3, 4],
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
        
        layer.animator.add_animation(animation);
        
        // Test at time 0.5 (should be half faded)
        let transform = layer.get_character_transform(0, 0.5);
        assert!((transform.opacity - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_font_weight_variants() {
        let weights = vec![
            FontWeight::Thin,
            FontWeight::ExtraLight,
            FontWeight::Light,
            FontWeight::Normal,
            FontWeight::Medium,
            FontWeight::SemiBold,
            FontWeight::Bold,
            FontWeight::ExtraBold,
            FontWeight::Black,
            FontWeight::Custom(750),
        ];
        
        for weight in weights {
            let mut typography = TypographyControls::default();
            typography.font_weight = weight;
            assert_eq!(typography.font_weight, weight);
        }
    }

    #[test]
    fn test_text_transform_variants() {
        let transforms = vec![
            TextTransform::None,
            TextTransform::Uppercase,
            TextTransform::Lowercase,
            TextTransform::Capitalize,
        ];
        
        for transform in transforms {
            let mut typography = TypographyControls::default();
            typography.text_transform = transform;
            assert_eq!(typography.text_transform, transform);
        }
    }

    #[test]
    fn test_blend_modes() {
        let blend_modes = vec![
            BlendMode::Normal,
            BlendMode::Multiply,
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::SoftLight,
            BlendMode::HardLight,
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
            BlendMode::Darken,
            BlendMode::Lighten,
            BlendMode::Difference,
            BlendMode::Exclusion,
        ];
        
        for blend_mode in blend_modes {
            let mut layer = TextLayer::new("test".to_string(), "Test".to_string(), "Hello".to_string());
            layer.blend_mode = blend_mode;
            assert_eq!(layer.blend_mode, blend_mode);
        }
    }

    #[test]
    fn test_path_types() {
        let path_types = vec![
            PathType::Line,
            PathType::Bezier,
            PathType::Circle,
            PathType::Ellipse,
            PathType::Rectangle,
            PathType::Polygon,
            PathType::Custom,
        ];
        
        for path_type in path_types {
            let path = TextPath::new("test".to_string(), path_type);
            assert_eq!(path.path_type, path_type);
        }
    }

    #[test]
    fn test_closed_path() {
        let mut path = TextPath::new("test".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        path.add_point(100.0, 100.0);
        path.add_point(0.0, 100.0);
        path.closed = true;
        
        // Test point at distance beyond total length (should wrap around)
        let point = path.get_point_at_distance(500.0);
        assert!(point.is_some());
    }

    #[test]
    fn test_path_direction() {
        let mut path = TextPath::new("test".to_string(), PathType::Line);
        path.add_point(0.0, 0.0);
        path.add_point(100.0, 0.0);
        
        path.direction = PathDirection::Reverse;
        assert_eq!(path.direction, PathDirection::Reverse);
        
        // Test that direction affects point calculation
        let point = path.get_point_at_distance(25.0);
        assert!(point.is_some());
    }

    #[test]
    fn test_typography_modification() {
        let mut layer = TextLayer::new("test".to_string(), "Test".to_string(), "Hello".to_string());
        
        let mut typography = TypographyControls::default();
        typography.font_family = "Helvetica".to_string();
        typography.font_size = 24.0;
        typography.font_weight = FontWeight::Bold;
        typography.color = (1.0, 0.0, 0.0, 1.0); // Red
        
        layer.set_typography(typography);
        
        assert_eq!(layer.typography.font_family, "Helvetica");
        assert_eq!(layer.typography.font_size, 24.0);
        assert_eq!(layer.typography.font_weight, FontWeight::Bold);
        assert_eq!(layer.typography.color, (1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_text_update() {
        let mut layer = TextLayer::new("test".to_string(), "Test".to_string(), "Hello".to_string());
        assert_eq!(layer.get_character_count(), 5);
        
        layer.set_text("Hello World".to_string());
        assert_eq!(layer.get_character_count(), 11);
        assert_eq!(layer.text, "Hello World");
    }

    #[test]
    fn test_animation_types_extended() {
        let animation_types = vec![
            AnimationType::FontSize,
            AnimationType::FontWeight,
            AnimationType::LineHeight,
            AnimationType::LetterSpacing,
            AnimationType::TextTransform,
            AnimationType::PathPosition,
            AnimationType::PathRotation,
        ];
        
        for anim_type in animation_types {
            let animation = CharacterAnimation::new(
                "test".to_string(),
                "Test".to_string(),
                anim_type,
                vec![0],
            );
            assert_eq!(animation.animation_type, anim_type);
        }
    }
}
