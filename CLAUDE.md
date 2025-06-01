# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build
```bash
# Development build (with hot-reloading and dev tools)
cargo build
# or using Bevy CLI
bevy build

# Release build
cargo build --release
bevy build --release
```

### Run
```bash
# Run development build (uses dev_native features)
cargo run
bevy run

# Run web build
bevy run web

# Run with specific features
cargo run --no-default-features --features dev
```

### Test
```bash
cargo test
```

### Lint
```bash
# Standard Rust linting
cargo clippy

# Bevy-specific lints
bevy lint
```

### Format
```bash
cargo fmt
```

## Architecture

This is a 2D Bevy game built using the Bevy New 2D template. The project follows a modular plugin-based architecture:

### Core Structure
- **main.rs**: Entry point that sets up the App with all plugins. Configures window settings, asset handling, and defines the main `AppSystems` execution order (TickTimers → RecordInput → Update).
- **Plugin System**: Each major feature is implemented as a Bevy plugin, making the codebase modular and maintainable.

### State Management
- **Screen States**: The game uses a state machine (`Screen` enum) to manage different screens: Splash → Title → Loading → Gameplay
- **Pause State**: A separate `Pause` state that controls whether game systems should run
- **Scoped Entities**: Both states use scoped entities, meaning entities are automatically cleaned up when states change

### Key Modules
- **screens/**: Contains the main game screens (splash, title, loading, gameplay) - each implemented as a plugin with its own state handling
- **menus/**: UI menu systems (main menu, settings, pause menu, credits)
- **demo/**: Game-specific logic including player movement, animation, and level systems
- **audio/**: Audio playback system supporting background music and sound effects
- **theme/**: UI theming system with consistent colors (palette.rs), widget styles, and interaction handling
- **asset_tracking/**: Asset loading and management system
- **dev_tools/**: Development-only debugging tools (only included with `dev` feature)

### System Organization
- Systems are grouped into `AppSystems` sets that run in order: timers first, then input recording, then general updates
- `PausableSystems` set for systems that should pause when the game is paused
- Each plugin typically adds its systems to the appropriate sets

### Asset Management
- Uses Bevy's asset system with `AssetMetaCheck::Never` for web compatibility
- Assets organized in `assets/` directory: audio (music/sound_effects) and images
- Hot-reloading enabled in development builds via `file_watcher` and `embedded_watcher` features

### Platform Support
- Native builds with Wayland support enabled
- Web builds supported with specific profile configurations
- Platform-specific optimizations in Cargo.toml (Linux uses clang/mold linker)