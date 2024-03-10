use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::{Character, CharacterStat},
    grid::WorldEntity,
    history::HistoryLog,
};

use super::{AbstractAction, Action, ActionResult};
use crate::game::magic::Focus;

#[derive(Debug)]
pub struct SacrificeAction {
    pub who: Entity,
}

pub fn a_sacrifice(who: Entity) -> AbstractAction {
    Box::new(SacrificeAction { who })
}

impl Action for SacrificeAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::ARC
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state =
            SystemState::<(ResMut<HistoryLog>, Query<(&mut Character, &WorldEntity)>)>::new(world);

        let (mut log, mut world_entity_query) = read_system_state.get_mut(world);

        log.add("You make the sacrifice to descend into the ruin.");
        vec![]
    }
}
