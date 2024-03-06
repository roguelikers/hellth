use bevy::{ecs::system::SystemState, prelude::*};

use super::*;
use crate::game::grid::{WorldData, WorldEntity};

pub struct MeleeAttackAction {
    pub entity: Entity,
    pub direction: IVec2,
}

pub fn a_melee(who: Entity, wher: IVec2) -> AbstractAction {
    Box::new(MeleeAttackAction {
        entity: who,
        direction: wher,
    })
}

impl Action for MeleeAttackAction {
    fn do_action(&self, world: &mut World) -> Vec<Box<dyn Action>> {
        let mut read_system_state =
            SystemState::<(Res<WorldData>, Query<&WorldEntity>)>::new(world);

        let (world_data, world_entities) = read_system_state.get(world);

        let Ok(WorldEntity {
            is_player: is_attacker_player,
            position,
            ..
        }) = world_entities.get(self.entity)
        else {
            return vec![];
        };

        let next_position = *position + self.direction;

        if let Some(other) = world_data.blocking.get(&next_position) {
            let Ok(WorldEntity {
                is_player: is_target_player,
                ..
            }) = world_entities.get(*other)
            else {
                return vec![];
            };

            if is_attacker_player != is_target_player {
                vec![a_hit(self.entity, *other)]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}
