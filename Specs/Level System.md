### Technical Document: Modular & Hot-Reloadable Level System

#### 1. Guiding Principles

*   **Data-Driven First:** Levels are primarily defined as simple JSON files (`.level.json`), allowing for rapid iteration by designers without recompiling the game.
*   **Hot-Reloading:** Changes to level files on disk are automatically detected and reflected in the running game, providing instant feedback. This is powered by `bevy_common_assets`.
*   **Extensible by Design:** The obstacle system uses a tagged enum (`ObstacleKind`). Adding new types of obstacles (e.g., one-way gates, hazards) is a matter of adding a variant to the enum and a corresponding spawn logic branch.
*   **Unified Data Structure:** A single `Level` struct serves as the canonical blueprint for all levels, whether they are loaded from a file or defined programmatically in code.
*   **Clear State Management:** The game uses a state machine (`AssetLoading` -> `Gameplay`) to ensure assets are fully loaded before gameplay begins.

#### 2. Level JSON Format (`*.level.json`)

The JSON structure is designed to be as flat and human-readable as possible. Note the `obstacle` objects, which now have a `type` tag directly at their top level alongside their properties.

**File:** `assets/levels/tutorial_1.level.json`
```json
{
  "name": "The Walled Garden",
  "play_area": [ 900.0, 600.0 ],
  "moodels": [
    { "mood": "Neutral", "position": [ -200.0, 100.0 ] },
    { "mood": "Happy", "position": [ 0.0, 0.0 ] }
  ],
  "obstacles": [
    {
      "position": [ 0.0, 150.0 ],
      "type": "Wall",
      "size": [ 400.0, 20.0 ]
    },
    {
      "position": [ -200.0, -50.0 ],
      "type": "Wall",
      "size": [ 20.0, 300.0 ]
    }
  ],
  "goal_zones": [
    {
      "position": [ 350.0, -200.0 ],
      "size": [ 150.0, 150.0 ],
      "target_mood": "Happy",
      "required_count": 1
    }
  ]
}
```

#### 3. System Architecture

1.  **Request:** A `SpawnLevel` event is sent to request a level (e.g., from a menu).
2.  **Load:** A system listens for this event.
    *   **File:** It uses `asset_server.load()` to get a `Handle<Level>` and places it in a `LevelHandle` resource. It then transitions the game to the `GameState::AssetLoading` state.
    *   **Programmatic:** It constructs the `Level` struct directly, places it in an `ActiveLevel` resource, and transitions the game to `GameState::Gameplay`.
3.  **Process:** A system running during `AssetLoading` waits for the `Assets<Level>` collection to contain the loaded level. Once it appears, the data is copied into the `ActiveLevel` resource, and the state transitions to `GameState::Gameplay`.
4.  **Spawn:** On entering `GameState::Gameplay`, the `spawn_level_entities` system runs once. It reads the data from the `ActiveLevel` resource and populates the world with all necessary entities (Moodels, Obstacles, Zones).
5.  **Hot-Reload:** While in `GameState::Gameplay`, a system listens for `AssetEvent::Modified`. If the currently loaded level asset is modified, it despawns the existing level, re-loads the level data into `ActiveLevel`, and triggers a state transition back into `Gameplay` to force a clean respawn.

---

### Production Code

This code is ready for implementation.

#### Main Application Setup (`main.rs`)

```rust
// main.rs

use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use avian2d::prelude::*;

// Make sure to define these mods in your project structure
// mod asset_tracking;
// mod audio;
// mod demo;
// mod menus;
// mod screens;
// mod theme;

use crate::demo::level::{self, Level}; // Import level module and Level struct

fn main() -> AppExit {
    App::new()
        .add_plugins(AppPlugin)
        .run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins.set(AssetPlugin {
                // Configure asset settings if needed
                ..default()
            }),
        );

        // Add Avian2D physics plugin
        app.add_plugins(PhysicsPlugins::default().with_length_unit(100.0));

        // Add the JsonAssetPlugin for our Level struct
        // It will look for files ending in ".level.json"
        app.add_plugins(JsonAssetPlugin::<Level>::new(&["level.json"]));

        // Add our custom game plugins
        app.add_plugins((
            // asset_tracking::plugin,
            // audio::plugin,
            demo::plugin, // This will contain the level::plugin
            // menus::plugin,
            // screens::plugin,
            // theme::plugin,
        ));

        // Add the game state
        app.init_state::<level::GameState>();

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        // Temporary system to kick off the first level
        app.add_systems(Startup, |mut ev: EventWriter<level::SpawnLevel>| {
            ev.send(level::SpawnLevel::FromFile("levels/tutorial_1.level.json".to_string()));
        });
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2dBundle::default()));
}
```

#### New File: `src/demo/level_library.rs`

```rust
//! A library of programmatically-defined levels.

use super::level::{GoalZoneData, Level, MoodelData, ObstacleData, ObstacleKind};
use crate::demo::mood::Mood;
use bevy::prelude::*;

/// Returns a programmatically defined `Level` based on a unique string ID.
pub fn get_level_by_id(id: &str) -> Option<Level> {
    match id {
        "tutorial_code" => Some(create_tutorial_from_code()),
        _ => None,
    }
}

fn create_tutorial_from_code() -> Level {
    Level {
        name: "Programmatic Tutorial".to_string(),
        play_area: Vec2::new(900.0, 600.0),
        moodels: vec![
            MoodelData {
                mood: Mood::Happy,
                position: Vec2::new(-200.0, 0.0),
            },
        ],
        obstacles: vec![ObstacleData {
            position: Vec2::new(0.0, 0.0),
            kind: ObstacleKind::Wall {
                size: Vec2::new(20.0, 300.0),
            },
        }],
        goal_zones: vec![GoalZoneData {
            position: Vec2::new(350.0, 0.0),
            size: Vec2::new(200.0, 200.0),
            target_mood: Mood::Happy,
            required_count: 1,
        }],
    }
}
```

#### Modified File: `src/demo/level.rs`

```rust
//! Data-driven and programmatic level loading, obstacles, and win conditions.

use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::level_library;
use crate::{
    demo::{
        mood::{spawn_moodel_bundle, Mood, MoodAssets},
        movement::PlayArea,
    },
    AppSystems, PausableSystems,
};

pub(super) fn plugin(app: &mut App) {
    // Register components, resources, and events
    app.register_type::<Obstacle>();
    app.register_type::<GoalZone>();
    app.register_type::<InGoalZone>();
    app.add_event::<SpawnLevel>();

    // Resources to manage level state
    app.init_resource::<LevelHandle>();
    app.init_resource::<ActiveLevel>();

    // Core systems for level lifecycle
    app.add_systems(Update, handle_spawn_requests);
    app.add_systems(
        Update,
        process_loaded_level.run_if(in_state(GameState::AssetLoading)),
    );
    app.add_systems(OnEnter(GameState::Gameplay), spawn_level_entities);
    app.add_systems(OnExit(GameState::Gameplay), teardown_level);

    // Gameplay logic systems
    app.add_systems(
        Update,
        (
            hot_reload_level, // For rapid iteration
            (
                update_moodels_in_zones,
                check_goal_zones_progress,
                check_win_condition.after(check_goal_zones_progress),
            )
                .chain()
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        )
            .run_if(in_state(GameState::Gameplay)),
    );
}

// --- States, Resources, Components & Events ---

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    None, // Or a Menu state
    AssetLoading,
    Gameplay,
}

#[derive(Event, Clone)]
pub enum SpawnLevel {
    FromFile(String),
    Programmatic(String),
}

#[derive(Resource, Default)]
struct LevelHandle(Handle<Level>);

#[derive(Resource, Default)]
struct ActiveLevel(Option<Level>);

#[derive(Component)]
pub struct LevelEntity;
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Obstacle;
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GoalZone {
    pub target_mood: Mood,
    pub required_count: u32,
    pub is_satisfied: bool,
}
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct InGoalZone(Entity);


// --- Core Data Structures (for Serialization) ---

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Level {
    pub name: String,
    pub play_area: Vec2,
    pub moodels: Vec<MoodelData>,
    pub obstacles: Vec<ObstacleData>,
    pub goal_zones: Vec<GoalZoneData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct MoodelData {
    pub mood: Mood,
    pub position: Vec2,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ObstacleData {
    pub position: Vec2,
    #[serde(flatten)]
    pub kind: ObstacleKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[serde(tag = "type")]
pub enum ObstacleKind {
    Wall { size: Vec2 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct GoalZoneData {
    pub position: Vec2,
    pub size: Vec2,
    pub target_mood: Mood,
    pub required_count: u32,
}

// --- Level Lifecycle Systems ---

fn teardown_level(mut commands: Commands, query: Query<Entity, With<LevelEntity>>) {
    info!("Tearing down level...");
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_spawn_requests(
    mut commands: Commands,
    mut spawn_requests: EventReader<SpawnLevel>,
    asset_server: Res<AssetServer>,
    mut level_handle: ResMut<LevelHandle>,
    mut active_level: ResMut<ActiveLevel>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(request) = spawn_requests.read().last() {
        match request {
            SpawnLevel::FromFile(path) => {
                info!("Loading level from file: {}", path);
                level_handle.0 = asset_server.load(path);
                active_level.0 = None; // Clear any previously active level
                next_state.set(GameState::AssetLoading);
            }
            SpawnLevel::Programmatic(id) => {
                info!("Loading programmatic level: {}", id);
                if let Some(level) = level_library::get_level_by_id(id) {
                    active_level.0 = Some(level);
                    level_handle.0 = Handle::default(); // Clear asset handle
                    next_state.set(GameState::Gameplay);
                } else {
                    error!("Could not find programmatic level with ID: {}", id);
                }
            }
        }
        spawn_requests.clear();
    }
}

fn process_loaded_level(
    mut level_assets: ResMut<Assets<Level>>,
    level_handle: Res<LevelHandle>,
    mut active_level: ResMut<ActiveLevel>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(loaded_level) = level_assets.get_mut(&level_handle.0) {
        info!("Level asset loaded, processing...");
        active_level.0 = Some(loaded_level.clone());
        next_state.set(GameState::Gameplay);
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_level_entities(
    mut commands: Commands,
    active_level: Res<ActiveLevel>,
    mut play_area: ResMut<PlayArea>,
    mood_assets: Res<MoodAssets>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(level) = &active_level.0 else {
        error!("Attempted to spawn level, but no active level data was found!");
        return;
    };

    info!("Spawning level entities for: {}", level.name);
    play_area.center = Vec2::ZERO;
    play_area.size = level.play_area;

    // Spawn Moodels
    for data in &level.moodels {
        commands.spawn((
            spawn_moodel_bundle(
                data.mood,
                &mood_assets,
                data.position.extend(0.0),
                350.0, // Default max speed, can be customized later
                time.elapsed_secs(),
            ),
            LevelEntity,
        ));
    }

    // Spawn Obstacles
    for data in &level.obstacles {
        match &data.kind {
            ObstacleKind::Wall { size } => {
                commands.spawn((
                    Name::new("Wall"),
                    Obstacle,
                    LevelEntity,
                    ColorMesh2dBundle {
                        mesh: meshes.add(Rectangle::new(size.x, size.y)).into(),
                        material: materials.add(Color::srgb(0.4, 0.4, 0.5)),
                        transform: Transform::from_xyz(data.position.x, data.position.y, 0.0),
                        ..default()
                    },
                    RigidBody::Static,
                    Collider::rectangle(size.x, size.y),
                ));
            }
        }
    }

    // Spawn Goal Zones
    for data in &level.goal_zones {
        commands.spawn((
            Name::new(format!("{:?} Goal Zone", data.target_mood)),
            GoalZone {
                target_mood: data.target_mood,
                required_count: data.required_count,
                is_satisfied: false,
            },
            LevelEntity,
            ColorMesh2dBundle {
                mesh: meshes.add(Rectangle::new(data.size.x, data.size.y)).into(),
                material: materials.add(data.target_mood.color().with_alpha(0.2)),
                transform: Transform::from_xyz(data.position.x, data.position.y, -1.0),
                ..default()
            },
            RigidBody::Static,
            Collider::rectangle(data.size.x, data.size.y),
            Sensor,
        ));
    }
}

fn hot_reload_level(
    mut asset_events: EventReader<AssetEvent<Level>>,
    level_handle: Res<LevelHandle>,
    mut active_level: ResMut<ActiveLevel>,
    level_assets: Res<Assets<Level>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in asset_events.read() {
        if let AssetEvent::Modified { id } = event {
            if *id == level_handle.0.id() {
                info!("Level asset modified, triggering hot-reload.");
                if let Some(updated_level) = level_assets.get(*id) {
                    active_level.0 = Some(updated_level.clone());
                    // Re-enter state to trigger a clean respawn via OnEnter/OnExit systems
                    next_state.set(GameState::Gameplay);
                }
            }
        }
    }
}


// --- Gameplay Logic Systems ---

fn update_moodels_in_zones(
    mut commands: Commands,
    mut collisions: EventReader<CollisionStarted>,
    moodel_query: Query<Entity, With<Mood>>, // More efficient to just check for marker component
    goal_zone_query: Query<Entity, With<GoalZone>>,
) {
    for CollisionStarted(entity1, entity2) in collisions.read() {
        let (moodel_entity, zone_entity) =
            if moodel_query.get(*entity1).is_ok() && goal_zone_query.get(*entity2).is_ok() {
                (*entity1, *entity2)
            } else if moodel_query.get(*entity2).is_ok() && goal_zone_query.get(*entity1).is_ok() {
                (*entity2, *entity1)
            } else {
                continue;
            };

        // Add InGoalZone component to the moodel. This overwrites any previous one,
        // which is correct behavior if it moves between overlapping zones.
        commands.entity(moodel_entity).insert(InGoalZone(zone_entity));
    }
}

fn check_goal_zones_progress(
    mut zone_query: Query<(Entity, &mut GoalZone, &Handle<ColorMaterial>)>,
    moodel_query: Query<(&Mood, &InGoalZone)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // This could be more efficient, but for a game jam, it's clear and correct.
    // A map could be used for larger numbers of zones.
    let moodels_in_zones: Vec<(Entity, Mood)> = moodel_query
        .iter()
        .map(|(mood, in_zone)| (in_zone.0, *mood))
        .collect();

    for (zone_entity, mut goal_zone, material_handle) in &mut zone_query {
        let current_count = moodels_in_zones
            .iter()
            .filter(|(moodel_zone_entity, moodel_mood)| {
                *moodel_zone_entity == zone_entity && *moodel_mood == goal_zone.target_mood
            })
            .count();

        goal_zone.is_satisfied = current_count as u32 >= goal_zone.required_count;

        // Update visual feedback for the zone
        if let Some(material) = materials.get_mut(material_handle) {
            let target_color = if goal_zone.is_satisfied {
                goal_zone.target_mood.color().with_alpha(0.6)
            } else {
                goal_zone.target_mood.color().with_alpha(0.2)
            };
            material.color = target_color;
        }
    }
}

fn check_win_condition(
    zone_query: Query<&GoalZone>,
    mut level_won: Local<bool>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>
) {
    // Reset the win flag when we enter a new gameplay state
    if current_state.is_changed() {
        *level_won = false;
    }
    if *level_won { return; }

    // If there are no goal zones defined, we can't win the level.
    if zone_query.is_empty() { return; }

    let all_satisfied = zone_query.iter().all(|zone| zone.is_satisfied);

    if all_satisfied {
        *level_won = true;
        info!("🎉 LEVEL COMPLETE! 🎉");
        // Here you would trigger a state change to a victory screen, e.g.:
        // next_state.set(GameState::Victory);
    }
}
```