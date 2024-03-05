use bevy::prelude::*;

use crate::game::{
    fov::{LastSeen, RecalculateFOVEvent},
    grid::{Grid, WorldData, WorldEntity},
    procgen::PlayerMarker,
    turns::TurnOrder,
};

#[derive(Event)]
pub struct MoveCommand {
    pub entity: Entity,
    pub direction: IVec2,
    pub cost: i32,
}

// if someone could google where i should put this to get it to work on EVERYTHING in this project,
// that would be great :D
#[allow(clippy::too_many_arguments)]
pub fn execute_move_command(
    mut move_commands: EventReader<MoveCommand>,
    mut fov_commands: EventWriter<RecalculateFOVEvent>,
    grid: Res<Grid>,
    world: Res<WorldData>,
    mut turn_order: ResMut<TurnOrder>,
    mut game_entities: Query<(&mut WorldEntity, &mut Transform)>,
    mut last_seens: Query<&mut LastSeen>,
    player_markers: Query<&PlayerMarker>,
) {
    for MoveCommand {
        entity,
        direction,
        cost,
    } in move_commands.read()
    {
        let Ok((mut world_entity, mut transform)) = game_entities.get_mut(*entity) else {
            return;
        };

        if !world.solid.contains(&(world_entity.position + *direction)) {
            world_entity.position += *direction;
            let (x, y) = grid.norm(world_entity.position);

            if let Ok(mut last_seen) = last_seens.get_mut(*entity) {
                if world.data.is_in_fov(x, y) {
                    *last_seen = LastSeen(Some(world_entity.position));
                }
            }

            if player_markers.contains(*entity) {
                // ok very broken :D
                // this should maybe not be done here but rather in the fov recalc, EXCEPT if we're the player
                transform.translation = grid.get_tile_position(world_entity.position).translation;

                fov_commands.send(RecalculateFOVEvent);
            }
        }

        turn_order.pushback(*cost);
    }
}

pub struct MoveCommandPlugin;
impl Plugin for MoveCommandPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_event::<MoveCommand>();
        bevy.add_systems(
            Update,
            execute_move_command.run_if(on_event::<MoveCommand>()),
        );
    }
}
