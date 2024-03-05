pub mod move_command;

use bevy::prelude::*;

use self::move_command::{MoveCommand, MoveCommandPlugin};

#[derive(Event, Debug)]
pub enum GameCommand {
    Move {
        entity: Entity,
        direction: IVec2,
        cost: i32,
    },
}

pub fn command_transmuter(
    mut game_commands: EventReader<GameCommand>,
    mut move_commands: EventWriter<MoveCommand>,
) {
    for game_command in game_commands.read() {
        println!("{:?}", game_command);
        match game_command {
            GameCommand::Move {
                entity,
                direction,
                cost,
            } => move_commands.send(MoveCommand {
                entity: *entity,
                direction: *direction,
                cost: *cost,
            }),
        }
    }
}
pub struct SvarogCommandsPlugin;
impl Plugin for SvarogCommandsPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_event::<GameCommand>();
        bevy.add_systems(First, command_transmuter);
        bevy.add_plugins(MoveCommandPlugin);
    }
}
