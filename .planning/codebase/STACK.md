---
generated: 2026-04-03
focus: tech
---

# Technology Stack

**Analysis Date:** 2026-04-03

## Languages

**Primary:**
- Rust (edition 2024) - All backend, core logic, overlay rendering, and frontend
- WASM (wasm32-unknown-unknown) - Frontend compilation target for Dioxus UI

**Secondary:**
- JavaScript - ECharts charting library loaded via CDN in the frontend
- Python - Conversion/migration scripts in `scripts/`
- TOML - Definition files for effects, encounters, sounds

## Runtime

**Environment:**
- Rust stable (currently rustc 1.94.0)
- Tauri 2 runtime (WebKit2GTK on Linux, WebView2 on Windows, WKWebView on macOS)
- Tokio async runtime (full features in core, sync-only in frontend WASM)

**Package Manager:**
- Cargo (workspace-based)
- Lockfile: `Cargo.lock` present

**WASM Target:**
- `wasm32-unknown-unknown` for the Dioxus frontend (`app/Cargo.toml`)

## Frameworks

**Core:**
- Tauri 2 - Desktop application framework (`app/src-tauri/Cargo.toml`)
- Dioxus 0.7.2 - Reactive UI framework targeting web/WASM (`app/Cargo.toml`)
- DataFusion 51 - SQL query engine for analytics (`core/Cargo.toml`)
- Arrow 57 / Parquet 57 - Columnar data format and storage (`core/Cargo.toml`)

**Rendering:**
- tiny-skia 0.11 - Software 2D rendering for overlays (`overlay/Cargo.toml`)
- cosmic-text 0.16 - Text shaping and layout (`overlay/Cargo.toml`)
- fontdb 0.23 - Font database (`overlay/Cargo.toml`)

**Build/Dev:**
- Dioxus CLI (`dx serve`, `dx bundle`) - Frontend dev server and bundling
- Tauri CLI (`cargo tauri`) - Application building and packaging
- cargo-binstall - Binary installation of CLI tools
- phf_codegen 0.13.1 - Compile-time perfect hash generation (`core/build.rs`)

## Key Dependencies

**Critical (core logic):**
- `memchr` 2.7.6 - SIMD-accelerated byte scanning for log parsing (`core/Cargo.toml`)
- `encoding_rs` 0.8 - Windows-1252 text decoding for SWTOR logs (`core/Cargo.toml`)
- `lasso` 0.7.3 - String interner with multi-threaded support (`core/Cargo.toml`)
- `rayon` 1.11.0 - Parallel iteration for parse-worker (`core/Cargo.toml`, `parse-worker/Cargo.toml`)
- `memmap2` 0.9.9 - Memory-mapped file reading (`core/Cargo.toml`, `parse-worker/Cargo.toml`)
- `hashbrown` 0.16.1 - Fast hash map implementation (`core/Cargo.toml`)
- `phf` 0.13.1 - Compile-time perfect hash maps (`core/Cargo.toml`)
- `notify` 8.2 - Filesystem watching for log file changes (`core/Cargo.toml`)
- `confy` 2.0.0 - Configuration file management via TOML (`core/Cargo.toml`)

**Infrastructure:**
- `tokio` 1.48.0 - Async runtime (full features in backend, sync-only in WASM) (`core/Cargo.toml`)
- `serde` 1.0 / `serde_json` 1.0 - Serialization throughout all crates
- `chrono` 0.4.42 - Date/time handling (`core/Cargo.toml`, `app/src-tauri/Cargo.toml`)
- `toml` 0.9 - TOML parsing for definitions and config (`core/Cargo.toml`)
- `thiserror` 2 - Error type derivation (`core/Cargo.toml`)
- `tracing` 0.1 / `tracing-subscriber` 0.3 / `tracing-appender` 0.2 - Structured logging (workspace deps)
- `rolling-file` 0.2 - Rolling log file output (workspace dep)
- `reqwest` 0.12 - HTTP client for Parsely uploads (`app/src-tauri/Cargo.toml`)
- `flate2` 1.1 - Gzip compression for uploads (`app/src-tauri/Cargo.toml`)
- `rodio` 0.19 - Audio playback for alerts (wav, vorbis, mp3) (`app/src-tauri/Cargo.toml`)
- `pulldown-cmark` 0.12 - Markdown rendering for boss notes (`app/src-tauri/Cargo.toml`)
- `quick-xml` 0.37 - XML parsing for Parsely API responses (`app/src-tauri/Cargo.toml`)
- `zip` 2 - ZIP extraction for icon packs (`app/src-tauri/Cargo.toml`, `overlay/Cargo.toml`)
- `clap` 4 - CLI argument parsing for validate tool (`validate/Cargo.toml`)

**Tauri Plugins:**
- `tauri-plugin-opener` 2 - OS default app opening
- `tauri-plugin-dialog` 2 - Native file/folder dialogs
- `tauri-plugin-global-shortcut` 2 - System-wide hotkeys
- `tauri-plugin-updater` 2 - Auto-update support
- `tauri-plugin-process` 2 - Process management
- `tauri-plugin-single-instance` 2 - Single instance enforcement
- `tauri-plugin-window-state` 2 - Window position persistence

**Platform-Specific:**
- Linux: `wayland-client` 0.31, `wayland-protocols` 0.32, `wayland-protocols-wlr` 0.3, `x11rb` 0.13, `ashpd` 0.12 (XDG portals), `rustix` 1.0
- Windows: `windows` 0.58 (Win32 API)
- macOS: `core-graphics` 0.24, `objc2` 0.6, `objc2-foundation` 0.3, `objc2-app-kit` 0.3, `dispatch` 0.2

**Frontend (WASM):**
- `dioxus` 0.7.2 - UI framework (`app/Cargo.toml`)
- `wasm-bindgen` 0.2 / `wasm-bindgen-futures` 0.4 - JS interop
- `web-sys` 0.3 / `js-sys` 0.3 - Browser API bindings
- `serde-wasm-bindgen` 0.6.5 - Serde for WASM bridge
- `gloo-timers` 0.3 - Timer utilities for WASM
- `getrandom` 0.3 (wasm_js) - Random number generation in WASM

**Non-Linux only:**
- `tts` 0.26 - Text-to-speech for alerts on Windows/macOS (`app/src-tauri/Cargo.toml`)

## Configuration

**Application Config:**
- Managed via `confy` crate writing TOML to `~/.config/baras/`
- User encounter definitions: `~/.config/baras/definitions/encounters/`
- Bundled definitions in `core/definitions/` (effects, encounters, sounds)

**Build Config:**
- `Cargo.toml` (workspace root) - Workspace members, shared deps, release profile
- `app/src-tauri/tauri.conf.json` - Tauri app config (window, bundle, plugins, updater)
- `app/Dioxus.toml` - Dioxus frontend config (asset dir, dev server, ECharts CDN)

**Release Profile** (`Cargo.toml`):
- LTO: thin (cross-crate optimization)
- codegen-units: 1 (maximum optimization)
- panic: abort (no unwinding overhead)

## Platform Requirements

**Development:**
- Rust stable toolchain (edition 2024)
- `wasm32-unknown-unknown` target for frontend
- Dioxus CLI (`dx`) and Tauri CLI (`cargo tauri`)
- Linux: libwebkit2gtk-4.1-dev, libappindicator3-dev, librsvg2-dev, libxdo-dev, libasound2-dev
- Frontend dev server runs on `http://localhost:1420`

**Production:**
- Linux: AppImage or .deb (ubuntu-24.04 base), requires WebKit2GTK 4.1
- Windows: NSIS installer (.exe)
- macOS: DMG or .app bundle (minimum macOS 10.13, aarch64)
- `baras-parse-worker` sidecar binary bundled alongside main app

**CI/CD:**
- GitHub Actions with `workflow_dispatch` triggers
- Runners: ubuntu-24.04 (Linux), windows-latest (Windows), macos-14/15 (macOS)
- Tauri signing keys via GitHub Secrets
- Auto-update manifest (`latest.json`) committed to master after release

---

*Stack analysis: 2026-04-03*
