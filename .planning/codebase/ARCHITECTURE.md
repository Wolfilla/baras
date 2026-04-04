---
generated: 2026-04-03
focus: arch
---

# Architecture

## Pattern Overview

**Overall:** Event-driven pipeline with service-oriented backend

**Key Characteristics:**
- Unidirectional data flow: log file -> parser -> signals -> service -> overlay updates
- Rust workspace with 7 crates: `types`, `core`, `overlay`, `app` (frontend), `app/src-tauri` (backend), `parse-worker`, `validate`
- Tauri bridges native backend to Dioxus WASM frontend
- Overlays run in dedicated OS threads with platform-native windows (not web-based)
- Subprocess architecture for historical parsing to avoid memory fragmentation

## Crate Dependency Graph

```
baras-types          (no deps - shared serializable types)
    ^
    |
baras-core           (depends on: baras-types)
    ^
    |
baras-overlay        (depends on: baras-core, baras-types)
    ^
    |
app (src-tauri)      (depends on: baras-core, baras-types, baras-overlay, tauri)

app-ui (frontend)    (depends on: baras-types, dioxus)

baras-parse-worker   (depends on: baras-core)

validate             (depends on: baras-core)  [CLI tool, not linked into app]
```

**Key rule:** `baras-types` is the only crate shared between the WASM frontend and native backend. All types that cross the Tauri IPC boundary live here.

## Layers

**Types Layer (`baras-types`):**
- Purpose: Shared serializable types for IPC between frontend and backend
- Location: `types/src/lib.rs`, `types/src/formatting.rs`
- Contains: Query result types (`DataTab`, `AbilityBreakdown`, `EntityBreakdown`, `TimeSeriesPoint`, `CombatLogRow`, etc.), overlay config types, formatting helpers
- Depends on: `serde` only
- Used by: `baras-core`, `app-ui` (WASM frontend), `app` (Tauri backend)

**Core Layer (`baras-core`):**
- Purpose: All business logic -- parsing, signal processing, encounter tracking, querying
- Location: `core/src/`
- Contains: Combat log parser, event processor, encounter state machine, effect tracker, timer manager, DataFusion queries, TOML definition loading
- Depends on: `baras-types`, DataFusion, Arrow, Parquet, chrono, memchr, encoding_rs
- Used by: `app` (backend), `baras-overlay`, `baras-parse-worker`, `validate`

**Overlay Layer (`baras-overlay`):**
- Purpose: Custom rendering engine for overlay windows (not web-based)
- Location: `overlay/src/`
- Contains: Platform backends (Wayland, X11, Windows, macOS), renderer (tiny-skia + cosmic-text), overlay implementations, reusable widgets
- Depends on: `baras-core`, `baras-types`, tiny-skia, cosmic-text, platform-specific windowing libs
- Used by: `app` (backend spawns overlay threads)

**Backend Layer (`app/src-tauri`):**
- Purpose: Tauri application -- coordinates service, state, overlays, and frontend commands
- Location: `app/src-tauri/src/`
- Contains: `CombatService` (background task), `SharedState`, `OverlayManager`, Tauri commands, overlay router, audio, hotkeys
- Depends on: `baras-core`, `baras-overlay`, `baras-types`, tauri
- Used by: Entry point; manages all runtime state

**Frontend Layer (`app/src` aka `app-ui`):**
- Purpose: Dioxus WASM UI for analytics, settings, encounter editing
- Location: `app/src/`
- Contains: Dioxus components, Tauri IPC API layer, frontend types
- Depends on: `baras-types`, dioxus (web target)
- Used by: Rendered in Tauri webview

## Data Flow

**Live Parsing Pipeline:**

1. `DirectoryWatcher` (`core/src/context/watcher.rs`) detects file changes via `notify` crate
2. Watcher sends `FileDetected`/`FileModified` to `CombatService` via `ServiceCommand`
3. `CombatService::run()` (`app/src-tauri/src/service/mod.rs`) reads new bytes via `Reader` (`core/src/combat_log/reader.rs`)
4. `LogParser::parse_line()` (`core/src/combat_log/parser.rs`) converts raw text to `CombatEvent`
5. `EventProcessor::process_event()` (`core/src/signal_processor/processor.rs`) produces `Vec<GameSignal>` + accumulates events
6. `CombatSignalHandler` (sync context in service) dispatches signals to:
   - `TimerManager` (`core/src/timers/manager.rs`) -- timer state machines
   - `EffectTracker` (`core/src/effects/tracker.rs`) -- raid frame effects
   - Encounter state (phase transitions, counters, boss detection)
7. Service sends `ServiceCommand` via `cmd_tx.try_send()` for async operations
8. Service computes overlay data and sends `OverlayUpdate` variants via `overlay_tx` channel
9. `spawn_overlay_router()` (`app/src-tauri/src/router.rs`) receives `OverlayUpdate` and routes to overlay threads via per-overlay `mpsc` channels
10. Each overlay thread receives `OverlayCommand::UpdateData(OverlayData::*)`, re-renders, and presents to screen

**Historical Parsing Pipeline:**

1. User selects file in frontend -> `OpenHistoricalFile` command
2. `CombatService` spawns `baras-parse-worker` subprocess (`parse-worker/src/main.rs`)
3. Parse-worker `mmap`s the file, uses `rayon::par_iter()` for parallel line parsing
4. `FastEncounterWriter` writes directly to Arrow builders in 50K-event batches
5. Writes LZ4-compressed Parquet files per encounter to `~/.config/baras/data/{session_id}/`
6. Outputs JSON summary to stdout (encounter list, byte position, player info)
7. Main app reads JSON output, loads encounter summaries, and continues from last byte for live tailing
8. If subprocess fails, falls back to `fallback_streaming_parse()` (sequential, in-process)

**Query Pipeline (Data Explorer):**

1. Frontend sends query command (e.g., `query_breakdown`) via Tauri invoke
2. `ServiceHandle` method on backend reads `SharedState` to find correct data source
3. `QueryContext` (`core/src/query/mod.rs`) manages DataFusion `SessionContext`:
   - Same file: reuses existing context (fast path)
   - New file: creates fresh context, clears caches
   - Live data: always re-registers MemTable from Arrow buffers
4. SQL query executed via DataFusion, results returned as typed structs
5. Frontend receives results as JSON via Tauri IPC

**State Management:**

- `SharedState` (`app/src-tauri/src/state/mod.rs`): Central state container, uses `std::sync::Mutex` for sync Tauri command contexts and `tokio::sync::RwLock` for async data
- `AtomicBool` flags for lock-free reads: `in_combat`, `watching`, `is_live_tailing`, overlay active flags, `game_running`
- `AutoHideState`: Centralized overlay suppression with multiple independent flags (`conversation_active`, `not_live_active`, `session_not_live`, `game_starting`); overlays hidden when ANY flag is true
- `SessionCache` (`core/src/state/cache.rs`): Per-session combat state including encounters, player info, boss definitions, timer context
- `AppConfig` (`core/src/context/config.rs`): Persisted configuration via `confy`

## Key Abstractions

**`CombatEvent`:**
- Purpose: Parsed representation of a single combat log line
- Location: `core/src/combat_log/combat_event.rs`
- Contains: timestamp, source/target entities, ability, effect, damage/heal values
- Pattern: Flat struct with interned strings (`IStr` via `lasso` crate)

**`GameSignal`:**
- Purpose: High-level events derived from raw combat events (21 variants)
- Location: `core/src/signal_processor/signal.rs`
- Contains: `CombatStarted`, `CombatEnded`, `EntityDeath`, `EffectApplied`, `EffectRemoved`, `AbilityActivated`, `DamageTaken`, `HealingDone`, `AreaEntered`, `BossEncounterDetected`, `BossHpChanged`, `PhaseChanged`, `CounterChanged`, etc.
- Pattern: Enum with per-variant data, all variants carry a `timestamp`

**`SignalHandler` trait:**
- Purpose: Interface for systems that react to game signals
- Location: `core/src/signal_processor/handler.rs`
- Pattern: `handle_signal(&mut self, signal: &GameSignal, encounter: Option<&CombatEncounter>)` + optional `on_encounter_start`/`on_encounter_end` hooks
- Implementors: `TimerManager`, `EffectTracker`, `ChallengeTracker`

**`Overlay` trait:**
- Purpose: Interface for overlay implementations
- Location: `overlay/src/overlays/mod.rs`
- Pattern: Each overlay (MetricOverlay, RaidOverlay, TimerOverlay, etc.) implements rendering and data update logic
- Implementations: 15 overlay types in `overlay/src/overlays/`

**`ServiceCommand` enum:**
- Purpose: Messages from Tauri commands to the background `CombatService`
- Location: `app/src-tauri/src/service/mod.rs` (line ~100)
- Contains: ~25 variants covering tailing, directory, definitions, overlay, timer operations
- Pattern: Sent via `mpsc::Sender<ServiceCommand>`, received in `CombatService::run()` select loop

**`OverlayUpdate` enum:**
- Purpose: Messages from service to overlay router for display updates
- Location: `app/src-tauri/src/service/mod.rs` (defined in service, used by router)
- Contains: `DataUpdated`, `EffectsUpdated`, `BossHealthUpdated`, `TimersAUpdated`, `AlertsFired`, `CombatStarted`, `CombatEnded`, `ConversationStarted`, `NotLiveStateChanged`, etc.
- Pattern: Sent via `mpsc::channel::<OverlayUpdate>(256)`, consumed by `spawn_overlay_router()`

**`OverlayCommand` enum:**
- Purpose: Per-overlay thread commands (data updates, config changes, shutdown)
- Location: `app/src-tauri/src/overlay/state.rs`
- Pattern: Each running overlay has its own `mpsc::Sender<OverlayCommand>` stored in `OverlayState`

**`CombatEncounter`:**
- Purpose: Tracks state of a single combat encounter (in-progress or completed)
- Location: `core/src/encounter/combat.rs`
- Contains: Player metrics, entity info, boss state, phase transitions, timing, encounter state machine
- States: `NotStarted` -> `InCombat` -> `PostCombat`

**`BossEncounterDefinition`:**
- Purpose: TOML-defined boss encounter with triggers, phases, counters, timers, notes
- Location: `core/src/dsl/definition.rs`, definitions at `core/definitions/encounters/`
- Pattern: Loaded from TOML files, supports bundled + user custom overlays via `_custom.toml`

## Entry Points

**Tauri Backend (`app/src-tauri/src/main.rs`):**
- Calls `app_lib::run()` which sets up Tauri with plugins, spawns `CombatService`, overlay router, hotkeys, tray
- Registers ~90 Tauri commands via `invoke_handler`

**Frontend WASM (`app/src/main.rs`):**
- Launches Dioxus app with `launch(App)`
- `App` component in `app/src/app.rs` is the root

**Parse Worker (`parse-worker/src/main.rs`):**
- CLI binary: `baras-parse-worker <file_path> <session_id> <output_dir> [definitions_dir]`
- Spawned as subprocess by `CombatService`

**Validate CLI (`validate/src/main.rs`):**
- CLI tool for replaying combat logs against boss definitions
- Used for development/testing, not shipped in production

**Overlay standalone (`overlay/src/main.rs`):**
- Development entry point for testing overlay rendering without full app
- 1742 lines, contains test overlay scenarios

## Error Handling

**Strategy:** `thiserror` for domain-specific error types with graceful degradation

**Patterns:**
- `core/src/combat_log/error.rs`: `ParseError` (line format, timestamp, entity) + `ReaderError` (file, mmap, encoding)
- `core/src/query/error.rs`: `QueryError` wrapping DataFusion, Arrow, column, SQL errors
- `core/src/storage/error.rs`: `StorageError` for parquet writing
- `core/src/dsl/error.rs`: Definition loading errors
- `core/src/timers/error.rs`: Timer definition errors
- Parse-worker failure: main app falls back to `fallback_streaming_parse()` (sequential in-process)
- Missing definitions: log warning, continue without features
- Overlay spawn failure: logged, other overlays continue

## Cross-Cutting Concerns

**Logging:** `tracing` crate throughout all crates. Backend initializes `tracing-subscriber` with `rolling-file` appender in `app/src-tauri/src/logging.rs`. Frontend uses `dioxus-logger`.

**String Interning:** `lasso` crate with multi-threaded `ThreadedRodeo`. `IStr` type alias used throughout `core` for entity names, ability names, effect names. Resolved via `resolve()` function. Prevents repeated allocation of frequently-seen strings.

**Validation:** `validate` crate replays logs against boss definitions with checkpoint verification. Input validation in Tauri commands returns `Result<T, String>` for frontend display.

**Audio:** Separate `AudioService` with `rodio` for sound playback. Communicated via `AudioSender` channel from signal handlers. TTS support on non-Linux platforms via `tts` crate; Linux uses `ashpd` portal.

**Configuration:** `confy` crate for `AppConfig` persistence. Overlay profiles support save/load/rename/delete with per-role defaults. Config stored at OS-standard config directory.

---

*Architecture analysis: 2026-04-03*
