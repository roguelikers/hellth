use std::collections::VecDeque;

use bevy::{ecs::system::SystemState, prelude::*, utils::HashMap};

use super::{
    actions::{
        ai_think_action::{AIBehaviour, AIThinkAction},
        AbstractAction, ActionEvent,
    },
    grid::WorldEntity,
    health::Health,
    procgen::PlayerMarker,
    turns::TurnOrder,
    GameStates,
};

#[derive(Component, Default)]
pub struct AIPlan(pub VecDeque<AbstractAction>);

#[derive(Component, Default)]
pub struct AIAgent(pub AIBehaviour);

pub fn ai_agents_act(
    mut turn_order: ResMut<TurnOrder>,
    player: Query<(Entity, &WorldEntity), With<PlayerMarker>>,
    mut non_players: Query<(&WorldEntity, &AIAgent, &mut AIPlan), Without<PlayerMarker>>,
    mut actions: EventWriter<ActionEvent>,
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
            if let Ok((_world_top, ai_agent, mut ai_plan)) = non_players.get_mut(top) {
                if ai_plan.0.is_empty() {
                    actions.send(ActionEvent(Box::new(AIThinkAction {
                        entity: top,
                        behaviour: ai_agent.0,
                    })));
                    turn_order.pushback(50);
                } else {
                    actions.send(ActionEvent(ai_plan.0.pop_front().unwrap()));
                    turn_order.pushback(100);
                }
            } else {
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
