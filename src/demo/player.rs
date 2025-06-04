//! Player-specific behavior.

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();

    // Shape tool input systems
    app.add_systems(
        Update,
        (
            handle_line_tool_input,
            handle_box_tool_input,
            handle_circle_tool_input,
        )
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

/// The player marker (no longer a visual entity).
pub fn player() -> impl Bundle {
    (
        Name::new("Player"),
        Player,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player;

/// Handle Line Tool input (Q key)
fn handle_line_tool_input(
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyQ) {
        // TODO: Implement line tool
        info!("Line tool activated!");
    }
}

/// Handle Box Tool input (W key)
fn handle_box_tool_input(
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyW) {
        // TODO: Implement box tool
        info!("Box tool activated!");
    }
}

/// Handle Circle Tool input (E key)
fn handle_circle_tool_input(
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyE) {
        // TODO: Implement circle tool
        info!("Circle tool activated!");
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    pub moodel: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            moodel: assets.load("images/Moodel.png"),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
