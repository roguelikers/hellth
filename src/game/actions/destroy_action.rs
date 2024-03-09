use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::CharacterStat,
    grid::{WorldData, WorldEntity},
    inventory::{CarriedItems, EquippedItems, Item},
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct DestroyAction {
    pub what: Entity,
}

pub fn a_destroy(what: Entity) -> AbstractAction {
    Box::new(DestroyAction { what })
}

impl Action for DestroyAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            ResMut<WorldData>,
            Query<&mut WorldEntity, With<Item>>,
            Query<(&mut CarriedItems, Option<&mut EquippedItems>)>,
        )>::new(world);

        let (mut world_data, mut world_entity_query, mut items_query) =
            read_system_state.get_mut(world);

        let Ok(world_entity) = world_entity_query.get_mut(self.what) else {
            return vec![];
        };

        world_data.blocking.remove(&world_entity.position);

        for (mut inventory, equipped) in &mut items_query {
            if let Some(position) = inventory.0.iter().position(|p| p == &self.what) {
                inventory.0.remove(position);
            }

            if let Some(mut equipment) = equipped {
                if let Some(position) = equipment.0.iter().position(|p| p == &self.what) {
                    equipment.0.remove(position);
                }
            }
        }

        {
            let mut to_remove = vec![self.what];
            if let Some(ch) = world.get::<Children>(self.what) {
                for c in ch.iter() {
                    to_remove.push(*c);
                }
            }

            for rem in to_remove {
                world.despawn(rem);
            }
        }

        vec![]
    }
}
