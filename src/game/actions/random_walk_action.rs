use bevy::prelude::*;

use crate::game::{actions::a_move, character::CharacterStat, feel::Random};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct RandomWalkAction {
    pub who: Entity,
}

pub fn a_random_walk(who: Entity) -> AbstractAction {
    Box::new(RandomWalkAction { who })
}

impl Action for RandomWalkAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::AGI
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let Some(mut rng) = world.get_resource_mut::<Random>() else {
            return vec![];
        };

        if rng.percent(80u32) {
            let extremes = [-1, 1];
            if rng.coin() {
                vec![a_move(
                    self.who,
                    IVec2::new(extremes[rng.gen(0..1) as usize], 0),
                )]
            } else {
                vec![a_move(
                    self.who,
                    IVec2::new(0, extremes[rng.gen(0..1) as usize]),
                )]
            }
        } else {
            vec![a_move(self.who, rng.gen2d(-1..1, -1..1))]
        }
    }
}
