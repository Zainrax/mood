## Technical Implementation Guide: Advanced Moodel AI (v2.0)

**Document Version:** 2.0
**Author:** AI Assistant
**Date:** 2023-10-27

### 1. Overview of Changes from v1.0

This document outlines the v2.0 update to the Moodel AI system. It addresses feedback from the initial implementation to create more robust, engaging, and believable behaviors.

The key changes are:

1.  **Boundary Avoidance:** A new system has been added that applies a gentle "avoidance" force as Moodels approach the play area boundaries. This prevents them from constantly scraping against walls and makes their movement within the area feel more natural and intentional.
2.  **Improved Flocking Behavior:** Happy Moodels now have a subtle, internal "jiggle" movement within their flocks. This ensures they generate physics collisions with each other, triggering mood interactions and preventing them from decaying to `Neutral` due to perceived isolation.
3.  **Dynamic `Calm` and `Neutral` Movement:** These moods no longer become completely static. They now exhibit a constant, slow, and gentle drift, making them feel more alive while remaining visually distinct from more energetic moods.
4.  **True "Rage" Chaos:** The magnetism logic has been corrected to ensure `Rage` Moodels are repelled by *all* other Moodels, including each other. This prevents "Rage clumps" and reinforces their role as chaotic agents.
5.  **Refined Force Combination:** The AI's final movement intent is now calculated by adding forces together without immediate normalization. This allows strong forces (like magnetism or boundary avoidance) to correctly override weaker ones (like wandering), resulting in more decisive actions.

### 2. Required Code Changes

The following changes should be applied to the project. The primary modification is a complete replacement of the `demo/ai.rs` file.

#### Action: Replace `src/demo/ai.rs`

Replace the entire contents of the file `src/demo/ai.rs` with the new, production-ready code below. This single file contains all the necessary logic for the v2.0 AI.

```rust
//! AI-controlled entities behavior.
//!
//! This module provides a mood-driven AI system. Entities exhibit complex
//! behaviors such as organic wandering, flocking/repulsion, boundary avoidance,
//! and state-based abilities, all determined by their current `Mood`.

use bevy::prelude::*;
use noisy_bevy::fbm_simplex_2d_seeded;
use rand::Rng;

use crate::{
    demo::{
        mood::Mood,
        movement::{MovementController, PlayArea},
    },
    AppSystems, PausableSystems,
};

pub(super) fn plugin(app: &mut App) {
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
            update_ai_magnetism.after(update_ai_behavior),
            // 3. Add a final, strong force to avoid play area boundaries.
            update_ai_boundary_avoidance.after(update_ai_magnetism),
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
            ability_cooldown: Timer::from_seconds(rng.gen_range(3.0..6.0), TimerMode::Once),
        }
    }
}

/// System that updates the AI's core action state and base movement intent.
fn update_ai_behavior(
    time: Res<Time>,
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
    let mut rng = rand::thread_rng();

    let potential_targets: Vec<_> = all_moodels
        .iter()
        .filter(|(_, _, mood)| **mood != Mood::Rage)
        .collect();

    for (entity, transform, mut controller, mut wander_state, mood) in &mut ai_query {
        wander_state.state_timer.tick(time.delta());
        wander_state.ability_cooldown.tick(time.delta());

        match &mut wander_state.action {
            AiAction::Wandering => {
                if *mood == Mood::Rage && wander_state.ability_cooldown.just_finished() {
                    let closest_target = potential_targets
                        .iter()
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
                        controller.intent = Vec2::ZERO;
                        continue;
                    }
                }
                controller.intent = get_wander_intent(mood, &mut wander_state, &time, &mut rng);
            }
            AiAction::Aiming { target } => {
                controller.intent = Vec2::ZERO;
                if wander_state.state_timer.just_finished() {
                    if let Ok((_, target_transform, _)) = all_moodels.get(*target) {
                        info!("Rage Moodel {:?} is CHARGING!", entity);
                        wander_state.action = AiAction::Charging {
                            target_pos: target_transform.translation.truncate(),
                        };
                        wander_state.state_timer.set_duration(std::time::Duration::from_secs_f32(1.5));
                        wander_state.state_timer.reset();
                    } else {
                        wander_state.action = AiAction::Wandering;
                    }
                }
            }
            AiAction::Charging { target_pos } => {
                controller.intent = (*target_pos - transform.translation.truncate()).normalize_or_zero();
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
    mut query: Query<(&Entity, &Transform, &Mood, &mut MovementController, &AiMagnetism)>,
    all_moodels: Query<(&Entity, &Transform, &Mood)>,
) {
    let positions: Vec<_> = all_moodels.iter().map(|(e, t, m)| (*e, *t, *m)).collect();

    for (entity, transform, mood, mut controller, magnetism) in &mut query {
        let mut cohesion_vec = Vec2::ZERO;
        let mut separation_vec = Vec2::ZERO;
        let mut friendly_neighbor_count = 0;

        for (other_entity, other_transform, other_mood) in &positions {
            if entity == other_entity { continue; }
            let distance = transform.translation.truncate().distance(other_transform.translation.truncate());

            if distance < magnetism.vision_radius {
                if distance < magnetism.separation_distance {
                    if let Some(dir) = (transform.translation.truncate() - other_transform.translation.truncate()).try_normalize() {
                        separation_vec += dir / (distance + 0.1);
                    }
                }
                let attraction_factor = get_attraction_factor(*mood, *other_mood);
                if attraction_factor != 0.0 {
                    cohesion_vec += (other_transform.translation.truncate() - transform.translation.truncate()) * attraction_factor;
                    if attraction_factor > 0.0 {
                        friendly_neighbor_count += 1;
                    }
                }
            }
        }

        if friendly_neighbor_count > 0 {
            cohesion_vec /= friendly_neighbor_count as f32;
        }

        let magnetic_force = (cohesion_vec * magnetism.cohesion_strength) + (separation_vec * magnetism.separation_strength);
        controller.intent += magnetic_force;
    }
}

/// **NEW v2.0:** System to steer AI away from the edges of the play area.
fn update_ai_boundary_avoidance(
    play_area: Res<PlayArea>,
    mut query: Query<(&Transform, &mut MovementController), With<AiEntity>>,
) {
    let avoidance_margin = 100.0; // How far from the edge the avoidance force starts
    let avoidance_strength = 2.0; // How strongly they are pushed away
    let half_size = play_area.size / 2.0;
    let boundaries = Rect::from_center_half_size(play_area.center, half_size);

    for (transform, mut controller) in &mut query {
        let pos = transform.translation.truncate();
        let mut avoidance_force = Vec2::ZERO;

        // Check distance to each of the four boundaries
        if pos.x < boundaries.min.x + avoidance_margin {
            avoidance_force.x = avoidance_strength;
        }
        if pos.x > boundaries.max.x - avoidance_margin {
            avoidance_force.x = -avoidance_strength;
        }
        if pos.y < boundaries.min.y + avoidance_margin {
            avoidance_force.y = avoidance_strength;
        }
        if pos.y > boundaries.max.y - avoidance_margin {
            avoidance_force.y = -avoidance_strength;
        }

        // Add the avoidance force to the final intent. This force is strong
        // and should override other behaviors when near a wall.
        controller.intent += avoidance_force;
    }
}

/// Generates a noise vector using different seeds for X and Y axes for more random movement.
fn get_2d_noise(input: f32, octaves: u8, seed: f32) -> Vec2 {
    let noise_x = fbm_simplex_2d_seeded(Vec2::new(input, 0.0), octaves, 0.5, 2.0, seed);
    let noise_y = fbm_simplex_2d_seeded(Vec2::new(0.0, input), octaves, 0.5, 2.0, seed + 1000.0);
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
        Mood::Rage => get_2d_noise(time_input * 0.5, 4, wander_state.noise_seed).normalize_or_zero(),
        Mood::Calm => get_2d_noise(time_input * 0.1, 1, wander_state.noise_seed).normalize_or_zero() * 0.5,
        Mood::Neutral => get_2d_noise(time_input * 0.3, 2, wander_state.noise_seed).normalize_or_zero() * 0.6,
        Mood::Sad => {
            if !wander_state.state_timer.finished() {
                get_2d_noise(time_input * 0.1, 1, wander_state.noise_seed).normalize_or_zero() * 0.4
            } else {
                if wander_state.state_timer.just_finished() {
                    wander_state.state_timer.set_duration(std::time::Duration::from_secs_f32(rng.gen_range(3.0..6.0)));
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
        (Rage, _) => -1.0,
        (Calm, Calm) => 0.2,
        (Neutral, _) => -0.1,
        _ => 0.0,
    }
}
```

### 3. Verification

After replacing the file, run the application and observe the following behaviors to confirm the changes have been successfully implemented:

-   [ ] **Boundary Avoidance:** Moodels should be seen turning away from the play area edges *before* they physically collide with them. The effect should be a gentle, natural course correction.
-   [ ] **Happy Flocks:** A group of `Happy` Moodels should cluster together but continue to move and jostle slightly, triggering collision sounds and mood interactions. They should not become a static, overlapping ball.
-   [ ] **Rage Repulsion:** `Rage` Moodels should actively push away from all other Moodels, including each other. You should not see two `Rage` Moodels moving together.
-   [ ] **Active Calm/Neutral:** `Calm` and `Neutral` Moodels should no longer be completely stationary for long periods. They should always have a slow, drifting motion.
-   [ ] **Sad Behavior:** `Sad` Moodels should still exhibit their characteristic behavior of drifting slowly for a time and then stopping completely for a few seconds.