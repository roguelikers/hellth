use bevy::{ecs::system::SystemState, prelude::*};
use bresenham::Bresenham;

use crate::game::{
    actions::a_fly,
    character::{Character, CharacterStat},
    grid::{Grid, WorldEntity},
    history::HistoryLog,
    inventory::{CarriedItems, CarriedMarker, Item},
    procgen::ClearLevel,
    turns::TurnTaker,
};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct ThrowAction {
    pub who: Entity,
    pub what: Entity,
    pub wher: IVec2,
}

pub fn a_throw(who: Entity, what: Entity, wher: IVec2) -> AbstractAction {
    Box::new(ThrowAction { who, what, wher })
}

impl Action for ThrowAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::STR
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            Query<(&mut WorldEntity, &mut Transform), Without<TurnTaker>>,
            Query<(&WorldEntity, Option<&Character>, Option<&mut CarriedItems>), With<TurnTaker>>,
            Query<(&Item, &mut Visibility)>,
            ResMut<HistoryLog>,
            Res<Grid>,
        )>::new(world);

        let (mut transforms, mut world_entities, mut items, mut log, grid) =
            read_system_state.get_mut(world);

        let Ok((person_entity, Some(character), Some(mut person_carrying))) =
            world_entities.get_mut(self.who)
        else {
            return vec![];
        };

        let str = character.strength;
        let will = character.willpower;

        let mut mark_carried = vec![];
        let item_entity = self.what;
        let Ok((item, mut vis)) = items.get_mut(item_entity) else {
            return vec![];
        };

        let Ok((mut item_world, mut transform)) = transforms.get_mut(item_entity) else {
            return vec![];
        };

        item_world.position = person_entity.position;
        let mut new_transform = grid.get_tile_position(item_world.position);
        new_transform.translation.z = transform.translation.z;
        *transform = new_transform;

        if let Some(carried_item) = person_carrying.0.iter().position(|i| *i == item_entity) {
            person_carrying.0.remove(carried_item);
            *vis = Visibility::Visible;
            log.add(&format!("{} threw {}.", person_entity.name, item.name));
            mark_carried.push(item_entity);
        }

        let a = person_entity.position;
        let b = self.wher;

        for marked in mark_carried {
            world.entity_mut(marked).remove::<CarriedMarker>();
            world.entity_mut(marked).insert(ClearLevel);
        }

        let max_dist = (((str.min(5) + will.min(5)) as f32) * 1.5).ceil() as usize;
        let b = Bresenham::new((a.x as isize, a.y as isize), (b.x as isize, b.y as isize));
        let mut path = b
            .into_iter()
            .map(|(x, y)| IVec2::new(x as i32, y as i32))
            .collect::<Vec<_>>();
        path.truncate(max_dist);

        vec![a_fly(self.what, path[1..].to_vec())]
    }
}
