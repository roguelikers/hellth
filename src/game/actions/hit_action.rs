use bevy::{ecs::system::SystemState, prelude::*};

use super::*;
use crate::game::{
    character::Character, feel::Random, grid::WorldEntity, health::Health, history::HistoryLog,
    inventory::EquippedItems, procgen::PlayerMarker,
};
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
            Query<(&mut Health, &mut Character, Option<&mut EquippedItems>)>,
            Query<&PlayerMarker>,
            Query<&WorldEntity>,
            Query<&mut Shake>,
            ResMut<Random>,
            ResMut<HistoryLog>,
        )>::new(world);
        let (mut world_health_query, player_query, world_query, mut shake_query, mut rng, mut log) =
            read_system_state.get_mut(world);

        let attacker_strength = {
            if let Ok((_, attacker_character, _attacker_equipped)) =
                world_health_query.get(self.attacker)
            {
                attacker_character.strength
            } else {
                1
            }
        };

        let Ok((mut target_health, mut target_character, _target_equipped)) =
            world_health_query.get_mut(self.target)
        else {
            return vec![];
        };

        let Ok(world_attacker) = world_query.get(self.attacker) else {
            return vec![];
        };
        let Ok(world_target) = world_query.get(self.target) else {
            return vec![];
        };

        let mut damage_amount = attacker_strength / 3;
        if damage_amount < 1 && rng.coin() {
            damage_amount = 1;
        }

        if rng.percent(
            100 - (target_character.agility * target_character.willpower).clamp(0, 50) as usize,
        ) {
            if world_target.name == "You" {
                log.add(&format!("{} move out of the way.", world_target.name));
            } else {
                log.add(&format!("{} moves out of the way.", world_target.name));
            }

            play_sfx("gameplay_surprise", world);
            return vec![];
        }

        let verb = if world_attacker.name == "You" {
            "do"
        } else {
            "does"
        };
        log.add(&format!(
            "{} {} {} damage to {}.",
            world_attacker.name,
            verb,
            damage_amount,
            world_target.name.to_lowercase()
        ));
        log.add("");
        let diff = target_health.normal_damage(damage_amount as usize);
        for (stat, val) in diff {
            target_character[stat] += val;
            {
                let e = target_character.counters.entry(stat).or_insert(0);
                *e += 1;
            }
        }

        if player_query.contains(self.target) {
            shake_query.single_mut().add_trauma(rng.gen(2..5) as f32 * 0.01);
        }

        let result = if target_health.hitpoints.is_empty() {
            vec![a_death(self.target)]
        } else {
            vec![]
        };

        play_sfx("gameplay_hit", world);
        result
    }
}
