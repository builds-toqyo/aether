

use aether_types::animation::{
    AnimationTrack, AnimationTrackCollection, TrackValue, InterpolationMethod,
    EasingFunction
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::interpolation::{AnimationInterpolator, InterpolationResult};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Seeking,
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState::Stopped
    }
}


#[derive(Debug, Clone)]
pub struct AnimationState {
    pub playback_state: PlaybackState,
    pub current_time: f64,
    pub start_time: f64,
    pub end_time: f64,
    pub playback_speed: f64,
    pub loop_animation: bool,
    pub duration: f64,
    pub reversed: bool,
}

impl AnimationState {

    pub fn new() -> Self {
        Self {
            playback_state: PlaybackState::Stopped,
            current_time: 0.0,
            start_time: 0.0,
            end_time: 1.0,
            playback_speed: 1.0,
            loop_animation: false,
            duration: 1.0,
            reversed: false,
        }
    }


    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration.max(0.0);
        self.end_time = self.start_time + self.duration;
    }


    pub fn set_time_range(&mut self, start_time: f64, end_time: f64) {
        self.start_time = start_time;
        self.end_time = end_time;
        self.duration = end_time - start_time;
        self.current_time = self.current_time.clamp(start_time, end_time);
    }


    pub fn is_at_end(&self) -> bool {
        if self.reversed {
            self.current_time <= self.start_time
        } else {
            self.current_time >= self.end_time
        }
    }


    pub fn is_at_start(&self) -> bool {
        if self.reversed {
            self.current_time >= self.end_time
        } else {
            self.current_time <= self.start_time
        }
    }


    pub fn normalized_time(&self) -> f64 {
        if self.duration <= 0.0 {
            0.0
        } else {
            ((self.current_time - self.start_time) / self.duration).clamp(0.0, 1.0)
        }
    }


    pub fn set_normalized_time(&mut self, normalized_time: f64) {
        let t = normalized_time.clamp(0.0, 1.0);
        self.current_time = self.start_time + t * self.duration;
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}


pub struct AnimationEngine {

    state: AnimationState,

    tracks: AnimationTrackCollection,

    interpolator: AnimationInterpolator,

    current_values: HashMap<String, InterpolationResult>,

    last_update: Instant,

    stats: AnimationEngineStats,
}

impl AnimationEngine {

    pub fn new() -> Self {
        Self {
            state: AnimationState::new(),
            tracks: AnimationTrackCollection::new("default".to_string()),
            interpolator: AnimationInterpolator::new(),
            current_values: HashMap::new(),
            last_update: Instant::now(),
            stats: AnimationEngineStats::default(),
        }
    }


    pub fn with_interpolator(interpolator: AnimationInterpolator) -> Self {
        Self {
            state: AnimationState::new(),
            tracks: AnimationTrackCollection::new("default".to_string()),
            interpolator,
            current_values: HashMap::new(),
            last_update: Instant::now(),
            stats: AnimationEngineStats::default(),
        }
    }


    pub fn add_track(&mut self, track: AnimationTrack) -> Result<(), String> {
        self.tracks.add_track(track)?;
        self.update_time_range();
        Ok(())
    }


    pub fn remove_track(&mut self, track_id: &str) -> Option<AnimationTrack> {
        let track = self.tracks.remove_track(track_id);
        if track.is_some() {
            self.current_values.remove(track_id);
            self.update_time_range();
        }
        track
    }


    pub fn get_track(&self, track_id: &str) -> Option<&AnimationTrack> {
        self.tracks.get_track(track_id)
    }


    pub fn get_tracks(&self) -> Vec<&AnimationTrack> {
        self.tracks.get_tracks()
    }


    pub fn get_enabled_tracks(&self) -> Vec<&AnimationTrack> {
        self.tracks.get_enabled_tracks()
    }


    pub fn play(&mut self) {
        if self.state.playback_state == PlaybackState::Stopped {
            if self.state.is_at_end() && !self.state.loop_animation {
                self.state.current_time = self.state.start_time;
            }
        }
        self.state.playback_state = PlaybackState::Playing;
        self.last_update = Instant::now();
        self.stats.playbacks_started += 1;
    }


    pub fn pause(&mut self) {
        self.state.playback_state = PlaybackState::Paused;
    }


    pub fn stop(&mut self) {
        self.state.playback_state = PlaybackState::Stopped;
        self.state.current_time = self.state.start_time;
        self.last_update = Instant::now();
    }


    pub fn seek(&mut self, time: f64) {
        self.state.playback_state = PlaybackState::Seeking;
        self.state.current_time = time.clamp(self.state.start_time, self.state.end_time);
        self.evaluate_at_time(self.state.current_time);
        self.state.playback_state = PlaybackState::Paused;
    }


    pub fn seek_normalized(&mut self, normalized_time: f64) {
        let time = self.state.start_time + normalized_time * self.state.duration;
        self.seek(time);
    }


    pub fn update(&mut self) -> &HashMap<String, InterpolationResult> {
        if self.state.playback_state != PlaybackState::Playing {
            return &self.current_values;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        self.last_update = now;


        let delta_seconds = elapsed.as_secs_f64() * self.state.playback_speed;

        if self.state.reversed {
            self.state.current_time -= delta_seconds;
        } else {
            self.state.current_time += delta_seconds;
        }


        if self.state.is_at_end() {
            if self.state.loop_animation {
                if self.state.reversed {
                    self.state.current_time = self.state.end_time;
                } else {
                    self.state.current_time = self.state.start_time;
                }
                self.stats.loops_completed += 1;
            } else {
                self.state.playback_state = PlaybackState::Stopped;
            }
        }

        if self.state.is_at_start() && self.state.reversed {
            if self.state.loop_animation {
                self.state.current_time = self.state.end_time;
                self.stats.loops_completed += 1;
            } else {
                self.state.playback_state = PlaybackState::Stopped;
            }
        }


        self.evaluate_at_time(self.state.current_time);
        self.stats.frames_evaluated += 1;

        &self.current_values
    }


    pub fn evaluate_at_time(&mut self, time: f64) -> &HashMap<String, InterpolationResult> {
        self.current_values.clear();

        for track in self.tracks.get_enabled_tracks() {
            let result = self.interpolator.interpolate_at_time(&track.keyframes, time);
            self.current_values.insert(track.id.clone(), result);
        }

        &self.current_values
    }


    pub fn get_track_value(&self, track_id: &str) -> Option<&InterpolationResult> {
        self.current_values.get(track_id)
    }


    pub fn get_track_value_as_float(&self, track_id: &str) -> Option<f64> {
        self.current_values.get(track_id)
            .and_then(|result| result.value.as_float())
    }


    pub fn get_track_value_as_vector3(&self, track_id: &str) -> Option<[f64; 3]> {
        self.current_values.get(track_id)
            .and_then(|result| result.value.as_vector3())
    }


    pub fn get_track_value_as_color(&self, track_id: &str) -> Option<[f64; 4]> {
        self.current_values.get(track_id)
            .and_then(|result| result.value.as_color())
    }


    pub fn set_playback_speed(&mut self, speed: f64) {
        self.state.playback_speed = speed.max(0.0);
    }


    pub fn set_loop(&mut self, loop_animation: bool) {
        self.state.loop_animation = loop_animation;
    }


    pub fn set_reversed(&mut self, reversed: bool) {
        self.state.reversed = reversed;
    }


    pub fn get_state(&self) -> &AnimationState {
        &self.state
    }


    pub fn get_state_mut(&mut self) -> &mut AnimationState {
        &mut self.state
    }


    pub fn get_stats(&self) -> &AnimationEngineStats {
        &self.stats
    }


    pub fn reset_stats(&mut self) {
        self.stats = AnimationEngineStats::default();
        self.interpolator.reset_stats();
    }


    pub fn clear_tracks(&mut self) {
        self.tracks.clear();
        self.current_values.clear();
        self.state.set_time_range(0.0, 1.0);
    }


    fn update_time_range(&mut self) {
        if let Some((min_time, max_time)) = self.tracks.time_range() {
            self.state.set_time_range(min_time, max_time);
        } else {
            self.state.set_time_range(0.0, 1.0);
        }
    }
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone, Default)]
pub struct AnimationEngineStats {

    pub playbacks_started: u64,

    pub loops_completed: u64,

    pub frames_evaluated: u64,

    pub avg_evaluation_time_ms: f64,

    pub total_evaluation_time_ms: f64,
}

impl AnimationEngineStats {

    pub fn fps(&self) -> f64 {
        if self.total_evaluation_time_ms > 0.0 {
            (self.frames_evaluated as f64) / (self.total_evaluation_time_ms / 1000.0)
        } else {
            0.0
        }
    }


    pub fn avg_time_per_frame_ms(&self) -> f64 {
        if self.frames_evaluated > 0 {
            self.total_evaluation_time_ms / self.frames_evaluated as f64
        } else {
            0.0
        }
    }
}


pub struct AnimationEngineBuilder {
    engine: AnimationEngine,
}

impl AnimationEngineBuilder {

    pub fn new() -> Self {
        Self {
            engine: AnimationEngine::new(),
        }
    }


    pub fn duration(mut self, duration: f64) -> Self {
        self.engine.state.set_duration(duration);
        self
    }


    pub fn playback_speed(mut self, speed: f64) -> Self {
        self.engine.set_playback_speed(speed);
        self
    }


    pub fn loop_animation(mut self, loop_animation: bool) -> Self {
        self.engine.set_loop(loop_animation);
        self
    }


    pub fn track(mut self, track: AnimationTrack) -> Self {
        let _ = self.engine.add_track(track);
        self
    }


    pub fn tracks(mut self, tracks: Vec<AnimationTrack>) -> Self {
        for track in tracks {
            let _ = self.engine.add_track(track);
        }
        self
    }


    pub fn build(self) -> AnimationEngine {
        self.engine
    }
}

impl Default for AnimationEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::animation::{Keyframe, KeyframeData, TrackType, ParameterBinding};

    #[test]
    fn test_animation_engine_creation() {
        let engine = AnimationEngine::new();

        assert_eq!(engine.state.playback_state, PlaybackState::Stopped);
        assert_eq!(engine.state.current_time, 0.0);
        assert_eq!(engine.tracks.track_count(), 0);
    }

    #[test]
    fn test_track_addition() {
        let mut engine = AnimationEngine::new();

        let binding = ParameterBinding::new("object1".to_string(), "position".to_string());
        let track = AnimationTrack::with_default_binding(
            "track1".to_string(),
            "Position Track".to_string(),
            TrackType::Position,
            "object1".to_string()
        );


        let mut track_with_keyframe = track;
        track_with_keyframe.add_keyframe(KeyframeData::Vector3(Keyframe::new(0.0, [0.0, 0.0, 0.0])));

        engine.add_track(track_with_keyframe).unwrap();

        assert_eq!(engine.tracks.track_count(), 1);
        assert!(engine.get_track("track1").is_some());
    }

    #[test]
    fn test_playback_controls() {
        let mut engine = AnimationEngine::new();


        assert_eq!(engine.state.playback_state, PlaybackState::Stopped);


        engine.play();
        assert_eq!(engine.state.playback_state, PlaybackState::Playing);


        engine.pause();
        assert_eq!(engine.state.playback_state, PlaybackState::Paused);


        engine.stop();
        assert_eq!(engine.state.playback_state, PlaybackState::Stopped);
        assert_eq!(engine.state.current_time, engine.state.start_time);
    }

    #[test]
    fn test_seeking() {
        let mut engine = AnimationEngine::new();

        engine.seek(0.5);
        assert_eq!(engine.state.current_time, 0.5);
        assert_eq!(engine.state.playback_state, PlaybackState::Paused);

        engine.seek_normalized(0.75);
        assert_eq!(engine.state.normalized_time(), 0.75);
    }

    #[test]
    fn test_animation_engine_builder() {
        let binding = ParameterBinding::new("object1".to_string(), "position".to_string());
        let track = AnimationTrack::with_default_binding(
            "track1".to_string(),
            "Position Track".to_string(),
            TrackType::Position,
            "object1".to_string()
        );

        let engine = AnimationEngineBuilder::new()
            .duration(2.0)
            .playback_speed(1.5)
            .loop_animation(true)
            .track(track)
            .build();

        assert_eq!(engine.state.duration, 2.0);
        assert_eq!(engine.state.playback_speed, 1.5);
        assert!(engine.state.loop_animation);
        assert_eq!(engine.tracks.track_count(), 1);
    }
}
