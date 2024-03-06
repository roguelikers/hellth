use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{actions::death_action::DeathAction, grid::WorldEntity, health::Health};

use super::Action;

#[derive(Event)]
pub struct HitAction {
    pub attacker: Entity,
    pub target: Entity,
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
            vec![Box::new(DeathAction {
                target: self.target,
            })]
        } else {
            vec![]
        }
    }
}
