use bevy::prelude::*;
use doryen_fov::{FovAlgorithm, FovRecursiveShadowCasting};

use super::{
    grid::{Grid, WorldData, WorldEntity, FOV},
    health::Health,
    inventory::CarriedMarker,
    procgen::PlayerMarker,
};

#[derive(Event)]
pub struct RecalculateFOVEvent;

#[derive(Component)]
pub struct Sight(pub u32);

pub fn on_new_fov_added(
    query: Query<Added<FOV>>,
    mut recalc_event: EventWriter<RecalculateFOVEvent>,
) {
    for _ in &query {
        recalc_event.send(RecalculateFOVEvent);
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::collapsible_else_if)]
pub fn recalculate_fov(
    mut recalc_event: EventReader<RecalculateFOVEvent>,
    player_entity: Query<(&WorldEntity, &Health, &Sight), With<PlayerMarker>>,
    grid: Option<Res<Grid>>,
    map: Option<ResMut<WorldData>>,
    mut non_players: Query<(Entity, &WorldEntity, &mut Transform), Without<PlayerMarker>>,
    carried: Query<&CarriedMarker>,
    mut sprites: Query<&mut TextureAtlasSprite>,
    mut visibility: Query<&mut Visibility>,
    mut fov: Local<FovRecursiveShadowCasting>,
) {
    if !recalc_event.is_empty() {
        recalc_event.clear();
    } else {
        return;
    }

    let Some(grid) = grid else {
        return;
    };

    let Some(mut map) = map else {
        return;
    };

    let Ok((player_in_world, health, sight)) = &player_entity.get_single() else {
        return;
    };

    if health.hitpoints.is_empty() {
        grid.entities.iter().for_each(|(_pos, e)| {
            let Ok(mut vis) = visibility.get_mut(*e) else {
                return;
            };

            let Ok(mut sprite) = sprites.get_mut(*e) else {
                return;
            };

            *vis = Visibility::Visible;
            sprite.color = Color::ORANGE_RED;
        });

        for (non_player_entity, world_entity, mut transform) in &mut non_players {
            transform.translation = grid.get_tile_position(world_entity.position).translation;

            let Ok(mut vis) = visibility.get_mut(non_player_entity) else {
                return;
            };

            let Ok(mut sprite) = sprites.get_mut(non_player_entity) else {
                return;
            };

            *vis = Visibility::Visible;
            sprite.color = Color::RED;
        }

        return;
    }

    map.data.clear_fov();

    {
        let (x, y) = grid.norm(player_in_world.position);
        fov.compute_fov(&mut map.data, x, y, sight.0 as usize, true);
    }

    grid.entities.iter().for_each(|(pos, e)| {
        let Ok(mut vis) = visibility.get_mut(*e) else {
            return;
        };

        let Ok(mut sprite) = sprites.get_mut(*e) else {
            return;
        };

        let (x, y) = grid.norm(*pos);
        if map.data.is_in_fov(x, y) {
            map.memory.insert(*pos);
            sprite.color = Color::WHITE;
            *vis = Visibility::Visible;
        } else if map.memory.contains(pos) {
            sprite.color = Color::GRAY;
            *vis = Visibility::Visible;
        } else {
            sprite.color = Color::BLACK;
            *vis = Visibility::Hidden;
        }
    });

    for (non_player_entity, world_entity, mut transform) in &mut non_players {
        let Ok(mut vis) = visibility.get_mut(non_player_entity) else {
            continue;
        };

        let Ok(mut sprite) = sprites.get_mut(non_player_entity) else {
            continue;
        };
        let (x, y) = grid.norm(world_entity.position);

        if carried.contains(non_player_entity) {
            continue;
        }

        if map.data.is_in_fov(x, y) {
            *vis = Visibility::Visible;
            sprite.color = Color::WHITE;
            transform.translation = grid.get_tile_position(world_entity.position).translation;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}
