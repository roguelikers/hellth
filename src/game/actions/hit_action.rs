use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::health::Health;

use super::*;

pub struct HitAction {
    pub attacker: Entity,
    pub target: Entity,
}

pub fn a_hit(attacker: Entity, target: Entity) -> AbstractAction {
    Box::new(HitAction { attacker, target })
}

impl Action for HitAction {
    fn do_action(&self, world: &mut World) -> Vec<Box<dyn Action>> {
        let mut read_system_state = SystemState::<Query<&mut Health>>::new(world);
        let mut world_health_query = read_system_state.get_mut(world);

        let Ok(mut target_health) = world_health_query.get_mut(self.target) else {
            return vec![];
        };

        target_health.normal_damage(1);

        if target_health.hitpoints.is_empty() {
            vec![a_death(self.target)]
        } else {
            vec![]
        }
    }
}
