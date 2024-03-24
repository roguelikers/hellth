use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    actions::play_sfx, character::{Character, CharacterStat}, grid::WorldEntity, health::Health, history::HistoryLog
};

use super::{AbstractAction, Action, ActionResult};
use crate::game::magic::Focus;

#[derive(Debug)]
pub struct FocusAction {
    pub who: Entity,
}

pub fn a_focus(who: Entity) -> AbstractAction {
    Box::new(FocusAction { who })
}

impl Action for FocusAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::ARC
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            ResMut<HistoryLog>,
            Query<(&mut Character, &WorldEntity, &mut Focus, &Health)>,
        )>::new(world);

        let (mut log, mut world_entity_query) = read_system_state.get_mut(world);

        let Ok((char, entity, mut focus, health)) = world_entity_query.get_mut(self.who) else {
            return vec![];
        };

        focus.0 += 1;
        if focus.0 >= health.hitpoints.len() as u32 {
            focus.0 = 0;
        }

        let mut log_written = false;
        if entity.is_player {
            if focus.0 > 0 {
                log.add(&format!("Your focus is raised to {}.", focus.0));
                log_written = true;
            } else {
                log.add("You focus. You can implant consumed thaumaturgy deeper into your soul.");
                log_written = true;
            }
        } else {
            log.add(&format!("{} focuses.", entity.name));
            log_written = true;
        }
        log.add("");
        
        if entity.is_player {
            play_sfx("gameplay_surprise", world);
        }
        
        vec![]
    }
}
