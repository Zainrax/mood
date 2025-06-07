//! AI-controlled entities behavior.
//!
//! This module provides a mood-driven AI system. Entities exhibit complex
//! behaviors such as organic wandering, flocking/repulsion, boundary avoidance,
//! and state-based abilities, all determined by their current `Mood`.

use bevy::prelude::*;
use noisy_bevy::fbm_simplex_2d_seeded;
use rand::Rng;
use std::ops::Range;

use crate::{
    AppSystems, PausableSystems,
    demo::{
        mood::Mood,
        movement::{MovementController, PlayArea},
    },
};

/// Configuration resource for AI behavior parameters
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct AiConfig {
    // Magnetism strengths
    pub cohesion_strength: f32,
    pub separation_strength: f32,
    // Boundary avoidance
    pub avoidance_margin: f32,
    pub avoidance_strength: f32,
    // Rage-specific tuning
    pub rage_charge_aim_duration: f32,
    pub rage_charge_duration: f32,
    pub rage_charge_cooldown: Range<f32>,
    pub rage_max_hits_before_burnout: u32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            cohesion_strength: 0.05,
            separation_strength: 0.01,
            avoidance_margin: 100.0,
            avoidance_strength: 2.0,
            rage_charge_aim_duration: 0.75,
            rage_charge_duration: 1.5,
            rage_charge_cooldown: 5.0..8.0,
            rage_max_hits_before_burnout: 3,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    // Register and initialize the new AiConfig resource
    app.register_type::<AiConfig>();
    app.init_resource::<AiConfig>();

    // Register all AI components
    app.register_type::<AiEntity>();
    app.register_type::<AiWanderState>();
    app.register_type::<AiMagnetism>();
    app.register_type::<AiAction>();

    // Add the new AI systems to the update schedule in a specific order
    app.add_systems(
        Update,
        (
            // 1. Set the base wander/action intent.
            update_ai_behavior,
            // 2. Modify the intent with social flocking/repulsion forces.
            update_ai_magnetism,
            // 3. Add a final, strong force to avoid play area boundaries.
            update_ai_boundary_avoidance,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Marker component for AI-controlled entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct AiEntity;

/// Configuration for AI spawn positions and behavior
pub struct AiSpawnConfig {
    pub positions: Vec<Vec3>,
    pub max_speed: f32,
}

impl Default for AiSpawnConfig {
    fn default() -> Self {
        Self {
            positions: vec![
                // Original positions
                Vec3::new(-200.0, 100.0, 0.0),
                Vec3::new(200.0, -100.0, 0.0),
                Vec3::new(0.0, 200.0, 0.0),
                Vec3::new(-150.0, -150.0, 0.0),
                Vec3::new(300.0, 0.0, 0.0),
                // Additional positions for more interactions
                Vec3::new(-100.0, 0.0, 0.0),
                Vec3::new(100.0, 150.0, 0.0),
                Vec3::new(-250.0, -50.0, 0.0),
                Vec3::new(250.0, 100.0, 0.0),
                Vec3::new(0.0, -200.0, 0.0),
                Vec3::new(-350.0, 0.0, 0.0),
                Vec3::new(350.0, -50.0, 0.0),
                Vec3::new(-50.0, 250.0, 0.0),
                Vec3::new(50.0, -250.0, 0.0),
                Vec3::new(150.0, 50.0, 0.0),
            ],
            max_speed: 350.0,
        }
    }
}

/// Component for mood-based attraction/repulsion forces.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AiMagnetism {
    /// How far this entity can "see" others to react to them.
    pub vision_radius: f32,
    /// The strength of the force to move towards "friendly" neighbors.
    pub cohesion_strength: f32,
    /// The strength of the force to move away from all nearby neighbors to avoid crowding.
    pub separation_strength: f32,
    /// The ideal distance to maintain from other entities.
    pub separation_distance: f32,
}

/// Represents the specific action an AI is currently performing.
#[derive(Debug, Clone, Reflect, PartialEq)]
pub enum AiAction {
    /// Default state: moving around using noise-based wandering.
    Wandering,
    /// A rage-specific state: pausing to lock onto a target.
    Aiming { target: Entity },
    /// A rage-specific state: moving at high speed towards a target's last known position.
    Charging { target_pos: Vec2 },
}

/// State component for AI behavior, incorporating actions and timers.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AiWanderState {
    /// Generic timer for controlling the duration of states (e.g., how long to pause).
    pub state_timer: Timer,
    /// A random seed for Perlin noise to give each entity a unique movement path.
    pub noise_seed: f32,
    /// The current action the AI is performing.
    pub action: AiAction,
    /// Cooldown timer for special abilities like the Rage charge.
    pub ability_cooldown: Timer,
    /// Counter for rage burnout - tracks hits during charge sequence.
    pub charge_hit_count: u32,
}

impl AiWanderState {
    /// Creates a new, default wander state for an AI.
    pub fn new() -> Self {
        let mut rng = rand::rng();
        Self {
            state_timer: Timer::from_seconds(rng.random_range(0.5..1.5), TimerMode::Once),
            noise_seed: rng.random(),
            action: AiAction::Wandering,
            ability_cooldown: Timer::from_seconds(rng.random_range(3.0..6.0), TimerMode::Once),
            charge_hit_count: 0, // Initialize to zero
        }
    }
}

/// System that updates the AI's core action state and base movement intent.
fn update_ai_behavior(
    time: Res<Time>,
    config: Res<AiConfig>, // Get the config resource
    all_moodels: Query<(Entity, &Transform, &Mood)>,
    mut ai_query: Query<
        (
            Entity,
            &Transform,
            &mut MovementController,
            &mut AiWanderState,
            &Mood,
        ),
        With<AiEntity>,
    >,
) {
    let mut rng = rand::rng();

    let potential_targets: Vec<_> = all_moodels
        .iter()
        .filter(|(_, _, mood)| **mood != Mood::Rage)
        .collect();

    for (entity, transform, mut controller, mut wander_state, mood) in &mut ai_query {
        wander_state.state_timer.tick(time.delta());
        wander_state.ability_cooldown.tick(time.delta());

        match wander_state.action.clone() {
            AiAction::Wandering => {
                if *mood == Mood::Rage && wander_state.ability_cooldown.just_finished() {
                    let closest_target = potential_targets
                        .iter()
                        .filter(|(target_entity, _, _)| *target_entity != entity)
                        .min_by_key(|(_, target_transform, _)| {
                            let distance =
                                transform.translation.distance(target_transform.translation);
                            (distance * 100.0) as u32
                        });

                    if let Some((target_entity, _, _)) = closest_target {
                        info!("Rage Moodel {:?} is AIMING at {:?}", entity, target_entity);
                        wander_state.action = AiAction::Aiming {
                            target: *target_entity,
                        };
                        // Use config value for aim duration
                        wander_state
                            .state_timer
                            .set_duration(std::time::Duration::from_secs_f32(
                                config.rage_charge_aim_duration,
                            ));
                        wander_state.state_timer.reset();
                        // Reset hit count for the new charge sequence
                        wander_state.charge_hit_count = 0;
                        controller.intent = Vec2::ZERO;
                        continue;
                    }
                }
                controller.intent = get_wander_intent(mood, &mut wander_state, &time, &mut rng);
            }
            AiAction::Aiming { target } => {
                controller.intent = Vec2::ZERO;
                if wander_state.state_timer.just_finished() {
                    if let Ok((_, target_transform, _)) = all_moodels.get(target) {
                        info!("Rage Moodel {:?} is CHARGING!", entity);
                        wander_state.action = AiAction::Charging {
                            target_pos: target_transform.translation.truncate(),
                        };
                        // Use config value for charge duration
                        wander_state
                            .state_timer
                            .set_duration(std::time::Duration::from_secs_f32(
                                config.rage_charge_duration,
                            ));
                        wander_state.state_timer.reset();
                    } else {
                        wander_state.action = AiAction::Wandering;
                    }
                }
            }
            AiAction::Charging { target_pos } => {
                controller.intent =
                    (target_pos - transform.translation.truncate()).normalize_or_zero();
                if wander_state.state_timer.just_finished() {
                    info!("Rage Moodel {:?} charge complete.", entity);
                    wander_state.action = AiAction::Wandering;
                    // Use config value for cooldown
                    wander_state
                        .ability_cooldown
                        .set_duration(std::time::Duration::from_secs_f32(
                            rng.random_range(config.rage_charge_cooldown.clone()),
                        ));
                    wander_state.ability_cooldown.reset();
                }
            }
        }
    }
}

/// System that applies flocking and repulsion forces, respecting priority actions.
fn update_ai_magnetism(
    config: Res<AiConfig>, // Get the config resource
    mut query: Query<
        (
            Entity,
            &Transform,
            &Mood,
            &mut MovementController,
            &AiWanderState,
            &AiMagnetism,
        ),
        With<AiEntity>,
    >,
    all_moodels: Query<(Entity, &Transform, &Mood)>,
) {
    let positions: Vec<_> = all_moodels.iter().collect();

    for (entity, transform, mood, mut controller, wander_state, magnetism) in &mut query {
        // PRIORITY OVERRIDE: If the AI is not wandering, skip all magnetism.
        if wander_state.action != AiAction::Wandering {
            continue;
        }
        let mut cohesion_vec = Vec2::ZERO;
        let mut separation_vec = Vec2::ZERO;
        let mut friendly_neighbor_count = 0;

        for (other_entity, other_transform, other_mood) in &positions {
            if entity == *other_entity {
                continue;
            }
            let distance = transform
                .translation
                .truncate()
                .distance(other_transform.translation.truncate());

            if distance < magnetism.vision_radius {
                if distance < magnetism.separation_distance {
                    if let Some(dir) = (transform.translation.truncate()
                        - other_transform.translation.truncate())
                    .try_normalize()
                    {
                        separation_vec += dir / (distance + 0.1);
                    }
                }
                let attraction_factor = get_attraction_factor(*mood, **other_mood);
                if attraction_factor != 0.0 {
                    cohesion_vec += (other_transform.translation.truncate()
                        - transform.translation.truncate())
                        * attraction_factor;
                    if attraction_factor > 0.0 {
                        friendly_neighbor_count += 1;
                    }
                }
            }
        }

        if friendly_neighbor_count > 0 {
            cohesion_vec /= friendly_neighbor_count as f32;
        }

        // Use config values for strengths
        let magnetic_force = (cohesion_vec * config.cohesion_strength)
            + (separation_vec * config.separation_strength);
        controller.intent += magnetic_force;
    }
}

/// System to steer AI away from play area edges, respecting priority actions.
fn update_ai_boundary_avoidance(
    play_area: Res<PlayArea>,
    config: Res<AiConfig>, // Get the config resource
    mut query: Query<(&Transform, &mut MovementController, &AiWanderState), With<AiEntity>>,
) {
    let half_size = play_area.size / 2.0;
    let boundaries = Rect::from_center_half_size(play_area.center, half_size);

    for (transform, mut controller, wander_state) in &mut query {
        // PRIORITY OVERRIDE: If the AI is not wandering, skip boundary avoidance.
        if wander_state.action != AiAction::Wandering {
            continue;
        }
        let pos = transform.translation.truncate();
        let mut avoidance_force = Vec2::ZERO;

        // Use config values for margin and strength
        if pos.x < boundaries.min.x + config.avoidance_margin {
            avoidance_force.x = config.avoidance_strength;
        }
        if pos.x > boundaries.max.x - config.avoidance_margin {
            avoidance_force.x = -config.avoidance_strength;
        }
        if pos.y < boundaries.min.y + config.avoidance_margin {
            avoidance_force.y = config.avoidance_strength;
        }
        if pos.y > boundaries.max.y - config.avoidance_margin {
            avoidance_force.y = -config.avoidance_strength;
        }

        // Add the avoidance force to the final intent. This force is strong
        // and should override other behaviors when near a wall.
        controller.intent += avoidance_force;
    }
}

/// Generates a noise vector using different seeds for X and Y axes for more random movement.
fn get_2d_noise(input: f32, octaves: u8, seed: f32) -> Vec2 {
    let noise_x = fbm_simplex_2d_seeded(Vec2::new(input, 0.0), octaves as usize, 0.5, 2.0, seed);
    let noise_y = fbm_simplex_2d_seeded(
        Vec2::new(0.0, input),
        octaves as usize,
        0.5,
        2.0,
        seed + 1000.0,
    );
    Vec2::new(noise_x, noise_y)
}

/// Helper function to calculate the base wandering direction using Perlin noise.
fn get_wander_intent(
    mood: &Mood,
    wander_state: &mut AiWanderState,
    time: &Time,
    rng: &mut impl Rng,
) -> Vec2 {
    let time_input = time.elapsed_secs();
    match mood {
        Mood::Happy => {
            let flock_wander = get_2d_noise(time_input * 0.2, 2, wander_state.noise_seed);
            flock_wander * 0.3
        }
        Mood::Rage => {
            get_2d_noise(time_input * 0.5, 4, wander_state.noise_seed).normalize_or_zero()
        }
        Mood::Calm => {
            get_2d_noise(time_input * 0.1, 1, wander_state.noise_seed).normalize_or_zero() * 0.5
        }
        Mood::Neutral => {
            get_2d_noise(time_input * 0.3, 2, wander_state.noise_seed).normalize_or_zero() * 0.6
        }
        Mood::Sad => {
            if !wander_state.state_timer.finished() {
                get_2d_noise(time_input * 0.1, 1, wander_state.noise_seed).normalize_or_zero() * 0.4
            } else {
                if wander_state.state_timer.just_finished() {
                    wander_state
                        .state_timer
                        .set_duration(std::time::Duration::from_secs_f32(
                            rng.random_range(3.0..6.0),
                        ));
                    wander_state.state_timer.reset();
                }
                Vec2::ZERO
            }
        }
    }
}

/// Helper function to define the magnetic interaction between two moods.
fn get_attraction_factor(my_mood: Mood, other_mood: Mood) -> f32 {
    use Mood::*;
    match (my_mood, other_mood) {
        (Happy, Happy) => 1.0,
        (Happy, Calm) => 0.3,
        (Sad, Sad) => 0.5,
        (Sad, Happy) => -0.8,
        (Sad, Rage) => -1.0,
        (Rage, Rage) => -1.0,
        (Rage, Sad) => -1.0,
        (Rage, _) => 0.2,
        (Calm, Calm) => 0.2,
        (Neutral, _) => -0.1,
        _ => 0.0,
    }
}
