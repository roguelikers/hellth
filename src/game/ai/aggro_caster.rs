use crate::game::{actions::*, ai::AIStrategy};
use bevy::prelude::*;

use super::{get_player, get_positions_and_health, AIBehaviour, AbstractAIBehaviour};

#[derive(Debug)]
pub struct AggroCasterAIThinking;

pub fn ai_aggro_caster() -> AbstractAIBehaviour {
    Box::new(AggroCasterAIThinking)
}

impl AIBehaviour for AggroCasterAIThinking {
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

        let distance = (player_pos.distance_squared(enemy_pos) as f32).sqrt();

        if enemy_hp.hitpoints.len() + 1 >= player_hp.hitpoints.len() && distance < 1.45 {
            return vec![a_track(entity, player)];
        }

        let mut bravery = if enemy_hp.hitpoints.len() > player_hp.hitpoints.len() {
            7
        } else {
            -1
        };

        if distance > 10.0 {
            return vec![a_random_walk(entity), a_track(entity, player)];
        }

        if distance > 1.44 {
            bravery += 4;
        } else {
            bravery += 10;
        }

        if bravery > 0 {
            if distance > 4.0 {
                vec![
                    a_track(entity, player),
                    a_behave(entity, AIStrategy::Caster),
                ]
            } else {
                vec![
                    a_track(entity, player),
                    a_flee(entity, player),
                    a_random_walk(entity),
                ]
            }
        } else {
            vec![
                a_flee(entity, player),
                a_random_walk(entity),
                a_track(entity, player),
            ]
        }
    }
}
