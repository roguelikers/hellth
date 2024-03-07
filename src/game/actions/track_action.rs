use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    actions::a_move, character::CharacterStat, feel::Random, grid::WorldEntity,
    procgen::PlayerMarker,
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct TrackAction {
    pub who: Entity,
    pub target: Entity,
}

pub fn a_track(who: Entity, target: Entity) -> AbstractAction {
    Box::new(TrackAction { who, target })
}

impl Action for TrackAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIS
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut world_state = SystemState::<(
            Query<&WorldEntity>,
            Query<Entity, With<PlayerMarker>>,
            ResMut<Random>,
        )>::new(world);
        let (world_state_query, player_entities, mut rng) = world_state.get_mut(world);

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

        let dp = *player_position - *npc_position;
        let norm_dp = dp.clamp(IVec2::new(-1, -1), IVec2::new(1, 1));

        if rng.percent(70u32) {
            if rng.coin() {
                vec![a_move(self.who, IVec2::new(norm_dp.x, 0))]
            } else {
                vec![a_move(self.who, IVec2::new(0, norm_dp.y))]
            }
        } else {
            vec![a_move(self.who, norm_dp)]
        }
    }
}
