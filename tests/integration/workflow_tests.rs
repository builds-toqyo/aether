#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::collections::HashMap;


    #[derive(Debug, Clone)]
    pub struct TestProject {
        pub id: String,
        pub name: String,
        pub path: Option<PathBuf>,
        pub media: Vec<TestMediaItem>,
        pub timeline: TestTimeline,
        pub settings: ProjectSettings,
        pub modified: bool,
    }

    #[derive(Debug, Clone)]
    pub struct ProjectSettings {
        pub resolution: (u32, u32),
        pub frame_rate: f64,
        pub sample_rate: u32,
        pub color_space: String,
    }

    impl Default for ProjectSettings {
        fn default() -> Self {
            Self {
                resolution: (1920, 1080),
                frame_rate: 30.0,
                sample_rate: 48000,
                color_space: "sRGB".to_string(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct TestMediaItem {
        pub id: String,
        pub path: PathBuf,
        pub media_type: MediaType,
        pub duration_ms: u64,
        pub metadata: HashMap<String, String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum MediaType {
        Video,
        Audio,
        Image,
    }

    #[derive(Debug, Clone, Default)]
    pub struct TestTimeline {
        pub tracks: Vec<TestTrack>,
        pub duration_ms: u64,
        pub playhead_ms: u64,
    }

    #[derive(Debug, Clone)]
    pub struct TestTrack {
        pub id: String,
        pub name: String,
        pub track_type: TrackType,
        pub clips: Vec<TestClip>,
        pub muted: bool,
        pub locked: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum TrackType {
        Video,
        Audio,
    }

    #[derive(Debug, Clone)]
    pub struct TestClip {
        pub id: String,
        pub media_id: String,
        pub start_ms: u64,
        pub duration_ms: u64,
        pub in_point_ms: u64,
        pub out_point_ms: u64,
        pub effects: Vec<TestEffect>,
    }

    #[derive(Debug, Clone)]
    pub struct TestEffect {
        pub id: String,
        pub effect_type: String,
        pub parameters: HashMap<String, f64>,
    }


    pub struct WorkflowEngine {
        pub project: Option<TestProject>,
        pub undo_stack: Vec<WorkflowAction>,
        pub redo_stack: Vec<WorkflowAction>,
    }

    #[derive(Debug, Clone)]
    pub enum WorkflowAction {
        CreateProject(String),
        ImportMedia(String),
        AddClip(String, String, u64),
        RemoveClip(String),
        MoveClip(String, u64),
        TrimClip(String, u64, u64),
        AddEffect(String, TestEffect),
        RemoveEffect(String, String),
        Save,
    }

    impl WorkflowEngine {
        pub fn new() -> Self {
            Self {
                project: None,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
            }
        }

        pub fn create_project(&mut self, name: &str) -> Result<(), String> {
            if self.project.is_some() {
                return Err("Project already open".to_string());
            }

            self.project = Some(TestProject {
                id: format!("proj_{}", uuid_mock()),
                name: name.to_string(),
                path: None,
                media: Vec::new(),
                timeline: TestTimeline::default(),
                settings: ProjectSettings::default(),
                modified: false,
            });

            self.undo_stack.push(WorkflowAction::CreateProject(name.to_string()));
            self.redo_stack.clear();
            Ok(())
        }

        pub fn import_media(&mut self, path: &str) -> Result<String, String> {
            let project = self.project.as_mut()
                .ok_or("No project open")?;

            let media_id = format!("media_{}", uuid_mock());
            let media_type = if path.ends_with(".mp4") || path.ends_with(".mov") {
                MediaType::Video
            } else if path.ends_with(".mp3") || path.ends_with(".wav") {
                MediaType::Audio
            } else {
                MediaType::Image
            };

            project.media.push(TestMediaItem {
                id: media_id.clone(),
                path: PathBuf::from(path),
                media_type,
                duration_ms: 10000,
                metadata: HashMap::new(),
            });

            project.modified = true;
            self.undo_stack.push(WorkflowAction::ImportMedia(media_id.clone()));
            self.redo_stack.clear();
            Ok(media_id)
        }

        pub fn add_clip_to_timeline(&mut self, media_id: &str, track_id: &str, start_ms: u64) -> Result<String, String> {
            let project = self.project.as_mut()
                .ok_or("No project open")?;

            let media = project.media.iter()
                .find(|m| m.id == media_id)
                .ok_or("Media not found")?
                .clone();

            let track = project.timeline.tracks.iter_mut()
                .find(|t| t.id == track_id)
                .ok_or("Track not found")?;

            let clip_id = format!("clip_{}", uuid_mock());
            track.clips.push(TestClip {
                id: clip_id.clone(),
                media_id: media_id.to_string(),
                start_ms,
                duration_ms: media.duration_ms,
                in_point_ms: 0,
                out_point_ms: media.duration_ms,
                effects: Vec::new(),
            });


            let clip_end = start_ms + media.duration_ms;
            if clip_end > project.timeline.duration_ms {
                project.timeline.duration_ms = clip_end;
            }

            project.modified = true;
            self.undo_stack.push(WorkflowAction::AddClip(clip_id.clone(), track_id.to_string(), start_ms));
            self.redo_stack.clear();
            Ok(clip_id)
        }

        pub fn add_track(&mut self, name: &str, track_type: TrackType) -> Result<String, String> {
            let project = self.project.as_mut()
                .ok_or("No project open")?;

            let track_id = format!("track_{}", uuid_mock());
            project.timeline.tracks.push(TestTrack {
                id: track_id.clone(),
                name: name.to_string(),
                track_type,
                clips: Vec::new(),
                muted: false,
                locked: false,
            });

            project.modified = true;
            Ok(track_id)
        }

        pub fn add_effect(&mut self, clip_id: &str, effect_type: &str, params: HashMap<String, f64>) -> Result<String, String> {
            let project = self.project.as_mut()
                .ok_or("No project open")?;

            let clip = project.timeline.tracks.iter_mut()
                .flat_map(|t| t.clips.iter_mut())
                .find(|c| c.id == clip_id)
                .ok_or("Clip not found")?;

            let effect_id = format!("effect_{}", uuid_mock());
            let effect = TestEffect {
                id: effect_id.clone(),
                effect_type: effect_type.to_string(),
                parameters: params,
            };

            clip.effects.push(effect.clone());
            project.modified = true;
            self.undo_stack.push(WorkflowAction::AddEffect(clip_id.to_string(), effect));
            self.redo_stack.clear();
            Ok(effect_id)
        }

        pub fn trim_clip(&mut self, clip_id: &str, new_in: u64, new_out: u64) -> Result<(), String> {
            let project = self.project.as_mut()
                .ok_or("No project open")?;

            let clip = project.timeline.tracks.iter_mut()
                .flat_map(|t| t.clips.iter_mut())
                .find(|c| c.id == clip_id)
                .ok_or("Clip not found")?;

            if new_out <= new_in {
                return Err("Invalid trim points".to_string());
            }

            clip.in_point_ms = new_in;
            clip.out_point_ms = new_out;
            clip.duration_ms = new_out - new_in;

            project.modified = true;
            self.undo_stack.push(WorkflowAction::TrimClip(clip_id.to_string(), new_in, new_out));
            self.redo_stack.clear();
            Ok(())
        }

        pub fn save_project(&mut self, path: &str) -> Result<(), String> {
            let project = self.project.as_mut()
                .ok_or("No project open")?;

            project.path = Some(PathBuf::from(path));
            project.modified = false;
            self.undo_stack.push(WorkflowAction::Save);
            Ok(())
        }

        pub fn can_undo(&self) -> bool {
            !self.undo_stack.is_empty()
        }

        pub fn can_redo(&self) -> bool {
            !self.redo_stack.is_empty()
        }

        pub fn undo_count(&self) -> usize {
            self.undo_stack.len()
        }
    }

    fn uuid_mock() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        format!("{:08x}", COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    #[test]
    fn test_create_new_project() {
        let mut engine = WorkflowEngine::new();

        assert!(engine.create_project("Test Project").is_ok());
        assert!(engine.project.is_some());
        assert_eq!(engine.project.as_ref().unwrap().name, "Test Project");
    }

    #[test]
    fn test_import_media() {
        let mut engine = WorkflowEngine::new();
        engine.create_project("Test").unwrap();

        let media_id = engine.import_media("/path/to/video.mp4").unwrap();
        assert!(!media_id.is_empty());

        let project = engine.project.as_ref().unwrap();
        assert_eq!(project.media.len(), 1);
        assert_eq!(project.media[0].media_type, MediaType::Video);
    }

    #[test]
    fn test_add_track_and_clip() {
        let mut engine = WorkflowEngine::new();
        engine.create_project("Test").unwrap();

        let track_id = engine.add_track("Video 1", TrackType::Video).unwrap();
        let media_id = engine.import_media("/path/to/video.mp4").unwrap();
        let clip_id = engine.add_clip_to_timeline(&media_id, &track_id, 0).unwrap();

        assert!(!clip_id.is_empty());

        let project = engine.project.as_ref().unwrap();
        assert_eq!(project.timeline.tracks.len(), 1);
        assert_eq!(project.timeline.tracks[0].clips.len(), 1);
    }

    #[test]
    fn test_add_effect_to_clip() {
        let mut engine = WorkflowEngine::new();
        engine.create_project("Test").unwrap();

        let track_id = engine.add_track("Video 1", TrackType::Video).unwrap();
        let media_id = engine.import_media("/path/to/video.mp4").unwrap();
        let clip_id = engine.add_clip_to_timeline(&media_id, &track_id, 0).unwrap();

        let mut params = HashMap::new();
        params.insert("brightness".to_string(), 0.5);
        params.insert("contrast".to_string(), 1.2);

        let effect_id = engine.add_effect(&clip_id, "color_correction", params).unwrap();
        assert!(!effect_id.is_empty());

        let project = engine.project.as_ref().unwrap();
        let clip = &project.timeline.tracks[0].clips[0];
        assert_eq!(clip.effects.len(), 1);
        assert_eq!(clip.effects[0].effect_type, "color_correction");
    }

    #[test]
    fn test_trim_clip() {
        let mut engine = WorkflowEngine::new();
        engine.create_project("Test").unwrap();

        let track_id = engine.add_track("Video 1", TrackType::Video).unwrap();
        let media_id = engine.import_media("/path/to/video.mp4").unwrap();
        let clip_id = engine.add_clip_to_timeline(&media_id, &track_id, 0).unwrap();

        engine.trim_clip(&clip_id, 1000, 5000).unwrap();

        let project = engine.project.as_ref().unwrap();
        let clip = &project.timeline.tracks[0].clips[0];
        assert_eq!(clip.in_point_ms, 1000);
        assert_eq!(clip.out_point_ms, 5000);
        assert_eq!(clip.duration_ms, 4000);
    }

    #[test]
    fn test_save_project() {
        let mut engine = WorkflowEngine::new();
        engine.create_project("Test").unwrap();
        engine.import_media("/path/to/video.mp4").unwrap();

        assert!(engine.project.as_ref().unwrap().modified);

        engine.save_project("/path/to/project.aether").unwrap();

        assert!(!engine.project.as_ref().unwrap().modified);
        assert!(engine.project.as_ref().unwrap().path.is_some());
    }

    #[test]
    fn test_undo_tracking() {
        let mut engine = WorkflowEngine::new();
        engine.create_project("Test").unwrap();

        assert!(engine.can_undo());
        assert_eq!(engine.undo_count(), 1);

        engine.import_media("/path/to/video.mp4").unwrap();
        assert_eq!(engine.undo_count(), 2);
    }

    #[test]
    fn test_complete_workflow() {
        let mut engine = WorkflowEngine::new();


        engine.create_project("My Video Project").unwrap();


        let video_track = engine.add_track("Video 1", TrackType::Video).unwrap();
        let audio_track = engine.add_track("Audio 1", TrackType::Audio).unwrap();


        let video_id = engine.import_media("/footage/clip1.mp4").unwrap();
        let audio_id = engine.import_media("/audio/music.mp3").unwrap();


        let video_clip = engine.add_clip_to_timeline(&video_id, &video_track, 0).unwrap();
        let _audio_clip = engine.add_clip_to_timeline(&audio_id, &audio_track, 0).unwrap();


        let mut color_params = HashMap::new();
        color_params.insert("saturation".to_string(), 1.2);
        engine.add_effect(&video_clip, "color_correction", color_params).unwrap();


        engine.trim_clip(&video_clip, 500, 8000).unwrap();


        engine.save_project("/projects/my_video.aether").unwrap();


        let project = engine.project.as_ref().unwrap();
        assert_eq!(project.name, "My Video Project");
        assert_eq!(project.media.len(), 2);
        assert_eq!(project.timeline.tracks.len(), 2);
        assert!(!project.modified);
    }

    #[test]
    fn test_multiple_clips_on_track() {
        let mut engine = WorkflowEngine::new();
        engine.create_project("Test").unwrap();

        let track_id = engine.add_track("Video 1", TrackType::Video).unwrap();


        let media1 = engine.import_media("/clip1.mp4").unwrap();
        let media2 = engine.import_media("/clip2.mp4").unwrap();
        let media3 = engine.import_media("/clip3.mp4").unwrap();


        engine.add_clip_to_timeline(&media1, &track_id, 0).unwrap();
        engine.add_clip_to_timeline(&media2, &track_id, 10000).unwrap();
        engine.add_clip_to_timeline(&media3, &track_id, 20000).unwrap();

        let project = engine.project.as_ref().unwrap();
        assert_eq!(project.timeline.tracks[0].clips.len(), 3);
        assert_eq!(project.timeline.duration_ms, 30000);
    }

    #[test]
    fn test_project_settings() {
        let mut engine = WorkflowEngine::new();
        engine.create_project("4K Project").unwrap();

        let project = engine.project.as_mut().unwrap();
        project.settings.resolution = (3840, 2160);
        project.settings.frame_rate = 60.0;

        assert_eq!(project.settings.resolution, (3840, 2160));
        assert_eq!(project.settings.frame_rate, 60.0);
    }
}
