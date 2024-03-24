
use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{actions::a_destroy, character::CharacterStat, health::Health, history::HistoryLog, player::Achievements, procgen::PlayerMarker};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct FortuneAction {
    pub what: Entity,
}

pub fn a_fortune(what: Entity) -> AbstractAction {
    Box::new(FortuneAction { what })
}

impl Action for FortuneAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let message = {
            if let Some(mut ach) = world.get_resource_mut::<Achievements>() {
                ach.messages.pop().unwrap_or("...nothing at all!".to_string())
            } else {
                "...something sadly illegible...".to_string()
            }
        };
        
        if let Some(mut log) = world.get_resource_mut::<HistoryLog>() {
            log.add("You examine the scroll you found. It says:");
            log.add(&message);
            log.add("You feel your psyche heal a bit in contact with the outside world.");
        }

        let mut read_system_state = SystemState::<
            Query<&mut Health, With<PlayerMarker>>
        >::new(world);

        let mut health_query = read_system_state.get_mut(world);

        let Ok(mut health) = health_query.get_single_mut() else {
            return vec![];
        };

        health.normal_heal(1);
        
        vec![ a_destroy(self.what) ]
    }
}
