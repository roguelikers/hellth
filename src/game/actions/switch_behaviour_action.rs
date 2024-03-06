use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::ai::{AIAgent, AIStrategy};

use super::*;

#[derive(Debug)]
pub struct SwitchBehaviourAction {
    pub entity: Entity,
    pub behaviour: AIStrategy,
}

pub fn a_behave(entity: Entity, behaviour: AIStrategy) -> AbstractAction {
    Box::new(SwitchBehaviourAction { entity, behaviour })
}

impl Action for SwitchBehaviourAction {
    fn do_action(&self, world: &mut World) -> Vec<AbstractAction> {
        let mut ai_agent_state = SystemState::<Query<&mut AIAgent>>::new(world);
        let mut ai_agent_query = ai_agent_state.get_mut(world);

        let Ok(mut ai_agent) = ai_agent_query.get_mut(self.entity) else {
            return vec![];
        };

        ai_agent.0 = self.behaviour;
        vec![]
    }
}
