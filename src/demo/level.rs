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
    screens::Screen,
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
        process_loaded_level.run_if(resource_exists::<LevelLoadingState>),
    );
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
            .run_if(in_state(Screen::Gameplay)),
    );
}

// --- Resources, Components & Events ---

/// Resource to track that we're loading a level
#[derive(Resource)]
struct LevelLoadingState;

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

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub play_area: Vec2,
    pub moodels: Vec<MoodelData>,
    pub obstacles: Vec<ObstacleData>,
    pub goal_zones: Vec<GoalZoneData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodelData {
    pub mood: Mood,
    pub position: Vec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObstacleData {
    pub position: Vec2,
    #[serde(flatten)]
    pub kind: ObstacleKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ObstacleKind {
    Wall { size: Vec2 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        commands.entity(entity).despawn();
    }
}

fn handle_spawn_requests(
    mut commands: Commands,
    mut spawn_requests: EventReader<SpawnLevel>,
    asset_server: Res<AssetServer>,
    mut level_handle: ResMut<LevelHandle>,
    mut active_level: ResMut<ActiveLevel>,
) {
    if let Some(request) = spawn_requests.read().last() {
        match request {
            SpawnLevel::FromFile(path) => {
                info!("Loading level from file: {}", path);
                level_handle.0 = asset_server.load(path);
                active_level.0 = None; // Clear any previously active level
                commands.insert_resource(LevelLoadingState);
            }
            SpawnLevel::Programmatic(id) => {
                info!("Loading programmatic level: {}", id);
                if let Some(level) = level_library::get_level_by_id(id) {
                    active_level.0 = Some(level);
                    level_handle.0 = Handle::default(); // Clear asset handle
                    
                    // Spawn level immediately
                    commands.run_system_cached(spawn_level_entities);
                } else {
                    error!("Could not find programmatic level with ID: {}", id);
                }
            }
        }
        spawn_requests.clear();
    }
}

fn process_loaded_level(
    mut commands: Commands,
    mut level_assets: ResMut<Assets<Level>>,
    level_handle: Res<LevelHandle>,
    mut active_level: ResMut<ActiveLevel>,
) {
    if let Some(loaded_level) = level_assets.get_mut(&level_handle.0) {
        info!("Level asset loaded, processing...");
        active_level.0 = Some(loaded_level.clone());
        commands.remove_resource::<LevelLoadingState>();
        
        // Spawn level
        commands.run_system_cached(spawn_level_entities);
    }
}

/// System to spawn level entities - this is now called via one_shot system
pub fn spawn_level_entities(world: &mut World) {
    // First despawn any existing level entities
    let entities_to_despawn: Vec<Entity> = world
        .query_filtered::<Entity, With<LevelEntity>>()
        .iter(world)
        .collect();
    
    for entity in entities_to_despawn {
        world.entity_mut(entity).despawn();
    }

    // Get resources we need
    let active_level = world.resource::<ActiveLevel>();
    let Some(level) = &active_level.0 else {
        error!("Attempted to spawn level, but no active level data was found!");
        return;
    };
    let level = level.clone(); // Clone to avoid borrow issues

    info!("Spawning level entities for: {}", level.name);
    
    // Update play area
    let mut play_area = world.resource_mut::<PlayArea>();
    play_area.center = Vec2::ZERO;
    play_area.size = level.play_area;

    // Get other resources
    let mood_assets = world.resource::<MoodAssets>().clone();
    let time = world.resource::<Time>();
    let elapsed_secs = time.elapsed_secs();

    // Spawn Moodels
    for data in &level.moodels {
        world.spawn((
            spawn_moodel_bundle(
                data.mood,
                &mood_assets,
                data.position.extend(0.0),
                350.0, // Default max speed, can be customized later
                elapsed_secs,
            ),
            LevelEntity,
            StateScoped(Screen::Gameplay),
        ));
    }

    // Spawn Obstacles
    for data in &level.obstacles {
        match &data.kind {
            ObstacleKind::Wall { size } => {
                let mut meshes = world.resource_mut::<Assets<Mesh>>();
                let mesh = meshes.add(Rectangle::new(size.x, size.y));
                
                let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
                let material = materials.add(Color::srgb(0.4, 0.4, 0.5));
                
                world.spawn((
                    Name::new("Wall"),
                    Obstacle,
                    LevelEntity,
                    StateScoped(Screen::Gameplay),
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    Transform::from_xyz(data.position.x, data.position.y, 0.0),
                    RigidBody::Static,
                    Collider::rectangle(size.x, size.y),
                ));
            }
        }
    }

    // Spawn Goal Zones
    for data in &level.goal_zones {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Rectangle::new(data.size.x, data.size.y));
        
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let material = materials.add(data.target_mood.color().with_alpha(0.2));
        
        world.spawn((
            Name::new(format!("{:?} Goal Zone", data.target_mood)),
            GoalZone {
                target_mood: data.target_mood,
                required_count: data.required_count,
                is_satisfied: false,
            },
            LevelEntity,
            StateScoped(Screen::Gameplay),
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(data.position.x, data.position.y, -1.0),
            RigidBody::Static,
            Collider::rectangle(data.size.x, data.size.y),
            Sensor,
        ));
    }
}

fn hot_reload_level(
    mut commands: Commands,
    mut asset_events: EventReader<AssetEvent<Level>>,
    level_handle: Res<LevelHandle>,
    mut active_level: ResMut<ActiveLevel>,
    level_assets: Res<Assets<Level>>,
) {
    for event in asset_events.read() {
        if let AssetEvent::Modified { id } = event {
            if *id == level_handle.0.id() {
                info!("Level asset modified, triggering hot-reload.");
                if let Some(updated_level) = level_assets.get(*id) {
                    active_level.0 = Some(updated_level.clone());
                    // Re-spawn level
                    commands.run_system_cached(spawn_level_entities);
                }
            }
        }
    }
}

// --- Gameplay Logic Systems ---

fn update_moodels_in_zones(
    mut commands: Commands,
    mut collisions: EventReader<CollisionStarted>,
    moodel_query: Query<Entity, With<Mood>>,
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

        // Add InGoalZone component to the moodel
        commands.entity(moodel_entity).insert(InGoalZone(zone_entity));
    }
}

fn check_goal_zones_progress(
    mut zone_query: Query<(Entity, &mut GoalZone, &MeshMaterial2d<ColorMaterial>)>,
    moodel_query: Query<(&Mood, &InGoalZone)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
) {
    // If there are no goal zones defined, we can't win the level.
    if zone_query.is_empty() { return; }

    let all_satisfied = zone_query.iter().all(|zone| zone.is_satisfied);

    if all_satisfied && !*level_won {
        *level_won = true;
        info!("🎉 LEVEL COMPLETE! 🎉");
        // TODO: Trigger victory screen or next level
    }
}

// --- Backwards Compatibility ---

/// Legacy function to spawn the default level
/// This is called from gameplay.rs OnEnter(Screen::Gameplay)
pub fn spawn_level(mut ev: EventWriter<SpawnLevel>) {
    // Load the tutorial level from JSON file
    ev.write(SpawnLevel::FromFile("levels/tutorial_1.level.json".to_string()));
    
    // Alternative: Load a programmatic level
    // ev.write(SpawnLevel::Programmatic("tutorial_code".to_string()));
}