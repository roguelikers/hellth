use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::{Character, CharacterStat},
    grid::{Grid, WorldEntity},
    history::HistoryLog,
    inventory::{CarriedItems, CarriedMarker, EquippedItems, Item},
    procgen::ClearLevel,
    turns::TurnTaker,
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct DropAction {
    pub who: Entity,
    pub what: Vec<Entity>,
}

pub fn a_drop(who: Entity, what: Vec<Entity>) -> AbstractAction {
    Box::new(DropAction { who, what })
}

impl Action for DropAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        // curse: this action fails if your willpower is too low
        CharacterStat::WIL
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            Query<(&mut WorldEntity, &mut Transform), Without<TurnTaker>>,
            Query<
                (
                    &WorldEntity,
                    &mut Character,
                    Option<&mut CarriedItems>,
                    Option<&mut EquippedItems>,
                ),
                With<TurnTaker>,
            >,
            Query<(&Item, &mut Visibility)>,
            ResMut<HistoryLog>,
            Res<Grid>,
        )>::new(world);

        let (mut transforms, mut world_entities, mut items, mut log, grid) =
            read_system_state.get_mut(world);

        let Ok((
            person_entity,
            mut person_char,
            Some(mut person_carrying),
            Some(mut person_equipped),
        )) = world_entities.get_mut(self.who)
        else {
            return vec![];
        };

        let mut mark_carried = vec![];
        for item_entity in &self.what {
            let Ok((item, mut vis)) = items.get_mut(*item_entity) else {
                continue;
            };

            let Ok((mut item_world, mut transform)) = transforms.get_mut(*item_entity) else {
                continue;
            };

            item_world.position = person_entity.position;
            let mut new_transform = grid.get_tile_position(item_world.position);
            new_transform.translation.z = transform.translation.z;
            *transform = new_transform;

            let mut log_written = false;
            if let Some(carried_item) = person_carrying.0.iter().position(|i| i == item_entity) {
                person_carrying.0.remove(carried_item);
                if let Some(pos) = person_equipped.0.iter().position(|i| i == item_entity) {
                    person_equipped.0.remove(pos);

                    for (stat, val) in &item.equip_stat_changes {
                        person_char[*stat] -= *val;
                    }

                    log.add(&format!("{} unequipped {}.", person_entity.name, item.name));
                    log_written = true;
                }
                *vis = Visibility::Visible;
                mark_carried.push(*item_entity);
            }
            if log_written {
                log.add("");
            }
        }

        for marked in mark_carried {
            world.entity_mut(marked).remove::<CarriedMarker>();
            world.entity_mut(marked).insert(ClearLevel);
        }

        vec![]
    }
}
