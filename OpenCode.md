# OpenCode.md

## Commands
- Build: `cargo build` (dev), `cargo build --release` (prod)
- Run: `cargo run` (dev), `bevy run web`
- Test: `cargo test [<test_name_or_path>]` (e.g., `cargo test specific_test_fn` or `cargo test module::specific_test_fn`)
- Lint: `cargo clippy && bevy lint`
- Format: `cargo fmt`

## Code Style & Conventions (Bevy & Rust)
- **Primary Source**: Refer to `CLAUDE.md` for detailed architecture and project structure.
- **Framework**: Bevy game engine, utilizing a modular plugin-based architecture.
- **State Management**: Game flow is managed by the `Screen` enum and a separate `Pause` state. Entities are typically state-scoped.
- **Imports**: Group imports: `bevy::prelude::*` is common, followed by external crates, then project-specific modules (`crate::...`).
- **Formatting**: Strictly adhere to `cargo fmt` output.
- **Naming Conventions**:
    - Types (Structs, Enums, Traits): `PascalCase`
    - Functions, methods, variables, modules: `snake_case`
    - Constants: `UPPER_SNAKE_CASE`
- **Error Handling**: Employ standard Rust `Result<T, E>` and `Option<T>`. Use Bevy's logging macros (`info!`, `warn!`, `error!`) for diagnostics.
- **Bevy ECS**: Leverage Bevy's Entity Component System: `Component`, `Resource`, `System`, `Event`.
- **Bevy Systems**: Organize systems into `AppSystems` sets (e.g., `TickTimers`, `RecordInput`, `Update`) and `PausableSystems` for systems that respect the `Pause` state.
- **Modularity**: Implement distinct features as plugins within their respective modules (e.g., `src/screens/`, `src/menus/`, `src/demo/`).
- **Comments**: Use `///` for doc comments on public APIs. Use `//` for explanations of non-obvious implementation details.
