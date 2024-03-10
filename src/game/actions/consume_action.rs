use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    actions::a_destroy,
    character::{Character, CharacterStat},
    grid::WorldEntity,
    health::Health,
    history::HistoryLog,
    inventory::Item,
    magic::Focus,
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct ConsumeAction {
    pub who: Entity,
    pub what: Entity,
}

pub fn a_consume(who: Entity, what: Entity) -> AbstractAction {
    Box::new(ConsumeAction { who, what })
}

impl Action for ConsumeAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            ResMut<HistoryLog>,
            Query<&Item>,
            Query<(&mut Character, &WorldEntity, &mut Health, &mut Focus)>,
        )>::new(world);

        let (mut log, item_query, mut world_entity_query) = read_system_state.get_mut(world);

        let Ok(item) = item_query.get(self.what) else {
            return vec![];
        };

        if let Ok((mut character, world_entity, mut health, mut focus)) =
            world_entity_query.get_mut(self.who)
        {
            let mut message = vec![format!("{} consumed {}.", world_entity.name, item.name)];

            if focus.0 > 0 && world_entity.is_player {
                message
                    .push("You are focused, enchanting deeper reaches of your soul.".to_string());
            }

            let hp_total = (health.hitpoints.len() - 1) as isize;
            let mut already_missed = false;
            for (index, effect_val) in item.equip_stat_changes.iter().enumerate() {
                let pos = hp_total - index as isize - focus.0 as isize;

                if pos < 0 && !already_missed {
                    message.push("Part of the spell missed.".to_string());
                    already_missed = true;
                    continue;
                }

                if let Some(hp) = health.hitpoints.get_mut(pos as usize) {
                    for (effect, val) in hp.enchant(*effect_val) {
                        character[effect] += val;

                        {
                            let e = character.counters.entry(effect).or_insert(0);
                            *e += 1;
                        }

                        message.push(format!(
                            "{} {} {} by {}.",
                            world_entity.name,
                            if val > 0 { "raise" } else { "lower" },
                            format!("{:?}", effect).to_uppercase(),
                            val.abs()
                        ));
                    }
                }
            }

            focus.0 = 0;

            log.add(&message.join(" "));
        }

        vec![a_destroy(self.what)]
    }
}
