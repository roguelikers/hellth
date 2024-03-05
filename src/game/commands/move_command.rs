use bevy::prelude::*;

use crate::game::{
    fov::RecalculateFOVEvent,
    grid::{GameEntity, Grid, WorldData},
    procgen::PlayerMarker,
    turns::TurnOrder,
};

#[derive(Event)]
pub struct MoveCommand {
    pub entity: Entity,
    pub direction: IVec2,
    pub cost: i32,
}

pub fn execute_move_command(
    mut move_commands: EventReader<MoveCommand>,
    mut fov_commands: EventWriter<RecalculateFOVEvent>,
    grid: Res<Grid>,
    map: Res<WorldData>,
    mut turn_order: ResMut<TurnOrder>,
    mut game_entities: Query<(&mut GameEntity, &mut Transform)>,
    player_markers: Query<&PlayerMarker>,
) {
    for MoveCommand {
        entity,
        direction,
        cost,
    } in move_commands.read()
    {
        let Ok((mut game_entity, mut transform)) = game_entities.get_mut(*entity) else {
            return;
        };

        if !map.solid.contains(&(game_entity.position + *direction)) {
            game_entity.position += *direction;
            transform.translation = grid.get_tile_position(game_entity.position).translation;

            if player_markers.contains(*entity) {
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
