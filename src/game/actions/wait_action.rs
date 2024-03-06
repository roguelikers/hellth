use std::{thread, time::Duration};

use bevy::prelude::*;

use super::{AbstractAction, Action};

pub struct WaitAction;

pub fn a_wait() -> AbstractAction {
    Box::new(WaitAction)
}

impl Action for WaitAction {
    fn do_action(&self, _world: &mut World) -> Vec<AbstractAction> {
        thread::sleep(Duration::from_millis(18));
        vec![]
    }
}
