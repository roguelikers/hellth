use bevy::prelude::*;

use super::{commands::move_command::MoveCommand, feel::Random, turns::StartTurnEvent};

// i'm running away from actually doing anything here yet
#[derive(Component)]
pub struct AIAgent;

pub fn ai_responds_to_start_turn(
    mut start_turn_events: EventReader<StartTurnEvent>,
    agents: Query<&mut AIAgent>,
    mut rng: ResMut<Random>,
    mut move_commands: EventWriter<MoveCommand>,
    //mut turn_order: ResMut<TurnOrder>,
) {
    for StartTurnEvent(entity) in start_turn_events.read() {
        if let Ok(_agent) = agents.get(*entity) {
            // todo: add scripting support for this part for quick iteration
            // TODO: silly logic to see if it works
            // TODO: make sure that everyone has an agent!

            // we can't use commands to do multiple agents in the same frame
            // i need to figure out how to do this without taking a big number of arguments in

            move_commands.send(MoveCommand {
                entity: *entity,
                direction: rng.gen2d(-1..2, -1..2),
                cost: 100,
            });
        }
    }
}

pub struct SvarogAIPlugin;

impl Plugin for SvarogAIPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_systems(
            Update,
            ai_responds_to_start_turn.run_if(on_event::<StartTurnEvent>()),
        );
    }
}
