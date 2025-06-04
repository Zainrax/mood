//! Spawn the main level.

use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    demo::{
        ai::AiSpawnConfig,
        mood::{spawn_moodel_bundle, Mood, MoodAssets},
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
) {
    
    // Get spawn configuration
    let spawn_config = AiSpawnConfig::default();

    // Spawn the level entity with player and Moodel entities
    let mut level = commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
    ));

    level.with_children(|parent| {
        // Spawn player (now just a marker entity)
        parent.spawn(player());
        
        // Spawn Moodels starting in Neutral mood
        for (i, position) in spawn_config.positions.iter().enumerate() {
            parent.spawn(spawn_moodel_bundle(
                Mood::Neutral,
                &mood_assets,
                *position,
                spawn_config.max_speed,
            )).insert(Name::new(format!("Neutral Moodel {}", i + 1)));
        }
        
        // Spawn music
        parent.spawn((
            Name::new("Gameplay Music"),
            music(level_assets.music.clone())
        ));
    });
}