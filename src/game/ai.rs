use bevy::prelude::*;

use crate::game::actions::move_action::MoveAction;

use super::{
    actions::ActionEvent, feel::Random, grid::WorldEntity, procgen::PlayerMarker, turns::TurnOrder,
    GameStates,
};

#[derive(Component)]
pub struct AIAgent;

pub fn ai_agents_act(
    mut turn_order: ResMut<TurnOrder>,
    player: Query<(Entity, &WorldEntity), With<PlayerMarker>>,
    _non_players: Query<(Entity, &WorldEntity), Without<PlayerMarker>>,
    mut rng: ResMut<Random>,
    mut actions: EventWriter<ActionEvent>,
) {
    let Ok((player_entity, _player_world)) = player.get_single() else {
        println!("NO PLAYER!");
        return;
    };

    let Some(top) = turn_order.peek() else {
        return;
    };

    if top == player_entity {
        return;
    }

    while turn_order.peek() != Some(player_entity) {
        if let Some(top) = turn_order.peek() {
            // we're going to waste their time and do nothing until we fix the player
            actions.send(ActionEvent(Box::new(MoveAction {
                entity: top,
                direction: rng.gen2d(-1..2, -1..2),
            })));

            turn_order.pushback(100);
        }
    }
}

pub struct SvarogAIPlugin;

impl Plugin for SvarogAIPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_systems(Update, ai_agents_act.run_if(in_state(GameStates::Game)));
    }
}
