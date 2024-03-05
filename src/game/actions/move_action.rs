use bevy::prelude::*;

use crate::game::{
    fov::{LastSeen, RecalculateFOVEvent},
    grid::{Grid, WorldData, WorldEntity},
    procgen::PlayerMarker,
    turns::TurnOrder,
};

use super::Action;

#[derive(Event)]
pub struct MoveAction {
    pub entity: Entity,
    pub direction: IVec2,
}

enum MoveResult {
    MoveSucceed {
        next_position: IVec2,
        new_transform: Transform,
        is_in_fov: bool,
    },
    #[allow(dead_code)]
    PushActor {
        push_direction: IVec2,
        pushed_entity: Entity,
    },
    PushSolid,
}

impl Action for MoveAction {
    fn do_action(&self, world: &mut World) -> Vec<Box<dyn Action>> {
        // this is the read-only part
        let move_result = {
            let Some(grid) = world.get_resource::<Grid>() else {
                return vec![];
            };

            let Some(world_data) = world.get_resource::<WorldData>() else {
                return vec![];
            };
            let Some(WorldEntity { position, .. }) = world.get::<WorldEntity>(self.entity) else {
                return vec![];
            };

            let next_position = *position + self.direction;
            let (x, y) = grid.norm(next_position);

            if !world_data.solid.contains(&next_position) {
                MoveResult::MoveSucceed {
                    next_position,
                    new_transform: grid.get_tile_position(next_position),
                    is_in_fov: world_data.data.is_in_fov(x, y),
                }
            } else {
                // todo: push non-solid here too
                MoveResult::PushSolid
            }
        };

        // by the end of this, we have free'd the world, so we can now do mut stuff
        match move_result {
            MoveResult::MoveSucceed {
                next_position,
                new_transform,
                is_in_fov,
            } => {
                if let Some(mut world_entity) = world.get_mut::<WorldEntity>(self.entity) {
                    world_entity.position = next_position;
                }

                if let Some(mut last_seen) = world.get_mut::<LastSeen>(self.entity) {
                    if is_in_fov {
                        *last_seen = LastSeen(Some(next_position));
                    }
                }

                if world.get::<PlayerMarker>(self.entity).is_some() {
                    if let Some(mut transform) = world.get_mut::<Transform>(self.entity) {
                        transform.translation = new_transform.translation;
                    }

                    world.send_event(RecalculateFOVEvent);
                }

                vec![]
            }

            _ => {
                vec![]
            }
        }
    }
}
