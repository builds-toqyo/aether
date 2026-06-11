use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TimelineError {
    InvalidTrack(String),
    InvalidClip(String),
    InvalidTime(String),
    OperationError(String),
}

impl fmt::Display for TimelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimelineError::InvalidTrack(msg) => write!(f, "Invalid track: {}", msg),
            TimelineError::InvalidClip(msg) => write!(f, "Invalid clip: {}", msg),
            TimelineError::InvalidTime(msg) => write!(f, "Invalid time: {}", msg),
            TimelineError::OperationError(msg) => write!(f, "Operation error: {}", msg),
        }
    }
}

impl Error for TimelineError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ClipType {
    Video,
    Audio,
    Image,
    Text,
    Effect,
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub id: String,
    pub clip_type: ClipType,
    pub start_time: f64,
    pub duration: f64,
    pub source_path: Option<String>,
    pub properties: HashMap<String, String>,
}

impl Clip {
    pub fn new(id: String, clip_type: ClipType, start_time: f64, duration: f64) -> Self {
        Self {
            id,
            clip_type,
            start_time,
            duration,
            source_path: None,
            properties: HashMap::new(),
        }
    }

    pub fn with_source(mut self, source_path: String) -> Self {
        self.source_path = Some(source_path);
        self
    }

    pub fn add_property(mut self, key: String, value: String) -> Self {
        self.properties.insert(key, value);
        self
    }

    pub fn end_time(&self) -> f64 {
        self.start_time + self.duration
    }

    pub fn contains_time(&self, time: f64) -> bool {
        time >= self.start_time && time < self.end_time()
    }
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub clips: Vec<Clip>,
    pub is_muted: bool,
    pub is_locked: bool,
}

impl Track {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            clips: Vec::new(),
            is_muted: false,
            is_locked: false,
        }
    }

    pub fn add_clip(&mut self, clip: Clip) -> Result<(), TimelineError> {

        for existing_clip in &self.clips {
            if existing_clip.clip_type == clip.clip_type &&
               ((clip.start_time >= existing_clip.start_time && clip.start_time < existing_clip.end_time()) ||
                (clip.end_time() > existing_clip.start_time && clip.end_time() <= existing_clip.end_time())) {
                return Err(TimelineError::OperationError(
                    format!("Clip overlaps with existing clip {} of the same type", existing_clip.id)
                ));
            }
        }

        self.clips.push(clip);
        Ok(())
    }

    pub fn remove_clip(&mut self, clip_id: &str) -> Result<Clip, TimelineError> {
        if let Some(index) = self.clips.iter().position(|clip| clip.id == clip_id) {
            Ok(self.clips.remove(index))
        } else {
            Err(TimelineError::InvalidClip(format!("Clip with id {} not found", clip_id)))
        }
    }

    pub fn clips_at_time(&self, time: f64) -> Vec<&Clip> {
        self.clips.iter()
            .filter(|clip| clip.contains_time(time))
            .collect()
    }
}

pub struct TimelineConfig {
    pub fps: u32,
    pub duration: f64,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            duration: 60.0,
        }
    }
}

pub struct Timeline {
    config: TimelineConfig,
    tracks: HashMap<String, Track>,
    current_time: f64,
    state: Arc<Mutex<TimelineState>>,
}

struct TimelineState {
    is_playing: bool,
    playback_speed: f64,
    last_update_time: std::time::Instant,
}

impl Timeline {
    pub fn new(config: TimelineConfig) -> Self {
        let state = TimelineState {
            is_playing: false,
            playback_speed: 1.0,
            last_update_time: std::time::Instant::now(),
        };

        Self {
            config,
            tracks: HashMap::new(),
            current_time: 0.0,
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn add_track(&mut self, track: Track) -> Result<(), TimelineError> {
        if self.tracks.contains_key(&track.id) {
            return Err(TimelineError::InvalidTrack(
                format!("Track with id {} already exists", track.id)
            ));
        }

        self.tracks.insert(track.id.clone(), track);
        Ok(())
    }

    pub fn remove_track(&mut self, track_id: &str) -> Result<Track, TimelineError> {
        if let Some(track) = self.tracks.remove(track_id) {
            Ok(track)
        } else {
            Err(TimelineError::InvalidTrack(
                format!("Track with id {} not found", track_id)
            ))
        }
    }

    pub fn get_track(&self, track_id: &str) -> Result<&Track, TimelineError> {
        self.tracks.get(track_id).ok_or_else(|| {
            TimelineError::InvalidTrack(format!("Track with id {} not found", track_id))
        })
    }

    pub fn get_track_mut(&mut self, track_id: &str) -> Result<&mut Track, TimelineError> {
        self.tracks.get_mut(track_id).ok_or_else(|| {
            TimelineError::InvalidTrack(format!("Track with id {} not found", track_id))
        })
    }

    pub fn add_clip_to_track(&mut self, track_id: &str, clip: Clip) -> Result<(), TimelineError> {
        let track = self.get_track_mut(track_id)?;
        track.add_clip(clip)
    }


    pub fn remove_clip_from_track(&mut self, track_id: &str, clip_id: &str) -> Result<Clip, TimelineError> {
        let track = self.get_track_mut(track_id)?;
        track.remove_clip(clip_id)
    }


    pub fn seek(&mut self, time: f64) -> Result<(), TimelineError> {
        if time < 0.0 || time > self.config.duration {
            return Err(TimelineError::InvalidTime(
                format!("Time {} is outside timeline bounds (0 to {})", time, self.config.duration)
            ));
        }

        self.current_time = time;
        Ok(())
    }


    pub fn play(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.is_playing = true;
        state.last_update_time = std::time::Instant::now();
    }


    pub fn pause(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.is_playing = false;
    }


    pub fn set_playback_speed(&mut self, speed: f64) -> Result<(), TimelineError> {
        if speed <= 0.0 {
            return Err(TimelineError::OperationError(
                format!("Invalid playback speed: {}", speed)
            ));
        }

        let mut state = self.state.lock().unwrap();
        state.playback_speed = speed;
        Ok(())
    }


    pub fn update(&mut self) -> Result<f64, TimelineError> {
        let mut state = self.state.lock().unwrap();

        if state.is_playing {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(state.last_update_time).as_secs_f64();
            state.last_update_time = now;


            self.current_time += elapsed * state.playback_speed;


            if self.current_time >= self.config.duration {
                self.current_time = self.config.duration;
                state.is_playing = false;
            }
        }

        Ok(self.current_time)
    }


    pub fn active_clips(&self) -> HashMap<String, Vec<&Clip>> {
        let mut result = HashMap::new();

        for (track_id, track) in &self.tracks {
            if !track.is_muted {
                let clips = track.clips_at_time(self.current_time);
                if !clips.is_empty() {
                    result.insert(track_id.clone(), clips);
                }
            }
        }

        result
    }


    pub fn current_time(&self) -> f64 {
        self.current_time
    }


    pub fn duration(&self) -> f64 {
        self.config.duration
    }


    pub fn set_duration(&mut self, duration: f64) -> Result<(), TimelineError> {
        if duration <= 0.0 {
            return Err(TimelineError::OperationError(
                format!("Invalid duration: {}", duration)
            ));
        }

        self.config.duration = duration;


        if self.current_time > duration {
            self.current_time = duration;
        }

        Ok(())
    }


    pub fn tracks(&self) -> &HashMap<String, Track> {
        &self.tracks
    }


    pub fn is_playing(&self) -> bool {
        self.state.lock().unwrap().is_playing
    }
}


pub fn create_default_timeline() -> Timeline {
    Timeline::new(TimelineConfig::default())
}
