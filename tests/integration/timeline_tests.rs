#[cfg(test)]
mod tests {
    use std::collections::HashMap;


    #[derive(Debug, Clone)]
    pub struct Timeline {
        pub tracks: Vec<Track>,
        pub duration_ms: u64,
        pub playhead_ms: u64,
        pub in_point_ms: Option<u64>,
        pub out_point_ms: Option<u64>,
        pub markers: Vec<Marker>,
    }

    impl Timeline {
        pub fn new() -> Self {
            Self {
                tracks: Vec::new(),
                duration_ms: 0,
                playhead_ms: 0,
                in_point_ms: None,
                out_point_ms: None,
                markers: Vec::new(),
            }
        }

        pub fn add_track(&mut self, name: &str, track_type: TrackType) -> usize {
            let track = Track {
                id: self.tracks.len(),
                name: name.to_string(),
                track_type,
                clips: Vec::new(),
                muted: false,
                solo: false,
                locked: false,
                height: 60,
            };
            self.tracks.push(track);
            self.tracks.len() - 1
        }

        pub fn remove_track(&mut self, index: usize) -> Option<Track> {
            if index < self.tracks.len() {
                Some(self.tracks.remove(index))
            } else {
                None
            }
        }

        pub fn add_clip(&mut self, track_index: usize, clip: Clip) -> Result<usize, String> {
            let track = self.tracks.get_mut(track_index)
                .ok_or("Track not found")?;

            if track.locked {
                return Err("Track is locked".to_string());
            }


            let clip_end = clip.start_ms + clip.duration_ms;
            for existing in &track.clips {
                let existing_end = existing.start_ms + existing.duration_ms;
                if clip.start_ms < existing_end && clip_end > existing.start_ms {
                    return Err("Clip overlaps with existing clip".to_string());
                }
            }

            let clip_index = track.clips.len();
            track.clips.push(clip);


            if clip_end > self.duration_ms {
                self.duration_ms = clip_end;
            }

            Ok(clip_index)
        }

        pub fn remove_clip(&mut self, track_index: usize, clip_index: usize) -> Result<Clip, String> {
            let track = self.tracks.get_mut(track_index)
                .ok_or("Track not found")?;

            if track.locked {
                return Err("Track is locked".to_string());
            }

            if clip_index >= track.clips.len() {
                return Err("Clip not found".to_string());
            }

            Ok(track.clips.remove(clip_index))
        }

        pub fn move_clip(&mut self, track_index: usize, clip_index: usize, new_start_ms: u64) -> Result<(), String> {
            let track = self.tracks.get_mut(track_index)
                .ok_or("Track not found")?;

            if track.locked {
                return Err("Track is locked".to_string());
            }

            let clip = track.clips.get_mut(clip_index)
                .ok_or("Clip not found")?;

            let duration = clip.duration_ms;
            let new_end = new_start_ms + duration;


            for (i, other) in track.clips.iter().enumerate() {
                if i != clip_index {
                    let other_end = other.start_ms + other.duration_ms;
                    if new_start_ms < other_end && new_end > other.start_ms {
                        return Err("Move would cause overlap".to_string());
                    }
                }
            }

            clip.start_ms = new_start_ms;


            self.recalculate_duration();

            Ok(())
        }

        pub fn split_clip(&mut self, track_index: usize, clip_index: usize, split_point_ms: u64) -> Result<usize, String> {
            let track = self.tracks.get_mut(track_index)
                .ok_or("Track not found")?;

            if track.locked {
                return Err("Track is locked".to_string());
            }

            let clip = track.clips.get(clip_index)
                .ok_or("Clip not found")?
                .clone();

            let clip_end = clip.start_ms + clip.duration_ms;
            if split_point_ms <= clip.start_ms || split_point_ms >= clip_end {
                return Err("Split point must be within clip".to_string());
            }


            let original_duration = split_point_ms - clip.start_ms;
            track.clips[clip_index].duration_ms = original_duration;
            track.clips[clip_index].out_point_ms = clip.in_point_ms + original_duration;


            let new_clip = Clip {
                id: format!("{}_split", clip.id),
                media_id: clip.media_id.clone(),
                start_ms: split_point_ms,
                duration_ms: clip_end - split_point_ms,
                in_point_ms: clip.in_point_ms + original_duration,
                out_point_ms: clip.out_point_ms,
                speed: clip.speed,
                effects: clip.effects.clone(),
            };

            let new_index = track.clips.len();
            track.clips.push(new_clip);

            Ok(new_index)
        }

        pub fn trim_clip(&mut self, track_index: usize, clip_index: usize, trim_start: i64, trim_end: i64) -> Result<(), String> {
            let track = self.tracks.get_mut(track_index)
                .ok_or("Track not found")?;

            if track.locked {
                return Err("Track is locked".to_string());
            }

            let clip = track.clips.get_mut(clip_index)
                .ok_or("Clip not found")?;


            if trim_start != 0 {
                let new_start = (clip.start_ms as i64 + trim_start).max(0) as u64;
                let new_in = (clip.in_point_ms as i64 + trim_start).max(0) as u64;
                clip.start_ms = new_start;
                clip.in_point_ms = new_in;
                clip.duration_ms = (clip.duration_ms as i64 - trim_start).max(0) as u64;
            }


            if trim_end != 0 {
                clip.duration_ms = (clip.duration_ms as i64 + trim_end).max(0) as u64;
                clip.out_point_ms = clip.in_point_ms + clip.duration_ms;
            }

            self.recalculate_duration();
            Ok(())
        }

        pub fn add_marker(&mut self, time_ms: u64, name: &str, color: &str) {
            self.markers.push(Marker {
                time_ms,
                name: name.to_string(),
                color: color.to_string(),
                notes: None,
            });
            self.markers.sort_by_key(|m| m.time_ms);
        }

        pub fn set_work_area(&mut self, in_point: u64, out_point: u64) -> Result<(), String> {
            if out_point <= in_point {
                return Err("Out point must be after in point".to_string());
            }
            self.in_point_ms = Some(in_point);
            self.out_point_ms = Some(out_point);
            Ok(())
        }

        pub fn clear_work_area(&mut self) {
            self.in_point_ms = None;
            self.out_point_ms = None;
        }

        fn recalculate_duration(&mut self) {
            self.duration_ms = self.tracks.iter()
                .flat_map(|t| t.clips.iter())
                .map(|c| c.start_ms + c.duration_ms)
                .max()
                .unwrap_or(0);
        }

        pub fn get_clips_at_time(&self, time_ms: u64) -> Vec<(usize, usize, &Clip)> {
            let mut result = Vec::new();
            for (track_idx, track) in self.tracks.iter().enumerate() {
                for (clip_idx, clip) in track.clips.iter().enumerate() {
                    if time_ms >= clip.start_ms && time_ms < clip.start_ms + clip.duration_ms {
                        result.push((track_idx, clip_idx, clip));
                    }
                }
            }
            result
        }

        pub fn ripple_delete(&mut self, track_index: usize, clip_index: usize) -> Result<(), String> {
            let track = self.tracks.get_mut(track_index)
                .ok_or("Track not found")?;

            if track.locked {
                return Err("Track is locked".to_string());
            }

            let clip = track.clips.get(clip_index)
                .ok_or("Clip not found")?;

            let gap = clip.duration_ms;
            let delete_start = clip.start_ms;

            track.clips.remove(clip_index);


            for clip in &mut track.clips {
                if clip.start_ms > delete_start {
                    clip.start_ms -= gap;
                }
            }

            self.recalculate_duration();
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    pub struct Track {
        pub id: usize,
        pub name: String,
        pub track_type: TrackType,
        pub clips: Vec<Clip>,
        pub muted: bool,
        pub solo: bool,
        pub locked: bool,
        pub height: u32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum TrackType {
        Video,
        Audio,
    }

    #[derive(Debug, Clone)]
    pub struct Clip {
        pub id: String,
        pub media_id: String,
        pub start_ms: u64,
        pub duration_ms: u64,
        pub in_point_ms: u64,
        pub out_point_ms: u64,
        pub speed: f64,
        pub effects: Vec<Effect>,
    }

    impl Clip {
        pub fn new(id: &str, media_id: &str, start_ms: u64, duration_ms: u64) -> Self {
            Self {
                id: id.to_string(),
                media_id: media_id.to_string(),
                start_ms,
                duration_ms,
                in_point_ms: 0,
                out_point_ms: duration_ms,
                speed: 1.0,
                effects: Vec::new(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Effect {
        pub id: String,
        pub effect_type: String,
        pub parameters: HashMap<String, f64>,
    }

    #[derive(Debug, Clone)]
    pub struct Marker {
        pub time_ms: u64,
        pub name: String,
        pub color: String,
        pub notes: Option<String>,
    }

    #[test]
    fn test_create_timeline() {
        let timeline = Timeline::new();
        assert_eq!(timeline.tracks.len(), 0);
        assert_eq!(timeline.duration_ms, 0);
    }

    #[test]
    fn test_add_track() {
        let mut timeline = Timeline::new();
        let index = timeline.add_track("Video 1", TrackType::Video);

        assert_eq!(index, 0);
        assert_eq!(timeline.tracks.len(), 1);
        assert_eq!(timeline.tracks[0].name, "Video 1");
    }

    #[test]
    fn test_add_multiple_tracks() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);
        timeline.add_track("Video 2", TrackType::Video);
        timeline.add_track("Audio 1", TrackType::Audio);

        assert_eq!(timeline.tracks.len(), 3);
    }

    #[test]
    fn test_remove_track() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);
        timeline.add_track("Video 2", TrackType::Video);

        let removed = timeline.remove_track(0);
        assert!(removed.is_some());
        assert_eq!(timeline.tracks.len(), 1);
        assert_eq!(timeline.tracks[0].name, "Video 2");
    }

    #[test]
    fn test_add_clip() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);

        let clip = Clip::new("clip1", "media1", 0, 5000);
        let result = timeline.add_clip(0, clip);

        assert!(result.is_ok());
        assert_eq!(timeline.tracks[0].clips.len(), 1);
        assert_eq!(timeline.duration_ms, 5000);
    }

    #[test]
    fn test_clip_overlap_prevention() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);

        let clip1 = Clip::new("clip1", "media1", 0, 5000);
        let clip2 = Clip::new("clip2", "media2", 2000, 5000);

        timeline.add_clip(0, clip1).unwrap();
        let result = timeline.add_clip(0, clip2);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("overlaps"));
    }

    #[test]
    fn test_add_non_overlapping_clips() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);

        let clip1 = Clip::new("clip1", "media1", 0, 5000);
        let clip2 = Clip::new("clip2", "media2", 5000, 5000);

        timeline.add_clip(0, clip1).unwrap();
        timeline.add_clip(0, clip2).unwrap();

        assert_eq!(timeline.tracks[0].clips.len(), 2);
        assert_eq!(timeline.duration_ms, 10000);
    }

    #[test]
    fn test_move_clip() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);

        let clip = Clip::new("clip1", "media1", 0, 5000);
        timeline.add_clip(0, clip).unwrap();

        timeline.move_clip(0, 0, 10000).unwrap();

        assert_eq!(timeline.tracks[0].clips[0].start_ms, 10000);
        assert_eq!(timeline.duration_ms, 15000);
    }

    #[test]
    fn test_split_clip() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);

        let clip = Clip::new("clip1", "media1", 0, 10000);
        timeline.add_clip(0, clip).unwrap();

        let new_index = timeline.split_clip(0, 0, 5000).unwrap();

        assert_eq!(timeline.tracks[0].clips.len(), 2);
        assert_eq!(timeline.tracks[0].clips[0].duration_ms, 5000);
        assert_eq!(timeline.tracks[0].clips[new_index].start_ms, 5000);
        assert_eq!(timeline.tracks[0].clips[new_index].duration_ms, 5000);
    }

    #[test]
    fn test_split_clip_invalid_point() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);

        let clip = Clip::new("clip1", "media1", 0, 10000);
        timeline.add_clip(0, clip).unwrap();


        assert!(timeline.split_clip(0, 0, 0).is_err());

        assert!(timeline.split_clip(0, 0, 10000).is_err());

        assert!(timeline.split_clip(0, 0, 15000).is_err());
    }

    #[test]
    fn test_trim_clip() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);

        let clip = Clip::new("clip1", "media1", 0, 10000);
        timeline.add_clip(0, clip).unwrap();


        timeline.trim_clip(0, 0, 1000, 0).unwrap();

        assert_eq!(timeline.tracks[0].clips[0].start_ms, 1000);
        assert_eq!(timeline.tracks[0].clips[0].duration_ms, 9000);
    }

    #[test]
    fn test_locked_track() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);
        timeline.tracks[0].locked = true;

        let clip = Clip::new("clip1", "media1", 0, 5000);
        let result = timeline.add_clip(0, clip);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("locked"));
    }

    #[test]
    fn test_add_marker() {
        let mut timeline = Timeline::new();
        timeline.add_marker(5000, "Scene 1", "red");
        timeline.add_marker(10000, "Scene 2", "blue");
        timeline.add_marker(2000, "Intro", "green");

        assert_eq!(timeline.markers.len(), 3);

        assert_eq!(timeline.markers[0].time_ms, 2000);
        assert_eq!(timeline.markers[1].time_ms, 5000);
        assert_eq!(timeline.markers[2].time_ms, 10000);
    }

    #[test]
    fn test_work_area() {
        let mut timeline = Timeline::new();

        timeline.set_work_area(5000, 15000).unwrap();

        assert_eq!(timeline.in_point_ms, Some(5000));
        assert_eq!(timeline.out_point_ms, Some(15000));
    }

    #[test]
    fn test_invalid_work_area() {
        let mut timeline = Timeline::new();

        let result = timeline.set_work_area(15000, 5000);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_work_area() {
        let mut timeline = Timeline::new();
        timeline.set_work_area(5000, 15000).unwrap();
        timeline.clear_work_area();

        assert!(timeline.in_point_ms.is_none());
        assert!(timeline.out_point_ms.is_none());
    }

    #[test]
    fn test_get_clips_at_time() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);
        timeline.add_track("Audio 1", TrackType::Audio);

        timeline.add_clip(0, Clip::new("v1", "m1", 0, 10000)).unwrap();
        timeline.add_clip(1, Clip::new("a1", "m2", 0, 10000)).unwrap();

        let clips = timeline.get_clips_at_time(5000);
        assert_eq!(clips.len(), 2);

        let clips = timeline.get_clips_at_time(15000);
        assert_eq!(clips.len(), 0);
    }

    #[test]
    fn test_ripple_delete() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);

        timeline.add_clip(0, Clip::new("c1", "m1", 0, 5000)).unwrap();
        timeline.add_clip(0, Clip::new("c2", "m2", 5000, 5000)).unwrap();
        timeline.add_clip(0, Clip::new("c3", "m3", 10000, 5000)).unwrap();


        timeline.ripple_delete(0, 1).unwrap();

        assert_eq!(timeline.tracks[0].clips.len(), 2);

        assert_eq!(timeline.tracks[0].clips[1].start_ms, 5000);
    }

    #[test]
    fn test_timeline_duration_update() {
        let mut timeline = Timeline::new();
        timeline.add_track("Video 1", TrackType::Video);

        timeline.add_clip(0, Clip::new("c1", "m1", 0, 5000)).unwrap();
        assert_eq!(timeline.duration_ms, 5000);

        timeline.add_clip(0, Clip::new("c2", "m2", 10000, 5000)).unwrap();
        assert_eq!(timeline.duration_ms, 15000);

        timeline.remove_clip(0, 1).unwrap();
        assert_eq!(timeline.duration_ms, 5000);
    }
}
