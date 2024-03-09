use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::Character,
    grid::{WorldData, WorldEntity},
    turns::{TurnOrder, TurnOrderEntity},
};

use super::*;

#[derive(Debug)]
pub struct DeathAction {
    pub entity: Entity,
}

pub fn a_death(who: Entity) -> AbstractAction {
    Box::new(DeathAction { entity: who })
}

impl Action for DeathAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let result = {
            let mut read_system_state = SystemState::<(
                ResMut<WorldData>,
                ResMut<TurnOrder>,
                Query<(&Character, &mut WorldEntity)>,
            )>::new(world);
            let (mut world_data, mut turn_order, mut world_entity_query) =
                read_system_state.get_mut(world);

            let Ok((character, world_entity)) = world_entity_query.get_mut(self.entity) else {
                return vec![];
            };

            world_data.blocking.remove(&world_entity.position);

            turn_order.order.remove(&TurnOrderEntity {
                entity: self.entity,
            });

            if !world_entity.is_player {
                let stats = make_item(character);

                if !stats.is_empty() {
                    vec![a_leave_bones(stats, world_entity.position)]
                } else {
                    vec![]
                }
            } else {
                vec![a_leave_bones(vec![], world_entity.position)]
            }
        };

        {
            let mut to_remove = vec![self.entity];
            if let Some(ch) = world.get::<Children>(self.entity) {
                for c in ch.iter() {
                    to_remove.push(*c);
                }
            }

            for rem in to_remove {
                world.despawn(rem);
            }
        }
        result
    }
}

fn make_item(char: &Character) -> Vec<(CharacterStat, i32)> {
    let mut result = vec![];
    let stats = [
        CharacterStat::STR,
        CharacterStat::ARC,
        CharacterStat::INT,
        CharacterStat::WIS,
        CharacterStat::WIL,
        CharacterStat::AGI,
    ];

    for stat in &stats {
        let val = char[*stat];
        match val {
            x if x > 3 => result.push((*stat, 1)),
            x if x < 3 => result.push((*stat, -1)),
            _ => {}
        };
    }

    result
}
