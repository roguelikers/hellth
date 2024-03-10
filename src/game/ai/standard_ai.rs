use crate::game::actions::*;
use bevy::prelude::*;

use super::{get_player, get_positions_and_health, AIBehaviour, AIStrategy, AbstractAIBehaviour};

#[derive(Debug)]
pub struct StandardAIThinking;

pub fn ai_standard() -> AbstractAIBehaviour {
    Box::new(StandardAIThinking)
}

impl AIBehaviour for StandardAIThinking {
    fn do_thinking(&self, entity: Entity, world: &mut World) -> Vec<AbstractAction> {
        let player = {
            let Some(player) = get_player(world) else {
                return vec![a_random_walk(entity)];
            };

            player
        };

        let stats = { get_positions_and_health(world, &[entity, player]) };

        let Some((player_pos, player_hp)) = stats.get(&player).cloned().unwrap_or_default() else {
            return vec![];
        };

        let Some((enemy_pos, enemy_hp)) = stats.get(&entity).cloned().unwrap_or_default() else {
            return vec![];
        };

        let distance = (player_pos.distance_squared(enemy_pos) as f32).sqrt();

        if enemy_hp.hitpoints.len() + 1 >= player_hp.hitpoints.len() && distance < 3.0 {
            return vec![
                a_track(entity, player),
                a_track(entity, player),
                a_track(entity, player),
            ];
        }

        let mut bravery = if enemy_hp.hitpoints.len() > player_hp.hitpoints.len() {
            2
        } else {
            -1
        };

        if player_hp.hitpoints.len() == player_hp.size {
            bravery += 5;
        }

        if enemy_hp.hitpoints.len() < enemy_hp.size / 2 {
            bravery -= 2;
        }

        if enemy_hp.hitpoints.len() < enemy_hp.size / 4 {
            bravery -= 2;
        }

        if distance > 10.0 {
            return vec![a_random_walk(entity)];
        }

        if distance > 1.44 {
            bravery += 2;
        } else {
            bravery += 10;
        }

        if bravery > 0 {
            vec![
                a_track(entity, player),
                a_track(entity, player),
                a_track(entity, player),
                a_random_walk(entity),
            ]
        } else {
            vec![
                a_flee(entity, player),
                a_flee(entity, player),
                a_behave(entity, AIStrategy::RandomMove),
                a_flee(entity, player),
            ]
        }
    }
}
