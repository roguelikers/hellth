use bevy::{ecs::system::SystemState, prelude::*};

use super::*;
use crate::game::{
    character::Character, feel::Random, grid::WorldEntity, health::Health, history::HistoryLog,
    inventory::EquippedItems, procgen::PlayerMarker,
};
use bevy_trauma_shake::Shake;

#[derive(Debug)]
pub struct HealAction {
    pub entity: Entity,
}

pub fn a_heal(entity: Entity) -> AbstractAction {
    Box::new(HealAction { entity })
}

impl Action for HealAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIS
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            Query<(&mut Health, &mut Character, &WorldEntity)>,
            ResMut<HistoryLog>,
        )>::new(world);

        let (mut world_health_query, mut log) =
            read_system_state.get_mut(world);

        let Ok((mut target_health, mut target_character, world_target)) =
            world_health_query.get_mut(self.entity)
        else {
            return vec![];
        };

        target_health.normal_heal(1);

        log.add(&format!(
            "{} heals up.",
            world_target.name,
        ));
        log.add("");
        
        play_sfx("gameplay_surprise", world);
        vec![]
    }
}
