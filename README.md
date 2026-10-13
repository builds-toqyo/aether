# Aether - Professional Video Editing & Compositing

**A DaVinci Resolve + Adobe Fusion-style application built with Rust and Next.js**

This is the unified monorepo for the Aether project, containing both the Rust (Tauri) backend and the Next.js frontend. Aether aims to provide professional video editing and node-based compositing capabilities.

## Project Goal

Build a **DaVinci Resolve + Adobe Fusion** equivalent with:
- **Professional video editing** with multi-track timeline
- **Node-based compositing** (Fusion-style workflow)
- **Advanced color grading** with scopes and LUT support
- **Real-time GPU acceleration** for smooth playback
- **Cross-platform desktop application** (Windows, macOS, Linux)

## Current Status (Updated June 2026)

**Implemented (Frontend):**
- Node graph UI — 9 components including NodeComponent, ConnectionComponent, NodeGraphCanvas (`src/components/NodeGraph/`)
- Color scopes UI — Histogram, Vectorscope, WaveformScope, ScopeControls (`src/components/ColorScopes/`)
- Timeline UI with tracks, clips, ruler, playhead (`src/components/timeline/`)
- Preview viewer component (`src/components/Preview/`)
- Undo/redo history and clipboard (cut/copy/paste) for timeline clips
- Multicam panel — create clips from selected media, switch active angles, delete clips (`src/components/MulticamPanel.tsx`)
- Plugin manager — load/unload/scan plugins with hook display (`src/components/PluginManager.tsx`)
- Zustand store with slices for project, media, timeline, export, import, multicam, plugins
- `useTauriAPI` hook wiring all backend commands to frontend

**Implemented (Backend):**
- Node graph execution engine with ExecutionContext (`aether_api/src/commands/node_ops/`)
- FFmpeg video decoding (`ffmpeg-next 8.1.0`)
- Multi-track timeline data model with GES integration (`aether_core/src/engine/editing/timeline.rs`)
- File manager and media import with GStreamer 0.25 pipelines (`MediaImporter`, `FileManager`)
- Blur node (`nodes/basic/blur/`)
- Preview engine with frame caching and `AppSink` frame extraction (`PreviewEngine`) — `get_preview_info` and `preview_seek` wired to real engine data
- Animation keyframe framework with full easing suite (30 variants: Linear, Quad, Cubic, Quart, Quint, Sine, Expo, Circ, Back, Elastic, Bounce — all In/Out/InOut) (`aether_core/src/animation/`)
- Font metrics via `fontdue` with system font discovery (macOS/Linux/Windows paths) (`aether_core/src/text/typography.rs`)
- Shape layer path-to-shape conversion via `PathShape` primitive (`aether_core/src/shapes/primitives/path_shape.rs`)
- OCIO/ACES color pipeline — runtime `libloading` FFI with matrix fallback when OCIO library is unavailable (`aether_core/src/color/aces/ocio.rs`)
- Multicam editing — 8 Tauri commands (`multicam_create`, `add_angle`, `remove_angle`, `set_active_angle`, `get_clip`, `list_clips`, `delete_clip`, `sync_by_timecode`) with in-memory registry (`aether_api/src/commands/multicam.rs`)
- Plugin architecture — manifest-based `.dylib`/`.so`/`.dll` loading, hook registry (`OnProjectLoad`, `OnProjectSave`, `OnRenderStart`, `OnRenderComplete`, `ProcessFrame`, `ProcessAudio`, `CustomEffect`), 5 Tauri commands (`aether_api/src/commands/plugin.rs`)
- GStreamer thread safety — `EditingEngineProxy` uses dedicated background thread with `glib::MainContext` and mpsc message passing (no `unsafe impl Send/Sync`)
- GStreamer 0.25 API compatibility verified (`cargo check` passes) — uses `ElementFactory::make()`, `UriClipAsset::request_sync()`, `TransitionClip::new()`, `Effect::new()`

**Partially Implemented / In Development:**
- Hardware acceleration (CUDA, VAAPI, VideoToolbox, AMF) — GStreamer hardware decoding/encoding implemented with auto-detection; native GPU compute (wgpu) provides cross-platform acceleration
- Audio processing with GStreamer — preview engine video path works; audio playback path is incomplete
- GStreamer-based rendering/export — `EncodingProfileBuilder` creates profiles; full timeline → encodebin → filesink pipeline needs completion
- Timeline GES sync — Rust timeline model exists; `sync_to_ges()` implemented
- Real-time preview frame data pipeline — `PreviewEngine` extracts frames via `AppSink`; base64/binary buffer delivery to frontend canvas not wired
- Color scopes backend — frontend UI components exist; backend data generation (histogram/vectorscope/waveform pixel analysis) is stubbed
- Node graph ↔ Timeline integration — node execution engine and timeline both exist; connection between node outputs and timeline clip effects implemented
- Plugin effect execution — hook registry and manifest loading work; actual frame/audio processing callback invocation not implemented

**Missing Features:**
- Keyboard shortcut system (global `keydown` listener for `Ctrl+Z`, `Ctrl+S`, `Space`, etc.)
- Export dialog UI (format, resolution, quality, output path selection)
- Media import file picker integration (Tauri `dialog.open()` → `media_import` command)
- Project open/save file dialogs (Tauri `dialog.open()` / `dialog.save()` → `project_load` / `project_save`)
- GPU shader execution for real-time node effects (wgpu compute pipeline exists but not wired to node execution)
- Project persistence — projects exist only in memory; save/load to disk is stubbed
- Full text rendering engine — font metrics exist but glyph rasterization and text layer compositing not implemented
- Advanced multicam sync — timecode parsing and audio waveform alignment are stubs
- Plugin marketplace / registry UI

## Technology Stack (2026 Compatible)

**Backend (Rust):**
- **Tauri 2.8.5** — Desktop app framework
- **FFmpeg** (via `ffmpeg-next 8.1.0` / `ffmpeg-sys-next 8.1.0`) — Video decoding/encoding
- **GStreamer 0.25.x** (`gstreamer 0.25.1`, `ges 0.25.0`) — Audio/video pipeline (editing engine)
- **wgpu 29.0.3** — GPU compute for node execution
- **glib 0.22.7** — GStreamer main loop and context
- **Hardware acceleration** — CUDA, VAAPI, VideoToolbox, AMF (feature flags exist, stubs not wired)

**Frontend (Next.js):**
- **Next.js 16.2.4** — React framework
- **React 19.2.0** — UI library
- **TypeScript 6.0.3** — Type safety
- **TailwindCSS 4.1.16** — Styling
- **Zustand 5.0.13** — State management
- **Tauri API 2.11.0** — Frontend-backend IPC

## Development Roadmap

### Phase 1: Backend-Frontend Wiring (2-3 weeks)
- Wire preview frame extraction to frontend canvas (`PreviewEngine` → Tauri command → base64 image)
- Wire stubbed timeline commands (~12 TODOs in `timeline.rs`) to real `Timeline` methods
- Wire color scopes backend data pipeline (histogram/vectorscope/waveform pixel analysis)
- Wire media import file picker (Tauri `dialog.open()` → `media_import`)
- Wire project open/save dialogs (`dialog.open()` → `project_load`, `dialog.save()` → `project_save`)
- Add keyboard shortcuts system (global `keydown` listener)

### Phase 2: GStreamer Pipeline Completion (2-3 weeks)
- Complete export pipeline (`IntermediateExporter` → encodebin → filesink)
- Add audio playback path to `PreviewEngine`
- Verify `gst_pbutils::EncodingProfileBuilder` compatibility with GStreamer 0.25.x
- Fix GStreamer preview `get_state` signature if needed

### Phase 3: Node-Based Compositing (4-6 weeks)
- Connect node graph to timeline clips (node output as clip effect/overlay)
- Add compositing nodes (Merge, Transform, Color, Keyer)
- GPU shader execution via wgpu compute pipeline

### Phase 4: Real-time Preview & Hardware Acceleration (3-4 weeks)
- Working frame caching and adaptive quality
- Hardware acceleration detection and usage (CUDA, VAAPI, VideoToolbox, AMF)
- Performance stats from real pipeline

### Phase 5: Advanced Features (4-5 weeks)
- Text rendering engine with real glyph rasterization
- Advanced multicam sync (timecode/audio waveform alignment)
- Plugin effect execution (frame/audio processing hook invocation)
- Plugin marketplace / registry UI
- Project persistence (SQLite or JSON on disk)

## Quick Start

1. **Install dependencies:**
   ```sh
   npm install
   cargo build
   ```

2. **Start development:**
   ```sh
   npm run tauri dev
   ```

3. **Build production:**
   ```sh
   npm run build
   npm run tauri build
   ```

## Project Structure

```
/aether/
├── .gitignore
├── Cargo.toml
├── package.json
├── next.config.mjs
├── tsconfig.json
├── assets/
├── tests/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   ├── icons/
│   └── crates/
│       ├── aether_core/
│       ├── aether_api/
│       ├── aether_types/
│       └── aether_cli/
└── src/
    ├── public/
    │   └── icons/
    ├── app/
    ├── components/
    ├── hooks/
    ├── styles/
    └── utils/
```

## Architecture

### Backend (Rust, Tauri)
- **Directory:** `src-tauri/`
- **Main entry:** `src/main.rs` (desktop app)
- **Crates:**
  - `aether_core`: Video/audio processing, rendering engine
  - `aether_api`: IPC commands and events (`#[tauri::command]`)
  - `aether_types`: Shared Rust types for IPC
  - `aether_cli`: Command-line interface

### Frontend (Next.js)
- **Directory:** `src/`
- **App Router:** `src/app/`
- **Components:** `src/components/`
- **Hooks:** `src/hooks/`
- **IPC Utility:** `src/utils/ipc.ts` (Tauri IPC wrapper)

### IPC Communication
- Frontend calls Rust via `@tauri-apps/api`
- Rust exposes commands in `aether_api` with `#[tauri::command]`
- Shared types in `aether_types` ensure type safety

## Development

### Prerequisites
- **Rust 1.80+** (latest stable recommended)
- **Node.js 22+**
- **System dependencies:** FFmpeg, GStreamer development libraries

### Hardware Acceleration (Optional)

Aether supports hardware-accelerated video decoding and encoding via GStreamer plugins. This significantly improves performance on systems with supported GPUs.

**Supported Platforms:**

| Platform | Decoding | Encoding | SDK Required |
|----------|---------|----------|--------------|
| Linux (Intel) | vaapidecode | vaapiencode_h264, vaapiencode | libva, libva-intel-driver, gstreamer-vaapi |
| macOS | videotoolboxdec | vtenc_h264, videotoolboxenc | macOS VideoToolbox (built-in), gstreamer-apple |
| Windows | d3d11dec | d3d11h264enc, d3d11enc | DirectX 11, gstreamer-libav |
| NVIDIA (cross-platform) | nvdec, nvv4l2decoder | nvh264enc, nvenc | NVIDIA CUDA Toolkit, gstreamer-nvcodec |
| AMD (cross-platform) | amfdec | amfh264enc, amfenc | AMD AMF SDK, gstreamer-amf |

**Installation:**

- **Linux (Intel):** `sudo apt install gstreamer1.0-vaapi libva2 intel-media-va-driver-nonfree`
- **macOS:** GStreamer includes VideoToolbox support via `gstreamer-apple` plugins
- **Windows:** Install GStreamer with `libav` plugins for D3D11 support
- **NVIDIA:** Install CUDA Toolkit and GStreamer NVENC/NVDEC plugins
- **AMD:** Install AMD AMF SDK and GStreamer AMF plugins

**Note:** Hardware acceleration is optional. Aether will automatically detect available hardware and fall back to software encoding/decoding if unavailable.

### Setup
```sh
# Clone and install
git clone <repository>
cd aether
npm install
cargo build

# Start development server
npm run tauri dev
```

### Build
```sh
# Development build
npm run build
npm run tauri build

# Release build (optimized)
npm run build
npm run tauri build -- --target release
```

## Known Issues

1. **~90 stubbed API commands** — Many Tauri commands return mock data or `"TODO"` errors. Breakdown: editing ~32, rendering ~39, preview ~26, timeline ~12, node_ops ~4. See `plans.md` §2.1 for full list.
2. **Frontend dialogs not wired** — Media import, Open Project, Save As, and Quick Export menu items show placeholder toasts instead of calling Tauri `dialog.open()` / `dialog.save()`.
3. **Preview frame data not delivered to frontend** — `PreviewEngine` extracts frames via `AppSink`, but the Tauri command response pipeline (base64 image or binary buffer) is not wired to the frontend canvas.
4. **Color scopes backend is stubbed** — Frontend UI components (Histogram, Vectorscope, WaveformScope) exist, but backend pixel analysis data generation is not implemented.
5. **GStreamer audio pipeline incomplete** — Video preview path works; audio playback path through `PreviewEngine` is not fully wired.
6. **Export pipeline incomplete** — `IntermediateExporter` creates an encoding profile via `EncodingProfileBuilder`, but the full timeline → encodebin → filesink pipeline needs completion.
7. **Hardware acceleration is stubbed** — CUDA, VAAPI, VideoToolbox, and AMF detection/init code exists but does not call real APIs.
8. **Node graph ↔ Timeline not connected** — Node execution engine and timeline model both exist independently; no pipeline connects node outputs to timeline clip effects.
9. **Plugin hooks not executed** — Hook registry and manifest loading work, but actual frame/audio processing callbacks are not invoked during render/preview.
10. **No project persistence** — Projects exist only in memory. Save/load to disk is stubbed.

## Contributing

**Areas needing contribution:**
- **Backend-frontend wiring** — Connect ~90 stubbed Tauri commands to real engine methods (highest impact)
- **Frontend dialogs** — Wire Tauri `dialog.open()` / `dialog.save()` for media import, project open/save, export
- **Preview frame pipeline** — Deliver `PreviewEngine` frames to frontend canvas (base64 or binary buffer)
- **Color scopes backend** — Implement histogram/vectorscope/waveform pixel analysis
- **GStreamer export pipeline** — Complete timeline → encodebin → filesink wiring
- **GStreamer audio pipeline** — Complete audio playback path in `PreviewEngine`
- **GPU shaders** — Wire wgpu compute pipeline to node execution engine
- **Hardware acceleration** — Wire CUDA, VAAPI, VideoToolbox, AMF detection to real APIs
- **Project persistence** — Implement SQLite or JSON project save/load
- **Keyboard shortcuts** — Global `keydown` listener with shortcut map
- **Node ↔ Timeline integration** — Connect node graph outputs to timeline clip effects
- **Testing** — Unit tests and integration tests for core engine modules

**Guidelines:**
- Follow Rust and Next.js best practices
- Use IPC for frontend-backend communication
- Keep shared types in `aether_types`
- Add tests for new features
- Document public APIs

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔗 Related Projects

- **DaVinci Resolve** - Inspiration for editing workflow
- **Adobe Fusion** - Node-based compositing reference
- **Blackmagic Design** - Professional color grading standards
