use bevy::{prelude::*, render::camera::CameraUpdateSystem, transform::TransformSystem};

use super::{
    fov::RecalculateFOVEvent,
    grid::{GameEntity, Grid, WorldData},
    procgen::PlayerMarker,
    GameStates,
};

pub fn character_controls(
    mut fov_events: EventWriter<RecalculateFOVEvent>,
    grid: Option<Res<Grid>>,
    map: Res<WorldData>,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut GameEntity, &mut Transform), With<PlayerMarker>>,
) {
    let Some(grid) = grid else {
        return;
    };

    let Ok((mut player_game_entity, mut transform)) = player_query.get_single_mut() else {
        return;
    };

    let maybe_move = if keys.just_pressed(KeyCode::Up) {
        Some(IVec2::new(0, 1))
    } else if keys.just_pressed(KeyCode::Down) {
        Some(IVec2::new(0, -1))
    } else if keys.just_pressed(KeyCode::Left) {
        Some(IVec2::new(-1, 0))
    } else if keys.just_pressed(KeyCode::Right) {
        Some(IVec2::new(1, 0))
    } else {
        None
    };

    if let Some(direction) = maybe_move {
        if !map
            .solid
            .contains(&(player_game_entity.position + direction))
        {
            player_game_entity.position += direction;
            transform.translation = grid
                .get_tile_position(player_game_entity.position)
                .translation;
            fov_events.send(RecalculateFOVEvent);
        }
    }
}

pub struct SvarogPlayerPlugin;
impl Plugin for SvarogPlayerPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_systems(
            Update,
            character_controls
                .before(TransformSystem::TransformPropagate)
                .before(CameraUpdateSystem)
                .run_if(in_state(GameStates::Game)),
        );
    }
}
