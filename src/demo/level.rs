//! Data-driven and programmatic level loading, obstacles, and win conditions.
//! Includes enhanced visuals, real-time feedback, and robust event-driven zone tracking.

use avian2d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::level_library;
use crate::{
    AppSystems, PausableSystems,
    demo::{
        mood::{Mood, MoodAssets, spawn_moodel_bundle},
        movement::PlayArea,
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Register components, resources, and events
    app.register_type::<Obstacle>();
    app.register_type::<GoalZone>();
    app.register_type::<PulseAnimation>();
    app.register_type::<AnimateScale>();
    app.add_event::<SpawnLevel>();

    // Resources to manage level state
    app.init_resource::<LevelHandle>();
    app.init_resource::<ActiveLevel>();
    app.init_resource::<FontHandle>();

    // Core systems for level lifecycle
    app.add_systems(Update, handle_spawn_requests);
    app.add_systems(
        Update,
        process_loaded_level.run_if(resource_exists::<LevelLoadingState>),
    );

    // Gameplay logic systems
    app.add_systems(
        Update,
        (
            hot_reload_level,
            (
                handle_zone_collisions,
                // --- THE FIX: Split the conflicting system into two ---
                update_zone_state.after(handle_zone_collisions),
                update_zone_visuals.after(update_zone_state),
                // ---
                check_win_condition.after(update_zone_visuals),
                // Animation systems
                animate_scale_pop,
                pulse_zone_animation,
            )
                .chain()
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

// --- Resources, Components & Events ---

#[derive(Resource, Default)]
pub struct FontHandle(pub Handle<Font>);

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

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct GoalZone {
    pub target_mood: Mood,
    pub required_count: u32,
    pub current_count: u32,
    pub is_satisfied: bool,
    pub entities_inside: HashSet<Entity>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GoalZoneText;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PulseAnimation {
    pub initial_alpha: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AnimateScale {
    pub timer: Timer,
    pub initial_scale: Vec3,
}

// --- Core Data Structures ---

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
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_spawn_requests(
    mut commands: Commands,
    mut spawn_requests: EventReader<SpawnLevel>,
    asset_server: Res<AssetServer>,
    mut level_handle: ResMut<LevelHandle>,
    mut active_level: ResMut<ActiveLevel>,
) {
    if let Some(request) = spawn_requests.read().last().cloned() {
        match request {
            SpawnLevel::FromFile(path) => {
                info!("Loading level from file: {}", path);
                level_handle.0 = asset_server.load(path);
                active_level.0 = None;
                commands.insert_resource(LevelLoadingState);
            }
            SpawnLevel::Programmatic(id) => {
                info!("Loading programmatic level: {}", id);
                if let Some(level) = level_library::get_level_by_id(&id) {
                    active_level.0 = Some(level);
                    level_handle.0 = Handle::default();
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
    if let Some(loaded_level) = level_assets.get(&level_handle.0) {
        info!("Level asset loaded, processing...");
        active_level.0 = Some(loaded_level.clone());
        commands.remove_resource::<LevelLoadingState>();
        commands.run_system_cached(spawn_level_entities);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_level_entities(world: &mut World) {
    // First despawn any existing level entities
    let entities_to_despawn: Vec<Entity> = world
        .query_filtered::<Entity, With<LevelEntity>>()
        .iter(world)
        .collect();

    for entity in entities_to_despawn {
        world.entity_mut(entity).despawn_recursive();
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
    let font_handle = world.resource::<FontHandle>().0.clone();

    // Spawn Moodels
    for data in &level.moodels {
        world.spawn((
            spawn_moodel_bundle(
                data.mood,
                &mood_assets,
                data.position.extend(0.0),
                350.0, // Default max speed
                elapsed_secs,
            ),
            LevelEntity,
            StateScoped(Screen::Gameplay),
        ));
    }

    // Spawn Obstacles with enhanced visuals
    for data in &level.obstacles {
        match &data.kind {
            ObstacleKind::Wall { size } => {
                let border_thickness = 4.0;
                let parent_entity = world
                    .spawn((
                        Name::new("Wall"),
                        Obstacle,
                        LevelEntity,
                        StateScoped(Screen::Gameplay),
                        Transform::from_xyz(data.position.x, data.position.y, 0.0),
                        Visibility::default(),
                        RigidBody::Static,
                        Collider::rectangle(size.x, size.y),
                    ))
                    .id();

                // Border (slightly larger, darker)
                let mut meshes = world.resource_mut::<Assets<Mesh>>();
                let border_mesh = meshes.add(Rectangle::new(size.x, size.y));
                let inner_mesh = meshes.add(Rectangle::new(
                    size.x - border_thickness,
                    size.y - border_thickness,
                ));

                let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
                let border_material = materials.add(Color::srgb(0.25, 0.25, 0.3));
                let inner_material = materials.add(Color::srgb(0.4, 0.4, 0.5));

                // Spawn border as child
                let border_entity = world
                    .spawn((
                        Mesh2d(border_mesh),
                        MeshMaterial2d(border_material),
                        Transform::from_xyz(0.0, 0.0, 0.0),
                    ))
                    .id();

                // Spawn inner fill as child
                let inner_entity = world
                    .spawn((
                        Mesh2d(inner_mesh),
                        MeshMaterial2d(inner_material),
                        Transform::from_xyz(0.0, 0.0, 0.1),
                    ))
                    .id();

                // Set up parent-child relationships
                world
                    .entity_mut(parent_entity)
                    .add_child(border_entity)
                    .add_child(inner_entity);
            }
        }
    }

    // Spawn Goal Zones with child text
    for data in &level.goal_zones {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Rectangle::new(data.size.x, data.size.y));

        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let material = materials.add(data.target_mood.color().with_alpha(0.2));

        let zone_entity = world
            .spawn((
                Name::new(format!("{:?} Goal Zone", data.target_mood)),
                GoalZone {
                    target_mood: data.target_mood,
                    required_count: data.required_count,
                    current_count: 0,
                    is_satisfied: false,
                    entities_inside: HashSet::new(),
                },
                LevelEntity,
                StateScoped(Screen::Gameplay),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(data.position.x, data.position.y, -1.0),
                RigidBody::Static,
                Collider::rectangle(data.size.x, data.size.y),
                Sensor,
            ))
            .id();

        // Spawn the text as a child of the zone
        let text_entity = world
            .spawn((
                GoalZoneText,
                Text2d::new(format!("0 / {}", data.required_count)),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE.with_alpha(0.6)),
                TextLayout::new_with_justify(JustifyText::Center),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ))
            .id();

        // Set up parent-child relationship
        world.entity_mut(zone_entity).add_child(text_entity);
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
                    commands.run_system_cached(spawn_level_entities);
                }
            }
        }
    }
}

// --- Gameplay Logic and Visual Feedback Systems ---

/// Handles both CollisionStarted and CollisionEnded events for zones.
fn handle_zone_collisions(
    mut commands: Commands,
    mut started: EventReader<CollisionStarted>,
    mut ended: EventReader<CollisionEnded>,
    moodel_query: Query<(Entity, &Mood, &Transform)>,
    mut zone_query: Query<(Entity, &mut GoalZone)>,
) {
    // Handle entities entering the zone
    for CollisionStarted(entity1, entity2) in started.read() {
        let (moodel_entity, zone_entity) =
            if moodel_query.get(*entity1).is_ok() && zone_query.get(*entity2).is_ok() {
                (*entity1, *entity2)
            } else if moodel_query.get(*entity2).is_ok() && zone_query.get(*entity1).is_ok() {
                (*entity2, *entity1)
            } else {
                continue;
            };

        if let Ok((_, mut goal_zone)) = zone_query.get_mut(zone_entity) {
            goal_zone.entities_inside.insert(moodel_entity);
            if let Ok((_, mood, transform)) = moodel_query.get(moodel_entity) {
                if *mood == goal_zone.target_mood {
                    commands.entity(moodel_entity).insert(AnimateScale {
                        timer: Timer::from_seconds(0.25, TimerMode::Once),
                        initial_scale: transform.scale,
                    });
                }
            }
        }
    }

    // Handle entities leaving the zone
    for CollisionEnded(entity1, entity2) in ended.read() {
        let (moodel_entity, zone_entity) =
            if moodel_query.get(*entity1).is_ok() && zone_query.get(*entity2).is_ok() {
                (*entity1, *entity2)
            } else if moodel_query.get(*entity2).is_ok() && zone_query.get(*entity1).is_ok() {
                (*entity2, *entity1)
            } else {
                continue;
            };

        if let Ok((_, mut goal_zone)) = zone_query.get_mut(zone_entity) {
            goal_zone.entities_inside.remove(&moodel_entity);
        }
    }
}

/// **NEW System 1:** Recalculates the score and satisfaction state for each zone.
/// This system performs the MUTABLE operations on GoalZone.
fn update_zone_state(mut zone_query: Query<&mut GoalZone>, moodel_query: Query<&Mood>) {
    for mut goal_zone in &mut zone_query {
        let mut current_valid_count = 0;
        for entity_in_zone in &goal_zone.entities_inside {
            if let Ok(mood) = moodel_query.get(*entity_in_zone) {
                if *mood == goal_zone.target_mood {
                    current_valid_count += 1;
                }
            }
        }
        goal_zone.current_count = current_valid_count;
        goal_zone.is_satisfied = goal_zone.current_count >= goal_zone.required_count;
    }
}

/// **NEW System 2:** Updates the visuals (text, color) based on the pre-calculated state.
/// This system only performs IMMUTABLE reads of GoalZone.
fn update_zone_visuals(
    mut commands: Commands,
    // Note: This query now has an immutable `&GoalZone`
    zone_query: Query<
        (Entity, &GoalZone, &MeshMaterial2d<ColorMaterial>, &Children),
        Changed<GoalZone>,
    >,
    mut text_query: Query<(&mut Text2d, &mut TextColor), With<GoalZoneText>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (zone_entity, goal_zone, material_handle, children) in &zone_query {
        // Update text
        for child in children.iter() {
            if let Ok((mut text, mut text_color)) = text_query.get_mut(child) {
                **text = format!("{} / {}", goal_zone.current_count, goal_zone.required_count);
                text_color.0 = if goal_zone.is_satisfied {
                    Color::BLACK
                } else {
                    Color::BLACK.with_alpha(0.6)
                };
            }
        }

        // Update zone background and add/remove pulse animation
        if let Some(material) = materials.get_mut(material_handle) {
            let initial_alpha = if goal_zone.is_satisfied { 0.6 } else { 0.2 };
            material.color = goal_zone.target_mood.color().with_alpha(initial_alpha);

            if goal_zone.is_satisfied {
                commands
                    .entity(zone_entity)
                    .insert(PulseAnimation { initial_alpha });
            } else {
                commands.entity(zone_entity).remove::<PulseAnimation>();
            }
        }
    }
}

fn check_win_condition(zone_query: Query<&GoalZone>, mut level_won: Local<bool>) {
    // If there are no goal zones defined, we can't win the level.
    if zone_query.is_empty() {
        return;
    }

    let all_satisfied = zone_query.iter().all(|zone| zone.is_satisfied);

    if all_satisfied && !*level_won {
        *level_won = true;
        info!("🎉 LEVEL COMPLETE! 🎉");
        // TODO: Trigger victory screen or next level
    }
}

// --- Animation Systems ---

fn animate_scale_pop(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut AnimateScale)>,
) {
    for (entity, mut transform, mut anim) in &mut query {
        anim.timer.tick(time.delta());
        let progress = anim.timer.fraction();
        // A simple parabola: goes from 0 to 1 back to 0
        let pop = 1.0 + ((-4.0 * progress * progress) + (4.0 * progress)) * 0.2;
        transform.scale = anim.initial_scale * pop;

        if anim.timer.finished() {
            transform.scale = anim.initial_scale;
            commands.entity(entity).remove::<AnimateScale>();
        }
    }
}

fn pulse_zone_animation(
    time: Res<Time>,
    query: Query<(&MeshMaterial2d<ColorMaterial>, &PulseAnimation)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (material_handle, anim) in &query {
        if let Some(material) = materials.get_mut(material_handle) {
            // Pulse alpha between initial_alpha and initial_alpha + 0.2
            let pulse = (time.elapsed_secs() * 3.0).sin() * 0.5 + 0.5; // Ranges from 0 to 1
            material.color.set_alpha(anim.initial_alpha + pulse * 0.2);
        }
    }
}

// --- Backwards Compatibility ---

/// Legacy function to spawn the default level
/// This is called from gameplay.rs OnEnter(Screen::Gameplay)
pub fn spawn_level(mut ev: EventWriter<SpawnLevel>) {
    // Load the tutorial level from JSON file
    ev.write(SpawnLevel::FromFile(
        "levels/tutorial_1.level.json".to_string(),
    ));

    // Alternative: Load a programmatic level
    // ev.write(SpawnLevel::Programmatic("tutorial_code".to_string()));
}
