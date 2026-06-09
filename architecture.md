# Aether Architecture

## Overview

Aether is a Tauri-based desktop video editor with a Rust backend. The application is organized as a Cargo workspace with three crates, following a layered architecture:

- **`aether_core`** — Engine layer (GStreamer, FFmpeg, GPU compute)
- **`aether_api`** — API/command layer (Tauri commands, state management)
- **`aether_types`** — Shared types (graph, parameters, serialization)

The frontend (not covered here) is a Tauri webview that invokes Rust commands via the Tauri IPC bridge.

---

## Crate Dependency Graph

```
src-tauri (Tauri app binary)
├── aether_api  ← commands, AppState, Tauri handler registration
│   ├── aether_core  ← EditingEngine, PreviewEngine, Exporter, Timeline
│   └── aether_types ← Graph, ParameterValue, PinDataType
│
├── aether_core
│   └── aether_types
│
└── aether_types  (leaf crate, no internal deps)
```

- `src-tauri` depends on all three crates and registers the Tauri command handler.
- `aether_api` exposes Tauri commands and holds `AppState`. It depends on `aether_core` for `EditingEngine` and `aether_types` for the node graph.
- `aether_core` is the engine layer. It does not depend on `aether_api`.
- `aether_types` is a small shared-types crate with no external dependencies.

---

## Application State (`AppState`)

`AppState` lives in `aether_api/src/state.rs` and is managed by Tauri via `.manage(AppState::new())`.

```rust
pub struct AppState {
    pub graph: Mutex<Graph>,                           // Node graph
    pub execution_results: Mutex<HashMap<Uuid, ParameterValue>>,
    pub node_execution_order: Mutex<Vec<Uuid>>,
    pub rendering_state: Mutex<RenderingState>,         // Render queue & jobs
    pub editing_engine: Mutex<Option<EditingEngine>>,   // GStreamer/GES engine
}
```

Tauri commands receive `State<'_, AppState>` and lock the relevant field:

```rust
#[tauri::command]
pub async fn preview_get_settings(state: State<'_, AppState>) -> Result<PreviewSettings, String> {
    let editing_engine = state.editing_engine.lock().map_err(|e| format!("{}", e))?;
    if let Some(engine) = editing_engine.as_ref() {
        let preview = engine.preview();
        let preview_guard = preview.lock().map_err(|e| format!("{}", e))?;
        // ... query preview engine
    }
}
```

**Important:** `Mutex<EditingEngine>` (not `Arc<Mutex<...>>`) is used here because Tauri's `State<T>` requires `T: Send + Sync`, and `EditingEngine` currently has `unsafe impl Send/Sync` (see Thread Safety section).

---

## EditingEngine & Sub-Engines

`EditingEngine` (in `aether_core/src/engine/editing/mod.rs`) is the root engine. It owns and initializes all GStreamer/GES objects and exposes sub-engines:

```
EditingEngine
├── ges_timeline: Option<ges::Timeline>       ← GES timeline (project data)
├── ges_pipeline: Option<ges::Pipeline>        ← GES playback/render pipeline
├── importer: Arc<Mutex<MediaImporter>>       ← Media import & proxy generation
├── preview_engine: Arc<Mutex<PreviewEngine>> ← Live preview / player
└── timeline: Arc<Mutex<Timeline>>              ← Rust-side timeline model
```

### `init_project()` flow

When `project_init` is called from the frontend:

1. Tauri command `project_init` in `aether_api/src/commands/editing.rs` locks `state.editing_engine`.
2. Creates `EditingEngine::new()` → calls `gst::init()` and `ges::init()`.
3. Calls `editing_engine.init_project(Some(path))`.
4. `init_project` creates a `ges::Timeline::new_audio_video()`.
5. Creates a `ges::Pipeline::new()` and attaches the timeline.
6. Passes the pipeline to `PreviewEngine::set_pipeline()`.
7. Passes the timeline to `Timeline::set_ges_timeline()`.

After this, the preview engine and timeline are wired to the same GES pipeline.

### Sub-engine accessors

```rust
impl EditingEngine {
    pub fn timeline(&self) -> Arc<Mutex<Timeline>>;
    pub fn importer(&self) -> Arc<Mutex<MediaImporter>>;
    pub fn preview(&self) -> Arc<Mutex<PreviewEngine>>;
}
```

These are used by Tauri commands to reach specific sub-systems:
- Timeline commands (`timeline_add_clip`, `timeline_move_clip`, etc.) call `engine.timeline()`.
- Preview commands (`preview_get_settings`, `preview_get_performance_stats`) call `engine.preview()`.
- Media commands (`media_import`) call `engine.importer()`.

---

## Data Flow: Frontend → Backend

### Example: Preview Seek

```
Frontend JS/TS
       |
       v
Tauri IPC invoke("preview_seek", { time: 5.0 })
       |
       v
#[tauri::command]
pub async fn preview_seek(request: PreviewSeekRequest, state: State<'_, AppState>)
       |
       v
state.editing_engine.lock()
       |
       v
EditingEngine (ges::Pipeline)
       |
       v
PreviewEngine::seek(time)
       |
       v
GStreamer pipeline seek (gst::Event::new_seek())
```

**Current state:** `preview_seek` is stubbed. It validates the input time but does **not** call `engine.preview().seek()`. See `plans.md` §7.1.

### Example: Timeline Add Clip

```
Frontend JS/TS
       |
       v
invoke("timeline_add_clip", { track_id, media_id, position, duration })
       |
       v
#[tauri::command] timeline_add_clip(request, state)
       |
       v
state.editing_engine.lock() → engine.timeline().lock()
       |
       v
Timeline::add_clip(clip_info)  ← updates Rust model
       |
       v
Timeline::sync_to_ges()        ← pushes changes to GES timeline
```

**Current state:** Some timeline commands are stubbed with `"TODO"` returns. See `plans.md` §2.1.

---

## Tauri Command Registration

Commands are registered in two places (legacy split):

### `aether_api/src/lib.rs`

The **primary** handler with all modern commands (preview, timeline, project, media, rendering, nodes):

```rust
pub fn init_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            create_node, delete_node, connect_nodes, ...,
            get_preview_info, preview_playback_control, preview_seek, ...,
            project_init, project_save, ...,
            rendering_start_job, rendering_cancel_job, ...,
        ])
}
```

### `src-tauri/src/lib.rs`

An **older/legacy** handler with a subset of editing/rendering commands:

```rust
pub fn init_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            create_project, open_project, save_project, ...,
            start_rendering, get_export_progress, ...,
        ])
}
```

**Note:** The main entry point (`src-tauri/src/main.rs`) currently calls `src_tauri::init_app()`, which uses the legacy handler. The `aether_api::init_app()` handler is not currently wired into the binary. This is a known architectural gap — either consolidate or switch main.rs to use `aether_api::init_app()`.

---

## Thread Safety Architecture

### Current State (Workaround)

GStreamer/GES objects (`ges::Timeline`, `ges::Pipeline`, `gst::Element`, etc.) are GObject-based and do **not** implement `Send` or `Sync`.

To store `EditingEngine` in Tauri `State`, the codebase uses:

```rust
// aether_core/src/engine/editing/mod.rs:34-37
unsafe impl Send for EditingEngine {}
unsafe impl Sync for EditingEngine {}
```

This compiles but is **unsafe** because GStreamer objects are bound to the thread/main context that created them.

### Planned Fix (from `plans.md`)

Replace direct ownership with an **mpsc proxy pattern**:

```
Tauri Command Thread
       |
       v
AppState.editing_engine: Mutex<EditingEngineProxy>
       |
       +-- Sender<EngineCommand> --+
                                   |
                                   v
                          Dedicated GStreamer Thread
                          (owns real EditingEngine + GES objects)
                                   |
                                   +-- Receiver<EngineCommand>
                                   |
                                   v
                              processes commands
                                   |
                                   +-- sends EngineResponse back
```

`EditingEngineProxy` is `Send + Sync` naturally because it only holds a `Sender`. All actual GES API calls happen on the dedicated thread.

The same pattern applies to `Exporter` and `GstExporter`.

---

## Node System (Graph-Based Compositing)

Aether uses a node-graph system for effects and compositing, separate from the timeline:

```
AppState.graph: Mutex<Graph>
```

`Graph` is defined in `aether_types` and contains:
- `nodes: HashMap<Uuid, Node>`
- `connections: HashMap<Uuid, Connection>`

Node commands (`create_node`, `connect_nodes`, `execute_graph`) live in `aether_api/src/commands/node_ops/`.

**Execution flow:**
1. Frontend creates nodes and connections.
2. `execute_graph` triggers `NodeExecutor::execute()` per node in topological order.
3. Results are stored in `AppState.execution_results`.
4. `get_node_result` retrieves output values by pin ID.

**Note:** The node system and timeline system are currently separate. Future integration will allow node graphs to be applied as effects on timeline clips.

---

## Rendering Pipeline

Two rendering paths exist:

### 1. Timeline Renderer (`aether_core/src/engine/rendering/`)

Uses `TimelineRenderer` + `Renderer` to render the timeline to a file:

```
TimelineRenderer
├── Timeline (Rust model)
├── Renderer (FFmpeg-based)
│   ├── VideoEncoder (ffmpeg-next)
│   ├── AudioEncoder (ffmpeg-next)
│   └── Frame processing pipeline
└── Output file
```

### 2. GStreamer Exporter (`GstExporter`)

Uses GES pipeline with an encoding profile to render directly:

```
GstExporter
├── ges::Timeline
├── ges::Pipeline
├── EncodingProfile (video + audio)
└── Output file
```

**Current issues:** GES API changes broke `set_render_settings()` and encoding profile construction. See `plans.md` §3.2, §3.3.

---

## File Organization

```
src-tauri/
├── Cargo.toml                 # Workspace root + Tauri binary
├── src/
│   ├── main.rs                # Entry point
│   └── lib.rs                 # Legacy AppState + command handler
│
└── crates/
    ├── aether_api/
    │   ├── src/
    │   │   ├── lib.rs          # Modern command handler registration
    │   │   ├── state.rs        # AppState definition
    │   │   └── commands/
    │   │       ├── editing.rs      # Project, media, import
    │   │       ├── timeline.rs     # Timeline operations
    │   │       ├── preview.rs      # Preview / player
    │   │       ├── rendering.rs    # Render queue & jobs
    │   │       └── node_ops/       # Node graph commands
    │   │           ├── creation.rs
    │   │           ├── connections.rs
    │   │           ├── execution.rs
    │   │           └── queries.rs
    │   └── Cargo.toml
    │
    ├── aether_core/
    │   ├── src/
    │   │   ├── engine/
    │   │   │   ├── editing/
    │   │   │   │   ├── mod.rs          # EditingEngine
    │   │   │   │   ├── preview.rs      # PreviewEngine
    │   │   │   │   ├── timeline.rs     # Timeline model
    │   │   │   │   ├── import.rs       # MediaImporter
    │   │   │   │   ├── effects.rs      # Effects & transitions
    │   │   │   │   ├── export.rs       # IntermediateExporter
    │   │   │   │   └── types.rs        # EditingError, MediaInfo, etc.
    │   │   │   ├── rendering/
    │   │   │   │   ├── renderer.rs     # FFmpeg renderer
    │   │   │   │   ├── gst_exporter.rs # GES exporter
    │   │   │   │   ├── export.rs       # Export state/progress
    │   │   │   │   └── formats.rs      # ContainerFormat, VideoFormat, AudioFormat
    │   │   │   └── video_decoder.rs    # Standalone video decoder
    │   │   ├── nodes/                # Node system
    │   │   ├── color/                # Color grading / ACES
    │   │   ├── text/                 # Typography
    │   │   ├── animation/            # Animation & easing
    │   │   ├── shapes/               # Vector shapes
    │   │   └── modules/              # File manager, metadata
    │   └── Cargo.toml
    │
    └── aether_types/
        ├── src/
        │   ├── lib.rs
        │   ├── graph.rs            # Graph, Node, Connection
        │   └── parameter.rs        # ParameterValue, PinDataType
        └── Cargo.toml
```

---

## Known Architectural Gaps

1. **Dual `init_app()` handlers** — `src-tauri/src/lib.rs` and `aether_api/src/lib.rs` both define command handlers. Only the former is used at runtime. Consolidation needed.

2. **`unsafe impl Send/Sync`** on `EditingEngine` — Must be replaced with mpsc proxy pattern before production use.

3. **Stubbed preview commands** — `get_preview_info`, `preview_seek`, `preview_get_frame`, etc. return mock data. They need to be wired to `PreviewEngine`.

4. **Stubbed timeline commands** — Many timeline operations return `"TODO"` errors.

5. **GStreamer API drift** — `parse_launch`, `set_render_settings`, `Transition::new()`, etc. were removed in GStreamer 0.25+.

6. **ffmpeg-next API drift** — `codec::find_by_name`, `codec_params` direct access changed in 7.x/8.x.

7. **No project persistence** — Projects exist only in memory. Save/load is stubbed.

8. **Node system ↔ Timeline integration** — Node graphs are not yet connected to timeline clips.

---

## Data Flow Summary Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Frontend (Tauri Webview)                     │
│  React / Vue / Svelte → Tauri IPC invoke() / listen()               │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Tauri Runtime                                │
│  Command Router → State<'_, AppState> → Lock relevant Mutex field   │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    ▼                           ▼
        ┌──────────────────┐        ┌────────────────────┐
        │  Node Graph      │        │  EditingEngine     │
        │  (aether_types)  │        │  (aether_core)     │
        │                  │        │                    │
        │  Graph           │        │  ├── PreviewEngine│
        │  ├── Node        │        │  ├── Timeline     │
        │  ├── Connection  │        │  ├── MediaImporter│
        │  └── Parameter   │        │  └── Exporter     │
        └──────────────────┘        └────────────────────┘
                    │                           │
                    ▼                           ▼
        ┌──────────────────┐        ┌────────────────────┐
        │  NodeExecutor    │        │  GStreamer / GES     │
        │  (Rust compute)  │        │  (C FFI pipeline)    │
        │                  │        │                    │
        │  wgpu shaders    │        │  ges::Pipeline      │
        │  CPU fallback    │        │  gst::Element       │
        └──────────────────┘        └────────────────────┘
                                                 │
                                                 ▼
                                       ┌──────────────────┐
                                       │  Output          │
                                       │  ├── Preview window
                                       │  ├── Rendered file
                                       │  └── Thumbnails
                                       └──────────────────┘
```
