Technical Implementation Guide: Advanced Moodel AI
Document Version: 1.0
Author: AI Assistant
Date: 2023-10-27
1. Overview
This document outlines the implementation of an advanced AI system for Moodels. The goal is to replace the current simple wandering logic with a more dynamic, stateful, and expressive system where a Moodel's behavior is directly driven by its Mood.
The new system introduces three core features:
Mood-Driven Movement: Moodels will use Perlin noise for organic, meandering paths. The characteristics of their movement (speed, complexity, pauses) will change based on their current Mood.
Mood-Based Magnetism: Moodels will exhibit flocking and repulsion behaviors. Happy Moodels will group together, while enraged ones will push others away, creating dynamic social interactions.
Stateful Abilities: Moodels now have a simple state machine. This is showcased with the Rage mood, which will now periodically pause, aim at another Moodel, and execute a high-speed "charge."
2. Project Setup
A new dependency is required for Perlin noise generation.
Action: Add the noisy_bevy crate to your project's Cargo.toml file.
# In your [dependencies] section
noisy_bevy = "0.5.0"
Use code with caution.
Toml
3. AI System Implementation (demo/ai.rs)
The demo/ai.rs file will be completely replaced with the following code. This new implementation is self-contained and includes all the logic for the new behaviors.
Action: Replace the entire contents of src/demo/ai.rs with the code below.
//! AI-controlled entities behavior.
//!
//! This module provides a mood-driven AI system. Entities exhibit complex
//! behaviors such as organic wandering, flocking/repulsion, and state-based
//! abilities, all determined by their current `Mood`.

use bevy::prelude::*;
use noisy_bevy::{fbm_simplex_2d_seeded};
use rand::Rng;

use crate::{
    demo::{mood::Mood, movement::MovementController},
    AppSystems, PausableSystems,
};

pub(super) fn plugin(app: &mut App) {
    // Register all new and modified components
    app.register_type::<AiEntity>();
    app.register_type::<AiWanderState>();
    app.register_type::<AiMagnetism>();
    app.register_type::<AiAction>();

    // Add the new AI systems to the update schedule
    app.add_systems(
        Update,
        (
            // The behavior system runs first to set the base intent and handle state changes.
            update_ai_behavior,
            // The magnetism system runs after to modify the intent with social forces.
            update_ai_magnetism.after(update_ai_behavior),
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Marker component for AI-controlled entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct AiEntity;

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
}

impl AiWanderState {
    /// Creates a new, default wander state for an AI.
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            state_timer: Timer::from_seconds(rng.gen_range(0.5..1.5), TimerMode::Once),
            noise_seed: rng.gen(),
            action: AiAction::Wandering,
            // Initial cooldown so they don't all charge at the start.
            ability_cooldown: Timer::from_seconds(rng.gen_range(3.0..6.0), TimerMode::Once),
        }
    }
}

/// System that updates the AI's core action state and base movement intent.
/// This acts as a state machine, especially for the Rage mood's charge ability.
fn update_ai_behavior(
    time: Res<Time>,
    all_moodels: Query<(Entity, &Transform, &Mood)>,
    mut ai_query: Query<(Entity, &Transform, &mut MovementController, &mut AiWanderState, &Mood), With<AiEntity>>,
) {
    let mut rng = rand::thread_rng();

    // Collect potential targets for the Rage charge to avoid query conflicts.
    let potential_targets: Vec<_> = all_moodels.iter()
        .filter(|(_, _, mood)| **mood != Mood::Rage)
        .collect();

    for (entity, transform, mut controller, mut wander_state, mood) in &mut ai_query {
        // Tick timers
        wander_state.state_timer.tick(time.delta());
        wander_state.ability_cooldown.tick(time.delta());

        // --- AI STATE MACHINE ---
        match &mut wander_state.action {
            AiAction::Wandering => {
                // If this is a Rage moodel and its ability is ready, look for a target.
                if *mood == Mood::Rage && wander_state.ability_cooldown.just_finished() {
                    let closest_target = potential_targets.iter()
                        .filter(|(target_entity, _, _)| *target_entity != entity)
                        .min_by_key(|(_, target_transform, _)| {
                            let distance = transform.translation.distance(target_transform.translation);
                            (distance * 100.0) as u32
                        });

                    if let Some((target_entity, _, _)) = closest_target {
                        info!("Rage Moodel {:?} is AIMING at {:?}", entity, target_entity);
                        wander_state.action = AiAction::Aiming { target: *target_entity };
                        wander_state.state_timer.set_duration(std::time::Duration::from_secs_f32(0.75));
                        wander_state.state_timer.reset();
                        controller.intent = Vec2::ZERO; // Stop moving
                        continue; // Skip wander logic this frame
                    }
                }

                // --- Default Wander Logic ---
                // (This code runs if the entity is in the Wandering state)
                let base_intent = get_wander_intent(mood, &mut wander_state, &time, &mut rng);
                controller.intent = base_intent;
            }

            AiAction::Aiming { target } => {
                controller.intent = Vec2::ZERO; // Stay still
                if wander_state.state_timer.just_finished() {
                    if let Ok((_, target_transform, _)) = all_moodels.get(*target) {
                        info!("Rage Moodel {:?} is CHARGING!", entity);
                        wander_state.action = AiAction::Charging {
                            target_pos: target_transform.translation.truncate(),
                        };
                        wander_state.state_timer.set_duration(std::time::Duration::from_secs_f32(1.5));
                        wander_state.state_timer.reset();
                    } else {
                        // Target disappeared, go back to wandering.
                        wander_state.action = AiAction::Wandering;
                    }
                }
            }

            AiAction::Charging { target_pos } => {
                // Move with great purpose towards the last known position.
                let direction = (*target_pos - transform.translation.truncate()).normalize_or_zero();
                controller.intent = direction;

                if wander_state.state_timer.just_finished() {
                    info!("Rage Moodel {:?} charge complete.", entity);
                    wander_state.action = AiAction::Wandering;
                    wander_state.ability_cooldown.set_duration(std::time::Duration::from_secs_f32(rng.gen_range(5.0..8.0)));
                    wander_state.ability_cooldown.reset();
                }
            }
        }
    }
}

/// System that applies flocking and repulsion forces to AI entities.
fn update_ai_magnetism(
    mut query: Query<(Entity, &Transform, &Mood, &mut MovementController, &AiMagnetism), With<AiEntity>>,
    all_moodels: Query<(Entity, &Transform, &Mood)>,
) {
    // Collect all positions into a Vec to safely iterate while modifying the query.
    // This avoids the classic "cannot borrow Query as mutable more than once" error.
    let positions: Vec<_> = all_moodels.iter().collect();

    for (entity, transform, mood, mut controller, magnetism) in &mut query {
        let mut cohesion_vec = Vec2::ZERO;
        let mut separation_vec = Vec2::ZERO;
        let mut friendly_neighbor_count = 0;

        for (other_entity, other_transform, other_mood) in &positions {
            if entity == *other_entity { continue; }

            let distance = transform.translation.truncate().distance(other_transform.translation.truncate());

            if distance < magnetism.vision_radius {
                // 1. Separation: Push away from all neighbors to avoid crowding.
                if distance < magnetism.separation_distance {
                    if let Some(dir) = (transform.translation.truncate() - other_transform.translation.truncate()).try_normalize() {
                        // Force is stronger the closer the entities are.
                        separation_vec += dir / (distance + 0.1);
                    }
                }

                // 2. Cohesion/Repulsion: Attract or repel based on mood interaction.
                let attraction_factor = get_attraction_factor(*mood, *other_mood);
                if attraction_factor != 0.0 {
                    cohesion_vec += (other_transform.translation.truncate() - transform.translation.truncate()) * attraction_factor;
                    if attraction_factor > 0.0 {
                        friendly_neighbor_count += 1;
                    }
                }
            }
        }

        // Average the cohesion vector to get the center of mass of "friends".
        if friendly_neighbor_count > 0 {
            cohesion_vec /= friendly_neighbor_count as f32;
        }

        // Combine the forces.
        let magnetic_force = (cohesion_vec * magnetism.cohesion_strength)
                           + (separation_vec * magnetism.separation_strength);
        
        // Add the magnetic force to the base wander intent and re-normalize.
        controller.intent = (controller.intent + magnetic_force).normalize_or_zero();
    }
}

/// Helper function to calculate the base wandering direction using Perlin noise.
fn get_wander_intent(
    mood: &Mood,
    wander_state: &mut AiWanderState,
    time: &Time,
    rng: &mut impl Rng,
) -> Vec2 {
    let mut intent = Vec2::ZERO;
    match mood {
        Mood::Happy | Mood::Rage => {
            let noise_input = time.elapsed_secs() * 0.5;
            let noise = fbm_simplex_2d_seeded(Vec2::splat(noise_input), 4, 0.5, 2.0, wander_state.noise_seed);
            intent = noise.normalize_or_zero();
        }
        Mood::Calm | Mood::Neutral => {
            if !wander_state.state_timer.finished() {
                let noise_input = time.elapsed_secs() * 0.2;
                let noise = fbm_simplex_2d_seeded(Vec2::splat(noise_input), 2, 0.5, 2.0, wander_state.noise_seed);
                intent = noise.normalize_or_zero();
            } else if wander_state.state_timer.just_finished() {
                wander_state.state_timer.set_duration(std::time::Duration::from_secs_f32(rng.gen_range(2.0..4.0)));
                wander_state.state_timer.reset();
            }
        }
        Mood::Sad => {
            if !wander_state.state_timer.finished() {
                let noise_input = time.elapsed_secs() * 0.1;
                let noise = fbm_simplex_2d_seeded(Vec2::splat(noise_input), 1, 0.5, 2.0, wander_state.noise_seed);
                intent = noise.normalize_or_zero();
            } else if wander_state.state_timer.just_finished() {
                wander_state.state_timer.set_duration(std::time::Duration::from_secs_f32(rng.gen_range(3.0..6.0)));
                wander_state.state_timer.reset();
            }
        }
    }
    intent
}

/// Helper function to define the magnetic interaction between two moods.
/// Returns: > 0 for attraction, < 0 for repulsion, 0 for neutral.
fn get_attraction_factor(my_mood: Mood, other_mood: Mood) -> f32 {
    use Mood::*;
    match (my_mood, other_mood) {
        // Happy loves company
        (Happy, Happy) => 1.0,
        (Happy, Calm)  => 0.3,
        // Sad huddles with other Sad, but avoids energetic moods
        (Sad, Sad)     => 0.5,
        (Sad, Happy)   => -0.8,
        (Sad, Rage)    => -1.0,
        // Rage wants to be alone
        (Rage, _) if my_mood != other_mood => -1.0,
        // Calm is content
        (Calm, Calm)   => 0.2,
        _ => 0.0,
    }
}
Use code with caution.
Rust
4. Spawning Logic Update (mood.rs)
The spawn_moodel_bundle function needs to be updated to add the new AiMagnetism and AiWanderState components to each Moodel.
Action: In src/demo/mood.rs, modify the spawn_moodel_bundle function as shown below. You will also need to add the new imports at the top of the file.
Add these imports to the top of src/demo/mood.rs:
// ... existing imports
use crate::demo::ai::{AiEntity, AiWanderState, AiMagnetism};
// ...
Use code with caution.
Rust
Replace the spawn_moodel_bundle function with this new version:
/// Creates a bundle for spawning a Moodel with a specific mood.
pub fn spawn_moodel_bundle(
    mood: Mood,
    mood_assets: &MoodAssets,
    position: Vec3,
    max_speed: f32,
    current_time: f32,
) -> impl Bundle {
    // Note: The MoodelBundle struct itself does not need to change,
    // as we are adding the new components directly here.
    (
        MoodelBundle {
            name: Name::new(format!("{:?} Moodel", mood)),
            ai_entity: AiEntity,
            mood,
            mood_entity: MoodEntity {
                isolation_timer: Timer::from_seconds(3.0, TimerMode::Repeating),
                mood_stability: 0.0,
                last_interaction_time: current_time,
            },
            ai_wander: AiWanderState::new(), // This is now handled by the tuple below
            sprite: Sprite {
                image: mood_assets.get_sprite(mood),
                color: mood.color(),
                custom_size: Some(Vec2::new(134.0, 208.0)),
                ..default()
            },
            transform: Transform::from_translation(position).with_scale(Vec3::splat(0.5)),
            movement_controller: MovementController {
                max_speed: max_speed * mood.speed_multiplier(),
                ..default()
            },
            play_area_bounded: PlayAreaBounded {
                restitution: match mood {
                    Mood::Happy => 0.9,
                    Mood::Rage => 0.7,
                    Mood::Sad => 0.3,
                    _ => 0.6,
                },
            },
            movement_smoothing: MovementSmoothing {
                acceleration: match mood {
                    Mood::Happy => 1000.0,
                    Mood::Rage => 2500.0, // Rage needs very high acceleration for charging
                    Mood::Calm => 400.0,
                    Mood::Sad => 300.0,
                    _ => 600.0,
                },
                deceleration: match mood {
                    Mood::Rage => 1500.0,
                    Mood::Happy => 1200.0,
                    Mood::Calm => 1000.0,
                    Mood::Sad => 600.0,
                    _ => 1000.0,
                },
                ..default()
            },
            // Physics components...
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(50.0),
            linear_velocity: LinearVelocity::default(),
            angular_velocity: AngularVelocity::default(),
            collision_layers: CollisionLayers::default(),
            collision_events: CollisionEventsEnabled,
            restitution: Restitution::new(match mood {
                Mood::Happy => 0.9,
                Mood::Rage => 0.7,
                Mood::Sad => 0.3,
                _ => 0.6,
            }),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            mass: Mass(2.0),
            friction: Friction::new(0.1),
            gravity_scale: GravityScale(0.0),
            external_force: ExternalForce::default(),
        },
        // Add the new AI components directly to the entity
        AiWanderState::new(),
        AiMagnetism {
            vision_radius: 250.0,
            cohesion_strength: 0.05,
            separation_strength: 0.5,
            separation_distance: 80.0,
        },
    )
}
Use code with caution.
Rust
Note: The original AiWanderState field in MoodelBundle can be removed or ignored, as we are now adding the component directly in the spawning function's returned tuple. For cleanliness, it is recommended to remove it from the MoodelBundle struct definition.
5. Conclusion
After implementing these changes, the simulation will feature significantly more intelligent and expressive AI. Moodels will behave in ways that are visually consistent with their emotional state, leading to a more dynamic and engaging experience. The system is designed to be extensible, allowing for new actions and social behaviors to be added in the future.