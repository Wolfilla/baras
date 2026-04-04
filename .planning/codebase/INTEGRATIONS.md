---
generated: 2026-04-03
focus: tech
---

# External Integrations

**Analysis Date:** 2026-04-03

## APIs & External Services

**Parsely.io (Log Upload):**
- Purpose: Upload combat log encounters for external analysis/sharing
- Endpoint: `https://parsely.io/api/upload2` (POST, multipart form)
- SDK/Client: `reqwest` 0.12 HTTP client
- Auth: Username/password sent as form fields (stored in app config via `confy`)
- Implementation: `app/src-tauri/src/commands/parsely.rs`
- Features: Full file upload, encounter-specific line range extraction with trailing window, gzip compression, guild log support
- Response: XML parsed manually via string matching and `quick-xml`

**Tauri Auto-Updater:**
- Purpose: In-app update checking, download, and installation
- Endpoint: `https://raw.githubusercontent.com/baras-app/baras/master/latest.json`
- SDK/Client: `tauri-plugin-updater` 2
- Auth: Public key embedded in `app/src-tauri/tauri.conf.json`
- Implementation: `app/src-tauri/src/updater.rs`
- Flow: Startup check (3s delay) -> cache Update object -> user-triggered download/install -> app restart

**ECharts (Frontend Charting):**
- Purpose: Data visualization charts in the Data Explorer
- Loaded via CDN: `https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js`
- Config: `app/Dioxus.toml` (web.resource.dev and web.resource.release)
- Used in: `app/src/components/data_explorer.rs`, `app/src/components/charts_panel.rs`
- Integration: Called from WASM via `js-sys`/`web-sys` bindings

## Data Storage

**Apache Parquet (Historical Encounter Data):**
- Format: Columnar Parquet files with LZ4/Snappy/Zstd compression
- Client: Arrow 57 + Parquet 57 crates
- Query: DataFusion 51 SQL engine (`core/src/query/mod.rs`)
- Writer: `FastEncounterWriter` in parse-worker writes 50K event batches
- Location: User data directory, one parquet file per combat log

**TOML Definition Files:**
- Purpose: Game data definitions (effects, encounters, sounds, timers)
- Parser: `toml` 0.9
- Bundled: `core/definitions/effects/` (hots.toml, dots.toml, dcds.toml, custom.toml)
- Bundled: `core/definitions/encounters/` (operations/, flashpoints/, other/)
- Bundled: `core/definitions/sounds/` (MP3/WAV alert sounds, TTS voice packs)
- User custom: `~/.config/baras/definitions/encounters/` with `_custom.toml` suffix
- JSON: `core/definitions/absorbs.json` (absorb effect data)

**Application Config:**
- Manager: `confy` 2.0.0 with TOML backend
- Location: `~/.config/baras/` (via `dirs` crate)
- Implementation: `core/src/context/config.rs`

**File Storage:**
- Local filesystem only, no cloud storage
- Combat logs read from SWTOR game directory (user-configured)
- Memory-mapped file reading via `memmap2` for large log files

**Caching:**
- `CachedText` LRU cache (512 entries) for overlay text layout reuse
- `QueryContext` reuses DataFusion `SessionContext` for same-file queries
- No external caching service

## File Format Integrations

**SWTOR Combat Logs (Input):**
- Format: Windows-1252 encoded text, bracket-delimited fields
- Parser: Custom high-performance parser in `core/src/combat_log/parser.rs`
- Techniques: `memchr` SIMD scanning, fixed-size stack arrays, manual digit extraction
- Encoding: `encoding_rs` for Windows-1252 -> UTF-8 conversion
- Watcher: `notify` 8.2 crate for filesystem change detection (`core/src/context/watcher.rs`)

**Parquet (Output):**
- Written by parse-worker subprocess via Arrow builders
- Read by DataFusion SQL queries in the main app
- Compression: LZ4 (parse-worker), Snappy, Zstd supported

**ZIP Archives:**
- Purpose: Icon pack extraction
- Crate: `zip` 2 with deflate support
- Used in: `overlay/Cargo.toml`, `app/src-tauri/Cargo.toml`

**Markdown:**
- Purpose: Boss encounter notes display
- Parser: `pulldown-cmark` 0.12 in backend, custom inline parser in overlay
- Implementation: `app/src-tauri/Cargo.toml`, `overlay/src/overlays/notes.rs`

**XML:**
- Purpose: Parsing Parsely API responses, StarParse timer import
- Crate: `quick-xml` 0.37
- Implementation: `app/src-tauri/src/commands/parsely.rs`

## Authentication & Identity

**Auth Provider:** None (no user accounts)
- Parsely.io credentials stored locally in app config
- No OAuth, no tokens, no session management

## Audio

**Audio Playback:**
- Crate: `rodio` 0.19 (wav, vorbis, mp3 decoders)
- Purpose: Timer/alert sound effects
- Implementation: `app/src-tauri/src/audio/service.rs`
- Sound files: `core/definitions/sounds/` (bundled MP3/WAV)

**Text-to-Speech:**
- Crate: `tts` 0.26 (Windows and macOS only, not available on Linux)
- Purpose: Spoken alert notifications
- Implementation: `app/src-tauri/src/commands/service.rs`

## Platform Overlay Backends

**Wayland (Linux primary):**
- Protocol: wlr-layer-shell via `wayland-protocols-wlr` 0.3
- Client: `wayland-client` 0.31
- Shared memory: `memmap2` 0.9 + `rustix` 1.0 for SHM buffers
- Location: `overlay/src/platform/` (wayland module)

**X11 (Linux fallback):**
- Client: `x11rb` 0.13 with RandR, Shape, SHM extensions
- Purpose: Transparent overlay windows with input passthrough (XShape)
- Location: `overlay/src/platform/` (x11 module)

**Windows:**
- API: Win32 via `windows` 0.58 crate
- Features: GDI, WindowsAndMessaging, KeyboardAndMouse, LibraryLoader
- Location: `overlay/src/platform/` (windows module)

**macOS:**
- APIs: `core-graphics` 0.24 for CGContext, `objc2` 0.6 ecosystem for AppKit
- GCD: `dispatch` 0.2 for main thread operations
- Features: NSWindow, NSView, NSScreen, NSEvent, NSApplication
- Location: `overlay/src/platform/` (macos module)

## Linux Desktop Integration

**XDG Desktop Portal:**
- Crate: `ashpd` 0.12 (Linux only)
- Purpose: Open URLs in sandboxed/immutable distro environments (Flatpak, Bazzite, etc.)
- Implementation: `app/src-tauri/src/commands/url.rs`
- Fallback: `tauri-plugin-opener` (uses xdg-open)

## Monitoring & Observability

**Structured Logging:**
- Framework: `tracing` 0.1 + `tracing-subscriber` 0.3 with env-filter
- Output: Rolling log files via `tracing-appender` 0.2 + `rolling-file` 0.2
- No external error tracking or APM service

**Process Monitoring:**
- Purpose: Auto-hide overlays when SWTOR game is not running
- Implementation: `AtomicBool` flags with `Ordering::SeqCst` for lock-free reads
- No external monitoring service

## CI/CD & Deployment

**Hosting:**
- GitHub Releases (binary distribution)
- Repo: `baras-app/baras` on GitHub

**CI Pipeline:**
- GitHub Actions (`.github/workflows/build.yml`, `.github/workflows/release.yml`)
- Build workflow: manual dispatch, per-platform toggle (Linux/Windows/macOS)
- Release workflow: manual dispatch, builds all platforms, creates GitHub Release, updates `latest.json`
- Artifacts: AppImage + .deb (Linux), NSIS .exe (Windows), DMG + .app (macOS)

**Signing:**
- Tauri update signing via `TAURI_SIGNING_PRIVATE_KEY` secret
- Public key in `app/src-tauri/tauri.conf.json` for client verification

## Webhooks & Callbacks

**Incoming:** None
**Outgoing:** None (only outbound HTTP to Parsely.io and GitHub for updates)

## Sidecar Process

**parse-worker:**
- Binary: `baras-parse-worker` (bundled as Tauri external binary)
- Purpose: Parallel historical log parsing in subprocess to prevent memory fragmentation
- Communication: JSON over stdout
- Platform binaries: `baras-parse-worker-{target-triple}` in `app/src-tauri/binaries/`
- Config: `app/src-tauri/tauri.conf.json` `bundle.externalBin`

---

*Integration audit: 2026-04-03*
