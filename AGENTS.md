# AGENTS.md — Keyboard Player

Rust + Bevy 0.18.1 typing practice game. Edition 2024. Desktop (macOS/Windows/Linux).

## Build / Run / Test

```bash
cargo build                          # compile (dev profile, deps optimized via profile.dev.package.*)
cargo run                            # launch game (requires assets/ directory)
cargo test                           # run all unit tests
cargo test --lib                     # skip doc-tests, unit tests only
cargo test <test_name>               # single test by name, e.g. cargo test split_english_basic
cargo test <module_path>::            # tests in a module, e.g. cargo test states::playing::tests::
cargo clippy                         # lint (no clippy.toml — use defaults)
cargo fmt                            # format (no rustfmt.toml — use defaults)
cargo fmt -- --check                 # verify formatting without modifying
```

No CI pipeline. Validate locally with `cargo clippy && cargo test && cargo fmt -- --check`.

## Project Structure

```
src/
├── main.rs              # Entry: plugin registration, camera, resource init
├── states/              # Game state machine (MainMenu → Selection → Playing → Result)
│   ├── mod.rs           # GameState enum + StatesPlugin
│   ├── menu.rs          # Main menu UI + input
│   ├── selection.rs     # Language/Grade/Difficulty selection (keyboard + mouse)
│   ├── playing.rs       # Core typing gameplay, HUD, pause, cursor, completion
│   └── result.rs        # Score display + new record celebration
├── systems/             # Pure logic systems (not tied to a single state)
│   ├── difficulty.rs    # Hidden character generation for hard mode
│   └── scoring.rs       # (reserved)
├── resources/           # Bevy Resources
│   ├── font_assets.rs   # FontAssets (FromWorld init)
│   ├── game_config.rs   # User selection config (language, grade, difficulty)
│   └── game_data.rs     # TextLibrary + CurrentPassage
├── data/                # Data models and loading
│   ├── text_model.rs    # Language, Grade, Difficulty, TextPassage
│   └── text_loader.rs   # Scan + parse assets/texts/*.json
├── audio/
│   └── sfx.rs           # SfxHandles (FromWorld) + SfxEvent + playback system
├── storage/
│   └── records.rs       # PracticeRecord, RecordStore, local JSON persistence
├── components/          # (reserved for shared ECS components)
└── ui/                  # (reserved for reusable UI helpers)
assets/
├── texts/               # Passage JSON files: {lang}_{grade}_{num}.json
├── fonts/               # Ark Pixel font (OFL-1.1)
├── audio/               # Kenney CC0 sound effects (.ogg)
└── images/              # Static image assets
```

## Bevy 0.18 API (Critical Differences)

These differ from older Bevy tutorials and docs found online:

| Concept | Bevy 0.18 API |
|---|---|
| Entity cleanup on state exit | `DespawnOnExit(GameState::Foo)` (NOT `StateScoped`) |
| Event definition | `#[derive(Message)]` (NOT `#[derive(Event)]`) |
| Event read/write | `MessageReader<T>` / `MessageWriter<T>`, `.write()` to send |
| Child entity spawning | `ChildSpawnerCommands` (NOT `ChildBuilder`) |
| Despawn children | `despawn_related::<Children>()` (NOT `despawn_descendants()`) |
| Window resolution | `WindowResolution` accepts `(u32, u32)` |
| Single-entity query | `Single<&mut Window>` |
| Keyboard text input | `KeyboardInput.text` field |
| IME events | `MessageReader<Ime>` |
| Resource with AssetServer | `FromWorld` + `init_resource::<T>()` in App setup |

## Code Style

### Imports

```rust
// 1. Standard library
use std::collections::HashMap;
use std::path::Path;

// 2. External crates — bevy::prelude::* is the one glob import allowed
use bevy::prelude::*;
use bevy::input::keyboard::{Key, KeyboardInput};
use rand::Rng;
use serde::Deserialize;

// 3. Crate-internal — always explicit paths, never glob
use crate::data::text_model::{Difficulty, Grade, Language};
use super::GameState;
```

### Naming

| Kind | Convention | Example |
|---|---|---|
| Files | `snake_case.rs` | `text_loader.rs`, `game_config.rs` |
| Functions / variables | `snake_case` | `handle_typing_input`, `current_line` |
| Types / Enums / Components | `PascalCase` | `TypingState`, `GameState`, `RefLineText` |
| Constants | `SCREAMING_SNAKE` | `MAX_CHARS_EN`, `COLOR_CORRECT` |
| Enum variants | `PascalCase` | `Language::En`, `Difficulty::Hard` |

### Constants and Colors

Define UI constants at the top of each file as `const`. Use `Color::srgb()` / `Color::srgba()`:

```rust
const COLOR_CORRECT: Color = Color::srgb(0.29, 0.85, 0.50);
const BTN_NORMAL: Color = Color::srgb(0.15, 0.15, 0.25);
```

Background color: `#1a1a2e` — `Color::srgb(0.102, 0.102, 0.180)`.

### Bevy Patterns

- **Plugin per module**: Each feature module exports a `pub struct FooPlugin;` implementing `Plugin`.
- **State-scoped systems**: `OnEnter(GameState::X)` for setup, `OnExit` for cleanup. `Update` systems gated with `.run_if(in_state(GameState::X))`.
- **System chaining**: Related update systems use `.chain()` when order matters.
- **Resources**: Game state tracked via `#[derive(Resource)]` structs. Insert with `commands.insert_resource(...)`, remove in cleanup with `commands.remove_resource::<T>()`.
- **Components as markers**: Newtype tuple structs, e.g. `struct RefLineText(usize);`, `struct HudTimer;`.
- **Struct init**: Use `..default()` to fill remaining fields.
- **Entity cleanup**: Always attach `DespawnOnExit(GameState::X)` to root UI nodes.

### Error Handling

- `eprintln!` for non-fatal runtime errors (missing files, parse failures) — the game continues.
- `.unwrap_or_default()` for recoverable deserialization or file I/O.
- `.expect("message")` only when the invariant is a hard programmer error.
- Never `unwrap()` in non-test code. Tests may use `unwrap()`.
- No `panic!` in production paths.

### Testing

Tests live in `#[cfg(test)] mod tests` at the bottom of each source file. No separate test directories.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(kpm: f64) -> PracticeRecord { /* helper */ }

    #[test]
    fn empty_store_best_kpm_is_none() {
        let store = RecordStore::default();
        assert!(store.best_kpm().is_none());
    }
}
```

- Use deterministic seeded RNG for randomness-dependent tests: `StdRng::seed_from_u64(42)`.
- Test pure logic only (splitting, scoring, difficulty generation, serialization). No Bevy ECS tests.
- Test helpers are private `fn` inside the `tests` module.

### Module System

- Each directory has a `mod.rs` re-exporting its submodules with `pub mod foo;`.
- `main.rs` declares top-level modules with `mod audio; mod data;` etc.
- Visibility: items are private by default. Use `pub` only on types and functions consumed by other modules.

## Assets / Data

- Passage files: `assets/texts/{lang}_{grade}_{NNN}.json` — flat directory, auto-scanned at startup.
- JSON schema: `{ id, language, grade, title, author?, content }`.
- Fonts: `assets/fonts/ark-pixel-12px-zh.otf` (Chinese + English pixel font).
- Audio: `assets/audio/{correct_key,wrong_key,line_complete,record_break}.ogg`.
- User records persist to `dirs::data_local_dir()/kb-player/records.json`.

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| bevy | 0.18.1 | Game engine (ECS, rendering, audio, input) |
| serde + serde_json | 1 | JSON serialization for passages and records |
| rand | 0.8 | Random passage selection + difficulty hiding |
| dirs | 6 | Cross-platform user data directory |
