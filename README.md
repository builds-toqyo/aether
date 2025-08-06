# Aether Monorepo

This is the unified monorepo for the Aether project, containing both the Rust (Tauri) backend and the Next.js frontend.

---

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

---

## Orchestration

- **Root `package.json`**: Contains scripts for frontend (Next.js) and Tauri orchestration (`tauri dev`, `tauri build`).
- **Root `Cargo.toml`**: Rust workspace for all backend crates.
- **`src-tauri/tauri.conf.json`**: Tauri configuration. Defines how the app launches, which dev server to use, and how assets are bundled.

### Development
- Run `npm install` to install frontend dependencies.
- Run `cargo build` to build Rust crates.
- Run `npm run tauri dev` to launch the full Tauri app with the Next.js frontend.

---

## Backend (Rust, Tauri)
- **Directory:** `src-tauri/`
- **Main entry:** `src/main.rs` (desktop app), `src/lib.rs` (mobile entry, if needed)
- **Crates:**
  - `aether_core`: Main application logic
  - `aether_api`: IPC commands and events (with `#[tauri::command]`)
  - `aether_types`: Shared Rust types for IPC
  - `aether_cli`: Command-line interface

---

## Frontend (Next.js)
- **Directory:** `src/`
- **App Router:** `src/app/`
- **Reusable Components:** `src/components/`
- **Hooks:** `src/hooks/`
- **IPC Utility:** `src/utils/ipc.ts` (wrapper for Tauri IPC)
- **Static Assets:** `src/public/`

---

## IPC Communication (Frontend ↔ Backend)

- Frontend uses `src/utils/ipc.ts` to call Rust commands via `@tauri-apps/api`.
- Rust backend exposes functions in `aether_api` with the `#[tauri::command]` attribute.
- Shared data types are defined in `aether_types` for type-safe serialization between Rust and TypeScript.

**Example:**
- TypeScript calls `callRust('my_command', { arg1: value })` in `ipc.ts`.
- Rust receives the call in a function marked with `#[tauri::command]` in `aether_api`.
- Data is exchanged using types from `aether_types`.

---

## How to Build and Run

1. Install dependencies:
   ```sh
   npm install
   cargo build
   ```
2. Start development:
   ```sh
   npm run tauri dev
   ```
3. Build production app:
   ```sh
   npm run build
   npm run tauri build
   ```

---

## Contributing
- Follow Rust and Next.js best practices for each respective part of the codebase.
- Use IPC for communication between frontend and backend.
- Keep shared types in `aether_types` for consistency.

---

For more details, see the documentation in each subdirectory.
