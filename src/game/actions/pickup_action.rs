use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::CharacterStat,
    grid::WorldEntity,
    inventory::{CarriedItems, CarriedMarker, Item},
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
        let mut read_system_state = SystemState::<(
            Query<(&WorldEntity, Option<&mut CarriedItems>)>,
            Query<(&Item, &mut Visibility)>,
        )>::new(world);

        let (mut world_entities, mut items) = read_system_state.get_mut(world);

        let Ok((person_entity, Some(mut person_carrying))) = world_entities.get_mut(self.who)
        else {
            return vec![];
        };

        let mut mark_carried = vec![];
        for item_entity in &self.what {
            let Ok((item, mut vis)) = items.get_mut(*item_entity) else {
                continue;
            };

            person_carrying.0.push(*item_entity);
            *vis = Visibility::Hidden;

            mark_carried.push(*item_entity);

            if person_entity.is_player {
                println!("Picked up {:?}", item);
            }
        }

        for marked in mark_carried {
            world.entity_mut(marked).insert(CarriedMarker);
        }

        vec![]
    }
}
