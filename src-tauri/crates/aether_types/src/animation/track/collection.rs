

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;

use super::track::AnimationTrack;
use super::types::TrackType;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTrackCollection {

    tracks: HashMap<String, AnimationTrack>,

    name: String,
}

impl AnimationTrackCollection {

    pub fn new(name: String) -> Self {
        Self {
            tracks: HashMap::new(),
            name,
        }
    }


    pub fn add_track(&mut self, track: AnimationTrack) -> Result<(), String> {
        track.validate()?;
        self.tracks.insert(track.id.clone(), track);
        Ok(())
    }


    pub fn remove_track(&mut self, track_id: &str) -> Option<AnimationTrack> {
        self.tracks.remove(track_id)
    }


    pub fn get_track(&self, track_id: &str) -> Option<&AnimationTrack> {
        self.tracks.get(track_id)
    }


    pub fn get_track_mut(&mut self, track_id: &str) -> Option<&mut AnimationTrack> {
        self.tracks.get_mut(track_id)
    }


    pub fn get_tracks(&self) -> Vec<&AnimationTrack> {
        self.tracks.values().collect()
    }


    pub fn get_tracks_mut(&mut self) -> Vec<&mut AnimationTrack> {
        self.tracks.values_mut().collect()
    }


    pub fn get_tracks_by_type(&self, track_type: TrackType) -> Vec<&AnimationTrack> {
        self.tracks.values()
            .filter(|track| track.track_type == track_type)
            .collect()
    }


    pub fn get_tracks_by_type_mut(&mut self, track_type: TrackType) -> Vec<&mut AnimationTrack> {
        self.tracks.values_mut()
            .filter(|track| track.track_type == track_type)
            .collect()
    }


    pub fn get_tracks_by_target(&self, target_id: &str) -> Vec<&AnimationTrack> {
        self.tracks.values()
            .filter(|track| track.binding.target_id == target_id)
            .collect()
    }


    pub fn get_tracks_by_target_mut(&mut self, target_id: &str) -> Vec<&mut AnimationTrack> {
        self.tracks.values_mut()
            .filter(|track| track.binding.target_id == target_id)
            .collect()
    }


    pub fn get_enabled_tracks(&self) -> Vec<&AnimationTrack> {
        self.tracks.values()
            .filter(|track| track.enabled && !track.muted)
            .collect()
    }


    pub fn get_enabled_tracks_by_type(&self, track_type: TrackType) -> Vec<&AnimationTrack> {
        self.tracks.values()
            .filter(|track| track.track_type == track_type && track.enabled && !track.muted)
            .collect()
    }


    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }


    pub fn track_count_by_type(&self, track_type: TrackType) -> usize {
        self.tracks.values()
            .filter(|track| track.track_type == track_type)
            .count()
    }


    pub fn clear(&mut self) {
        self.tracks.clear();
    }


    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Collection name cannot be empty".to_string());
        }

        for (id, track) in &self.tracks {
            if id != &track.id {
                return Err(format!("Track ID mismatch: key '{}' vs track.id '{}'", id, track.id));
            }

            track.validate()?;
        }

        Ok(())
    }


    pub fn description(&self) -> String {
        format!("Track collection '{}' with {} tracks", self.name, self.track_count())
    }


    pub fn get_target_ids(&self) -> Vec<String> {
        let mut targets: Vec<String> = self.tracks.values()
            .map(|track| track.binding.target_id.clone())
            .collect();
        targets.sort();
        targets.dedup();
        targets
    }


    pub fn get_values_at_time(&self, time: f64) -> HashMap<String, super::value::TrackValue> {
        let mut values = HashMap::new();

        for track in self.get_enabled_tracks() {
            if let Some(value) = track.get_value_at_time(time) {
                values.insert(track.id.clone(), value);
            }
        }

        values
    }


    pub fn get_values_at_time_for_target(&self, time: f64, target_id: &str) -> HashMap<String, super::value::TrackValue> {
        let mut values = HashMap::new();

        for track in self.get_tracks_by_target(target_id) {
            if track.enabled && !track.muted {
                if let Some(value) = track.get_value_at_time(time) {
                    values.insert(track.id.clone(), value);
                }
            }
        }

        values
    }


    pub fn time_range(&self) -> Option<(f64, f64)> {
        if self.tracks.is_empty() {
            return None;
        }

        let mut min_time = f64::INFINITY;
        let mut max_time = f64::NEG_INFINITY;

        for track in self.tracks.values() {
            if let Some((track_min, track_max)) = track.time_range() {
                min_time = min_time.min(track_min);
                max_time = max_time.max(track_max);
            }
        }

        if min_time == f64::INFINITY || max_time == f64::NEG_INFINITY {
            None
        } else {
            Some((min_time, max_time))
        }
    }


    pub fn clone_with_name(&self, new_name: String) -> Self {
        let mut cloned = self.clone();
        cloned.name = new_name;
        cloned
    }


    pub fn merge(&mut self, other: AnimationTrackCollection) -> Result<(), String> {
        for (id, track) in other.tracks {
            if self.tracks.contains_key(&id) {
                return Err(format!("Track ID '{}' already exists in collection", id));
            }
            self.tracks.insert(id, track);
        }
        Ok(())
    }


    pub fn filter_tracks<F>(&self, predicate: F) -> AnimationTrackCollection
    where
        F: Fn(&AnimationTrack) -> bool,
    {
        let mut filtered = AnimationTrackCollection::new(format!("{} (filtered)", self.name));

        for track in self.tracks.values() {
            if predicate(track) {
                filtered.tracks.insert(track.id.clone(), track.clone());
            }
        }

        filtered
    }


    pub fn get_statistics(&self) -> TrackCollectionStats {
        let mut stats = TrackCollectionStats::default();

        for track in self.tracks.values() {
            stats.total_tracks += 1;

            if track.enabled {
                stats.enabled_tracks += 1;
            }

            if track.muted {
                stats.muted_tracks += 1;
            }

            stats.total_keyframes += track.keyframe_count();

            match track.track_type {
                TrackType::Position => stats.position_tracks += 1,
                TrackType::Rotation => stats.rotation_tracks += 1,
                TrackType::Scale => stats.scale_tracks += 1,
                TrackType::Opacity => stats.opacity_tracks += 1,
                TrackType::Color => stats.color_tracks += 1,
                TrackType::Custom => stats.custom_tracks += 1,
            }
        }

        stats
    }
}

impl fmt::Display for AnimationTrackCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnimationTrackCollection({})", self.description())
    }
}

impl Default for AnimationTrackCollection {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

/// Statistics for track collection
#[derive(Debug, Clone, Default)]
pub struct TrackCollectionStats {
    /// Total number of tracks
    pub total_tracks: usize,
    /// Number of enabled tracks
    pub enabled_tracks: usize,
    /// Number of muted tracks
    pub muted_tracks: usize,
    /// Total number of keyframes
    pub total_keyframes: usize,
    /// Number of position tracks
    pub position_tracks: usize,
    /// Number of rotation tracks
    pub rotation_tracks: usize,
    /// Number of scale tracks
    pub scale_tracks: usize,
    /// Number of opacity tracks
    pub opacity_tracks: usize,
    /// Number of color tracks
    pub color_tracks: usize,
    /// Number of custom tracks
    pub custom_tracks: usize,
}

impl TrackCollectionStats {
    /// Get percentage of enabled tracks
    pub fn enabled_percentage(&self) -> f64 {
        if self.total_tracks == 0 {
            0.0
        } else {
            (self.enabled_tracks as f64 / self.total_tracks as f64) * 100.0
        }
    }

    /// Get percentage of muted tracks
    pub fn muted_percentage(&self) -> f64 {
        if self.total_tracks == 0 {
            0.0
        } else {
            (self.muted_tracks as f64 / self.total_tracks as f64) * 100.0
        }
    }

    /// Get average keyframes per track
    pub fn average_keyframes_per_track(&self) -> f64 {
        if self.total_tracks == 0 {
            0.0
        } else {
            self.total_keyframes as f64 / self.total_tracks as f64
        }
    }
}

impl fmt::Display for TrackCollectionStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Track Collection Statistics:")?;
        writeln!(f, "  Total tracks: {}", self.total_tracks)?;
        writeln!(f, "  Enabled tracks: {} ({:.1}%)", self.enabled_tracks, self.enabled_percentage())?;
        writeln!(f, "  Muted tracks: {} ({:.1}%)", self.muted_tracks, self.muted_percentage())?;
        writeln!(f, "  Total keyframes: {}", self.total_keyframes)?;
        writeln!(f, "  Average keyframes per track: {:.1}", self.average_keyframes_per_track())?;
        writeln!(f, "  Track types:")?;
        writeln!(f, "    Position: {}", self.position_tracks)?;
        writeln!(f, "    Rotation: {}", self.rotation_tracks)?;
        writeln!(f, "    Scale: {}", self.scale_tracks)?;
        writeln!(f, "    Opacity: {}", self.opacity_tracks)?;
        writeln!(f, "    Color: {}", self.color_tracks)?;
        writeln!(f, "    Custom: {}", self.custom_tracks)?;
        Ok(())
    }
}


pub struct AnimationTrackCollectionBuilder {
    collection: AnimationTrackCollection,
}

impl AnimationTrackCollectionBuilder {

    pub fn new(name: String) -> Self {
        let collection = AnimationTrackCollection::new(name);
        Self { collection }
    }


    pub fn track(mut self, track: AnimationTrack) -> Self {
        let _ = self.collection.add_track(track);
        self
    }


    pub fn tracks(mut self, tracks: Vec<AnimationTrack>) -> Self {
        for track in tracks {
            let _ = self.collection.add_track(track);
        }
        self
    }


    pub fn build(self) -> Result<AnimationTrackCollection, String> {
        self.collection.validate()?;
        Ok(self.collection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::keyframe::{Keyframe, KeyframeData};
    use crate::animation::interpolation::{InterpolationMethod, EasingFunction};
    use super::types::{TrackType, ParameterBinding};
    use super::value::TrackValue;

    #[test]
    fn test_track_collection() {
        let mut collection = AnimationTrackCollection::new("Test Collection".to_string());

        let binding1 = ParameterBinding::new("object1".to_string(), "position".to_string());
        let track1 = AnimationTrack::new(
            "track1".to_string(),
            "Position Track".to_string(),
            TrackType::Position,
            binding1,
        );

        let binding2 = ParameterBinding::new("object1".to_string(), "opacity".to_string());
        let track2 = AnimationTrack::new(
            "track2".to_string(),
            "Opacity Track".to_string(),
            TrackType::Opacity,
            binding2,
        );

        collection.add_track(track1).unwrap();
        collection.add_track(track2).unwrap();

        assert_eq!(collection.track_count(), 2);
        assert!(collection.get_track("track1").is_some());
        assert!(collection.get_track("track2").is_some());

        let position_tracks = collection.get_tracks_by_type(TrackType::Position);
        assert_eq!(position_tracks.len(), 1);

        assert!(collection.validate().is_ok());
    }

    #[test]
    fn test_track_collection_filtering() {
        let mut collection = AnimationTrackCollection::new("Test Collection".to_string());

        let binding = ParameterBinding::new("object1".to_string(), "position".to_string());
        let mut track1 = AnimationTrack::new(
            "track1".to_string(),
            "Position Track".to_string(),
            TrackType::Position,
            binding,
        );


        track1.add_keyframe(KeyframeData::Vector3(Keyframe::new(0.0, [1.0, 2.0, 3.0])));

        collection.add_track(track1).unwrap();


        let enabled_tracks = collection.get_enabled_tracks();
        assert_eq!(enabled_tracks.len(), 1);


        let position_tracks = collection.get_tracks_by_type(TrackType::Position);
        assert_eq!(position_tracks.len(), 1);

        let opacity_tracks = collection.get_tracks_by_type(TrackType::Opacity);
        assert_eq!(opacity_tracks.len(), 0);
    }

    #[test]
    fn test_track_collection_statistics() {
        let mut collection = AnimationTrackCollection::new("Test Collection".to_string());

        let binding1 = ParameterBinding::new("object1".to_string(), "position".to_string());
        let mut track1 = AnimationTrack::new(
            "track1".to_string(),
            "Position Track".to_string(),
            TrackType::Position,
            binding1,
        );
        track1.add_keyframe(KeyframeData::Vector3(Keyframe::new(0.0, [1.0, 2.0, 3.0])));

        let binding2 = ParameterBinding::new("object1".to_string(), "opacity".to_string());
        let mut track2 = AnimationTrack::new(
            "track2".to_string(),
            "Opacity Track".to_string(),
            TrackType::Opacity,
            binding2,
        );
        track2.add_keyframe(KeyframeData::Float(Keyframe::new(0.0, 1.0)));

        collection.add_track(track1).unwrap();
        collection.add_track(track2).unwrap();

        let stats = collection.get_statistics();
        assert_eq!(stats.total_tracks, 2);
        assert_eq!(stats.enabled_tracks, 2);
        assert_eq!(stats.position_tracks, 1);
        assert_eq!(stats.opacity_tracks, 1);
        assert_eq!(stats.total_keyframes, 2);
        assert_eq!(stats.average_keyframes_per_track(), 1.0);
    }

    #[test]
    fn test_track_collection_builder() {
        let binding = ParameterBinding::new("object1".to_string(), "position".to_string());
        let mut track = AnimationTrack::new(
            "track1".to_string(),
            "Position Track".to_string(),
            TrackType::Position,
            binding,
        );
        track.add_keyframe(KeyframeData::Vector3(Keyframe::new(0.0, [1.0, 2.0, 3.0])));

        let collection = AnimationTrackCollectionBuilder::new("Test Collection".to_string())
            .track(track)
            .build();

        assert!(collection.is_ok());

        let collection = collection.unwrap();
        assert_eq!(collection.track_count(), 1);
        assert_eq!(collection.name, "Test Collection");
    }
}
