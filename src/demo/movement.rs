//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Constrain movement within defined play area boundaries.
//! - Smooth acceleration and deceleration for fluid movement.

use bevy::{prelude::*, window::PrimaryWindow};
use avian2d::prelude::*;

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();
    app.register_type::<ScreenWrap>();
    app.register_type::<PlayAreaBounded>();
    app.register_type::<MovementSmoothing>();
    app.init_resource::<PlayArea>();

    app.add_systems(
        Update,
        (
            apply_movement_with_smoothing,
            apply_play_area_boundaries,
            apply_screen_wrap,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            max_speed: 400.0,
        }
    }
}

/// Resource defining the playable area boundaries
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct PlayArea {
    /// Center of the play area
    pub center: Vec2,
    /// Size of the play area (width, height)
    pub size: Vec2,
}

impl Default for PlayArea {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            size: Vec2::new(800.0, 600.0), // Default play area size
        }
    }
}

/// Component for entities that should be constrained to the play area
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayAreaBounded {
    /// How bouncy the collision with boundaries should be (0.0 - 1.0)
    pub restitution: f32,
}

impl Default for PlayAreaBounded {
    fn default() -> Self {
        Self { restitution: 0.8 }
    }
}

/// Component for smooth movement acceleration/deceleration
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementSmoothing {
    /// Current velocity (smoothed)
    pub current_velocity: Vec2,
    /// Acceleration rate (how quickly to reach target velocity)
    pub acceleration: f32,
    /// Deceleration rate (how quickly to stop)
    pub deceleration: f32,
}

impl Default for MovementSmoothing {
    fn default() -> Self {
        Self {
            current_velocity: Vec2::ZERO,
            acceleration: 800.0, // Units per second squared
            deceleration: 1200.0, // Units per second squared
        }
    }
}


fn apply_movement_with_smoothing(
    time: Res<Time>,
    mut movement_query: Query<(
        &MovementController,
        &LinearVelocity,
        &mut ExternalForce,
        Option<&mut MovementSmoothing>,
    )>,
) -> Result {
    let delta = time.delta_secs();
    
    for (controller, current_velocity, mut force, smoothing) in &mut movement_query {
        // Clear existing forces to prevent accumulation
        force.clear();
        
        if let Some(mut smoothing) = smoothing {
            // Calculate target velocity
            let target_velocity = controller.max_speed * controller.intent;
            
            // Apply acceleration or deceleration
            let diff = target_velocity - smoothing.current_velocity;
            let diff_magnitude = diff.length();
            
            if diff_magnitude > 0.01 {
                // Determine if we're accelerating or decelerating
                let is_accelerating = target_velocity.length() > smoothing.current_velocity.length();
                let rate = if is_accelerating {
                    smoothing.acceleration
                } else {
                    smoothing.deceleration
                };
                
                // Apply smooth acceleration
                let max_change = rate * delta;
                if diff_magnitude <= max_change {
                    smoothing.current_velocity = target_velocity;
                } else {
                    let normalized_diff = diff / diff_magnitude;
                    smoothing.current_velocity += normalized_diff * max_change;
                }
            } else {
                smoothing.current_velocity = target_velocity;
            }
            
            // Calculate force needed to reach desired velocity
            let velocity_diff = smoothing.current_velocity - current_velocity.0;
            force.apply_force(velocity_diff * 20.0); // Even gentler force to allow proper collision physics
        } else {
            // No smoothing component, apply direct force
            let target_velocity = controller.max_speed * controller.intent;
            let velocity_diff = target_velocity - current_velocity.0;
            force.apply_force(velocity_diff * 20.0);
        }
    }
    Ok(())
}

fn apply_play_area_boundaries(
    play_area: Res<PlayArea>,
    mut bounded_query: Query<(
        &mut Transform,
        &mut LinearVelocity,
        Option<&mut MovementSmoothing>,
        &PlayAreaBounded,
    )>,
) -> Result {
    let half_size = play_area.size / 2.0;
    let min_bounds = play_area.center - half_size;
    let max_bounds = play_area.center + half_size;
    
    for (mut transform, mut velocity, smoothing, bounded) in &mut bounded_query {
        let mut position = transform.translation.xy();
        let mut vel = velocity.0;
        let mut bounced = false;
        
        // Check X bounds
        if position.x < min_bounds.x {
            position.x = min_bounds.x;
            vel.x = vel.x.abs() * bounded.restitution;
            bounced = true;
        } else if position.x > max_bounds.x {
            position.x = max_bounds.x;
            vel.x = -vel.x.abs() * bounded.restitution;
            bounced = true;
        }
        
        // Check Y bounds
        if position.y < min_bounds.y {
            position.y = min_bounds.y;
            vel.y = vel.y.abs() * bounded.restitution;
            bounced = true;
        } else if position.y > max_bounds.y {
            position.y = max_bounds.y;
            vel.y = -vel.y.abs() * bounded.restitution;
            bounced = true;
        }
        
        // Apply changes
        if bounced {
            transform.translation = position.extend(transform.translation.z);
            velocity.0 = vel;
            
            // Update smoothing component if present
            if let Some(mut smoothing) = smoothing {
                smoothing.current_velocity = vel;
            }
        }
    }
    Ok(())
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenWrap;

fn apply_screen_wrap(
    window: Single<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform, With<ScreenWrap>>,
) -> Result {
    let size = window.size() + 256.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
    Ok(())
}
