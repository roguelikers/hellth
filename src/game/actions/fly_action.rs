use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    actions::a_break,
    character::CharacterStat,
    grid::{Grid, WorldData, WorldEntity},
    history::HistoryLog,
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct FlyAction {
    pub what: Entity,
    pub path: Vec<IVec2>,
    pub already_flying: bool,
}

pub fn a_fly(what: Entity, path: Vec<IVec2>, already_flying: bool) -> AbstractAction {
    Box::new(FlyAction {
        what,
        path,
        already_flying,
    })
}

impl Action for FlyAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            Query<(&mut WorldEntity, &mut Transform)>,
            ResMut<HistoryLog>,
            Res<Grid>,
            Res<WorldData>,
        )>::new(world);

        let (mut transforms, mut log, grid, world_data) = read_system_state.get_mut(world);

        let Ok((mut item_world, mut transform)) = transforms.get_mut(self.what) else {
            return vec![];
        };

        if self.already_flying && world_data.blocking.contains_key(&item_world.position) {
            // log.add(&format!(
            //     "HITTING BLOCKING {:?} AT {:?}",
            //     world_data.blocking.get(&item_world.position),
            //     item_world.position
            // ));
            return vec![a_break(self.what)];
        }

        if let Some(v) = self.path.first() {
            if world_data.solid.contains(v) {
                //log.add(&format!("HITTING SOLID AT {:?}", &item_world.position,));
                return vec![];
            }

            item_world.position = *v;
            let mut new_transform = grid.get_tile_position(item_world.position);
            new_transform.translation.z = transform.translation.z;
            *transform = new_transform;

            // log.add(&format!("FLYING AT {:?}", &item_world.position,));
            vec![a_fly(self.what, self.path[1..].to_vec(), true)]
        } else {
            log.add("The thrown item falls down.");
            log.add("");
            vec![a_break(self.what)]
        }
    }
}
