use bevy::{prelude::*, render::camera::CameraUpdateSystem, transform::TransformSystem};

use super::{
    actions::{move_action::MoveAction, ActionEvent},
    grid::{WorldData, WorldEntity},
    procgen::PlayerMarker,
    turns::TurnOrder,
    GameStates,
};

pub fn character_controls(
    mut turn_order: ResMut<TurnOrder>,
    map: Res<WorldData>,
    keys: Res<Input<KeyCode>>,
    player_query: Query<(Entity, &WorldEntity), With<PlayerMarker>>,
    mut actions: EventWriter<ActionEvent>,
) {
    if let Some(e) = turn_order.peek() {
        if !player_query.contains(e) {
            return;
        }
    }

    let Ok((entity, player_game_entity)) = player_query.get_single() else {
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
            actions.send(ActionEvent(Box::new(MoveAction { entity, direction })));
            turn_order.pushback(50);
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
