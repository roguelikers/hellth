pub mod move_command;

use bevy::prelude::*;

use self::move_command::{MoveCommand, MoveCommandPlugin};

pub struct SvarogCommandsPlugin;
impl Plugin for SvarogCommandsPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_plugins(MoveCommandPlugin);
    }
}
