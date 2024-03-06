use ai_think_action::switch_behaviour::a_behave;
use bevy::prelude::*;

use crate::game::ai::{get_player, get_positions_and_health, AIPlan};
use crate::game::feel::Random;

use super::*;

#[derive(Default, Clone, Copy, Debug)]
pub enum AIBehaviour {
    #[default]
    RandomMove,
    Standard,
}

pub struct AIThinkAction {
    pub entity: Entity,
    pub behaviour: AIBehaviour,
}

pub fn a_think(entity: Entity, behaviour: AIBehaviour) -> AbstractAction {
    Box::new(AIThinkAction { entity, behaviour })
}

impl Action for AIThinkAction {
    fn do_action(&self, world: &mut World) -> Vec<AbstractAction> {
        let Some(mut rng) = world.get_resource_mut::<Random>() else {
            return vec![];
        };

        match self.behaviour {
            AIBehaviour::RandomMove => {
                let mut result = vec![];
                for _ in 0..(rng.gen(1..5) as usize) {
                    result.push(a_random_walk(self.entity));
                }
                result.push(a_behave(self.entity, AIBehaviour::Standard));

                result
            }

            AIBehaviour::Standard => {
                let planned_actions = do_standard_ai_thinking(self.entity, world);
                if let Some(mut plan) = world.get_mut::<AIPlan>(self.entity) {
                    plan.0 = VecDeque::from(planned_actions);
                }
                vec![]
            }
        }
    }
}

fn do_standard_ai_thinking(entity: Entity, world: &mut World) -> Vec<AbstractAction> {
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

    let mut bravery = if enemy_hp.hitpoints.len() > player_hp.hitpoints.len() {
        2
    } else {
        -1
    };

    if player_hp.hitpoints.len() == player_hp.size {
        bravery -= 1;
    }

    if enemy_hp.hitpoints.len() < enemy_hp.size / 2 {
        bravery -= 2;
    }

    if enemy_hp.hitpoints.len() < enemy_hp.size / 4 {
        bravery -= 2;
    }

    let distance = (player_pos.distance_squared(enemy_pos) as f32).sqrt();

    if distance > 10.0 {
        return vec![a_behave(entity, AIBehaviour::RandomMove)];
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
            a_behave(entity, AIBehaviour::RandomMove),
        ]
    } else {
        vec![
            a_flee(entity, player),
            a_flee(entity, player),
            a_random_walk(entity),
            a_flee(entity, player),
        ]
    }
}
