# Copilot Instructions for voice-input-app

## Overview

This project is a cross-platform desktop application built with Tauri (Rust backend) and a React (TypeScript) frontend, bundled via Vite. It is designed for voice input, audio processing, and speech recognition, integrating both web and native capabilities.

### Architecture
- **Frontend**: React (TypeScript), Zustand for state management, Tailwind CSS for styling, Vite for bundling.
- **Backend**: Rust (Tauri), with custom audio capture and recognition logic.
- **Communication**: Frontend and backend communicate via Tauri commands (annotated with `#[tauri::command]` in Rust).
- **Directory Structure**:
  - `src/`: React app (components, hooks, stores, utils)
  - `src-tauri/`: Rust backend (audio, recognition, commands)

## Developer Workflows

### Build & Run
- **Development**: `npm run tauri dev` (runs both Vite and Tauri in dev mode)
- **Frontend only**: `npm run dev` (Vite dev server)
- **Build**: `npm run build` (TypeScript check + Vite build)
- **Tauri build**: `npm run tauri build` (builds Rust backend and bundles frontend)

### Debugging
- Rust: Use `rust-analyzer` in VS Code, or standard Rust debugging tools.
- Frontend: Standard React/Vite debugging (browser devtools, etc).
- Tauri: Errors from Rust backend are surfaced in the dev console; Vite config disables clearScreen for easier error tracing.

### Testing
- No explicit test scripts found; add tests as needed for both Rust and React code.

## Project Conventions & Patterns
- **State Management**: Zustand stores in `src/stores/`.
- **Hooks**: Custom React hooks in `src/hooks/` for audio, recognition, and Tauri command integration.
- **Component Organization**: Each major UI feature is a folder in `src/components/`.
- **Rust Modules**: Audio and recognition logic is modularized under `src-tauri/src/audio/` and `src-tauri/src/recognition/`.
- **Tauri Commands**: Rust functions exposed to JS are annotated with `#[tauri::command]`.
- **Frontend-Backend Integration**: Use `@tauri-apps/api` in React to call backend commands.
- **Styling**: Tailwind CSS, configured in `tailwind.config.js`.
- **TypeScript**: Strict mode enabled, modern module resolution.

## Integration Points & External Dependencies
- **Tauri**: Core for native shell, windowing, and backend logic.
- **@tauri-apps/api**: JS API for calling Rust commands.
- **cpal**: Rust audio I/O library.
- **Zustand**: State management in React.
- **Tailwind CSS**: Utility-first CSS framework.
- **Vite**: Fast dev server and build tool.

## Special Notes
- **Windows-specific**: The Rust lib name workaround in `Cargo.toml` is required for Windows builds.
- **Port Configuration**: Vite dev server runs on port 1420 (see `vite.config.js` and `tauri.conf.json`).
- **Frontend Output**: Built frontend is output to `dist/`, which Tauri uses as its frontend.
- **Security**: CSP is disabled by default; review before production.

## Getting Started
1. Install dependencies: `npm install` (Node.js) and `cargo install` (Rust toolchain).
2. Start development: `npm run tauri dev`.
3. Edit React code in `src/`, Rust code in `src-tauri/src/`.
4. Add new Tauri commands in Rust and expose them to JS as needed.

---

_This file is auto-generated to help AI coding agents and developers quickly understand the structure, workflows, and conventions of this codebase._
