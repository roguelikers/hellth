use std::{thread, time::Duration};

use bevy::prelude::*;

use crate::game::character::CharacterStat;

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct WaitAction;

pub fn a_wait() -> AbstractAction {
    Box::new(WaitAction)
}

impl Action for WaitAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&self, _world: &mut World) -> ActionResult {
        thread::sleep(Duration::from_millis(18));
        vec![]
    }
}
