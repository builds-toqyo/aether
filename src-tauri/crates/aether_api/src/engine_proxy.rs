use std::sync::mpsc::{channel, Sender};
use log::{error, info};

use aether_core::engine::editing::{
    EditingEngine, create_editing_engine,
    ImportOptions, MediaInfo,
    types::{EditingError, TrackType, ClipInfo},
};

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub id: String,
    pub track_type: TrackType,
    pub clips: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PreviewState {
    pub is_playing: bool,
    pub position: i64,
    pub dimensions: Option<(u32, u32)>,
    pub duration: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct TimelineInfo {
    pub tracks: Vec<TrackInfo>,
    pub clips: Vec<ClipInfo>,
    pub duration: i64,
}

pub enum EngineCommand {
    InitProject { path: Option<String>, resp: Sender<Result<(), EditingError>> },
    Shutdown { resp: Sender<Result<(), EditingError>> },
    GetTimelineInfo { resp: Sender<Result<TimelineInfo, EditingError>> },
    TimelineSeek { time: f64, resp: Sender<Result<(), EditingError>> },
    TimelineTrimClip { clip_id: String, new_duration: i64, resp: Sender<Result<(), EditingError>> },
    TimelineRemoveClip { clip_id: String, resp: Sender<Result<(), EditingError>> },
    TimelineAddClip { uri: String, track_type: TrackType, start_time: i64, duration: i64, in_point: i64, resp: Sender<Result<ClipInfo, EditingError>> },
    TimelineMoveClip { clip_id: String, new_start_time: i64, resp: Sender<Result<(), EditingError>> },
    TimelineCreateTrack { track_type: TrackType, resp: Sender<Result<TrackInfo, EditingError>> },
    TimelineDeleteTrack { track_id: String, resp: Sender<Result<(), EditingError>> },
    ImportMedia { path: String, options: ImportOptions, resp: Sender<Result<MediaInfo, EditingError>> },
    GetPreviewDimensions { resp: Sender<Result<(u32, u32), EditingError>> },
    GetPreviewState { resp: Sender<Result<PreviewState, EditingError>> },
    PreviewPlay { resp: Sender<Result<(), EditingError>> },
    PreviewPause { resp: Sender<Result<(), EditingError>> },
    PreviewStop { resp: Sender<Result<(), EditingError>> },
    PreviewSeek { position: i64, resp: Sender<Result<(), EditingError>> },
}

pub struct EditingEngineProxy {
    sender: Sender<EngineCommand>,
}

impl EditingEngineProxy {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<EngineCommand>();
        std::thread::spawn(move || run_engine_thread(receiver));
        Self { sender }
    }

    pub fn init_project(&self, path: Option<String>) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::InitProject { path, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn shutdown(&self) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::Shutdown { resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn get_timeline_info(&self) -> Result<TimelineInfo, EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::GetTimelineInfo { resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn timeline_seek(&self, time: f64) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::TimelineSeek { time, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn timeline_trim_clip(&self, clip_id: String, new_duration: i64) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::TimelineTrimClip { clip_id, new_duration, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn timeline_remove_clip(&self, clip_id: String) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::TimelineRemoveClip { clip_id, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn timeline_add_clip(&self, uri: String, track_type: TrackType, start_time: i64, duration: i64, in_point: i64) -> Result<ClipInfo, EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::TimelineAddClip { uri, track_type, start_time, duration, in_point, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn timeline_move_clip(&self, clip_id: String, new_start_time: i64) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::TimelineMoveClip { clip_id, new_start_time, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn timeline_create_track(&self, track_type: TrackType) -> Result<TrackInfo, EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::TimelineCreateTrack { track_type, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn timeline_delete_track(&self, track_id: String) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::TimelineDeleteTrack { track_id, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn import_media(&self, path: String, options: ImportOptions) -> Result<MediaInfo, EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::ImportMedia { path, options, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn get_preview_dimensions(&self) -> Result<(u32, u32), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::GetPreviewDimensions { resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn get_preview_state(&self) -> Result<PreviewState, EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::GetPreviewState { resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn preview_play(&self) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::PreviewPlay { resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn preview_pause(&self) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::PreviewPause { resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn preview_stop(&self) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::PreviewStop { resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }

    pub fn preview_seek(&self, position: i64) -> Result<(), EditingError> {
        let (tx, rx) = channel();
        self.sender.send(EngineCommand::PreviewSeek { position, resp: tx })
            .map_err(|_| EditingError::TimelineError("Engine thread disconnected".to_string()))?;
        rx.recv().map_err(|_| EditingError::TimelineError("Engine thread response lost".to_string()))?
    }
}

fn run_engine_thread(receiver: std::sync::mpsc::Receiver<EngineCommand>) {
    let main_context = glib::MainContext::new();

    info!("Starting EditingEngine background thread with MainContext");
    let result = main_context.with_thread_default(|| {
        match create_editing_engine() {
            Ok(mut engine) => {
                info!("EditingEngine initialized successfully on dedicated GStreamer thread");
                for cmd in receiver {
                    if let Err(e) = handle_command(&mut engine, cmd) {
                        error!("Error handling engine command: {}", e);
                    }
                    // Process any pending GStreamer events (bus messages, state changes, etc.)
                    while main_context.iteration(false) {}
                }
                info!("EditingEngine command channel closed; shutting down");
                let _ = engine.shutdown();
            }
            Err(err) => {
                error!("Failed to initialize EditingEngine: {}. Engine thread exiting.", err);
                let err_str = err.to_string();
                for cmd in receiver {
                    match cmd {
                        EngineCommand::InitProject { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::Shutdown { resp } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::GetTimelineInfo { resp } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::TimelineSeek { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::TimelineTrimClip { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::TimelineRemoveClip { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::TimelineAddClip { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::TimelineMoveClip { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::TimelineCreateTrack { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::TimelineDeleteTrack { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::ImportMedia { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::GetPreviewDimensions { resp } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::GetPreviewState { resp } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::PreviewPlay { resp } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::PreviewPause { resp } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::PreviewStop { resp } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                        EngineCommand::PreviewSeek { resp, .. } => { let _ = resp.send(Err(EditingError::GstreamerInitError(err_str.clone()))); }
                    };
                }
            }
        }
    });
    if let Err(e) = result {
        error!("Failed to set MainContext as thread default: {}", e);
    }
}

fn handle_command(engine: &mut EditingEngine, cmd: EngineCommand) -> Result<(), String> {
    match cmd {
        EngineCommand::InitProject { path, resp } => {
            let _ = resp.send(engine.init_project(path));
        }
        EngineCommand::Shutdown { resp } => {
            let _ = resp.send(engine.shutdown());
        }
        EngineCommand::GetTimelineInfo { resp } => {
            let timeline = engine.timeline();
            let result = (|| -> Result<TimelineInfo, EditingError> {
                let timeline_guard = timeline.lock()
                    .map_err(|_| EditingError::TimelineError("Timeline lock poisoned".to_string()))?;
                let clips = timeline_guard.get_clips();
                let duration = timeline_guard.get_duration();
                let mut tracks = Vec::new();
                for t in timeline_guard.get_video_tracks() {
                    tracks.push(TrackInfo { id: t.id.clone(), track_type: t.track_type, clips: t.clips.clone() });
                }
                for t in timeline_guard.get_audio_tracks() {
                    tracks.push(TrackInfo { id: t.id.clone(), track_type: t.track_type, clips: t.clips.clone() });
                }
                Ok(TimelineInfo { tracks, clips, duration })
            })();
            let _ = resp.send(result);
        }
        EngineCommand::TimelineSeek { time, resp } => {
            let preview = engine.preview();
            let result = (|| -> Result<(), EditingError> {
                let mut preview_guard = preview.lock()
                    .map_err(|_| EditingError::PreviewError("Preview lock poisoned".to_string()))?;
                let position_ns = (time * 1_000_000_000.0) as i64;
                preview_guard.seek(position_ns)
            })();
            let _ = resp.send(result);
        }
        EngineCommand::TimelineTrimClip { clip_id, new_duration, resp } => {
            let timeline = engine.timeline();
            let result = (|| -> Result<(), EditingError> {
                let mut timeline_guard = timeline.lock()
                    .map_err(|_| EditingError::TimelineError("Timeline lock poisoned".to_string()))?;
                timeline_guard.trim_clip(&clip_id, new_duration)
            })();
            let _ = resp.send(result);
        }
        EngineCommand::TimelineRemoveClip { clip_id, resp } => {
            let timeline = engine.timeline();
            let result = (|| -> Result<(), EditingError> {
                let mut timeline_guard = timeline.lock()
                    .map_err(|_| EditingError::TimelineError("Timeline lock poisoned".to_string()))?;
                timeline_guard.remove_clip(&clip_id)
            })();
            let _ = resp.send(result);
        }
        EngineCommand::TimelineAddClip { uri, track_type, start_time, duration, in_point, resp } => {
            let timeline = engine.timeline();
            let result = (|| -> Result<ClipInfo, EditingError> {
                let mut timeline_guard = timeline.lock()
                    .map_err(|_| EditingError::TimelineError("Timeline lock poisoned".to_string()))?;
                let clip = timeline_guard.add_clip(&uri, track_type, start_time, duration, in_point)?;
                Ok(clip.to_clip_info())
            })();
            let _ = resp.send(result);
        }
        EngineCommand::TimelineMoveClip { clip_id, new_start_time, resp } => {
            let timeline = engine.timeline();
            let result = (|| -> Result<(), EditingError> {
                let mut timeline_guard = timeline.lock()
                    .map_err(|_| EditingError::TimelineError("Timeline lock poisoned".to_string()))?;
                timeline_guard.move_clip(&clip_id, new_start_time)
            })();
            let _ = resp.send(result);
        }
        EngineCommand::TimelineCreateTrack { track_type, resp } => {
            let timeline = engine.timeline();
            let result = (|| -> Result<TrackInfo, EditingError> {
                let mut timeline_guard = timeline.lock()
                    .map_err(|_| EditingError::TimelineError("Timeline lock poisoned".to_string()))?;
                let track = match track_type {
                    TrackType::Video => timeline_guard.add_video_track()?,
                    TrackType::Audio => timeline_guard.add_audio_track()?,
                };
                Ok(TrackInfo { id: track.id, track_type: track.track_type, clips: track.clips })
            })();
            let _ = resp.send(result);
        }
        EngineCommand::TimelineDeleteTrack { track_id, resp } => {
            let timeline = engine.timeline();
            let result = (|| -> Result<(), EditingError> {
                let mut timeline_guard = timeline.lock()
                    .map_err(|_| EditingError::TimelineError("Timeline lock poisoned".to_string()))?;
                timeline_guard.remove_track(&track_id)
            })();
            let _ = resp.send(result);
        }
        EngineCommand::ImportMedia { path, options, resp } => {
            let importer = engine.importer();
            let result = (|| -> Result<MediaInfo, EditingError> {
                let mut importer_guard = importer.lock()
                    .map_err(|_| EditingError::ImportError("Importer lock poisoned".to_string()))?;
                importer_guard.import_media(&path, Some(options))
            })();
            let _ = resp.send(result);
        }
        EngineCommand::GetPreviewDimensions { resp } => {
            let preview = engine.preview();
            let result = (|| -> Result<(u32, u32), EditingError> {
                let preview_guard = preview.lock()
                    .map_err(|_| EditingError::PreviewError("Preview lock poisoned".to_string()))?;
                preview_guard.get_video_dimensions()
                    .ok_or(EditingError::PreviewError("No video dimensions available".to_string()))
            })();
            let _ = resp.send(result);
        }
        EngineCommand::GetPreviewState { resp } => {
            let preview = engine.preview();
            let result = (|| -> Result<PreviewState, EditingError> {
                let preview_guard = preview.lock()
                    .map_err(|_| EditingError::PreviewError("Preview lock poisoned".to_string()))?;
                Ok(PreviewState {
                    is_playing: preview_guard.is_playing(),
                    position: preview_guard.get_position().unwrap_or(0),
                    dimensions: preview_guard.get_video_dimensions(),
                    duration: preview_guard.get_duration(),
                })
            })();
            let _ = resp.send(result);
        }
        EngineCommand::PreviewPlay { resp } => {
            let preview = engine.preview();
            let result = (|| -> Result<(), EditingError> {
                let mut preview_guard = preview.lock()
                    .map_err(|_| EditingError::PreviewError("Preview lock poisoned".to_string()))?;
                preview_guard.play()
            })();
            let _ = resp.send(result);
        }
        EngineCommand::PreviewPause { resp } => {
            let preview = engine.preview();
            let result = (|| -> Result<(), EditingError> {
                let mut preview_guard = preview.lock()
                    .map_err(|_| EditingError::PreviewError("Preview lock poisoned".to_string()))?;
                preview_guard.pause()
            })();
            let _ = resp.send(result);
        }
        EngineCommand::PreviewStop { resp } => {
            let preview = engine.preview();
            let result = (|| -> Result<(), EditingError> {
                let mut preview_guard = preview.lock()
                    .map_err(|_| EditingError::PreviewError("Preview lock poisoned".to_string()))?;
                preview_guard.stop()
            })();
            let _ = resp.send(result);
        }
        EngineCommand::PreviewSeek { position, resp } => {
            let preview = engine.preview();
            let result = (|| -> Result<(), EditingError> {
                let mut preview_guard = preview.lock()
                    .map_err(|_| EditingError::PreviewError("Preview lock poisoned".to_string()))?;
                preview_guard.seek(position)
            })();
            let _ = resp.send(result);
        }
    }
    Ok(())
}
