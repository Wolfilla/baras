---
generated: 2026-04-03
focus: quality
---

# Coding Conventions

## Rust Edition and Workspace

**Edition:** 2024 (all crates)

**Workspace resolver:** 2

**Workspace lint config** (`Cargo.toml`):
```toml
[workspace.lints.clippy]
too_many_arguments = "allow"
```
All crates inherit with `[lints] workspace = true`.

## Naming Patterns

**Files:**
- `snake_case.rs` for all Rust source files
- Separate test files use `_tests.rs` suffix: `tracker_tests.rs`, `manager_tests.rs`, `processor_tests.rs`
- Submodules in directories with `mod.rs`: `core/src/effects/mod.rs`, `core/src/timers/mod.rs`

**Types:**
- PascalCase for structs, enums, traits: `CombatEncounter`, `GameSignal`, `EffectTracker`
- Enum variants use PascalCase: `EntityType::Player`, `OverlayData::Metrics`

**Functions:**
- snake_case: `parse_line()`, `handle_signal()`, `process_event()`
- Builder-style methods with `with_` prefix: `with_total()`, `with_color()`
- Boolean getters use `is_` prefix: `is_boss()`, `is_healing()`, `is_interactive()`

**Constants:**
- SCREAMING_SNAKE_CASE: `EFFECTS_DSL_VERSION`, `MAX_PROFILES`, `SHIELD_EFFECT_IDS`
- `phf` compile-time maps for game data lookups: `OPERATION_AREAS`, `FLASHPOINT_AREAS`, `ATTACK_TYPES`

**Modules:**
- Public modules use `pub mod` in parent's `mod.rs`
- Private test modules gated with `#[cfg(test)] mod _tests;`
- Backward compatibility aliases for renamed modules: `pub use dsl as boss;`, `pub use game_data as swtor_data;`

## Module Organization

**Pattern: `mod.rs` with submodule files + re-exports**

Each major module follows this structure in its `mod.rs`:

```rust
// Private submodules
mod active;
mod definition;
mod manager;

// Test module (conditional)
#[cfg(test)]
mod manager_tests;

// Public re-exports
pub use active::{ActiveTimer, TimerKey};
pub use definition::{TimerDefinition, TimerTrigger};
pub use manager::TimerManager;
```

See `core/src/effects/mod.rs`, `core/src/timers/mod.rs`, `core/src/signal_processor/mod.rs`.

**Crate-level re-exports** in `core/src/lib.rs`: All important types are re-exported from the crate root for convenient access by dependent crates. Use `pub use module::Type;` not `pub use module::*;` for specific types, but `pub use game_data::*;` for large lookup modules.

**Commands module** (`app/src-tauri/src/commands/mod.rs`): Uses `pub use submodule::*;` to flatten all commands into one namespace for Tauri's invoke handler.

## Section Delimiters

Use box-drawing characters for visual section separation throughout the codebase:

```rust
// ═══════════════════════════════════════════════════════════════════════════
// Major Section Header
// ═══════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────
// Minor Section Header
// ─────────────────────────────────────────────────────────────────────────
```

Double-line (`═`) for major sections, single-line (`─`) for subsections. Used extensively in all crates.

## Doc Comments

**Module-level:** Use `//!` doc comments at the top of each module file:
```rust
//! Effect tracking system
//!
//! This module provides:
//! - **Definitions**: Templates that describe what effects to track
//! - **Active instances**: Runtime state of currently active effects
```

Some modules include ASCII diagrams in doc comments (see `core/src/effects/mod.rs`).

**Function/type-level:** Use `///` sparingly. Prefer self-documenting names. Doc comments are added when the purpose or behavior is non-obvious.

## Derive Macros

**Common patterns:**
- Data types: `#[derive(Debug, Clone)]` (minimum for most structs)
- Enum variants: `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`
- Serializable configs: `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`
- Default-able types: Add `Default` derive or manual `impl Default`

**Serde attributes commonly used:**
```rust
#[serde(rename_all = "snake_case")]        // Enum variant naming
#[serde(rename_all = "camelCase")]         // Frontend types
#[serde(tag = "type")]                     // Tagged enums (e.g., Trigger)
#[serde(default)]                          // Optional fields with defaults
#[serde(alias = "id")]                     // Backward-compatible field names
#[serde(skip_serializing_if = "...")]      // Clean TOML output
#[serde(default = "path::to::fn")]         // Custom default functions
```

## Serde Defaults Module

`core/src/serde_defaults.rs` centralizes default value functions and skip predicates:
- `default_true()`, `default_timer_color()`, `default_countdown_start()`
- `is_false()`, `is_zero_f32()`, `is_empty_vec()`, `is_default_timer_color()`
- Use these instead of inline closures for `#[serde(default = "...")]` and `#[serde(skip_serializing_if = "...")]`

## Error Handling

**Pattern:** Domain-specific error types per module using `thiserror::Error`:
- `core/src/combat_log/error.rs` - `ParseError`, `ReaderError`
- `core/src/query/error.rs` - `QueryError`
- `core/src/dsl/error.rs` - `DslError`
- `core/src/storage/error.rs` - `StorageError`
- `core/src/timers/error.rs` - `TimerError`

**Structure:** Each error enum has contextual variants with structured fields:
```rust
#[derive(Debug, Error)]
pub enum DslError {
    #[error("failed to read {path}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
```

**Tauri commands** return `Result<T, String>` - errors are mapped to strings at the boundary.

**Graceful degradation:** Fallback to streaming parse if worker subprocess fails. Missing definitions are handled without panicking.

## Trait Pattern: SignalHandler

The `SignalHandler` trait (`core/src/signal_processor/handler.rs`) is the core abstraction for systems that react to game events:

```rust
pub trait SignalHandler {
    fn handle_signal(&mut self, signal: &GameSignal, encounter: Option<&CombatEncounter>);
    fn handle_signals(&mut self, signals: &[GameSignal], encounter: Option<&CombatEncounter>) { ... }
    fn on_encounter_start(&mut self, _encounter_id: u64) {}
    fn on_encounter_end(&mut self, _encounter_id: u64) {}
}
```

Implemented by: `EffectTracker`, `TimerManager`, and test harnesses.

## Trait Pattern: Overlay

The `Overlay` trait (`overlay/src/overlays/mod.rs`) defines the interface for all overlay windows:
- `update_data()`, `update_config()`, `render()`, `poll_events()`
- Default implementations for common operations (`set_click_through`, `set_move_mode`)
- Not required to be `Send` - overlays live in their own dedicated threads

## State Management

**Backend (`app/src-tauri/`):**
- `SharedState` uses `std::sync::Mutex` (not tokio) for synchronous Tauri command access
- `AtomicBool` flags with `Ordering::SeqCst` for lock-free hot-path reads
- `ServiceHandle` wraps `mpsc::Sender<ServiceCommand>` for async dispatch

**Frontend (Dioxus):**
- `use_signal()` for local component state
- `use_context()` / `use_context_provider()` for shared state (e.g., `ToastManager`)
- `spawn_local()` (aliased as `spawn`) for async Tauri calls
- Props passed via `#[derive(Props, Clone, PartialEq)]` structs

**Avoid bidirectional sync loops** between local signals and parent state. Initialize from parent in `use_signal`, sync one direction only.

## Frontend Patterns

**Component naming:** PascalCase functions with `#[component]` attribute:
```rust
#[component]
pub fn DataExplorerPanel(mut props: DataExplorerProps) -> Element {
```

The top-level `App()` function and some helpers use bare `fn` without `#[component]` (with `#![allow(non_snake_case)]`).

**Props:** Defined as separate structs with `#[derive(Props, Clone, PartialEq)]`:
```rust
#[derive(Props, Clone, PartialEq)]
pub struct RotationViewProps {
    pub encounter_idx: Option<u32>,
    pub selected_anchor: Signal<Option<i64>>,
    #[props(default)]
    pub on_range_change: Option<EventHandler<TimeRange>>,
}
```

**API layer** (`app/src/api.rs`): All Tauri backend calls go through typed wrapper functions. Two invoke patterns:
- `invoke()` for reads (silently ignores errors)
- `try_invoke()` for mutations (returns `Result<JsValue, String>`)

**Types mirror** (`app/src/types.rs`): Re-exports shared types from `baras-types` and defines frontend-specific types with `#[serde(rename_all = "camelCase")]`.

**Toast pattern** (`app/src/components/toast.rs`): Fixed-position div with auto-dismiss via `gloo_timers`. Access via `use_toast()` context hook.

## Shared Types Crate

`types/` (`baras-types`) contains serializable types shared between native backend and WASM frontend:
- Config types: `AppConfig`, `OverlaySettings`, `RaidOverlaySettings`
- Query types: `AbilityBreakdown`, `DataTab`, `BreakdownMode`
- UI state: `UiSessionState`, `DataExplorerState`
- Formatting utilities: `types/src/formatting.rs`

Types need `Serialize + Deserialize` and must be compatible with both native serde and `serde_wasm_bindgen`.

## Game Data Pattern

Static game data uses `phf` (perfect hash function) maps generated at build time:
- `core/build.rs` generates lookup tables from game data
- Modules like `core/src/game_data/bosses.rs`, `raids.rs`, `flashpoints.rs` expose lookup functions
- Pattern: `pub fn is_boss(npc_id: i64) -> bool` and `pub fn lookup_boss(npc_id: i64) -> Option<&BossInfo>`

## String Interning

`core/src/context/interner.rs` provides `IStr` (interned string) via the `lasso` crate:
- `intern(s: &str) -> IStr` - intern a string
- `resolve(key: IStr) -> &str` - resolve back to string
- `empty_istr()` - empty string sentinel
- Used for frequently repeated strings (entity names, ability names) to reduce memory

## Performance Conventions

- Fixed-size stack arrays instead of `Vec` for known-size collections
- `memchr_iter` for SIMD-accelerated scanning
- Manual digit parsing instead of string conversions
- `hashbrown::HashSet/HashMap` instead of `std::collections` in hot paths
- `encoding_rs` for Windows-1252 decoding (game log format)

## Import Organization

**Order (observed):**
1. Standard library (`std::`)
2. External crates (`chrono`, `serde`, `tokio`, etc.)
3. Workspace crates (`baras_core`, `baras_types`)
4. Internal crate modules (`crate::`, `super::`)

No barrel file patterns except in `mod.rs` re-exports. No path aliases configured.

---

*Convention analysis: 2026-04-03*
