//! Spawn the main level.

use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    demo::{
        ai::{spawn_ai_bundle, AiAssets, AiSpawnConfig},
        player::{PlayerAssets, player},
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
    player_assets: Res<PlayerAssets>,
    mut ai_assets: ResMut<AiAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Initialize AI assets to use the same sprite as player for now
    // In a real game, you'd load a different sprite for AI entities
    if ai_assets.sprite.is_weak() {
        ai_assets.sprite = player_assets.moodel.clone();
    }

    // Get AI spawn configuration
    let ai_config = AiSpawnConfig::default();

    // Spawn the level entity with player and AI entities
    let mut level = commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
    ));

    level.with_children(|parent| {
        // Spawn player
        parent.spawn(player(400.0, &player_assets, &mut texture_atlas_layouts));
        
        // Spawn AI entities using configuration
        for (i, position) in ai_config.positions.iter().enumerate() {
            parent.spawn(spawn_ai_bundle(
                ai_config.max_speed,
                &ai_assets,
                *position,
            )).insert(Name::new(format!("AI Entity {}", i + 1)));
        }
        
        // Spawn music
        parent.spawn((
            Name::new("Gameplay Music"),
            music(level_assets.music.clone())
        ));
    });
}