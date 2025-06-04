//! Mood system for Moodels - defines different emotional states and conversion logic.

use bevy::prelude::*;
use rand::Rng;
use avian2d::prelude::*;

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Mood>();
    app.register_type::<MoodAssets>();
    app.load_resource::<MoodAssets>();

    // Mood conversion systems
    app.add_systems(
        Update,
        (
            update_mood_based_on_collisions,
            update_mood_natural_progression,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// The different emotional states a Moodel can have
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub enum Mood {
    Neutral,
    Calm,
    Happy,
    Rage,
    Sad,
}

impl Mood {
    /// Get the movement speed multiplier for this mood
    pub fn speed_multiplier(self) -> f32 {
        match self {
            Mood::Neutral => 0.75, // 150 units/second baseline
            Mood::Calm => 0.5,     // 100 units/second baseline
            Mood::Happy => 1.0,    // 200 units/second baseline
            Mood::Rage => 1.5,     // 300 units/second baseline
            Mood::Sad => 0.375,    // 75 units/second baseline
        }
    }

    /// Get the color associated with this mood
    pub fn color(self) -> Color {
        match self {
            Mood::Neutral => Color::srgb(0.8, 0.8, 0.8), // Gray
            Mood::Calm => Color::srgb(0.3, 0.6, 1.0),    // Blue
            Mood::Happy => Color::srgb(1.0, 0.9, 0.2),   // Yellow
            Mood::Rage => Color::srgb(1.0, 0.2, 0.2),    // Red
            Mood::Sad => Color::srgb(0.6, 0.4, 0.8),     // Purple
        }
    }

    /// Get all mood variants for iteration
    pub fn all() -> [Mood; 5] {
        [
            Mood::Neutral,
            Mood::Calm,
            Mood::Happy,
            Mood::Rage,
            Mood::Sad,
        ]
    }

    /// Get the next mood in the cyclical progression: Happy -> Calm -> Sad -> Rage -> Happy
    /// Neutral can transition to any mood based on external factors
    pub fn next_in_cycle(self) -> Mood {
        match self {
            Mood::Neutral => Mood::Happy, // Neutral naturally progresses to Happy
            Mood::Happy => Mood::Calm,
            Mood::Calm => Mood::Sad,
            Mood::Sad => Mood::Rage,
            Mood::Rage => Mood::Happy,
        }
    }

    /// Get a random mood
    pub fn random(rng: &mut impl Rng) -> Self {
        let moods = Self::all();
        moods[rng.random_range(0..moods.len())]
    }
}

/// Resource containing all mood-related assets
#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct MoodAssets {
    #[dependency]
    pub neutral: Handle<Image>,
    #[dependency]
    pub calm: Handle<Image>,
    #[dependency]
    pub happy: Handle<Image>,
    #[dependency]
    pub rage: Handle<Image>,
    #[dependency]
    pub sad: Handle<Image>,
}

impl MoodAssets {
    /// Get the sprite handle for a specific mood
    pub fn get_sprite(&self, mood: Mood) -> Handle<Image> {
        match mood {
            Mood::Neutral => self.neutral.clone(),
            Mood::Calm => self.calm.clone(),
            Mood::Happy => self.happy.clone(),
            Mood::Rage => self.rage.clone(),
            Mood::Sad => self.sad.clone(),
        }
    }
}

impl FromWorld for MoodAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            neutral: assets.load("images/Moodel.png"), // Base sprite for Neutral
            calm: assets.load("images/MoodelCalm.png"),
            happy: assets.load("images/MoodelHappy.png"),
            rage: assets.load("images/MoodelRage.png"),
            sad: assets.load("images/MoodelSad.png"),
        }
    }
}

/// Component for entities that can change moods
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct MoodEntity {
    /// Current collision count
    pub collision_count: usize,
    /// Timer until next collision-based mood evaluation
    pub next_mood_check: Timer,
    /// Timer for natural cyclical mood progression
    pub next_cycle_progression: Timer,
    /// How long the current mood has been stable (for permanence)
    pub mood_stability: f32,
}

impl Default for MoodEntity {
    fn default() -> Self {
        Self {
            collision_count: 0,
            next_mood_check: Timer::from_seconds(0.5, TimerMode::Repeating), // Check every 0.5 seconds as per spec
            next_cycle_progression: Timer::from_seconds(8.0, TimerMode::Repeating), // Natural mood cycle every 8 seconds
            mood_stability: 0.0,
        }
    }
}

/// Updates Moodel moods based on collision count
fn update_mood_based_on_collisions(
    time: Res<Time>,
    mood_assets: Option<Res<MoodAssets>>,
    mut moodel_query: Query<(Entity, &mut Mood, &mut MoodEntity, &mut Sprite, &CollidingEntities)>,
) {
    // Early return if assets aren't loaded yet
    let Some(mood_assets) = mood_assets else {
        return;
    };

    let mut conversions = Vec::new();

    // Process each Moodel for mood changes based on collisions
    for (entity, current_mood, mut mood_entity, _sprite, colliding_entities) in &mut moodel_query {
        // Update collision count
        mood_entity.collision_count = colliding_entities.len();
        
        // Update timer
        mood_entity.next_mood_check.tick(time.delta());
        
        // Check for mood change based on collision count
        if mood_entity.next_mood_check.just_finished() && mood_entity.mood_stability < 3.0 {
            let collision_mood = match mood_entity.collision_count {
                0 => Mood::Sad,       // Isolated → Sad (loneliness)
                1..=2 => Mood::Calm,  // Small groups (2-3) → Calm (comfortable)
                3..=5 => Mood::Happy, // Medium groups (4-6) → Happy (social energy)
                _ => Mood::Rage,      // Large crowds (7+) → Rage (overwhelmed)
            };

            // Only change if collision mood is different and significant
            if collision_mood != *current_mood && mood_entity.mood_stability < 1.5 {
                conversions.push((entity, collision_mood));
                info!(
                    "Collision change: {:?} -> {:?} (collisions: {})",
                    current_mood, collision_mood, mood_entity.collision_count
                );
            }
        }
    }

    // Apply conversions
    for (entity, new_mood) in conversions {
        if let Ok((_, mut mood, mut mood_entity, mut sprite, _)) = moodel_query.get_mut(entity) {
            *mood = new_mood;
            sprite.image = mood_assets.get_sprite(new_mood);
            sprite.color = new_mood.color();

            // Reset stability when mood changes
            mood_entity.mood_stability = 0.0;
        }
    }
}

/// Updates Moodel moods based on natural cyclical progression
fn update_mood_natural_progression(
    time: Res<Time>,
    mood_assets: Option<Res<MoodAssets>>,
    mut moodel_query: Query<(&mut Mood, &mut MoodEntity, &mut Sprite)>,
) {
    // Early return if assets aren't loaded yet
    let Some(mood_assets) = mood_assets else {
        return;
    };

    let delta = time.delta_secs();

    for (mut mood, mut mood_entity, mut sprite) in &mut moodel_query {
        // Update timers and stability
        mood_entity.next_cycle_progression.tick(time.delta());
        mood_entity.mood_stability += delta;

        // Natural cyclical progression (overrides collision-based if mood is stable)
        if mood_entity.next_cycle_progression.just_finished() {
            let new_mood = mood.next_in_cycle();
            info!("Cyclical progression: {:?} -> {:?}", *mood, new_mood);
            
            *mood = new_mood;
            sprite.image = mood_assets.get_sprite(new_mood);
            sprite.color = new_mood.color();
            
            // Reset stability when mood changes
            mood_entity.mood_stability = 0.0;
        }
    }
}

/// Creates a bundle for spawning a Moodel with a specific mood
pub fn spawn_moodel_bundle(
    mood: Mood,
    mood_assets: &MoodAssets,
    position: Vec3,
    max_speed: f32,
) -> impl Bundle {
    use crate::demo::ai::{AiEntity, AiWanderState};
    use crate::demo::movement::{MovementController, ScreenWrap};

    (
        Name::new(format!("{:?} Moodel", mood)),
        AiEntity,
        mood,
        MoodEntity::default(),
        AiWanderState::new(),
        Sprite {
            image: mood_assets.get_sprite(mood),
            color: mood.color(),
            custom_size: Some(Vec2::new(134.0, 208.0)),
            ..default()
        },
        Transform::from_translation(position).with_scale(Vec3::splat(0.5)),
        MovementController {
            max_speed: max_speed * mood.speed_multiplier(),
            ..default()
        },
        ScreenWrap,
        // Physics components
        RigidBody::Dynamic,
        Collider::circle(30.0), // Radius of 30 pixels for the Moodel
        LinearVelocity::default(),
        AngularVelocity::default(),
        CollisionLayers::new([0], [0]), // All Moodels on same layer and collide with each other
    )
}
