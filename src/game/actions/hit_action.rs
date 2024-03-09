use bevy::{ecs::system::SystemState, prelude::*};

use super::*;
use crate::game::{character::Character, feel::Random, health::Health, procgen::PlayerMarker};
use bevy_trauma_shake::Shake;

#[derive(Debug)]
pub struct HitAction {
    pub attacker: Entity,
    pub target: Entity,
}

pub fn a_hit(attacker: Entity, target: Entity) -> AbstractAction {
    Box::new(HitAction { attacker, target })
}

impl Action for HitAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::STR
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            Query<(&mut Health, &mut Character)>,
            Query<&PlayerMarker>,
            Query<&mut Shake>,
            ResMut<Random>,
        )>::new(world);
        let (mut world_health_query, player_query, mut shake_query, mut rng) =
            read_system_state.get_mut(world);

        let attacker_strength = {
            if let Ok((_, attacker_character)) = world_health_query.get(self.attacker) {
                attacker_character.strength
            } else {
                1
            }
        };

        let Ok((mut target_health, mut character)) = world_health_query.get_mut(self.target) else {
            return vec![];
        };

        let mut damage_amount = attacker_strength / 3;
        if damage_amount < 1 && rng.coin() {
            damage_amount = 1;
        }

        let diff = target_health.normal_damage(damage_amount as usize);
        for (stat, val) in diff {
            character[stat] += val;
        }

        if player_query.contains(self.target) {
            shake_query.single_mut().add_trauma(0.05);
        } else {
            shake_query.single_mut().add_trauma(0.02);
        }

        if target_health.hitpoints.is_empty() {
            vec![a_death(self.target)]
        } else {
            vec![]
        }
    }
}
