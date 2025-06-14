// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
mod demo;
#[cfg(feature = "dev")]
mod dev_tools;
mod menus;
mod screens;
mod theme;

use avian2d::prelude::*;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_egui::EguiPlugin;
// Picking is now included in DefaultPlugins, no separate import needed
#[cfg(feature = "dev")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// Collision layers for separating Moodels and obstacles
pub const COLLISION_LAYER_MOODEL: LayerMask = LayerMask(0b0001);
pub const COLLISION_LAYER_OBSTACLE: LayerMask = LayerMask(0b0010);

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Mood".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        );

        // Add Avian2D physics plugin
        app.add_plugins(PhysicsPlugins::default().with_length_unit(100.0));
        
        // Add JSON asset plugin for Level assets
        app.add_plugins(JsonAssetPlugin::<crate::demo::level::Level>::new(&["level.json"]));

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            demo::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            #[cfg(feature = "dev")]
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            #[cfg(feature = "dev")]
            WorldInspectorPlugin::new(),
            menus::plugin,
            screens::plugin,
            theme::plugin,
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera and setup assets.
        app.add_systems(Startup, (spawn_camera, setup_assets));
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));
}

fn setup_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load a handwriting-style font
    // You can download fonts like "Kalam-Bold.ttf" from Google Fonts
    // or "ComicNeue-Bold.ttf" for a rounded, friendly look
    let font = asset_server.load("fonts/ComicNeue-Bold.ttf");
    // Fallback to default if font not found
    commands.insert_resource(crate::demo::level::FontHandle(font));
}
