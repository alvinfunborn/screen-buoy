# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is Screen Buoy?

A cross-platform (Windows/macOS) keyboard-driven screen navigation tool built with Tauri 2. Users press a global hotkey (default Alt+H) to display hint labels over UI elements and screen grid positions, then type hint characters to perform mouse actions (click, drag, scroll) without using a mouse.

## Build & Dev Commands

- **Dev mode:** `npm run tauri:dev` — starts Vite dev server + Tauri with hot reload
- **Build production:** `npm run build` then `cargo build --release` in `src-tauri/`
- **Frontend only:** `npm run dev` (Vite at localhost:1420)
- **Rust check:** `cd src-tauri && cargo check`
- **Rust clippy:** `cd src-tauri && cargo clippy`

No test suite exists currently.

## Architecture

### Two-process model (Tauri)
- **Frontend (React/TypeScript):** Settings UI (`src/`) and hint overlay (`overlay.html`). Settings use Ant Design + Tailwind. The overlay is a standalone HTML file (not React) that receives hints via Tauri events.
- **Backend (Rust):** All core logic in `src-tauri/src/`. Platform-specific code uses conditional compilation (`#[cfg(target_os)]`).

### Core flow
```
Global hotkey → collect UI elements → generate hints → create overlay windows (one per monitor) → emit hints to frontend → user types characters → keyboard hook filters/matches → execute mouse action
```

### Key modules (`src-tauri/src/`)
- **`lib.rs`** — Tauri app builder, tray setup, global shortcut registration, overlay window creation
- **`main.rs`** — Initialization sequence: COM init (Windows), config load, input hook, monitors, UI collection, shortcuts
- **`config/`** — TOML-based configuration loaded from `src-tauri/config.toml`. Uses `once_cell::Lazy` global. Frontend reads/writes config via Tauri commands
- **`element/`** — Background thread continuously collects UI elements via Windows UI Automation or macOS Accessibility. Two-tier caching: real-time for active window, cached for others
- **`hint/`** — `generator.rs` produces hints from character sets + grid positions. `hint.rs` stores active hints in global `HashMap`. `overlay.rs` manages per-monitor overlay windows
- **`input/`** — `hook/` installs global keyboard/mouse hooks (SetWindowsHookEx / Cocoa event taps). `keyboard.rs` is a state machine tracking typed hints, hold actions, drag state. `executor.rs` dispatches mouse actions
- **`window/`** — Enumerates visible windows, calculates occlusion/z-order. Platform-specific implementations
- **`monitor/`** — Multi-monitor enumeration and storage

### State management (Rust)
Global state uses `Lazy<Mutex<T>>` or `Lazy<RwLock<T>>` (from `once_cell`):
- `CONFIG` — application configuration
- `KEYBOARD_STATE` — input state machine
- `ACTIVE_HINTS_STORAGE` — current hints per overlay window
- `MONITORS_STORAGE` — detected monitors
- `OVERLAY_HANDLES_STORAGE` — overlay window handles

### IPC between frontend and backend
- **Commands (frontend→backend):** `get_config_for_frontend`, `save_config_for_frontend`, `get_hint_default_style`, `get_hint_types_styles`
- **Events (backend→frontend):** `show-hints`, `hide-hints`, `move-hints`, `filter-hints`, `rust-panic`

### Configuration
`src-tauri/config.toml` defines all settings: hint characters/styles/types, keybindings, mouse step sizes, keyboard mappings, system preferences, and UI automation intervals. Config search order: `./config.toml` → `./src-tauri/config.toml` → `../config.toml` (relative to exe).

## Platform-specific notes

- **Windows:** Requires COM initialization (APARTMENTTHREADED). Uses `windows` crate 0.61 for UI Automation, DWM API for overlay transparency, virtual key codes for keyboard mapping.
- **macOS:** Requires Accessibility permission (prompted on first run). Uses `core-graphics` for monitors/windows, Cocoa event taps for input hooks. Tauri built with `macos-private-api` feature.

## Frontend tech
React 18 + TypeScript + Vite 6 + Ant Design 5 + Tailwind CSS 3 + i18next (en/zh locales in `src/locales/`).
