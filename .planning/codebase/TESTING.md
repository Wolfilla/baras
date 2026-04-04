---
generated: 2026-04-03
focus: quality
---

# Testing Patterns

## Test Framework

**Runner:** Rust built-in test framework (`cargo test`)

**Assertion Library:** Standard `assert!`, `assert_eq!`, `assert!(matches!(...))` macros

**Run Commands:**
```bash
cargo test                           # Run all workspace tests
cargo test -p baras-core             # Run core crate tests only
cargo test -p baras-types            # Run types crate tests only
cargo test -p baras-validate         # Run validate crate tests only
cargo test -p app                    # Run Tauri backend tests only
cargo test -p baras-overlay          # Run overlay tests only
```

## Test Count and Distribution

**Total:** ~151 test functions across the workspace

| Crate / File | Tests | Focus |
|---|---|---|
| `core/src/timers/manager_tests.rs` | 24 | Timer activation by signals |
| `core/src/combat_log/parser/tests.rs` | 22 | Log line parsing (entities, details, damage, healing) |
| `types/src/formatting.rs` | 14 | Number/time formatting |
| `core/src/effects/tracker_tests.rs` | 14 | Effect lifecycle, alerts, multi-healer tracking |
| `core/src/signal_processor/processor_tests.rs` | 12 | End-to-end signal emission from log fixtures |
| `core/src/dsl/challenge.rs` | 11 | Challenge condition evaluation |
| `core/src/dsl/triggers/matchers.rs` | 9 | Ability/effect/entity selector matching |
| `core/src/timers/preferences.rs` | 6 | Timer preference persistence |
| `core/src/game_data/bosses.rs` | 6 | Boss lookup functions |
| `validate/src/replay/lag.rs` | 4 | Lag simulation |
| `validate/src/replay/clock.rs` | 4 | Virtual clock timing |
| `core/src/dsl/loader.rs` | 4 | TOML definition parsing |
| `core/src/context/area_index.rs` | 4 | Area indexing |
| Others | ~17 | Various utility tests |

## Test File Organization

**Two patterns used:**

**1. Inline `#[cfg(test)] mod tests` block** (most common):
```rust
// At bottom of source file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() { ... }
}
```
Used in: `types/src/lib.rs`, `types/src/formatting.rs`, `core/src/game_data/bosses.rs`, `core/src/context/area_index.rs`, `core/src/dsl/loader.rs`, `core/src/dsl/triggers/mod.rs`, `core/src/dsl/challenge.rs`, `validate/src/replay/clock.rs`, `validate/src/verification/checkpoint.rs`, `overlay/src/utils.rs`

**2. Separate `_tests.rs` file** (for large test suites):
```rust
// In mod.rs:
#[cfg(test)]
mod manager_tests;
```
Used in:
- `core/src/timers/manager_tests.rs` (1378 lines)
- `core/src/signal_processor/processor_tests.rs` (1826 lines)
- `core/src/effects/tracker_tests.rs` (897 lines)
- `core/src/combat_log/parser/tests.rs` (296 lines)

**Naming convention:** `{module}_tests.rs` for separate files, `mod tests` for inline blocks.

## Test Structure Patterns

### Helper Constructors

Tests define factory functions for complex test objects to reduce boilerplate:

```rust
// core/src/timers/manager_tests.rs
fn make_timer(id: &str, name: &str, trigger: TimerTrigger, duration: f32) -> TimerDefinition {
    TimerDefinition {
        id: id.to_string(),
        name: name.to_string(),
        trigger,
        duration_secs: duration,
        // ... explicit defaults for all fields
    }
}

fn make_encounter(combat_start: NaiveDateTime, combat_time_secs: f32) -> CombatEncounter { ... }

fn now() -> NaiveDateTime { Local::now().naive_local() }
```

Similar patterns in `core/src/effects/tracker_tests.rs` (`make_effect()`, `make_tracker()`, `effect_applied_signal()`) and `core/src/combat_log/parser/tests.rs` (`test_parser()`).

### Signal Factory Functions

Test files create typed signal constructors for readability:

```rust
fn effect_applied_signal(effect_id: i64, timestamp: NaiveDateTime) -> GameSignal {
    GameSignal::EffectApplied {
        effect_id,
        effect_name: empty_istr(),
        action_id: 0,
        // ... defaults for non-relevant fields
    }
}
```

### Fixture-Based Integration Tests

`core/src/signal_processor/processor_tests.rs` parses real combat log files and validates signal output:

```rust
fn collect_signals_from_fixture(fixture_path: &Path) -> Vec<GameSignal> { ... }
fn collect_signals_with_boss_defs(fixture_path: &Path, boss_config_path: &Path) -> Vec<GameSignal> { ... }
```

Fixture files live in `test-log-files/` (not committed to git). Tests that use fixtures will fail if the files are missing.

### Assertion Patterns

**Direct equality:**
```rust
assert_eq!(entity.entity_type, EntityType::Player);
assert_eq!(details.dmg_amount, 5765);
```

**Pattern matching for complex types:**
```rust
assert!(matches!(
    boss.phases[1].start_trigger,
    PhaseTrigger::BossHpBelow { hp_percent, .. } if (hp_percent - 50.0).abs() < 0.01
));
```

**Float comparison with tolerance:**
```rust
assert!((clock.combat_elapsed_secs() - 75.5).abs() < 0.001);
```

**Signal type checking via helper:**
```rust
fn signal_type_name(signal: &GameSignal) -> &'static str {
    match signal {
        GameSignal::CombatStarted { .. } => "CombatStarted",
        // ...
    }
}
```

## The Validate Crate

`validate/` is a standalone CLI tool (`baras-validate`) for testing boss encounter definitions against real combat logs. It is the closest thing to an integration test suite.

**Purpose:** Replay combat logs through the signal pipeline with boss definitions loaded, verifying timer behavior, phase transitions, and counter states.

**Usage:**
```bash
# From justfile
cargo run --bin baras-validate -- --boss revan --log test-log-files/operations/hm_tos_revan.txt
cargo run --bin baras-validate -- --boss propagator_core_xr53 --log test-log-files/operations/hm_propagator.txt
```

**Key features:**
- Replay modes: `--mode realtime` (1x speed with delays) or `--mode accelerated` (fast, default)
- Checkpoint verification: `--expect expectations.toml` validates timer states at specific combat times
- Output controls: `--quiet`, `--full`, `--verbose`, `--all-abilities`, `--all-entities`
- Encounter selection: `--encounter N` or `--latest`
- Time windowing: `--start-at MM:SS` / `--stop-at MM:SS`

**Architecture:**
- `validate/src/main.rs` - CLI entry point (clap-based), main replay loop
- `validate/src/replay/clock.rs` - Virtual clock for time simulation
- `validate/src/replay/lag.rs` - I/O lag simulation for realistic testing
- `validate/src/verification/checkpoint.rs` - TOML-based expectations system
- `validate/src/output/cli.rs` - Formatted terminal output

**Expectations format** (TOML):
```toml
[meta]
boss_id = "revan"
tolerance_secs = 0.5

[[checkpoint]]
at_secs = 10.0
active_timers = [{ id = "enrage", remaining_secs = [290.0, 290.5] }]
timers_fired = ["phase1_timer"]
```

## Mocking

**No mocking framework used.** Tests construct real objects with controlled inputs rather than mocking dependencies. The `SignalHandler` trait enables testing by feeding signals directly:

```rust
let mut manager = TimerManager::new();
manager.load_definitions(vec![timer]);
let signal = GameSignal::CombatStarted { timestamp: start, encounter_id: 1 };
manager.handle_signal(&signal, Some(&enc));
assert_eq!(manager.active_timers().len(), 1);
```

## Test Data

**Inline data:** Parser tests use inline combat log line strings:
```rust
let input = "@Galen Ayder#690129185314118|(-4700.43,-4750.48,710.03,-0.71)|(1/414851)";
```

**TOML strings:** Definition parsing tests use inline TOML:
```rust
let toml = r#"
[[boss]]
id = "test_boss"
name = "Test Boss"
...
"#;
let config: BossConfig = toml::from_str(toml).expect("Failed to parse TOML");
```

**Fixture files:** `test-log-files/` directory (gitignored) contains real combat logs for integration tests.

**External file dependencies:** One test in `app/src-tauri/src/commands/starparse.rs` depends on `scripts/starparse-timers v15.xml` using `env!("CARGO_MANIFEST_DIR")`.

## Coverage Gaps

**Well-tested areas:**
- Log line parsing (`parser/tests.rs`)
- Timer signal handling (24 tests)
- Effect tracking lifecycle (14 tests)
- Signal emission from combat events (12 fixture-based tests)
- TOML definition loading and serialization
- Number/time formatting utilities

**Not tested (or minimally tested):**
- **Query module** (`core/src/query/`): No tests for DataFusion SQL queries (overview, breakdown, timeline, rotation, usage, combat_log, effects)
- **Storage module** (`core/src/storage/`): No tests for parquet writing
- **Frontend components** (`app/src/`): No Dioxus component tests or WASM tests
- **Tauri commands** (`app/src-tauri/src/commands/`): Only 1 test (starparse XML parsing), no command integration tests
- **Overlay rendering** (`overlay/src/`): Only 3 utility tests, no rendering tests
- **Service layer** (`app/src-tauri/src/service/`): No tests for CombatService, handler, directory management
- **Context module** (`core/src/context/`): Only area_index tests (4), no tests for config, watcher, log files, parser session management
- **Encounter combat** (`core/src/encounter/`): No direct tests (covered indirectly by processor_tests fixtures)
- **Parse-worker** (`parse-worker/`): No tests

**Gaps by risk:**
- High: Query module - complex SQL generation untested
- High: Storage writer - data integrity critical
- Medium: Service layer - core application orchestration
- Medium: Encounter combat state machine
- Low: Frontend components (UI behavior)
- Low: Overlay rendering (visual output)

---

*Testing analysis: 2026-04-03*
