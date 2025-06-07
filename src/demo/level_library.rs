//! A library of programmatically-defined levels.

use super::level::{GoalZoneData, Level, MoodelData, ObstacleData, ObstacleKind};
use crate::demo::mood::Mood;
use bevy::prelude::*;

/// Returns a programmatically defined `Level` based on a unique string ID.
pub fn get_level_by_id(id: &str) -> Option<Level> {
    match id {
        "tutorial_code" => Some(create_tutorial_from_code()),
        _ => None,
    }
}

fn create_tutorial_from_code() -> Level {
    Level {
        name: "Programmatic Tutorial".to_string(),
        play_area: Vec2::new(900.0, 600.0),
        moodels: vec![
            MoodelData {
                mood: Mood::Happy,
                position: Vec2::new(-200.0, 0.0),
            },
        ],
        obstacles: vec![ObstacleData {
            position: Vec2::new(0.0, 0.0),
            kind: ObstacleKind::Wall {
                size: Vec2::new(20.0, 300.0),
            },
        }],
        goal_zones: vec![GoalZoneData {
            position: Vec2::new(350.0, 0.0),
            size: Vec2::new(200.0, 200.0),
            target_mood: Mood::Happy,
            required_count: 1,
        }],
    }
}