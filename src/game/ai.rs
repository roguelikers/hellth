use bevy::prelude::*;

use super::{commands::GameCommand, feel::Random, turns::StartTurnEvent};

// i'm running away from actually doing anything here yet
#[derive(Component)]
pub struct AIAgent;

pub fn ai_responds_to_start_turn(
    mut start_turn_events: EventReader<StartTurnEvent>,
    agents: Query<&mut AIAgent>,
    mut rng: ResMut<Random>,
    mut game_commands: EventWriter<GameCommand>,
) {
    for StartTurnEvent(entity) in start_turn_events.read() {
        if let Ok(_agent) = agents.get(*entity) {
            println!("#{:?} reacts", entity);

            // TODO: silly logic to see if it works
            game_commands.send(GameCommand::Move {
                entity: *entity,
                direction: rng.gen2d(-1..2, -1..2),
                cost: 100,
            });
        } else {
            println!("#{:?} has no reaction", entity);
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
