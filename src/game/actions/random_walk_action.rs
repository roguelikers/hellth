use bevy::prelude::*;

use crate::game::{actions::a_move, feel::Random};

use super::{AbstractAction, Action};

#[derive(Debug)]
pub struct RandomWalkAction {
    pub who: Entity,
}

pub fn a_random_walk(who: Entity) -> AbstractAction {
    Box::new(RandomWalkAction { who })
}

impl Action for RandomWalkAction {
    fn do_action(&self, world: &mut World) -> Vec<AbstractAction> {
        let Some(mut rng) = world.get_resource_mut::<Random>() else {
            return vec![];
        };

        vec![a_move(self.who, rng.gen2d(-1..2, -1..2))]
    }
}
