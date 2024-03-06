use std::{thread, time::Duration};

use bevy::prelude::*;

use super::Action;

#[derive(Event)]
pub struct WaitAction;

impl Action for WaitAction {
    fn do_action(&self, _world: &mut World) -> Vec<Box<dyn Action>> {
        thread::sleep(Duration::from_millis(18));
        vec![]
    }
}
