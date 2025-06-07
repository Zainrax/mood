//! Spawn the main level.

use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    demo::{
        ai::AiSpawnConfig,
        mood::{Mood, MoodAssets, create_mood_object_bundle, spawn_moodel_bundle},
        movement::PlayArea,
        player::player,
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    mood_assets: Res<MoodAssets>,
    mut play_area: ResMut<PlayArea>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Get spawn configuration
    let spawn_config = AiSpawnConfig::default();

    // Configure play area
    play_area.center = Vec2::ZERO;
    play_area.size = Vec2::new(900.0, 600.0);

    // Spawn the level entity with player and Moodel entities
    let mut level = commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
    ));

    level.with_children(|parent| {
        // Spawn play area boundary visualization
        let half_size = play_area.size / 2.0;
        let border_thickness = 4.0;
        let border_color = Color::srgba(0.3, 0.3, 0.5, 0.6);

        // Top border
        parent.spawn((
            Name::new("Top Border"),
            Sprite {
                color: border_color,
                custom_size: Some(Vec2::new(
                    play_area.size.x + border_thickness * 2.0,
                    border_thickness,
                )),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                play_area.center.x,
                play_area.center.y + half_size.y + border_thickness / 2.0,
                -1.0,
            )),
        ));

        // Bottom border
        parent.spawn((
            Name::new("Bottom Border"),
            Sprite {
                color: border_color,
                custom_size: Some(Vec2::new(
                    play_area.size.x + border_thickness * 2.0,
                    border_thickness,
                )),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                play_area.center.x,
                play_area.center.y - half_size.y - border_thickness / 2.0,
                -1.0,
            )),
        ));

        // Left border
        parent.spawn((
            Name::new("Left Border"),
            Sprite {
                color: border_color,
                custom_size: Some(Vec2::new(border_thickness, play_area.size.y)),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                play_area.center.x - half_size.x - border_thickness / 2.0,
                play_area.center.y,
                -1.0,
            )),
        ));

        // Right border
        parent.spawn((
            Name::new("Right Border"),
            Sprite {
                color: border_color,
                custom_size: Some(Vec2::new(border_thickness, play_area.size.y)),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                play_area.center.x + half_size.x + border_thickness / 2.0,
                play_area.center.y,
                -1.0,
            )),
        ));

        // Spawn player (now just a marker entity)
        parent.spawn(player());

        // Spawn Moodels starting in Neutral mood
        let current_time = time.elapsed_secs();
        for (i, position) in spawn_config.positions.iter().enumerate() {
            parent
                .spawn(spawn_moodel_bundle(
                    Mood::Neutral,
                    &mood_assets,
                    *position,
                    spawn_config.max_speed,
                    current_time,
                ))
                .insert(Name::new(format!("Neutral Moodel {}", i + 1)));
        }

        // Spawn mood objects around the play area
        let mood_objects = [
            (Mood::Rage, Vec3::new(-300.0, 200.0, 0.0)), // Top left - red triangle
         //   (Mood::Sad, Vec3::new(300.0, 200.0, 0.0)),   // Top right - blue circle
         //   (Mood::Happy, Vec3::new(-300.0, -200.0, 0.0)), // Bottom left - yellow star
         //   (Mood::Calm, Vec3::new(300.0, -200.0, 0.0)), // Bottom right - green circle
         //   (Mood::Rage, Vec3::new(0.0, 250.0, 0.0)),    // Top center - another rage trigger
         //   (Mood::Sad, Vec3::new(0.0, -250.0, 0.0)),    // Bottom center - another sad trigger
        ];

        for (mood, position) in mood_objects {
            parent.spawn(create_mood_object_bundle(
                mood,
                position,
                3.0, // 3 second cooldown between hits
                &mut meshes,
                &mut materials,
            ));
        }
    });
}
