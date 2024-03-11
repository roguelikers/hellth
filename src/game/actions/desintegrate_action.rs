use bevy::prelude::*;

use crate::game::character::CharacterStat;

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct DesintegrateAction {
    pub what: Entity,
}

pub fn a_desintegrate(what: Entity) -> AbstractAction {
    Box::new(DesintegrateAction { what })
}

impl Action for DesintegrateAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&mut self, world: &mut World) -> ActionResult {
        let mut to_remove = vec![self.what];
        if let Some(ch) = world.get::<Children>(self.what) {
            for c in ch.iter() {
                to_remove.push(*c);
            }
        }

        for rem in to_remove {
            world.despawn(rem);
        }

        vec![]
    }
}
