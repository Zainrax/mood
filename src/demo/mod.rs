//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

pub mod ai;
mod animation;
pub mod level;
mod level_library;
pub mod mood;
mod movement;
pub mod player;
mod player_input;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ai::plugin,
        animation::plugin,
        level::plugin,
        mood::plugin,
        movement::plugin,
        player::plugin,
        player_input::plugin,
    ));
}
