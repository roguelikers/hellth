use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::{Character, CharacterStat},
    grid::WorldEntity,
    history::HistoryLog,
    inventory::{CarriedItems, EquippedItems, Item, ItemType},
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct EquipAction {
    who: Entity,
    what: Entity,
}

pub fn a_equip(who: Entity, what: Entity) -> AbstractAction {
    Box::new(EquipAction { who, what })
}

impl Action for EquipAction {
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

        let mut message = vec![];
        if let Ok((mut character, world_entity, carried, mut equipped)) =
            world_entity_query.get_mut(self.who)
        {
            if carried.0.iter().any(|i| *i == self.what)
                && !equipped.0.iter().any(|i| *i == self.what)
            {
                message.push(format!("{} equipped {}.", world_entity.name, item.name));
                equipped.0.push(self.what);

                let count_weapons = equipped
                    .0
                    .iter()
                    .filter(|&i| {
                        if let Ok(item) = item_query.get(*i) {
                            item.item_type == ItemType::Weapon
                        } else {
                            false
                        }
                    })
                    .count();

                println!("Count weapons: {:?}", count_weapons);

                for (stat, val) in &item.equip_stat_changes {
                    character[*stat] += *val;
                    {
                        let e = character.counters.entry(*stat).or_insert(0);
                        *e += 1;
                    }
                    message.push(format!(
                        "{} {} {} by {}.",
                        world_entity.name,
                        if *val > 0 { "raise" } else { "lower" },
                        format!("{:?}", *stat).to_uppercase(),
                        val.abs()
                    ));
                }

                log.add(&message.join(" "));
            }
        }

        vec![]
    }
}
