use bevy::prelude::*;
use doryen_fov::{FovAlgorithm, FovRecursiveShadowCasting};

use super::{
    grid::{Grid, WorldData, WorldEntity, FOV},
    procgen::PlayerMarker,
};

#[derive(Event)]
pub struct RecalculateFOVEvent;

#[derive(Component)]
pub struct Sight(pub u32);

#[derive(Component, Default)]
pub struct LastSeen(pub Option<IVec2>);

pub fn on_new_fov_added(
    query: Query<Added<FOV>>,
    mut recalc_event: EventWriter<RecalculateFOVEvent>,
) {
    for _ in &query {
        recalc_event.send(RecalculateFOVEvent);
    }
}

macro_rules! norm_x {
    ($vec2:expr, $grid:ident) => {
        ($vec2.x + $grid.size.x / 2 + 1) as usize
    };
}

macro_rules! norm_y {
    ($vec2:expr, $grid:ident) => {
        ($vec2.y + $grid.size.y / 2 + 1) as usize
    };
}

#[allow(clippy::too_many_arguments)]
pub fn recalculate_fov(
    mut recalc_event: EventReader<RecalculateFOVEvent>,
    player_entity: Query<(&WorldEntity, &Sight), With<PlayerMarker>>,
    non_players: Query<(Entity, &WorldEntity), Without<PlayerMarker>>,
    grid: Option<Res<Grid>>,
    map: Option<ResMut<WorldData>>,
    mut last_seen: Query<&mut LastSeen>,
    mut sprites: Query<&mut TextureAtlasSprite>,
    mut visibility: Query<&mut Visibility>,
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

    let Ok((game_entity, sight)) = &player_entity.get_single() else {
        return;
    };

    let mut fov = FovRecursiveShadowCasting::new();

    map.data.clear_fov();
    fov.compute_fov(
        &mut map.data,
        norm_x!(game_entity.position, grid),
        norm_y!(game_entity.position, grid),
        sight.0 as usize,
        true,
    );

    grid.entities.iter().for_each(|(pos, e)| {
        let Ok(mut vis) = visibility.get_mut(*e) else {
            return;
        };

        let Ok(mut sprite) = sprites.get_mut(*e) else {
            return;
        };

        if map.data.is_in_fov(norm_x!(pos, grid), norm_y!(pos, grid)) {
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

    for (non_player_entity, non_player_game_entity) in &non_players {
        let Ok(mut vis) = visibility.get_mut(non_player_entity) else {
            continue;
        };

        let pos = non_player_game_entity.position;
        if map.data.is_in_fov(norm_x!(pos, grid), norm_y!(pos, grid)) {
            *vis = Visibility::Visible;
        } else {
            // we should remember the last seen position, maybe
            // but should do that via the _player's_ memory
            // which i could distribute over the other objects,
            // so that we wouldn't have to have a hash in one place
            *vis = Visibility::Hidden;

            // wait... hmmm... the real position shouldn't be used for positioning, even though it changes
            //
            //       model                        view
            //     position                    last_seen_at
            //        - this changes over time
            //        |------- bound to change ---->|
            //                                      - this is the only thing shown
            //
            if let Ok(mut last_seen_at) = last_seen.get_mut(non_player_entity) {
                *last_seen_at = LastSeen(Some(pos));
            }
        }
    }
}
