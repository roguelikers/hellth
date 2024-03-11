use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    actions::a_random_walk,
    character::{Character, CharacterStat},
    feel::Random,
    grid::{Grid, WorldData, WorldEntity},
    health::Health,
    history::HistoryLog,
    inventory::Item,
    magic::Focus,
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct InflictAction {
    pub who: Entity,
    pub target: Entity,
    pub artifact: Item,
}

pub fn a_inflict(who: Entity, target: Entity, artifact: Item) -> AbstractAction {
    Box::new(InflictAction {
        who,
        target,
        artifact,
    })
}

impl Action for InflictAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::STR
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            Query<(&mut WorldEntity, &mut Character, &mut Health)>,
            Query<Option<&Focus>>,
            Query<&Item>,
            ResMut<HistoryLog>,
            Res<Grid>,
            Res<WorldData>,
            ResMut<Random>,
        )>::new(world);

        let (mut entities, focus, _items, mut log, grid, world_data, mut rng) =
            read_system_state.get_mut(world);

        let ((x, y), name) = {
            if let Ok((attacker_entity, _attacker_char, _attacker_health)) =
                entities.get_mut(self.who)
            {
                (
                    grid.norm(attacker_entity.position),
                    attacker_entity.name.clone(),
                )
            } else {
                return vec![];
            }
        };

        let Ok((_target_entity, mut target_char, mut target_health)) =
            entities.get_mut(self.target)
        else {
            return vec![];
        };

        if !world_data.data.is_in_fov(x, y) {
            return vec![a_random_walk(self.who)];
        }

        log.add(&format!("{} chants in tongues.", name));

        if rng.percent(100 - 15u32 + target_char.arcana as u32 * 2) {
            log.add("You momentarily felt a spell affect you, but then it dissipates.");
            return vec![];
        }

        let focus = Focus(if let Ok(Some(focus)) = focus.get(self.who) {
            focus.0
        } else {
            0
        });

        let hp_total = (target_health.hitpoints.len() as isize - 1);
        let mut already_missed = false;

        let mut count = 0;

        for (index, effect_val) in self.artifact.equip_stat_changes.iter().enumerate() {
            let pos = hp_total - index as isize - focus.0 as isize;

            if pos < 0 && !already_missed {
                log.add("Your aura blocks part of an incoming spell.");
                return vec![];
            }

            if let Some(hp) = target_health.hitpoints.get_mut(pos as usize) {
                for (effect, val) in hp.enchant(*effect_val) {
                    target_char[effect] += val;

                    {
                        let e = target_char.counters.entry(effect).or_insert(0);
                        *e += 1;
                    }
                }
            }

            count += 1;
        }

        log.add(&format!(
            "You are afflicted by a curse with {} effects!",
            count
        ));

        vec![]
    }
}
