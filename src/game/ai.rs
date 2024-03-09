pub mod random_move_ai;
pub mod standard_ai;

use std::collections::VecDeque;
use std::fmt::Debug;

use bevy::{ecs::system::SystemState, prelude::*, utils::HashMap};

use crate::game::turns::TurnOrderEntity;

use self::{random_move_ai::ai_random_move, standard_ai::ai_standard};

use super::{
    actions::{a_think, AbstractAction, ActionEvent},
    character::Character,
    grid::WorldEntity,
    health::Health,
    procgen::PlayerMarker,
    turns::{EndTurnEvent, TurnOrder},
    GameStates,
};

#[derive(Default, Debug, Clone, Copy)]
pub enum AIStrategy {
    #[default]
    Standard,
    RandomMove,
}

impl From<AIStrategy> for AbstractAIBehaviour {
    fn from(value: AIStrategy) -> Self {
        match value {
            AIStrategy::Standard => ai_standard(),
            AIStrategy::RandomMove => ai_random_move(),
        }
    }
}

#[derive(Component, Default)]
pub struct PendingActions(pub VecDeque<AbstractAction>);

#[derive(Component, Debug, Default)]
pub struct AIAgent(pub AIStrategy);

pub type AbstractAIBehaviour = Box<dyn AIBehaviour>;

pub trait AIBehaviour: Send + Sync + Debug {
    fn do_thinking(&self, entity: Entity, world: &mut World) -> Vec<AbstractAction>;
}

pub fn ai_agents_act(
    mut turn_order: ResMut<TurnOrder>,
    player: Query<(Entity, &WorldEntity), With<PlayerMarker>>,
    mut non_players: Query<(&Character, &AIAgent, &mut PendingActions), Without<PlayerMarker>>,
    mut actions: EventWriter<ActionEvent>,
    mut turn_ended_events: EventWriter<EndTurnEvent>,
) {
    let Ok((player_entity, _player_world)) = player.get_single() else {
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
            if let Ok((character, ai_agent, mut pending)) = non_players.get_mut(top) {
                let current_energy = turn_order
                    .order
                    .get_priority(&TurnOrderEntity { entity: top })
                    .unwrap();

                if current_energy.0 == 0 {
                    turn_order.restart_turn();
                    turn_ended_events.send(EndTurnEvent);
                    return;
                }

                #[allow(unused_assignments)]
                let mut taken_action: Option<ActionEvent> = None;
                if pending.0.is_empty() {
                    taken_action = Some(ActionEvent(a_think(top, ai_agent.0.into())));
                } else {
                    taken_action = Some(ActionEvent(pending.0.pop_front().unwrap()));
                }

                if let Some(action) = taken_action {
                    let cost = character.calculate_cost(action.0.get_affiliated_stat());
                    // println!(
                    //     "{:?} ({} energy) decides to do {:?} for {} energy",
                    //     world_entity.name, current_energy.0, action.0, cost
                    // );
                    turn_order.pushback(cost);
                    actions.send(action);
                }
            } else {
                println!("SOMETHING IS OFF HERE!");
                turn_order.pushback(100);
            }
        }
    }
}

pub fn get_player(world: &mut World) -> Option<Entity> {
    let mut world_state = SystemState::<Query<Entity, With<PlayerMarker>>>::new(world);
    let player_query = world_state.get(world);
    let Ok(p) = player_query.get_single() else {
        return None;
    };

    Some(p)
}

pub fn get_positions_and_health(
    world: &mut World,
    entities: &[Entity],
) -> HashMap<Entity, Option<(IVec2, Health)>> {
    let mut world_state = SystemState::<Query<(&WorldEntity, &Health)>>::new(world);
    let world_state_query = world_state.get(world);

    let mut results = HashMap::new();
    for entity in entities {
        match world_state_query.get(*entity) {
            Ok((world_entity, health)) => {
                results.insert(*entity, Some((world_entity.position, health.clone())));
            }
            _ => {
                results.insert(*entity, None);
            }
        }
    }

    results
}

pub struct SvarogAIPlugin;

impl Plugin for SvarogAIPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_systems(Update, ai_agents_act.run_if(in_state(GameStates::Game)));
    }
}
