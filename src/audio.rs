use bevy::prelude::*;
use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Music>();
    app.register_type::<SoundEffect>();
    app.register_type::<AudioAssets>();

    app.add_event::<PlaySound>();
    app.load_resource::<AudioAssets>();

    app.add_systems(Update, (
        apply_global_volume.run_if(resource_changed::<GlobalVolume>),
        play_sound_effects,
    ));
}

/// A resource that holds handles to all our sound effects.
#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AudioAssets {
    #[dependency]
    pub correct_zone_entry: Handle<AudioSource>,
    #[dependency]
    pub mood_change: Handle<AudioSource>,
    #[dependency]
    pub mood_collision: Handle<AudioSource>,
}

impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            correct_zone_entry: assets.load("audio/sound_effects/button_click.ogg"),
            mood_change: assets.load("audio/sound_effects/button_hover.ogg"), 
            mood_collision: assets.load("audio/sound_effects/step1.ogg"),
        }
    }
}

/// An event to trigger a sound effect.
#[derive(Event)]
pub enum PlaySound {
    CorrectZoneEntry,
    MoodChanged { from: crate::demo::mood::Mood, to: crate::demo::mood::Mood },
    MoodCollision { mood1: crate::demo::mood::Mood, mood2: crate::demo::mood::Mood },
}

/// This system listens for `PlaySound` events and plays the corresponding sound.
fn play_sound_effects(
    mut commands: Commands,
    mut sound_events: EventReader<PlaySound>,
    audio_assets: Option<Res<AudioAssets>>,
) {
    // Early return if assets aren't loaded yet
    let Some(audio_assets) = audio_assets else {
        return;
    };

    // Process all sound events (can play multiple sounds simultaneously)
    for event in sound_events.read() {
        match event {
            PlaySound::CorrectZoneEntry => {
                commands.spawn(sound_effect(audio_assets.correct_zone_entry.clone()));
            }
            PlaySound::MoodChanged { from: _, to: _ } => {
                commands.spawn(sound_effect(audio_assets.mood_change.clone()));
            }
            PlaySound::MoodCollision { mood1: _, mood2: _ } => {
                commands.spawn(sound_effect(audio_assets.mood_collision.clone()));
            }
        }
    }
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "music" category (e.g. global background music, soundtrack).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Music;

/// A music audio instance.
pub fn music(handle: Handle<AudioSource>) -> impl Bundle {
    (
        AudioPlayer::new(handle),
        PlaybackSettings::LOOP,
        Music
    )
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "sound effect" category (e.g. footsteps, the sound of a magic spell, a door opening).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SoundEffect;

/// A sound effect audio instance.
pub fn sound_effect(handle: Handle<AudioSource>) -> impl Bundle {
    (
        AudioPlayer::new(handle),
        PlaybackSettings::DESPAWN,
        SoundEffect
    )
}

/// [`GlobalVolume`] doesn't apply to already-running audio entities, so this system will update them.
fn apply_global_volume(
    global_volume: Res<GlobalVolume>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink)>,
) {
    for (playback, mut sink) in &mut audio_query {
        sink.set_volume(global_volume.volume * playback.volume);
    }
}