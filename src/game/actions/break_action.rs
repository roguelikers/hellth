use std::{thread, time::Duration};

use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::CharacterStat,
    grid::{Grid, WorldEntity},
    history::HistoryLog,
    inventory::{Item, ItemType},
    turns::TurnTaker,
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct BreakAction {
    pub what: Entity,
}

pub fn a_break(what: Entity) -> AbstractAction {
    println!("BREAK {:?}", what);
    Box::new(BreakAction { what })
}

impl Action for BreakAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            Query<(&mut WorldEntity, &mut Transform)>,
            Query<(&Item, &mut Visibility)>,
            ResMut<HistoryLog>,
            Res<Grid>,
        )>::new(world);

        let (mut transforms, mut items, mut log, grid) = read_system_state.get_mut(world);

        let Ok((item, vis)) = items.get_mut(self.what) else {
            return vec![];
        };

        let breaks = match item.item_type {
            ItemType::Artifact => {
                log.add(&format!("Artifact {} breaks on impact!", item.name));
                true
            }
            ItemType::Potion => {
                //
                log.add(&format!("Potion {} breaks on impact!", item.name));
                true
            }

            _ => false,
        };

        if breaks {
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
