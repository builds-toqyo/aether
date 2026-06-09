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

## Current Status (Updated May 2026)

**Implemented (Frontend):**
- Node graph UI (`src/components/NodeGraph/`)
- Color scopes UI — waveform, vectorscope, histogram (`src/components/ColorScopes/`)
- Timeline UI with tracks, clips, ruler, playhead (`src/components/timeline/`)
- Preview viewer component (`src/components/Preview/`)

**Implemented (Backend):**
- Node graph execution engine (`aether_api/src/commands/node_ops/`)
- FFmpeg video decoding (`ffmpeg-next`)
- Multi-track timeline data model (`aether_core/src/engine/editing/timeline.rs`)
- File manager and media import (`MediaImporter`, `FileManager`)
- Blur node (`nodes/basic/blur/`)
- Preview engine skeleton with frame caching (`PreviewEngine`)
- Animation keyframe framework (`aether_core/src/animation/`)

**Partially Implemented / In Development:**
- Hardware acceleration (CUDA, VAAPI, VideoToolbox, AMF) — stubs exist, not wired
- Audio processing with GStreamer — API drift (GStreamer 0.25), needs rewrite
- Color grading (ACES) — placeholder logic, no real OCIO bindings
- GStreamer-based rendering/export — API broken in 0.25, needs migration
- Timeline GES sync — model exists, GES integration unverified

**Missing Features:**
- Multi-camera editing
- Plugin architecture
- Real-time preview with working frame cache (backend data pipeline)
- Animation easing functions
- Font metrics and text rendering engine
- Shape layer path-to-shape conversion

## Technology Stack (2026 Compatible)

**Backend (Rust):**
- **Tauri 2.8.5** — Desktop app framework
- **FFmpeg** (via `ffmpeg-next 8.1.0`) — Video decoding/encoding
- **GStreamer 0.25.x** — Audio/video pipeline (editing engine)
- **wgpu 29.0.3** — GPU compute for node execution
- **Hardware acceleration** — CUDA, VAAPI, VideoToolbox, AMF (planned)

**Frontend (Next.js):**
- **Next.js** — React framework (latest, see `package.json`)
- **React 19.2.0** — UI library
- **TypeScript** — Type safety (latest, see `package.json`)
- **TailwindCSS 4.1.16** — Styling

## Development Roadmap

### Phase 1: Compilation & API Stabilization (2-3 weeks)
- Fix GStreamer 0.25 API drift (`parse_launch`, `set_render_settings`, encoding profiles)
- Fix ffmpeg-next 8.x API changes (`codec::find_by_name`, `codec_params`)
- Remove `unsafe impl Send/Sync` workarounds via mpsc proxy pattern
- Pin all dependency versions

### Phase 2: Backend-Frontend Wiring (3-4 weeks)
- Wire stubbed preview commands to `PreviewEngine`
- Wire stubbed timeline commands to `Timeline`
- Wire color scopes backend data pipeline
- Implement project persistence (save/load)

### Phase 3: Node-Based Compositing (4-6 weeks)
- Connect node graph to timeline clips
- Add compositing nodes (Merge, Transform, Color)
- GPU shader execution via wgpu

### Phase 4: Real-time Preview (3-4 weeks)
- Working frame caching and adaptive quality
- Hardware acceleration detection and usage
- Performance stats from real pipeline

### Phase 5: Advanced Color & Motion (4-5 weeks)
- ACES/OCIO color pipeline
- Animation easing functions
- Font metrics and text rendering
- Multi-camera editing
- Plugin architecture

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

1. **GStreamer 0.25 API drift** — Several GStreamer APIs used in the codebase were removed in 0.25 (e.g., `gst::parse_launch`, `ges::Pipeline::set_render_settings`, `ges::Transition::new`). These need migration to the new API.
2. **`unsafe impl Send/Sync` workaround** — `EditingEngine`, `Exporter`, and `GstExporter` use `unsafe impl Send/Sync` to be stored in Tauri `State`. This is a temporary workaround until the mpsc proxy pattern is implemented.
3. **113 stubbed API commands** — Many Tauri commands return mock data or `"TODO"` errors instead of calling real engine methods. See `plans.md` for full list.
4. **Hardware acceleration is stubbed** — CUDA, VAAPI, VideoToolbox, and AMF detection/init code exists but does not call real APIs.

## Contributing

**Areas needing contribution:**
- **GStreamer 0.25 migration** — Update broken API calls to modern GES/gst signatures
- **Backend-frontend wiring** — Connect stubbed Tauri commands to real `EditingEngine` methods
- **GPU shaders** — Compute shaders for real-time node effects
- **Color science** — Real OCIO/ACES bindings or approximation
- **Testing** — Unit tests and integration tests

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
