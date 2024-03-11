use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::grid::{Grid, WorldEntity};

use super::*;

#[derive(Debug)]
pub struct TeleportAction {
    pub entity: Entity,
    pub place: IVec2,
}

pub fn a_teleport(who: Entity, wher: IVec2) -> AbstractAction {
    Box::new(TeleportAction {
        entity: who,
        place: wher,
    })
}

impl Action for TeleportAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::AGI
    }

    fn do_action(&mut self, world: &mut World) -> ActionResult {
        let mut read_system_state =
            SystemState::<(Res<Grid>, Query<(&mut WorldEntity, &mut Transform)>)>::new(world);

        let (grid, mut world_entities) = read_system_state.get_mut(world);

        let Ok((mut world_entity, mut transform)) = world_entities.get_mut(self.entity) else {
            return vec![];
        };

        let next_position = self.place;
        let mut new_transform = grid.get_tile_position(next_position);
        new_transform.translation.z = transform.translation.z;

        *transform = new_transform;
        world_entity.position = self.place;

        vec![]
    }
}
