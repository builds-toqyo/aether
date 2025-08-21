use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use gstreamer as gst;
use gstreamer_editing_services as ges;
use crate::engine::editing::types::{EditingError, ClipInfo, TrackType};

pub struct Timeline {
    ges_timeline: Option<ges::Timeline>,
    
    video_tracks: Vec<TimelineTrack>,
    audio_tracks: Vec<TimelineTrack>,
    
    clips: HashMap<String, TimelineClip>,
    
    duration: i64,
}

impl Timeline {
    pub fn new() -> Result<Self, EditingError> {
        Ok(Self {
            ges_timeline: None,
            video_tracks: Vec::new(),
            audio_tracks: Vec::new(),
            clips: HashMap::new(),
            duration: 0,
        })
    }
    
    pub fn set_ges_timeline(&mut self, timeline: ges::Timeline) -> Result<(), EditingError> {
        self.ges_timeline = Some(timeline.clone());
        
        if self.video_tracks.is_empty() {
            self.add_video_track()?;
        }
        
        if self.audio_tracks.is_empty() {
            self.add_audio_track()?;
        }
        
        Ok(())
    }
    
    pub fn add_video_track(&mut self) -> Result<TimelineTrack, EditingError> {
        let timeline = self.ges_timeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;
        
        let track = ges::VideoTrack::new()?;
        timeline.add_track(&track)?;
        
        let track_id = format!("video_{}", self.video_tracks.len());
        let timeline_track = TimelineTrack {
            id: track_id.clone(),
            track_type: TrackType::Video,
            ges_track: track.upcast::<ges::Track>(),
            clips: Vec::new(),
        };
        
        self.video_tracks.push(timeline_track.clone());
        
        Ok(timeline_track)
    }
    
    pub fn add_audio_track(&mut self) -> Result<TimelineTrack, EditingError> {
        let timeline = self.ges_timeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;
        
        let track = ges::AudioTrack::new()?;
        timeline.add_track(&track)?;
        
        let track_id = format!("audio_{}", self.audio_tracks.len());
        let timeline_track = TimelineTrack {
            id: track_id.clone(),
            track_type: TrackType::Audio,
            ges_track: track.upcast::<ges::Track>(),
            clips: Vec::new(),
        };
        
        self.audio_tracks.push(timeline_track.clone());
        
        Ok(timeline_track)
    }
    
    pub fn add_clip(&mut self, 
                   uri: &str, 
                   track_type: TrackType, 
                   start_time: i64, 
                   duration: i64,
                   in_point: i64) -> Result<TimelineClip, EditingError> {
        let timeline = self.ges_timeline.as_ref()
            .ok_or(EditingError::NotInitialized)?;
        
        let layer = if timeline.get_layers().is_empty() {
            timeline.append_layer()?
        } else {
            timeline.get_layer(0).ok_or(EditingError::TimelineError("No layers available".to_string()))?
        };
        
        let asset = ges::UriClipAsset::request_sync(uri)?;
        
        let clip = asset.extract()?;
        let clip = clip.downcast::<ges::Clip>()
            .map_err(|_| EditingError::TimelineError("Failed to downcast to Clip".to_string()))?;
        
        clip.set_start(start_time);
        clip.set_duration(duration);
        clip.set_inpoint(in_point);
        
        layer.add_clip(&clip)?;
        
        let clip_id = format!("clip_{}", self.clips.len());
        let timeline_clip = TimelineClip {
            id: clip_id.clone(),
            name: asset.get_id().to_string(),
            ges_clip: clip,
            track_type,
            start_time,
            duration,
            in_point,
            effects: Vec::new(),
        };
        
        self.clips.insert(clip_id.clone(), timeline_clip.clone());
        
        let clip_end = start_time + duration;
        if clip_end > self.duration {
            self.duration = clip_end;
        }
        
        Ok(timeline_clip)
    }
    
    pub fn move_clip(&mut self, clip_id: &str, new_start_time: i64) -> Result<(), EditingError> {
        let clip = self.clips.get_mut(clip_id)
            .ok_or(EditingError::InvalidParameter(format!("Clip not found: {}", clip_id)))?;
        
        clip.ges_clip.set_start(new_start_time);
        
        clip.start_time = new_start_time;
        
        let clip_end = new_start_time + clip.duration;
        if clip_end > self.duration {
            self.duration = clip_end;
        }
        
        Ok(())
    }
    
    pub fn trim_clip(&mut self, clip_id: &str, new_duration: i64) -> Result<(), EditingError> {
        let clip = self.clips.get_mut(clip_id)
            .ok_or(EditingError::InvalidParameter(format!("Clip not found: {}", clip_id)))?;
        
        clip.ges_clip.set_duration(new_duration);
        
        clip.duration = new_duration;
        
        self.update_duration();
        
        Ok(())
    }
    
    pub fn split_clip(&mut self, clip_id: &str, position: i64) -> Result<String, EditingError> {
        let clip = self.clips.get(clip_id)
            .ok_or(EditingError::InvalidParameter(format!("Clip not found: {}", clip_id)))?;
        
        if position <= clip.start_time || position >= clip.start_time + clip.duration {
            return Err(EditingError::InvalidParameter(
                format!("Split position {} is outside clip bounds", position)
            ));
        }
        
        let relative_position = position - clip.start_time;
        
        let (_, right_clip) = clip.ges_clip.split(relative_position)?;
        let right_clip = right_clip.downcast::<ges::Clip>()
            .map_err(|_| EditingError::TimelineError("Failed to downcast to Clip".to_string()))?;
        
        let right_clip_id = format!("clip_{}", self.clips.len());
        let right_timeline_clip = TimelineClip {
            id: right_clip_id.clone(),
            name: format!("{}_right", clip.name),
            ges_clip: right_clip.clone(),
            track_type: clip.track_type,
            start_time: position,
            duration: clip.duration - relative_position,
            in_point: clip.in_point + relative_position,
            effects: Vec::new(), // Effects need to be handled separately
        };
        
        let left_clip = self.clips.get_mut(clip_id).unwrap();
        left_clip.duration = relative_position;
        
        self.clips.insert(right_clip_id.clone(), right_timeline_clip);
        
        Ok(right_clip_id)
    }
    
    pub fn add_effect(&mut self, clip_id: &str, effect_type: &str) -> Result<TimelineEffect, EditingError> {
        let clip = self.clips.get_mut(clip_id)
            .ok_or(EditingError::InvalidParameter(format!("Clip not found: {}", clip_id)))?;
        
        let effect = ges::Effect::new(effect_type)?;
        
        clip.ges_clip.add(&effect)?;
        
        let effect_id = format!("effect_{}_{}_{}", clip_id, effect_type, clip.effects.len());
        let timeline_effect = TimelineEffect {
            id: effect_id.clone(),
            name: effect_type.to_string(),
            ges_effect: effect,
            parameters: HashMap::new(),
        };
        
        clip.effects.push(timeline_effect.clone());
        
        Ok(timeline_effect)
    }
    
    pub fn remove_clip(&mut self, clip_id: &str) -> Result<(), EditingError> {
        let clip = self.clips.get(clip_id)
            .ok_or(EditingError::InvalidParameter(format!("Clip not found: {}", clip_id)))?;
        
        let layer = clip.ges_clip.get_layer()
            .ok_or(EditingError::TimelineError("Clip has no layer".to_string()))?;
        
        layer.remove_clip(&clip.ges_clip)?;
        
        self.clips.remove(clip_id);
        
        self.update_duration();
        
        Ok(())
    }
    
    pub fn get_clips(&self) -> Vec<ClipInfo> {
        self.clips.values()
            .map(|clip| clip.to_clip_info())
            .collect()
    }
    
    pub fn get_clip(&self, clip_id: &str) -> Option<&TimelineClip> {
        self.clips.get(clip_id)
    }
    
    pub fn get_duration(&self) -> i64 {
        self.duration
    }
    
    fn update_duration(&mut self) {
        let mut max_duration = 0;
        
        for clip in self.clips.values() {
            let clip_end = clip.start_time + clip.duration;
            if clip_end > max_duration {
                max_duration = clip_end;
            }
        }
        
        self.duration = max_duration;
    }
    
    pub fn get_ges_timeline(&self) -> Option<&ges::Timeline> {
        self.ges_timeline.as_ref()
    }
}

#[derive(Clone)]
pub struct TimelineTrack {
    pub id: String,
    
    pub track_type: TrackType,
    
    pub ges_track: ges::Track,
    
    pub clips: Vec<String>,
}

#[derive(Clone)]
pub struct TimelineClip {
    pub id: String,
    
    pub name: String,
    
    pub ges_clip: ges::Clip,
    
    pub track_type: TrackType,
    
    pub start_time: i64,
    
    pub duration: i64,
    
    pub in_point: i64,
    
    pub effects: Vec<TimelineEffect>,
}

impl TimelineClip {
    pub fn to_clip_info(&self) -> ClipInfo {
        ClipInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            source_path: None, // Would need to extract from URI
            start_time: self.start_time,
            duration: self.duration,
            in_point: self.in_point,
            out_point: self.in_point + self.duration,
            track_type: self.track_type,
            effects: self.effects.iter().map(|e| e.to_effect_info()).collect(),
        }
    }
}

#[derive(Clone)]
pub struct TimelineEffect {
    pub id: String,
    
    pub name: String,
    
    pub ges_effect: ges::Effect,
    
    pub parameters: HashMap<String, String>,
}

impl TimelineEffect {
    pub fn to_effect_info(&self) -> crate::engine::editing::types::EffectInfo {
        crate::engine::editing::types::EffectInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            effect_type: self.name.clone(), // Using name as effect type
            parameters: self.parameters.clone(),
            start_time: 0, // Effects are applied to the entire clip duration by default
            duration: 0,   // Duration is the same as the clip
        }
    }
    
    pub fn set_parameter(&mut self, name: &str, value: &str) -> Result<(), EditingError> {
        self.ges_effect.set_property_from_str(name, value);
        
        self.parameters.insert(name.to_string(), value.to_string());
        
        Ok(())
    }
}
