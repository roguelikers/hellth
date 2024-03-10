use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::{Character, CharacterStat},
    grid::WorldEntity,
    history::HistoryLog,
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

fn get_focus_based_on_arc(arc: i32) -> u32 {
    match arc {
        i32::MIN..=0_i32 => 0,
        1 => 1,
        2 => 1,
        3 => 1,
        4 => 1,
        5 => 2,
        6 => 2,
        7 => 2,
        8 => 3,
        9 => 3,
        10_i32..=i32::MAX => 4,
    }
}

impl Action for FocusAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::ARC
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            ResMut<HistoryLog>,
            Query<(&mut Character, &WorldEntity, &mut Focus)>,
        )>::new(world);

        let (mut log, mut world_entity_query) = read_system_state.get_mut(world);

        let Ok((char, entity, mut focus)) = world_entity_query.get_mut(self.who) else {
            return vec![];
        };

        focus.0 += get_focus_based_on_arc(char.arcana);
        if entity.is_player {
            log.add("You focus. You can implant consumed thaumaturgy deeper into your soul.");
        } else {
            log.add(&format!("{} focuses.", entity.name));
        }
        vec![]
    }
}
