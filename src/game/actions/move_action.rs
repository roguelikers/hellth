use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    fov::{LastSeen, RecalculateFOVEvent},
    grid::{Grid, WorldData, WorldEntity},
    procgen::PlayerMarker,
};

use super::*;

pub struct MoveAction {
    pub entity: Entity,
    pub direction: IVec2,
}

pub fn a_move(who: Entity, wher: IVec2) -> AbstractAction {
    Box::new(MoveAction {
        entity: who,
        direction: wher,
    })
}

enum MoveResult {
    MoveSucceed {
        next_position: IVec2,
        new_transform: Transform,
        is_in_fov: bool,
    },
    #[allow(dead_code)]
    CancelMove,
}

impl Action for MoveAction {
    fn do_action(&self, world: &mut World) -> Vec<Box<dyn Action>> {
        if self.direction == IVec2::ZERO {
            return vec![];
        }

        // this is the read-only part
        let move_result = {
            let mut read_system_state =
                SystemState::<(Res<Grid>, Res<WorldData>, Query<&WorldEntity>)>::new(world);

            let (grid, world_data, world_entities) = read_system_state.get(world);

            let Ok(WorldEntity { position, .. }) = world_entities.get(self.entity) else {
                return vec![];
            };

            let next_position = *position + self.direction;
            let (x, y) = grid.norm(next_position);

            if !world_data.solid.contains(&next_position) {
                if !world_data.blocking.contains_key(&next_position) {
                    MoveResult::MoveSucceed {
                        next_position,
                        new_transform: grid.get_tile_position(next_position),
                        is_in_fov: world_data.data.is_in_fov(x, y),
                    }
                } else {
                    return vec![a_melee(self.entity, self.direction)];
                }
            } else {
                // todo: push non-solid here too
                MoveResult::CancelMove
            }
        };

        // by the end of this, we have free'd the world, so we can now do mut stuff
        match move_result {
            MoveResult::MoveSucceed {
                next_position,
                new_transform,
                is_in_fov,
            } => {
                let mut write_system_state = SystemState::<(
                    Query<&mut WorldEntity>,
                    Query<&mut LastSeen>,
                    Query<(&PlayerMarker, &mut Transform)>,
                    ResMut<WorldData>,
                    EventWriter<RecalculateFOVEvent>,
                )>::new(world);

                let (
                    mut world_entity_query,
                    mut last_seen_query,
                    mut player_transform_query,
                    mut world_data,
                    mut fov_events,
                ) = write_system_state.get_mut(world);

                if let Ok(mut world_entity) = world_entity_query.get_mut(self.entity) {
                    world_data.blocking.remove(&world_entity.position);
                    world_entity.position = next_position;
                    if world_entity.blocking {
                        world_data.blocking.insert(next_position, self.entity);
                    }
                }

                if is_in_fov {
                    if let Ok(mut last_seen) = last_seen_query.get_mut(self.entity) {
                        *last_seen = LastSeen(Some(next_position));
                    }
                }

                if let Ok((_, mut transform)) = player_transform_query.get_mut(self.entity) {
                    transform.translation = new_transform.translation;
                    fov_events.send(RecalculateFOVEvent);
                }

                vec![]
            }

            _ => {
                vec![]
            }
        }
    }
}
