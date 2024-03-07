use bevy::{ecs::system::SystemState, prelude::*};

use super::*;
use crate::game::{health::Health, procgen::PlayerMarker};
use bevy_trauma_shake::Shake;

#[derive(Debug)]
pub struct HitAction {
    pub attacker: Entity,
    pub target: Entity,
}

pub fn a_hit(attacker: Entity, target: Entity) -> AbstractAction {
    Box::new(HitAction { attacker, target })
}

impl Action for HitAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::STR
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state =
            SystemState::<(Query<&mut Health>, Query<&PlayerMarker>, Query<&mut Shake>)>::new(
                world,
            );
        let (mut world_health_query, player_query, mut shake_query) =
            read_system_state.get_mut(world);

        let Ok(mut target_health) = world_health_query.get_mut(self.target) else {
            return vec![];
        };

        target_health.normal_damage(1);

        if player_query.contains(self.target) {
            shake_query.single_mut().add_trauma(0.05);
        } else {
            shake_query.single_mut().add_trauma(0.02);
        }

        if target_health.hitpoints.is_empty() {
            vec![a_death(self.target)]
        } else {
            vec![]
        }
    }
}
