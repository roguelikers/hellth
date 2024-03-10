use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::CharacterStat,
    grid::WorldEntity,
    history::HistoryLog,
    inventory::{CarriedItems, CarriedMarker, Item},
    procgen::ClearLevel,
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct PickupAction {
    who: Entity,
    what: Vec<Entity>,
}

pub fn a_pickup(who: Entity, what: Vec<Entity>) -> AbstractAction {
    Box::new(PickupAction { who, what })
}

impl Action for PickupAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::AGI
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut clear_items = vec![];

        let mut read_system_state = SystemState::<(
            Query<(&WorldEntity, Option<&mut CarriedItems>)>,
            Query<(&Item, &mut Visibility)>,
            Query<(&ClearLevel)>,
            ResMut<HistoryLog>,
        )>::new(world);

        let (mut world_entities, mut items, clear, mut log) = read_system_state.get_mut(world);

        let Ok((person_entity, Some(mut person_carrying))) = world_entities.get_mut(self.who)
        else {
            return vec![];
        };

        let mut mark_carried = vec![];
        for item_entity in &self.what {
            let Ok((item, mut vis)) = items.get_mut(*item_entity) else {
                continue;
            };

            if person_carrying.0.len() < 9 {
                person_carrying.0.push(*item_entity);
                *vis = Visibility::Hidden;

                mark_carried.push(*item_entity);

                if person_entity.is_player {
                    log.add(&format!("Picked up {}.", item.name));
                    if clear.contains(*item_entity) {
                        clear_items.push(*item_entity);
                    }
                }
            } else {
                log.add("No more space for items. Drop something first.");
            }
        }

        {
            for marked in mark_carried {
                world.entity_mut(marked).insert(CarriedMarker);
            }
        }

        {
            for item in clear_items {
                world.entity_mut(item).remove::<ClearLevel>();
            }
        }

        vec![]
    }
}
