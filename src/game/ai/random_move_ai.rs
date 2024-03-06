use super::{AIBehaviour, AIStrategy, AbstractAIBehaviour};
use crate::game::actions::*;
use crate::game::feel::Random;
use bevy::prelude::*;

#[derive(Debug)]
pub struct RandomMoveAIThinking;

pub fn ai_random_move() -> AbstractAIBehaviour {
    Box::new(RandomMoveAIThinking)
}

impl AIBehaviour for RandomMoveAIThinking {
    fn do_thinking(&self, entity: Entity, world: &mut World) -> Vec<AbstractAction> {
        let mut result = vec![];

        let Some(mut rng) = world.get_resource_mut::<Random>() else {
            return vec![];
        };

        for _ in 0..(rng.gen(5..10) as usize) {
            result.push(a_random_walk(entity));
        }
        result.push(a_behave(entity, AIStrategy::Standard));

        result
    }
}
