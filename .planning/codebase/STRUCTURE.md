---
generated: 2026-04-03
focus: arch
---

# Codebase Structure

## Directory Layout

```
baras/
├── app/                        # Dioxus WASM frontend + Tauri backend
│   ├── assets/                 # Static assets (CSS, fonts, icons)
│   ├── src/                    # Dioxus WASM frontend source
│   ├── src-tauri/              # Tauri native backend source
│   ├── dist/                   # Built frontend output (committed)
│   └── Cargo.toml              # Frontend crate (app-ui)
├── core/                       # Business logic crate (baras-core)
│   ├── src/                    # Rust source modules
│   ├── data/                   # Generated data (build script output)
│   └── definitions/            # TOML definition files
├── overlay/                    # Custom rendering engine (baras-overlay)
│   └── src/                    # Overlay implementations + platform backends
├── types/                      # Shared types crate (baras-types)
│   └── src/                    # Serializable types for IPC
├── parse-worker/               # Subprocess binary (baras-parse-worker)
│   └── src/                    # Parallel historical log parser
├── validate/                   # CLI validation tool
│   └── src/                    # Log replay + checkpoint verification
├── test-log-files/             # Combat logs for testing (gitignored)
├── docs/                       # Documentation
│   └── wiki/                   # Wiki pages
├── scripts/                    # Build and conversion scripts
├── changelogs/                 # Release changelog files
├── etc/                        # Miscellaneous config files
├── icons/                      # Application icons
├── Cargo.toml                  # Workspace root
├── Cargo.lock                  # Dependency lockfile
└── justfile                    # Task runner recipes
```

## Directory Purposes

**`app/src/` (Frontend - `app-ui` crate):**
- Purpose: Dioxus WASM UI rendered in Tauri webview
- Contains: Components, Tauri API wrappers, frontend types, CSS
- Key files:
  - `main.rs`: Entry point, launches Dioxus app
  - `app.rs`: Root `App` component (2729 lines) -- tab navigation, overlay toggles, session state, event listeners
  - `api.rs`: Tauri `invoke()` / `try_invoke()` wrappers for all backend commands (1642 lines)
  - `types.rs`: Frontend-specific types mirroring backend with `#[serde(rename_all = "camelCase")]` (1156 lines)
  - `utils.rs`: Frontend utility functions

**`app/src/components/`:**
- Purpose: Dioxus UI components
- Key files:
  - `data_explorer.rs`: Full data explorer with 9 views (3591 lines)
  - `settings_panel.rs`: Overlay and app settings UI (2821 lines)
  - `effect_editor.rs`: Effect definition CRUD editor (2251 lines)
  - `charts_panel.rs`: Time series charts with DPS/HPS/DTPS (1379 lines)
  - `combat_log.rs`: Combat log viewer with search/filter (1307 lines)
  - `rotation_view.rs`: Rotation analysis display
  - `phase_timeline.rs`: Phase timeline visualization
  - `toast.rs`: Toast notification system
  - `parsely_upload_modal.rs`: Parsely.io upload integration
  - `hotkey_input.rs`: Global hotkey configuration
  - `ability_icon.rs`: Ability icon display
  - `class_icons.rs`: Class/role icon helpers
  - `encounter_types.rs`: Encounter type display helpers
  - `encounter_editor/`: Encounter editor subcomponents (directory)
  - `mod.rs`: Re-exports all components

**`app/src-tauri/src/` (Backend - `app` crate):**
- Purpose: Tauri native backend -- service orchestration, state, commands
- Key files:
  - `lib.rs`: `run()` function -- Tauri builder setup, plugin registration, service/overlay/hotkey spawning
  - `main.rs`: Calls `app_lib::run()`
  - `router.rs`: `spawn_overlay_router()` -- routes `OverlayUpdate` to overlay threads (637 lines)
  - `hotkeys.rs`: Global shortcut registration
  - `logging.rs`: Tracing subscriber + rolling file appender setup
  - `tray.rs`: System tray setup
  - `updater.rs`: Auto-update check via `tauri-plugin-updater`

**`app/src-tauri/src/commands/`:**
- Purpose: All Tauri-invokable command functions
- Key files:
  - `mod.rs`: Re-exports all command modules
  - `overlay.rs`: Overlay show/hide/toggle/refresh commands
  - `service.rs`: Tailing, config, session, profiles, directory, audio commands
  - `query.rs`: Data explorer query commands (breakdown, time series, combat log, rotation, etc.)
  - `encounters.rs`: Encounter editor CRUD commands (1791 lines)
  - `effects.rs`: Effect definition CRUD + import/export commands
  - `parsely.rs`: Parsely.io upload commands
  - `starparse.rs`: StarParse timer import
  - `url.rs`: URL opening with portal support

**`app/src-tauri/src/service/`:**
- Purpose: Background combat service -- main event loop, signal handling, state updates
- Key files:
  - `mod.rs`: `CombatService::run()` main loop, `ServiceCommand` enum, `OverlayUpdate` enum, `CombatSignalHandler`, all service logic (3787 lines)
  - `handler.rs`: `ServiceHandle` -- async methods for Tauri commands to interact with service (1580 lines)
  - `directory.rs`: Directory watcher integration, area index management
  - `process_monitor.rs`: Game process detection (checks if SWTOR is running)

**`app/src-tauri/src/state/`:**
- Purpose: Shared application state
- Key files:
  - `mod.rs`: `SharedState` struct (central state), `AutoHideState` (overlay suppression flags)
  - `raid_registry.rs`: `RaidSlotRegistry` -- persistent player-to-slot assignments for raid frames

**`app/src-tauri/src/overlay/`:**
- Purpose: Overlay lifecycle management (spawn, configure, destroy)
- Key files:
  - `mod.rs`: `SharedOverlayState` type alias, appearance helpers
  - `types.rs`: `MetricType` and `OverlayType` enums
  - `state.rs`: `OverlayState`, `OverlayHandle`, `OverlayCommand` -- runtime overlay tracking
  - `spawn.rs`: Factory functions for creating each overlay type on dedicated threads (1088 lines)
  - `manager.rs`: `OverlayManager` -- high-level show/hide/toggle/refresh operations (1138 lines)
  - `metrics.rs`: Metric entry creation helpers

**`app/src-tauri/src/audio/`:**
- Purpose: Audio playback service (sound effects, TTS)
- Contains: `AudioService`, `AudioEvent`, `AudioSender`

**`core/src/` (Core - `baras-core` crate):**
- Purpose: All business logic independent of UI framework

**`core/src/combat_log/`:**
- Purpose: Combat log parsing
- Key files:
  - `combat_event.rs`: `CombatEvent` struct, `EntityType` enum
  - `parser.rs` (or `parser/`): `LogParser` -- SIMD-accelerated line parser using `memchr`
  - `reader.rs`: `Reader` -- file reading with `encoding_rs` Windows-1252 decoding, streaming + mmap modes
  - `error.rs`: `ParseError`, `ReaderError`
  - `mod.rs`: Re-exports `CombatEvent`, `LogParser`, `Reader`

**`core/src/signal_processor/`:**
- Purpose: Event processing pipeline -- converts `CombatEvent` to `Vec<GameSignal>`
- Key files:
  - `processor.rs`: `EventProcessor` -- orchestrates signal generation, combat lifecycle (1625 lines)
  - `signal.rs`: `GameSignal` enum (21 variants)
  - `handler.rs`: `SignalHandler` trait
  - `combat_state.rs`: Combat start/end detection, entity tracking
  - `phase.rs`: Boss phase transition detection
  - `counter.rs`: Damage/ability counters per phase
  - `trigger_eval.rs`: Trigger condition evaluation
  - `challenge.rs`: Challenge metric tracking
  - `processor_tests.rs`: Unit tests (1826 lines)

**`core/src/encounter/`:**
- Purpose: Encounter state machine and metrics
- Key files:
  - `combat.rs`: `CombatEncounter`, `ActiveBoss` -- encounter lifecycle (1488 lines)
  - `metrics.rs`: `PlayerMetrics` -- per-player DPS/HPS/DTPS accumulation
  - `summary.rs`: `EncounterSummary`, `EncounterHistory` -- post-combat summaries
  - `entity_info.rs`: Entity tracking (players, NPCs, companions)
  - `effect_instance.rs`: Individual effect instance tracking
  - `shielding.rs`: Shield absorption tracking
  - `challenge.rs`: `ChallengeTracker` -- per-phase challenge metrics

**`core/src/query/`:**
- Purpose: DataFusion SQL queries over Arrow/Parquet data
- Key files:
  - `mod.rs`: `QueryContext` -- manages DataFusion `SessionContext` with smart caching
  - `breakdown.rs`: Ability/entity breakdown queries (DPS tables)
  - `overview.rs`: Raid overview queries
  - `time_series.rs`: DPS/HPS over time queries
  - `combat_log.rs`: Raw combat log viewer queries with pagination
  - `rotation.rs`: Rotation analysis queries
  - `effects.rs`: Effect uptime/window queries
  - `timeline.rs`: Encounter timeline queries
  - `usage.rs`: Ability usage queries
  - `column_helpers.rs`: Shared SQL column helpers
  - `error.rs`: `QueryError`

**`core/src/dsl/`:**
- Purpose: Boss encounter definition DSL (TOML-based)
- Key files:
  - `definition.rs`: `BossEncounterDefinition`, `PhaseDefinition`, `CounterDefinition`
  - `loader.rs`: TOML file loading with bundled + custom merge
  - `phase.rs`: Phase trigger types
  - `counter.rs`: Counter condition types
  - `condition.rs`: Trigger conditions
  - `audio.rs`: Audio alert configuration
  - `challenge.rs`: Challenge definition types
  - `entity_filter.rs`: Entity matching filters
  - `error.rs`: Definition errors
  - `triggers/`: Trigger type definitions and matchers

**`core/src/effects/`:**
- Purpose: Effect tracking (buffs/debuffs for raid frames)
- Key files:
  - `tracker.rs`: `EffectTracker` -- implements `SignalHandler`, tracks active effects (2337 lines)
  - `definition.rs`: `EffectDefinition`, `DefinitionSet` -- TOML effect definitions
  - `active.rs`: `ActiveEffect` -- runtime effect state

**`core/src/timers/`:**
- Purpose: Timer/alert system for boss encounters
- Key files:
  - `manager.rs`: `TimerManager` -- implements `SignalHandler`, manages timer state machines (1584 lines)
  - `definition.rs`: `TimerDefinition`, `TimerTrigger` -- TOML timer definitions
  - `active.rs`: `ActiveTimer` -- runtime timer state
  - `matching.rs`: Signal-to-trigger matching logic
  - `signal_handlers.rs`: Signal handler implementations
  - `preferences.rs`: Timer preference management
  - `error.rs`: Timer errors
  - `manager_tests.rs`: Unit tests (1378 lines)

**`core/src/context/`:**
- Purpose: Application context -- config, file management, string interning
- Key files:
  - `config.rs`: `AppConfig`, `OverlaySettings`, `OverlayProfile` and all config types
  - `log_files.rs`: `DirectoryIndex` -- file listing and metadata
  - `parser.rs`: `ParsingSession` -- active parsing session state, `DefinitionLoader`
  - `watcher.rs`: `DirectoryWatcher` -- filesystem monitoring via `notify`
  - `interner.rs`: `IStr` type, `intern()`, `resolve()` -- string interning via `lasso`
  - `area_index.rs`: Area index caching for log files
  - `background_tasks.rs`: Background task management
  - `error.rs`: Config and watcher errors

**`core/src/state/`:**
- Purpose: Session state and IPC types
- Key files:
  - `cache.rs`: `SessionCache` -- per-session state (encounters, players, bosses, timers)
  - `info.rs`: `AreaInfo` -- current area tracking
  - `ipc.rs`: `ParseWorkerOutput` -- JSON types for parse-worker subprocess communication

**`core/src/game_data/`:**
- Purpose: SWTOR game constants and lookups
- Key files:
  - `raid_bosses.rs`: Raid boss NPC ID lookups (4144 lines, generated)
  - `flashpoint_bosses.rs`: Flashpoint boss NPC ID lookups (2624 lines, generated)
  - Various constant definitions for effect IDs, ability IDs, entity types

**`core/src/storage/`:**
- Purpose: Parquet file writing for encounter data
- Key files:
  - `writer.rs`: `EncounterWriter` -- writes `EventRow` to Parquet
  - `error.rs`: `StorageError`

**`core/src/icons/`:**
- Purpose: Icon registry for ability/effect icons
- Contains: `IconRegistry`, icon loading and caching

**`core/definitions/`:**
- Purpose: Bundled TOML definition files
- Subdirs:
  - `effects/`: Effect definitions (buff/debuff tracking rules)
  - `encounters/operations/`: Raid boss definitions
  - `encounters/flashpoints/`: Flashpoint boss definitions
  - `encounters/other/`: World bosses, misc encounters
  - `sounds/`: Sound effect files organized by voice pack

**`overlay/src/` (Overlay - `baras-overlay` crate):**

**`overlay/src/overlays/`:**
- Purpose: Complete overlay implementations (15 types)
- Key files:
  - `metric.rs`: `MetricOverlay` -- DPS/HPS/DTPS/threat bars
  - `raid.rs`: `RaidOverlay` -- raid frames with effect icons (1328 lines)
  - `timers.rs`: `TimerOverlay` -- boss encounter timers
  - `boss_health.rs`: `BossHealthOverlay` -- boss HP bars
  - `personal.rs`: `PersonalOverlay` -- personal stats display
  - `effects.rs`: `EffectsOverlay` -- personal effect tracking
  - `effects_ab.rs`: `EffectsABOverlay` -- dual-panel effects (1006 lines)
  - `alerts.rs`: `AlertsOverlay` -- pop-up alert text
  - `challenges.rs`: `ChallengeOverlay` -- DPS/HPS per phase
  - `notes.rs`: `NotesOverlay` -- Markdown-formatted boss notes
  - `dot_tracker.rs`: `DotTrackerOverlay` -- DOT uptime tracking
  - `cooldowns.rs`: `CooldownOverlay` -- ability cooldown tracking
  - `combat_time.rs`: `CombatTimeOverlay` -- combat duration display
  - `operation_timer.rs`: `OperationTimerOverlay` -- operation-wide timer
  - `mod.rs`: `Overlay` trait + re-exports of all overlay types and data structs

**`overlay/src/platform/`:**
- Purpose: OS-specific window management backends
- Key files:
  - `wayland.rs`: Wayland backend via wlr-layer-shell (1685 lines)
  - `x11.rs`: X11 backend via XShape extension
  - `windows.rs`: Windows backend via Win32 API
  - `macos.rs`: macOS backend via objc2/AppKit
  - `mod.rs`: `NativeOverlay` trait, `OverlayConfig`, `PlatformError`, monitor detection

**`overlay/src/widgets/`:**
- Purpose: Reusable rendering components
- Key files:
  - `progress_bar.rs`: `ProgressBar` -- colored fill bars
  - `header.rs`: `Header` -- overlay title bar
  - `compound_row.rs`: Multi-element row layout
  - `labeled_value.rs`: `LabeledValue` -- label + value pairs
  - `colors.rs`: Color constants and helpers
  - `mod.rs`: Re-exports

**`overlay/src/` (other):**
- `renderer.rs`: `Renderer` -- tiny-skia drawing primitives
- `frame.rs`: `OverlayFrame` -- high-level rendering frame with text layout
- `manager.rs`: `OverlayWindow` -- platform window + renderer wrapper
- `icons.rs`: Icon loading and `IconCache`
- `class_icons.rs`: SWTOR class/role icon management
- `utils.rs`: Formatting helpers (`format_number`, `format_time`, `truncate_name`)
- `main.rs`: Standalone development entry point (1742 lines)

**`parse-worker/src/`:**
- Purpose: Subprocess for parallel historical parsing
- Key files:
  - `main.rs`: Complete binary -- `FastEncounterWriter` (Arrow builders), mmap, rayon parallel parsing, Parquet writing

**`validate/src/`:**
- Purpose: CLI tool for definition validation
- Key files:
  - `main.rs`: CLI entry point with clap argument parsing (1351 lines)
  - `replay/`: Log replay engine with virtual clock and lag simulation
  - `output/`: CLI output formatting
  - `verification/`: Checkpoint verification system

## Key File Locations

**Entry Points:**
- `app/src-tauri/src/main.rs`: Application binary entry
- `app/src-tauri/src/lib.rs`: `run()` -- Tauri setup, service/overlay spawning
- `app/src/main.rs`: Frontend WASM entry
- `parse-worker/src/main.rs`: Parse worker binary
- `validate/src/main.rs`: Validate CLI binary

**Configuration:**
- `Cargo.toml`: Workspace definition (7 members)
- `app/src-tauri/tauri.conf.json`: Tauri configuration (if present)
- `core/src/context/config.rs`: `AppConfig` and all settings types
- `app/src-tauri/capabilities/`: Tauri permission capabilities

**Core Logic:**
- `core/src/signal_processor/processor.rs`: Event processing state machine
- `app/src-tauri/src/service/mod.rs`: Service main loop + signal dispatch
- `app/src-tauri/src/router.rs`: Overlay update routing
- `core/src/timers/manager.rs`: Timer state machine
- `core/src/effects/tracker.rs`: Effect tracking engine

**Data Storage:**
- `core/src/storage/writer.rs`: Parquet encounter writer
- Runtime data: `~/.config/baras/data/{session_id}/` (Parquet files)
- Config: `~/.config/baras/` (via `confy`)
- User definitions: `~/.config/baras/definitions/encounters/`

## Naming Conventions

**Files:**
- `snake_case.rs` for all Rust source files
- `mod.rs` for module directories
- `*_tests.rs` for test files co-located with source (e.g., `processor_tests.rs`, `manager_tests.rs`, `tracker_tests.rs`)

**Crates:**
- `baras-core`, `baras-types`, `baras-overlay`, `baras-parse-worker` (hyphenated in Cargo.toml)
- Backend Tauri crate is just `app` (package name)
- Frontend crate is `app-ui`

**Directories:**
- Plural for collections: `overlays/`, `widgets/`, `commands/`, `definitions/`, `triggers/`
- Singular for namespaces: `encounter/`, `query/`, `state/`, `service/`

## Where to Add New Code

**New Overlay Type:**
1. Overlay implementation: `overlay/src/overlays/{name}.rs`
2. Register in `overlay/src/overlays/mod.rs` (add to `Overlay` trait exports)
3. Data type: Add variant to `OverlayData` enum in `overlay/src/overlays/mod.rs`
4. Config type: Add to `baras-types` (`types/src/lib.rs`) and `core/src/context/config.rs`
5. Spawn function: `app/src-tauri/src/overlay/spawn.rs`
6. Overlay type variant: `app/src-tauri/src/overlay/types.rs`
7. State channel: `app/src-tauri/src/overlay/state.rs` (add getter for tx channel)
8. Manager integration: `app/src-tauri/src/overlay/manager.rs` (show/hide/toggle)
9. Router case: `app/src-tauri/src/router.rs` (route `OverlayUpdate` to overlay thread)
10. Service update variant: `app/src-tauri/src/service/mod.rs` (add `OverlayUpdate` variant)
11. Tauri commands: `app/src-tauri/src/commands/overlay.rs`
12. Frontend toggle: `app/src/app.rs` (add enabled signal + UI toggle)

**New Tauri Command:**
1. Command function: `app/src-tauri/src/commands/{category}.rs`
2. Re-export in `app/src-tauri/src/commands/mod.rs`
3. Register in `invoke_handler` array in `app/src-tauri/src/lib.rs`
4. Frontend API wrapper: `app/src/api.rs`

**New Game Signal:**
1. Add variant to `GameSignal` in `core/src/signal_processor/signal.rs`
2. Add timestamp match arm in `GameSignal::timestamp()`
3. Emit from `EventProcessor::process_event()` in `core/src/signal_processor/processor.rs`
4. Handle in relevant `SignalHandler` implementors (`TimerManager`, `EffectTracker`, etc.)
5. If needed in service: handle in `CombatSignalHandler` in `app/src-tauri/src/service/mod.rs`

**New Query Type:**
1. Result type: `types/src/lib.rs` (shared between frontend and backend)
2. Query implementation: `core/src/query/{name}.rs`, register in `core/src/query/mod.rs`
3. `EncounterQuery` method: expose through the query context
4. Tauri command: `app/src-tauri/src/commands/query.rs`
5. ServiceHandle method: `app/src-tauri/src/service/handler.rs`
6. Frontend API: `app/src/api.rs`
7. Frontend component: `app/src/components/`

**New Boss Encounter Definition:**
1. TOML file: `core/definitions/encounters/{category}/{name}.toml`
2. If adding new NPC IDs: update `core/src/game_data/raid_bosses.rs` or `flashpoint_bosses.rs`

**New Effect Definition:**
1. TOML file: `core/definitions/effects/{name}.toml`

**New Timer Trigger Type:**
1. Trigger variant: `core/src/dsl/triggers/`
2. Matcher: `core/src/timers/matching.rs`
3. Signal handler integration: `core/src/timers/signal_handlers.rs`

**Utilities / Shared Helpers:**
- Core business logic helpers: `core/src/` (appropriate submodule)
- Overlay rendering helpers: `overlay/src/utils.rs` or `overlay/src/widgets/`
- Frontend helpers: `app/src/utils.rs`
- Shared types: `types/src/lib.rs`

## Special Directories

**`core/definitions/`:**
- Purpose: Bundled TOML files for boss encounters, effects, and sounds
- Generated: No (hand-authored)
- Committed: Yes
- User overrides via `_custom.toml` suffix in `~/.config/baras/definitions/`

**`core/data/`:**
- Purpose: Build-script generated data (e.g., `phf` hash maps)
- Generated: Yes (by `core/build.rs`)
- Committed: No (in `.gitignore`)

**`app/dist/`:**
- Purpose: Built frontend WASM output
- Generated: Yes (by Dioxus build)
- Committed: Yes (for Tauri bundling)

**`test-log-files/`:**
- Purpose: SWTOR combat log files for testing
- Generated: No
- Committed: No (gitignored)

**`scripts/`:**
- Purpose: Build scripts, icon conversion, timer conversion from other tools
- Contains: `converted-files/` with timer definitions converted from Orbs format

---

*Structure analysis: 2026-04-03*
