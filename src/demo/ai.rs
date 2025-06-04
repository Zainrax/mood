//! AI-controlled entities behavior.
//!
//! This module provides a wandering AI system where entities move in random
//! directions, changing course periodically. The AI uses a simple state-based
//! approach with configurable parameters.

use bevy::prelude::*;
use rand::Rng;

use crate::{
    AppSystems, PausableSystems,
    demo::movement::{MovementController, ScreenWrap},
};

/// Configuration constants for AI behavior
mod config {
    use std::ops::Range;

    /// Default maximum speed for AI entities
    pub const DEFAULT_AI_SPEED: f32 = 350.0;

    /// Scale factor for AI entity sprites
    pub const AI_SPRITE_SCALE: f32 = 0.5;

    /// Range of time (in seconds) an AI will move in one direction
    pub const WANDER_DURATION_RANGE: Range<f32> = 1.0..3.0;
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<AiEntity>();
    app.register_type::<AiWanderState>();
    app.init_resource::<AiAssets>();

    // Update AI movement behavior
    app.add_systems(
        Update,
        update_ai_wander_behavior
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Resource containing assets used by AI entities
#[derive(Resource, Default)]
pub struct AiAssets {
    /// Visual representation for AI entities
    pub sprite: Handle<Image>,
}

/// Creates a bundle for spawning an AI-controlled entity
///
/// # Arguments
/// * `max_speed` - Maximum movement speed of the AI entity
/// * `ai_assets` - Assets resource containing AI sprites
/// * `position` - Initial world position of the entity
pub fn spawn_ai_bundle(max_speed: f32, ai_assets: &AiAssets, position: Vec3) -> impl Bundle {
    (
        Name::new("AI Entity"),
        AiEntity,
        AiWanderState::new(),
        Sprite {
            image: ai_assets.sprite.clone(),
            custom_size: Some(Vec2::new(134.0, 208.0)),
            ..default()
        },
        Transform::from_translation(position).with_scale(Vec3::splat(config::AI_SPRITE_SCALE)),
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
    )
}

/// Marker component for AI-controlled entities
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
                Vec3::new(-200.0, 100.0, 0.0),
                Vec3::new(200.0, -100.0, 0.0),
                Vec3::new(0.0, 200.0, 0.0),
                Vec3::new(-150.0, -150.0, 0.0),
                Vec3::new(300.0, 0.0, 0.0),
            ],
            max_speed: config::DEFAULT_AI_SPEED,
        }
    }
}

/// State component for AI wandering behavior
///
/// This component tracks the current movement direction and timing
/// for an AI entity that wanders randomly around the game world.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AiWanderState {
    /// Current normalized direction vector
    direction: Vec2,
    /// Time remaining until next direction change
    time_until_change: f32,
}

impl AiWanderState {
    /// Creates a new wander state with random initial direction
    pub fn new() -> Self {
        let mut rng = rand::rng();
        Self {
            direction: Self::generate_random_direction(&mut rng),
            time_until_change: rng.random_range(config::WANDER_DURATION_RANGE),
        }
    }

    /// Updates the wander state with a new random direction
    fn randomize_direction(&mut self, rng: &mut impl Rng) {
        self.direction = Self::generate_random_direction(rng);
        self.time_until_change = rng.random_range(config::WANDER_DURATION_RANGE);
    }

    /// Generates a random unit vector for movement direction
    fn generate_random_direction(rng: &mut impl Rng) -> Vec2 {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        Vec2::new(angle.cos(), angle.sin())
    }
}

/// System that updates AI wandering behavior
///
/// This system handles the timer-based direction changes for all AI entities,
/// making them wander randomly around the game world.
fn update_ai_wander_behavior(
    time: Res<Time>,
    mut ai_query: Query<(&mut MovementController, &mut AiWanderState), With<AiEntity>>,
) {
    // Create RNG once per frame instead of once per entity for better performance
    let mut rng = rand::rng();

    for (mut controller, mut wander_state) in &mut ai_query {
        // Update direction change timer
        wander_state.time_until_change -= time.delta_secs();

        // Change direction when timer expires
        if wander_state.time_until_change <= 0.0 {
            wander_state.randomize_direction(&mut rng);
        }

        // Apply movement intent to the controller
        controller.intent = wander_state.direction;
    }
}
