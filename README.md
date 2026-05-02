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

**Implemented Features:**
- Video decoding with FFmpeg and hardware acceleration
- Audio processing with GStreamer
- Basic color grading system
- Multi-track timeline foundation
- File management and import/export
- Effects framework (blur, sharpen, crop, etc.)

**In Development:**
- Node-based compositing system
- Real-time GPU preview pipeline
- Advanced color tools (scopes, HDR)
- Motion graphics and keyframing

**Missing Features:**
- Node graph UI and execution engine
- Real-time preview with frame caching
- Color scopes (waveform, vectorscope, histogram)
- Animation system with keyframes
- Multi-camera editing
- Plugin architecture

## Technology Stack (2026 Compatible)

**Backend (Rust):**
- **Tauri 2.7.0** - Desktop app framework
- **FFmpeg 7.1.0** - Video decoding/encoding
- **GStreamer 0.24.1** - Audio processing
- **Hardware acceleration** - CUDA, VideoToolbox, VAAPI

**Frontend (Next.js):**
- **Next.js 16.2.4** - React framework
- **React 19.2.0** - UI library
- **TypeScript 6.0.3** - Type safety
- **TailwindCSS 4.2.4** - Styling

## Development Roadmap

### Phase 1: Node-Based Compositing (4-6 weeks)
- Node graph architecture
- Basic compositing nodes (Merge, Transform, Color)
- GPU execution engine

### Phase 2: Real-time Preview (3-4 weeks)
- GPU compute shaders
- Frame caching system
- Adaptive quality rendering

### Phase 3: Advanced Color Tools (3-4 weeks)
- Color scopes (waveform, vectorscope)
- HDR/ACES pipeline
- Professional LUT support

### Phase 4: Motion Graphics (4-5 weeks)
- Keyframing system
- Shape layers and text animation
- Masking and tracking

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

## Contributing

**Areas needing contribution:**
- **Node graph UI** - React component for visual node editing
- **GPU shaders** - Compute shaders for real-time effects
- **Color science** - ACES implementation and HDR support
- **Testing** - Unit tests and integration tests

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
