use crate::game::{actions::*, feel::Random};
use bevy::prelude::*;

use super::{get_player, get_positions_and_health, AIBehaviour, AbstractAIBehaviour};

#[derive(Debug)]
pub struct AggroAIThinking;

pub fn ai_aggro() -> AbstractAIBehaviour {
    Box::new(AggroAIThinking)
}

impl AIBehaviour for AggroAIThinking {
    fn do_thinking(&self, entity: Entity, world: &mut World) -> Vec<AbstractAction> {
        let player = {
            let Some(player) = get_player(world) else {
                return vec![a_random_walk(entity)];
            };

            player
        };

        let stats = { get_positions_and_health(world, &[entity, player]) };

        let Some((player_pos, player_hp)) = stats.get(&player).cloned().unwrap_or_default() else {
            return vec![a_random_walk(entity)];
        };

        let Some((enemy_pos, enemy_hp)) = stats.get(&entity).cloned().unwrap_or_default() else {
            return vec![a_random_walk(entity)];
        };

        let mut rng = world.get_resource_mut::<Random>().unwrap();

        vec![
            a_track(entity, player),
            a_track(entity, player),
            a_track(entity, player),
            if rng.percent(20u32) { a_wait() } else { a_track(entity, player) },
        ]
    }
}
