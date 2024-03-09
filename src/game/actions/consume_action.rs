use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    actions::a_destroy,
    character::{Character, CharacterStat},
    grid::WorldEntity,
    health::Health,
    history::HistoryLog,
    inventory::Item,
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
            Query<(&mut Character, &WorldEntity, &mut Health)>,
        )>::new(world);

        let (mut log, item_query, mut world_entity_query) = read_system_state.get_mut(world);

        let Ok(item) = item_query.get(self.what) else {
            return vec![];
        };

        if let Ok((mut character, world_entity, mut health)) = world_entity_query.get_mut(self.who)
        {
            log.add(&format!("{} consumed {}.", world_entity.name, item.name));

            for (index, effect_val) in item.equip_stat_changes.iter().enumerate() {
                if let Some(hp) = health.hitpoints.get_mut(index) {
                    for (effect, val) in hp.enchant(*effect_val) {
                        character[effect] += val;
                    }
                }
            }
        }

        vec![a_destroy(self.what)]
    }
}
