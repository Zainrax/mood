//! Handles player input for commanding Moodels. Selection is handled by observers in mood.rs.

use bevy::prelude::*;
use bevy_picking::prelude::{Click, Pointer};

use crate::demo::ai::{AiAction, AiWanderState};
use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<CommandMoodelEvent>();
    app.add_systems(
        Update,
        (
            handle_command_input,
            apply_moodel_command,
            manage_selection_visuals,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// A component marking an entity as selectable by the player.
#[derive(Component)]
pub struct Selectable;

/// A component marking an entity as currently selected.
#[derive(Component)]
pub struct Selected;

/// A marker for the visual selection ring entity.
#[derive(Component)]
pub struct SelectionRing;

/// An event fired when the player issues a move command.
#[derive(Event)]
struct CommandMoodelEvent {
    entity: Entity,
    destination: Vec2,
}

/// An observer that runs when the background is clicked, clearing any selection.
pub fn handle_background_click(
    _trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    selected_q: Query<Entity, With<Selected>>,
) {
    if let Ok(selected_entity) = selected_q.single() {
        info!(
            "Background clicked. Deselecting entity {:?}",
            selected_entity
        );
        commands.entity(selected_entity).remove::<Selected>();
    }
}

/// System that handles right-click commands for the selected Moodel.
fn handle_command_input(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    selected_q: Query<Entity, With<Selected>>,
    mut ev_command: EventWriter<CommandMoodelEvent>,
) {
    if mouse_buttons.just_pressed(MouseButton::Right) {
        if let Ok(selected_entity) = selected_q.single() {
            let Ok((camera, camera_transform)) = q_camera.single() else {
                return;
            };
            let Ok(window) = q_windows.single() else {
                return;
            };

            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    info!(
                        "Commanding entity {:?} to move to {:?}",
                        selected_entity, world_pos
                    );
                    ev_command.write(CommandMoodelEvent {
                        entity: selected_entity,
                        destination: world_pos,
                    });
                    commands.entity(selected_entity).remove::<Selected>();
                }
            }
        }
    }
}

/// Applies the move command to the AI state of the commanded Moodel.
fn apply_moodel_command(
    mut ev_command: EventReader<CommandMoodelEvent>,
    mut ai_q: Query<&mut AiWanderState>,
) {
    for ev in ev_command.read() {
        if let Ok(mut ai_state) = ai_q.get_mut(ev.entity) {
            ai_state.action = AiAction::MovingToTarget {
                destination: ev.destination,
            };
        }
    }
}

/// Spawns/despawns and manages a visual indicator for the selected entity.
fn manage_selection_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    added_q: Query<Entity, Added<Selected>>,
    mut removed: RemovedComponents<Selected>,
    children_q: Query<&Children>,
    ring_q: Query<Entity, With<SelectionRing>>,
    asset_server: Res<AssetServer>,
) {
    for entity in &added_q {
        info!("Entity {:?} selected. Spawning selection ring.", entity);
        let ring_entity = commands
            .spawn((
                Name::new("Selection Ring"),
                SelectionRing,
                Sprite {
                    image: asset_server.load("images/Selector.png"),
                    ..Default::default()
                },
                Transform::from_xyz(0.0, 0.0, 0.1),
            ))
            .id();
        commands.entity(entity).add_child(ring_entity);
    }

    for entity in removed.read() {
        if let Ok(children) = children_q.get(entity) {
            for child in children.iter() {
                if ring_q.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
        }
    }
}
