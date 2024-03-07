use bevy::prelude::*;

use crate::game::ai::{AbstractAIBehaviour, PendingActions};

use super::*;

#[derive(Debug)]
pub struct AIThinkAction {
    pub entity: Entity,
    pub behaviour: AbstractAIBehaviour,
}

pub fn a_think(entity: Entity, behaviour: AbstractAIBehaviour) -> AbstractAction {
    Box::new(AIThinkAction { entity, behaviour })
}

impl Action for AIThinkAction {
    fn do_action(&self, world: &mut World) -> Vec<AbstractAction> {
        let planned_actions = self.behaviour.do_thinking(self.entity, world);
        if let Some(mut plan) = world.get_mut::<PendingActions>(self.entity) {
            plan.0 = VecDeque::from_iter(planned_actions);
        }

        vec![]
    }
}
