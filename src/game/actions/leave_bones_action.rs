use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::CharacterStat,
    fov::RecalculateFOVEvent,
    grid::Grid,
    inventory::{ItemBuilder, ItemType},
    sprites::BONES,
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct LeaveBonesAction {
    pub stats: Vec<(CharacterStat, i32)>,
    pub pos: IVec2,
}

pub fn a_leave_bones(stats: Vec<(CharacterStat, i32)>, pos: IVec2) -> AbstractAction {
    Box::new(LeaveBonesAction { stats, pos })
}

impl Action for LeaveBonesAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIS
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let item = ItemBuilder::default()
            .with_name("Bones")
            .with_image(BONES)
            .with_type(ItemType::Artifact)
            .with_stats(&self.stats);

        //let (transform, atlas) = {
        let mut read_system_state =
            SystemState::<(Res<Grid>, EventWriter<RecalculateFOVEvent>)>::new(world);
        let (grid, mut fov_events) = read_system_state.get_mut(world);

        println!("Bones created at {:?}!", self.pos);

        let mut transform = grid.get_tile_position(self.pos);
        transform.translation.z = 1.0;
        let atlas = grid.atlas.clone_weak();
        fov_events.send(RecalculateFOVEvent);

        item.create_at_raw(self.pos, world, transform, atlas);

        vec![]
    }
}
