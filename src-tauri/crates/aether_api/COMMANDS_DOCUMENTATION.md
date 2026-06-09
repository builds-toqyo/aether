# Aether API Commands Documentation

This document provides comprehensive documentation for all Tauri commands implemented in the Aether video editing application.

## Table of Contents

- [Node Operations](#node-operations)
- [Timeline Operations](#timeline-operations)
- [Preview Operations](#preview-operations)
- [Editing Operations](#editing-operations)

---

## Node Operations

### `create_node`
Creates a new node in the node graph.

**Parameters:**
- `node_type: String` - Type of node to create
- `name: String` - Name for the node
- `position: Option<(f64, f64)>` - Optional position coordinates

**Returns:** `NodeResponse`

### `delete_node`
Deletes a node from the node graph.

**Parameters:**
- `node_id: String` - ID of the node to delete

**Returns:** `NodeResponse`

### `connect_nodes`
Creates a connection between two nodes.

**Parameters:**
- `source_node_id: String`
- `source_port_id: String`
- `target_node_id: String`
- `target_port_id: String`

**Returns:** `Connection`

### `disconnect_nodes`
Removes a connection between nodes.

**Parameters:**
- `connection_id: String` - ID of the connection to remove

**Returns:** `ConnectionResponse`

### `execute_graph`
Executes the node graph.

**Parameters:**
- `graph_id: String` - ID of the graph to execute

**Returns:** `ExecutionResult`

### `get_node_result`
Gets the result of a specific node execution.

**Parameters:**
- `node_id: String` - ID of the node

**Returns:** `NodeResult`

### `get_graph_info`
Gets information about the current graph.

**Returns:** `GraphInfo`

---

## Timeline Operations

### `get_timeline_info`
Gets comprehensive timeline information including tracks, clips, and playback state.

**Returns:** `TimelineInfo`
```rust
pub struct TimelineInfo {
    pub duration: f64,
    pub current_time: f64,
    pub is_playing: bool,
    pub tracks: Vec<TrackInfo>,
    pub clips: Vec<ClipInfo>,
    pub fps: f64,
    pub resolution: (u32, u32),
}
```

### `timeline_playback_control`
Controls timeline playback (play, pause, stop, seek, etc.).

**Parameters:** `PlaybackControlRequest`
```rust
pub struct PlaybackControlRequest {
    pub action: PlaybackAction, // Play, Pause, Stop, Seek, Next, Previous
    pub current_time: Option<f64>,
}
```

**Returns:** `TimelineResponse`

### `timeline_seek`
Seeks the timeline to a specific time.

**Parameters:** `TimelineSeekRequest`
```rust
pub struct TimelineSeekRequest {
    pub time: f64,
}
```

**Returns:** `TimelineResponse`

### `timeline_move_clip`
Moves a clip to a new position and/or track.

**Parameters:** `TimelineClipMoveRequest`
```rust
pub struct TimelineClipMoveRequest {
    pub clip_id: String,
    pub new_time: f64,
    pub new_track_id: Option<String>,
}
```

**Returns:** `TimelineResponse`

### `timeline_trim_clip`
Trims a clip at either the start or end edge.

**Parameters:** `TimelineClipTrimRequest`
```rust
pub struct TimelineClipTrimRequest {
    pub clip_id: String,
    pub edge: ClipEdge, // Start or End
    pub new_time: f64,
}
```

**Returns:** `TimelineResponse`

### `timeline_add_clip`
Adds a new clip to the timeline.

**Parameters:** `TimelineClipAddRequest`
```rust
pub struct TimelineClipAddRequest {
    pub track_id: String,
    pub source_file: String,
    pub position: f64,
    pub duration: Option<f64>,
    pub in_point: Option<f64>,
    pub out_point: Option<f64>,
}
```

**Returns:** `TimelineResponse`

### `timeline_remove_clip`
Removes a clip from the timeline.

**Parameters:** `TimelineClipRemoveRequest`
```rust
pub struct TimelineClipRemoveRequest {
    pub clip_id: String,
}
```

**Returns:** `TimelineResponse`

### `timeline_create_track`
Creates a new track in the timeline.

**Parameters:**
- `track_type: TrackType` - Video, Audio, Subtitle, or Effects
- `name: Option<String>` - Optional track name

**Returns:** `TimelineResponse`

### `timeline_delete_track`
Deletes a track from the timeline.

**Parameters:**
- `track_id: String` - ID of the track to delete

**Returns:** `TimelineResponse`

---

## Preview Operations

### `get_preview_info`
Gets current preview information including resolution, fps, and playback state.

**Returns:** `PreviewInfo`
```rust
pub struct PreviewInfo {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration: f64,
    pub current_time: f64,
    pub current_frame: u32,
    pub total_frames: u32,
    pub is_playing: bool,
    pub quality: PreviewQuality,
    pub format: FrameFormat,
}
```

### `preview_playback_control`
Controls preview playback.

**Parameters:** `PreviewPlaybackControlRequest`
```rust
pub struct PreviewPlaybackControlRequest {
    pub action: PlaybackAction, // Play, Pause, Stop, Seek, Next, Previous, StepForward, StepBackward
    pub current_time: Option<f64>,
}
```

**Returns:** `PreviewResponse`

### `preview_seek`
Seeks the preview to a specific time.

**Parameters:** `PreviewSeekRequest`
```rust
pub struct PreviewSeekRequest {
    pub time: f64,
}
```

**Returns:** `PreviewResponse`

### `preview_get_frame`
Gets a single frame at a specific timestamp.

**Parameters:** `PreviewFrameRequest`
```rust
pub struct PreviewFrameRequest {
    pub timestamp: f64,
    pub quality: Option<PreviewQuality>,
    pub format: Option<FrameFormat>,
}
```

**Returns:** `PreviewFrame`

### `preview_get_frame_range`
Gets multiple frames for a time range.

**Parameters:**
- `start_time: f64`
- `end_time: f64`
- `quality: Option<PreviewQuality>`
- `format: Option<FrameFormat>`

**Returns:** `Vec<PreviewFrame>`

### `preview_update_settings`
Updates preview rendering settings.

**Parameters:** `PreviewSettings`
```rust
pub struct PreviewSettings {
    pub quality: PreviewQuality,
    pub format: FrameFormat,
    pub scale: f64,
    pub show_safe_areas: bool,
    pub show_grid: bool,
    pub show_overlays: bool,
    pub background_color: String,
}
```

**Returns:** `PreviewResponse`

### `preview_get_settings`
Gets current preview settings.

**Returns:** `PreviewSettings`

### `preview_clear_cache`
Clears the preview frame cache.

**Returns:** `PreviewResponse`

### `preview_get_performance_stats`
Gets preview performance statistics.

**Returns:** `serde_json::Value` with performance metrics including:
- fps
- render_time_ms
- cache_hit_rate
- cache_size_mb
- frames_rendered
- frames_dropped
- memory_usage_mb
- gpu_usage_percent
- cpu_usage_percent

### `preview_export_frame`
Exports the current preview frame to a file.

**Parameters:**
- `timestamp: f64`
- `format: Option<String>` - Export format (png, jpg, etc.)
- `quality: Option<u8>` - Export quality (0-100)

**Returns:** `PreviewResponse`

---

## Editing Operations

### Project Management

#### `project_init`
Initializes a new project.

**Parameters:** `ProjectCreateRequest`
```rust
pub struct ProjectCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub fps: Option<f64>,
    pub resolution: Option<(u32, u32)>,
    pub template: Option<String>,
}
```

**Returns:** `ProjectInfo`

#### `project_save`
Saves the current project.

**Parameters:** `ProjectSaveRequest`
```rust
pub struct ProjectSaveRequest {
    pub project_id: String,
    pub file_path: Option<String>,
    pub auto_save: Option<bool>,
}
```

**Returns:** `EditingResponse`

#### `project_load`
Loads a project from file.

**Parameters:** `ProjectLoadRequest`
```rust
pub struct ProjectLoadRequest {
    pub file_path: String,
}
```

**Returns:** `ProjectInfo`

#### `project_get_recent`
Gets a list of recent projects.

**Parameters:**
- `limit: Option<usize>` - Maximum number of projects to return

**Returns:** `Vec<ProjectInfo>`

#### `project_auto_save`
Performs an auto-save of the project.

**Parameters:**
- `project_id: String`

**Returns:** `EditingResponse`

### Media Operations

#### `media_import`
Imports media files into the project.

**Parameters:** `MediaImportRequest`
```rust
pub struct MediaImportRequest {
    pub file_paths: Vec<String>,
    pub target_track: Option<String>,
    pub position: Option<f64>,
    pub auto_create_clips: Option<bool>,
}
```

**Returns:** `Vec<MediaInfo>`

#### `media_get_info`
Gets information about a specific media file.

**Parameters:**
- `media_id: String`

**Returns:** `MediaInfo`

#### `media_get_all`
Gets information about all imported media.

**Returns:** `Vec<MediaInfo>`

#### `media_remove`
Removes imported media from the project.

**Parameters:**
- `media_id: String`

**Returns:** `EditingResponse`

### Export Operations

#### `media_export`
Exports the project or timeline to a media file.

**Parameters:** `MediaExportRequest`
```rust
pub struct MediaExportRequest {
    pub output_path: String,
    pub format: ExportFormat, // Mp4, Avi, Mov, Mkv, Webm, Gif, etc.
    pub quality: ExportQuality, // Low, Medium, High, Ultra, Custom
    pub resolution: Option<(u32, u32)>,
    pub fps: Option<f64>,
    pub start_time: Option<f64>,
    pub end_time: Option<f64>,
    pub audio_settings: Option<AudioExportSettings>,
}
```

**Returns:** `EditingResponse`

#### `export_get_status`
Gets the status of an ongoing export.

**Parameters:**
- `export_id: String`

**Returns:** `serde_json::Value` with export status information

#### `export_cancel`
Cancels an ongoing export.

**Parameters:**
- `export_id: String`

**Returns:** `EditingResponse`

---

## Data Types

### Common Response Types

#### `TimelineResponse`
```rust
pub struct TimelineResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

#### `PreviewResponse`
```rust
pub struct PreviewResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

#### `EditingResponse`
```rust
pub struct EditingResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

### Enums

#### `TrackType`
- `Video`
- `Audio`
- `Subtitle`
- `Effects`

#### `ClipType`
- `Video`
- `Audio`
- `Image`
- `Text`
- `Effect`

#### `FrameFormat`
- `Rgba8`
- `Bgra8`
- `Rgb8`
- `Jpeg`
- `Png`
- `Nv12`
- `Yuv420`

#### `PreviewQuality`
- `Low`
- `Medium`
- `High`
- `Ultra`

#### `ExportFormat`
- `Mp4`
- `Avi`
- `Mov`
- `Mkv`
- `Webm`
- `Gif`
- `PngSequence`
- `JpegSequence`
- `AudioOnly`

#### `AudioCodec`
- `Aac`
- `Mp3`
- `Opus`
- `Flac`
- `Wav`

---

## Error Handling

All commands return `Result<T, String>` where the error string contains a descriptive error message. Common validation errors include:

- Empty required fields
- Negative values for time/position
- Invalid file paths
- Invalid IDs

---

## Usage Examples

### Timeline Operations
```typescript
// Get timeline info
const timelineInfo = await invoke('get_timeline_info');

// Play timeline
await invoke('timeline_playback_control', {
  action: 'play',
  currentTime: 0.0
});

// Move clip
await invoke('timeline_move_clip', {
  clipId: 'clip_1',
  newTime: 5.0,
  newTrackId: 'track_2'
});
```

### Preview Operations
```typescript
// Get preview frame
const frame = await invoke('preview_get_frame', {
  timestamp: 10.5,
  quality: 'High',
  format: 'Rgba8'
});

// Control preview playback
await invoke('preview_playback_control', {
  action: 'play',
  currentTime: 0.0
});
```

### Editing Operations
```typescript
// Create new project
const project = await invoke('project_init', {
  name: 'My Video Project',
  fps: 30.0,
  resolution: [1920, 1080]
});

// Import media
const media = await invoke('media_import', {
  file_paths: ['/path/to/video.mp4'],
  target_track: 'track_1',
  position: 0.0
});

// Export project
await invoke('media_export', {
  output_path: '/exports/video.mp4',
  format: 'Mp4',
  quality: 'High'
});
```

---

## Rendering Operations

### `rendering_start_job`
Starts a new rendering job.

**Parameters:** `RenderingRequest`
```rust
pub struct RenderingRequest {
    pub name: String,
    pub output_path: String,
    pub format: RenderFormat,
    pub quality: RenderQuality,
    pub resolution: Option<(u32, u32)>,
    pub fps: Option<f64>,
    pub start_time: Option<f64>,
    pub end_time: Option<f64>,
    pub video_settings: Option<VideoRenderSettings>,
    pub audio_settings: Option<AudioRenderSettings>,
    pub export_range: Option<ExportRange>,
    pub metadata: Option<RenderMetadata>,
}
```

**Returns:** `RenderingJob`

### `rendering_cancel_job`
Cancels a rendering job.

**Parameters:**
- `job_id: String` - ID of the job to cancel

**Returns:** `RenderingResponse`

### `rendering_pause_job`
Pauses a rendering job.

**Parameters:**
- `job_id: String` - ID of the job to pause

**Returns:** `RenderingResponse`

### `rendering_resume_job`
Resumes a paused rendering job.

**Parameters:**
- `job_id: String` - ID of the job to resume

**Returns:** `RenderingResponse`

### `rendering_get_job_status`
Gets the status of a specific rendering job.

**Parameters:**
- `job_id: String` - ID of the job

**Returns:** `RenderingJob`
```rust
pub struct RenderingJob {
    pub id: String,
    pub name: String,
    pub status: RenderingStatus,
    pub progress: f64,
    pub current_frame: u32,
    pub total_frames: u32,
    pub start_time: String,
    pub end_time: Option<String>,
    pub output_path: String,
    pub format: RenderFormat,
    pub quality: RenderQuality,
    pub resolution: (u32, u32),
    pub fps: f64,
    pub bitrate: u32,
    pub estimated_size: u64,
    pub actual_size: Option<u64>,
    pub error_message: Option<String>,
}
```

### `rendering_get_job_progress`
Gets detailed progress information for a rendering job.

**Parameters:**
- `job_id: String` - ID of the job

**Returns:** `RenderingProgress`
```rust
pub struct RenderingProgress {
    pub job_id: String,
    pub status: RenderingStatus,
    pub progress: f64,
    pub current_frame: u32,
    pub total_frames: u32,
    pub fps: f64,
    pub time_elapsed: f64,
    pub time_remaining: Option<f64>,
    pub current_stage: String,
    pub estimated_size: u64,
    pub actual_size: Option<u64>,
}
```

### `rendering_get_all_jobs`
Gets information about all rendering jobs.

**Returns:** `Vec<RenderingJob>`

### `rendering_get_queue`
Gets the rendering queue information.

**Returns:** `RenderingQueue`
```rust
pub struct RenderingQueue {
    pub active_jobs: Vec<RenderingJob>,
    pub queued_jobs: Vec<RenderingJob>,
    pub completed_jobs: Vec<RenderingJob>,
    pub failed_jobs: Vec<RenderingJob>,
    pub max_concurrent_jobs: usize,
    pub total_capacity: usize,
}
```

### `rendering_get_formats`
Gets available rendering formats.

**Returns:** `Vec<RenderFormatInfo>`
```rust
pub struct RenderFormatInfo {
    pub format: RenderFormat,
    pub name: String,
    pub description: String,
    pub extensions: Vec<String>,
    pub supports_video: bool,
    pub supports_audio: bool,
    pub recommended_for: Vec<String>,
    pub max_resolution: Option<(u32, u32)>,
    pub max_fps: Option<f64>,
    pub max_bitrate: Option<u32>,
}
```

### `rendering_get_presets`
Gets available rendering presets.

**Returns:** `Vec<RenderPresetInfo>`
```rust
pub struct RenderPresetInfo {
    pub name: String,
    pub description: String,
    pub format: RenderFormat,
    pub resolution: (u32, u32),
    pub fps: f64,
    pub bitrate: u32,
    pub quality: RenderQuality,
    pub preset: RenderPreset,
    pub video_codec: VideoCodec,
    pub audio_codec: AudioCodec,
}
```

### `rendering_estimate_time`
Estimates rendering time for given parameters.

**Parameters:**
- `resolution: (u32, u32)` - Video resolution
- `fps: f64` - Frame rate
- `duration: f64` - Video duration in seconds
- `quality: RenderQuality` - Render quality
- `format: RenderFormat` - Output format

**Returns:** `RenderingTimeEstimate`
```rust
pub struct RenderingTimeEstimate {
    pub estimated_seconds: f64,
    pub estimated_minutes: f64,
    pub estimated_hours: f64,
    pub confidence: f64,
    pub factors_used: Vec<String>,
}
```

### `rendering_get_performance_stats`
Gets rendering performance statistics.

**Returns:** `serde_json::Value` with performance metrics including:
- current_fps
- target_fps
- average_fps
- render_time_per_frame
- encoding_time_per_frame
- memory_usage_mb
- gpu_usage_percent
- cpu_usage_percent
- disk_write_speed_mbps
- frames_dropped
- bottleneck

### `rendering_cleanup_completed`
Cleans up completed rendering jobs.

**Parameters:**
- `older_than_hours: Option<u32>` - Clean up jobs older than this many hours
- `keep_count: Option<u32>` - Keep this many most recent jobs

**Returns:** `RenderingResponse`

---

## Implementation Notes

### Current Status
- All commands are implemented with mock data
- Commands include comprehensive validation
- Error handling is implemented for all operations
- Logging is integrated throughout

### Future Enhancements
- Replace mock data with actual editing engine integration
- Add real-time progress reporting for long operations
- Implement file system operations for media import/export
- Add undo/redo support for editing operations
- Integrate with actual timeline and preview engines

### Testing
- Unit tests are included for validation logic
- Mock data is used for testing command responses
- Error scenarios are tested where applicable
