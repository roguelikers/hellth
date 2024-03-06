use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{actions::a_move, grid::WorldEntity, procgen::PlayerMarker};

use super::{AbstractAction, Action};

#[derive(Debug)]
pub struct FleeAction {
    pub who: Entity,
    pub target: Entity,
}

pub fn a_flee(who: Entity, target: Entity) -> AbstractAction {
    Box::new(FleeAction { who, target })
}

impl Action for FleeAction {
    fn do_action(&self, world: &mut World) -> Vec<AbstractAction> {
        let mut world_state =
            SystemState::<(Query<&WorldEntity>, Query<Entity, With<PlayerMarker>>)>::new(world);
        let (world_state_query, player_entities) = world_state.get_mut(world);

        let Ok(player_entity) = player_entities.get_single() else {
            return vec![];
        };

        let Ok(WorldEntity {
            position: npc_position,
            ..
        }) = world_state_query.get(self.who)
        else {
            return vec![];
        };

        let Ok(WorldEntity {
            position: player_position,
            ..
        }) = world_state_query.get(player_entity)
        else {
            return vec![];
        };

        let dp = *npc_position - *player_position;
        let norm_dp = dp.clamp(IVec2::new(-1, -1), IVec2::new(1, 1));

        vec![a_move(self.who, norm_dp)]
    }
}
