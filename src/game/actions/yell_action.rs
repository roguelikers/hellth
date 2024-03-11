use std::{thread, time::Duration};

use bevy::prelude::*;

use crate::game::{character::CharacterStat, grid::WorldEntity, history::HistoryLog};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct YellAction {
    pub who: Entity,
}

pub fn a_yell(entity: Entity) -> AbstractAction {
    Box::new(YellAction { who: entity })
}

impl Action for YellAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::INT
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let name = world
            .get::<WorldEntity>(self.who)
            .map(|who| who.name.clone())
            .unwrap_or("someone".to_string());
        if let Some(mut log) = world.get_resource_mut::<HistoryLog>() {
            log.add(&format!("You hear {} yell!", name));
        }
        vec![]
    }
}
