---
generated: 2026-04-03
focus: concerns
---

# Codebase Concerns

## Tech Debt

### Oversized Files Exceeding 500-Line Guideline

The project has a 500-line guideline (except platform implementations). Many files significantly exceed this.

**Generated data files (acceptable):**
- `core/src/game_data/raid_bosses.rs` (4144 lines) - auto-generated boss data
- `core/src/game_data/flashpoint_bosses.rs` (2624 lines) - auto-generated boss data

**Platform implementations (acceptable per guidelines):**
- `overlay/src/platform/wayland.rs` (1685 lines)

**Service layer (should be split):**
- `app/src-tauri/src/service/mod.rs` (3787 lines) - CombatService run loop, 31 ServiceCommand variants, operation timer state, file indexing, overlay dispatch, process monitoring. Contains 55 functions and 87 `.clone()` calls.
  - Fix approach: Extract operation timer logic, file management, and overlay dispatch into dedicated modules.
- `app/src-tauri/src/service/handler.rs` (1580 lines) - Signal handler with 61 public functions.
  - Fix approach: Group handler methods by domain (combat, overlay, effects, timers) into sub-modules.

**Frontend components (should be split):**
- `app/src/components/data_explorer.rs` (3591 lines) - 9+ data views in one component.
- `app/src/components/settings_panel.rs` (2821 lines) - All settings in one file.
- `app/src/components/effect_editor.rs` (2251 lines)
- `app/src/components/charts_panel.rs` (1379 lines)
- `app/src/components/combat_log.rs` (1307 lines)
  - Fix approach: Extract each view/tab into its own component file.

**Core logic:**
- `core/src/effects/tracker.rs` (2337 lines, 56 functions)
- `core/src/signal_processor/processor.rs` (1625 lines)
- `core/src/timers/manager.rs` (1584 lines)
- `core/src/encounter/combat.rs` (1488 lines, 70 functions)
- `core/src/dsl/definition.rs` (958 lines)
- `core/src/dsl/challenge.rs` (951 lines)
  - Fix approach: Split struct definitions into separate files per the project guidelines.

**Shared types monolith:**
- `types/src/lib.rs` (3057 lines) - 74 structs/enums in a single file with no module organization.
  - Impact: Every change to any type requires scanning a 3000-line file. Hard to navigate.
  - Fix approach: Split into domain modules (overlay types, encounter types, config types, etc.) with a barrel re-export.

### Monolithic API Surface

- `app/src/api.rs` (1642 lines, 111 public functions) - All Tauri invoke wrappers in one file.
  - Fix approach: Split by domain (combat, overlay, effects, encounters, settings).

### Inconsistent Mutex Lock Error Handling

Two patterns coexist for the same `SharedState`:

- `operation_timer.lock().unwrap()` - 15 occurrences in `app/src-tauri/src/service/mod.rs`. Will panic on mutex poisoning.
- `raid_registry.lock().unwrap_or_else(|p| p.into_inner())` - Used elsewhere in the same file. Recovers from poisoning gracefully.
  - Impact: If any thread panics while holding `operation_timer`, the app crashes on next lock attempt.
  - Fix approach: Use `unwrap_or_else(|p| p.into_inner())` consistently, or switch to `parking_lot::Mutex` which doesn't poison.

### Tauri Commands Use `Result<_, String>` Everywhere

All 117 Tauri command return types in `app/src-tauri/src/commands/` use `Result<T, String>` instead of typed errors.
- Files: All files under `app/src-tauri/src/commands/`
- Impact: Loses error context, no structured error handling on frontend, `.map_err(|e| e.to_string())` scattered everywhere.
- Fix approach: Implement a unified `CommandError` type with `impl From<T>` for domain errors and `impl serde::Serialize`.

### Workspace-Level Clippy Suppression

- `Cargo.toml` sets `[workspace.lints.clippy] too_many_arguments = "allow"` globally.
- Additionally, 6 individual `#[allow(clippy::too_many_arguments)]` annotations exist in overlay code.
- Impact: Functions with many arguments indicate structural issues (missing config structs). The global suppression hides this.
- Fix approach: Introduce config/params structs for functions exceeding 5-6 arguments, then remove the workspace-level allow.

## Test Coverage Gaps

**Overall:** Only 20 out of 206 Rust source files contain any tests (9.7%).

**Well-tested areas (core crate only):**
- `core/src/combat_log/parser/tests.rs` - Parser tests
- `core/src/signal_processor/processor_tests.rs` (1826 lines)
- `core/src/effects/tracker_tests.rs` (897 lines)
- `core/src/timers/manager_tests.rs` (1378 lines)
- `core/src/dsl/` - Loader, triggers, challenges have tests

**No tests at all:**
- `overlay/` crate - Only `overlay/src/utils.rs` has a single test. Zero tests for any overlay rendering, platform backends, or overlay manager logic.
  - Risk: Rendering regressions go undetected. Platform-specific bugs in Wayland/X11/Windows/macOS code.
  - Priority: Medium (visual testing is hard, but at least data transform logic could be tested)

- `app/src-tauri/` - Only `commands/starparse.rs` has tests. Zero tests for:
  - `service/mod.rs` (3787 lines of service orchestration)
  - `service/handler.rs` (1580 lines of signal handling)
  - `overlay/manager.rs` (1138 lines)
  - `overlay/spawn.rs` (1088 lines with unsafe code)
  - All other command handlers
  - Risk: Service logic regressions, state management bugs.
  - Priority: High

- `app/src/` (Dioxus frontend) - Zero tests.
  - Priority: Low (WASM frontend testing is complex)

- `core/src/query/` - Zero tests for DataFusion query logic including `breakdown.rs` (872 lines).
  - Risk: Query result regressions, incorrect statistics.
  - Priority: High

- `core/src/encounter/combat.rs` (1488 lines, 70 functions) - Zero direct tests.
  - Priority: Medium (partially covered by processor_tests)

## Unsafe Code

**Platform overlay backends (justified):**
- `overlay/src/platform/wayland.rs` - 4 unsafe blocks for shared memory (memfd, mmap, raw pointer slices). Required by Wayland SHM protocol.
- `overlay/src/platform/x11.rs` - 4 unsafe blocks for X11 SHM buffer management. Required by X11 SHM extension.
- `overlay/src/platform/windows.rs` - ~15 unsafe blocks for Win32 API calls. Required by Windows API.
- `overlay/src/platform/macos.rs` - ~10 unsafe blocks for Objective-C runtime interop. Required by macOS AppKit.

**Memory-mapped file I/O (justified):**
- `core/src/combat_log/reader.rs` lines 50, 95 - `unsafe { Mmap::map(&file) }` for memory-mapped log reading. Standard pattern for mmap.

**Raw pointer wrapper (risk area):**
- `app/src-tauri/src/overlay/spawn.rs` - `SendPtr<T>` wraps `*mut T` with `unsafe impl Send + Sync`. Used to pass overlay pointers across thread boundaries on macOS.
  - Safety argument: "All access serialized through main queue via `dispatch::Queue::main().exec_sync()`"
  - Risk: If dispatch serialization is ever bypassed, undefined behavior. The 7 dereference sites (lines 292-358) all assume main-thread access.
  - Mitigation: Well-documented safety comments. Confined to macOS `#[cfg]` blocks.

**Shared memory Send impl (justified):**
- `overlay/src/platform/wayland.rs:439` - `unsafe impl Send for ShmBuffer {}`
- `overlay/src/platform/x11.rs:93` - `unsafe impl Send for ShmBuffer {}`
  - These wrap OS-level shared memory handles that are safe to send between threads.

## Performance Concerns

**Excessive cloning in service layer:**
- `app/src-tauri/src/service/mod.rs` has 87 `.clone()` calls. Many involve cloning `PathBuf`, `String`, and complex structs for event emission.
  - Impact: Unnecessary allocations in the hot path of event processing.
  - Fix approach: Use `Arc<str>` or `Arc<Path>` for frequently-cloned paths; pass references where lifetime allows.

**Potential lock contention:**
- `operation_timer` mutex is acquired 15+ times in the main service loop (`app/src-tauri/src/service/mod.rs`), often for brief reads.
  - Fix approach: Consider `RwLock` for read-heavy access, or inline the timer state into the service struct (it's only accessed from one task).

**No parser hot-path format! calls:**
- `core/src/combat_log/parser.rs` has zero `format!()` calls, confirming the hot path avoids allocations. This is good.

## Fragile Areas

**CombatService run loop:**
- Files: `app/src-tauri/src/service/mod.rs` lines 1210-1400+
- Why fragile: Single `tokio::select!` loop handles all 31 ServiceCommand variants plus timers, file watchers, and process monitoring. Adding new commands requires modifying the same massive match block.
- Safe modification: Add new ServiceCommand variants at the end of the enum and match block. Test signal flow manually.
- Test coverage: Zero.

**Signal processing pipeline:**
- Files: `core/src/signal_processor/processor.rs`, `core/src/context/parser.rs`
- Why fragile: The `process_event()` function orchestrates combat lifecycle, phase transitions, counters, timers, and effects. Changes to signal ordering can break downstream consumers.
- Test coverage: Good (1826-line test file), but integration-level only.

**Overlay spawn lifecycle:**
- Files: `app/src-tauri/src/overlay/spawn.rs`
- Why fragile: Raw pointer management with manual `Box::from_raw` cleanup. Memory leak if the cleanup path (line 358) is not reached.
- Safe modification: Do not change the pointer lifecycle without reviewing all 7 dereference sites.

## Dependencies

**Large dependency surface:**
- `datafusion` v51 + `arrow` v57 + `parquet` v57 - Heavy data processing stack. These are major dependencies that drive compile times and binary size.
  - Risk: Major version bumps require coordinated updates across all three.
  - Mitigation: Already using `default-features = false` to minimize feature surface.

**Platform-specific dependencies:**
- `wayland-client` 0.31 / `wayland-protocols` 0.32 / `wayland-protocols-wlr` 0.3 - Wayland compositor protocol bindings.
- `x11rb` 0.13 - X11 protocol.
- `windows` 0.58 - Win32 API.
- `objc2` 0.6 / `objc2-foundation` 0.3 / `objc2-app-kit` 0.3 - macOS Objective-C bridge.
  - Risk: objc2 ecosystem is relatively young (0.x versions). Breaking changes likely between minor versions.

**Audio dependency:**
- `rodio` 0.19 with wav/vorbis/mp3 features. Pulls in audio decoding libraries.
  - Note: Uses `default-features = false` which is good.

**`confy` 2.0.0 for config:**
- Used in `core/src/context/config.rs`.
  - Risk: confy is a small crate with infrequent updates. Config format changes could break user settings.

## Security Considerations

**Memory-mapped file access:**
- Files: `core/src/combat_log/reader.rs`
- Risk: If the combat log file is modified by another process while mmap'd, reads could return inconsistent data or (on some platforms) cause SIGBUS.
- Current mitigation: Used for historical file reading only (not live tailing). The parse-worker subprocess exits after use, limiting exposure.

**Subprocess spawning (parse-worker):**
- Files: `parse-worker/src/main.rs`, service code that spawns it
- Risk: Subprocess communicates via stdout JSON. Malformed output could cause deserialization failures.
- Current mitigation: Fallback to streaming parse if worker fails (documented in error handling section of CLAUDE.local.md).

## Missing Infrastructure

**No integration tests:**
- No `tests/` directory at workspace or crate level. All tests are unit tests within source files.
- Impact: No end-to-end testing of the signal pipeline, service commands, or overlay lifecycle.

**No benchmarks:**
- No `benches/` directory. Parser performance claims (SIMD scanning, stack arrays) are not validated by benchmarks.
- Fix approach: Add `criterion` benchmarks for `LogParser::parse_line()` and `EventProcessor::process_event()`.

---

*Concerns audit: 2026-04-03*
