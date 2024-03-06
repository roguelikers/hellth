use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    ai::AIAgent,
    grid::{WorldData, WorldEntity},
    health::Health,
    sprites::BONES,
    turns::{TurnOrder, TurnOrderEntity, TurnTaker},
};

use super::Action;
use crate::game::fov::RecalculateFOVEvent;

#[derive(Event)]
pub struct DeathAction {
    pub target: Entity,
}

impl Action for DeathAction {
    fn do_action(&self, world: &mut World) -> Vec<Box<dyn Action>> {
        let mut read_system_state = SystemState::<(
            Commands,
            ResMut<WorldData>,
            ResMut<TurnOrder>,
            Query<(&mut WorldEntity, &mut TextureAtlasSprite, &mut Transform)>,
            EventWriter<RecalculateFOVEvent>,
        )>::new(world);
        let (mut commands, mut world_data, mut turn_order, mut world_entity_query, mut fov_events) =
            read_system_state.get_mut(world);

        let Ok((world_entity, mut sprites, mut transform)) =
            world_entity_query.get_mut(self.target)
        else {
            return vec![];
        };

        world_data.blocking.remove(&world_entity.position);
        sprites.index = BONES.into();
        transform.translation.z -= 0.1;

        if !world_entity.is_player {
            fov_events.send(RecalculateFOVEvent);

            turn_order.order.remove(&TurnOrderEntity {
                entity: self.target,
            });
        }

        commands
            .entity(self.target)
            .remove::<TurnTaker>()
            .remove::<AIAgent>()
            .remove::<Health>();

        vec![]
    }
}
