use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::{Character, CharacterStat},
    grid::WorldEntity,
    history::HistoryLog,
    inventory::{CarriedItems, EquippedItems, Item},
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct UnequipAction {
    who: Entity,
    what: Entity,
}

pub fn a_unequip(who: Entity, what: Entity) -> AbstractAction {
    Box::new(UnequipAction { who, what })
}

impl Action for UnequipAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::AGI
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            ResMut<HistoryLog>,
            Query<&Item>,
            Query<(
                &mut Character,
                &WorldEntity,
                &CarriedItems,
                &mut EquippedItems,
            )>,
        )>::new(world);

        let (mut log, item_query, mut world_entity_query) = read_system_state.get_mut(world);

        let Ok(item) = item_query.get(self.what) else {
            return vec![];
        };

        if let Ok((mut character, world_entity, carried, mut equipped)) =
            world_entity_query.get_mut(self.who)
        {
            if carried.0.iter().any(|i| *i == self.what) {
                if let Some(pos) = equipped.0.iter().position(|i| *i == self.what) {
                    equipped.0.remove(pos);

                    for (stat, val) in &item.equip_stat_changes {
                        character[*stat] -= *val;
                    }

                    log.add(&format!("{} unequipped {}.", world_entity.name, item.name));
                }
            }
        }

        vec![]
    }
}
