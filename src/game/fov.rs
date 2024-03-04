use bevy::prelude::*;
use doryen_fov::{FovAlgorithm, FovRecursiveShadowCasting};

use super::grid::{GameEntity, Grid, WorldData, FOV};

#[derive(Event)]
pub struct RecalculateFOVEvent;

#[derive(Component)]
pub struct Sight(pub u32);

pub fn on_new_fov_added(
    query: Query<Entity, Added<FOV>>,
    mut recalc_event: EventWriter<RecalculateFOVEvent>,
) {
    for _ in &query {
        recalc_event.send(RecalculateFOVEvent);
    }
}

pub fn recalculate_fov(
    mut recalc_event: EventReader<RecalculateFOVEvent>,
    fovs: Query<(Entity, &GameEntity, &Sight)>,
    grid: Option<Res<Grid>>,
    map: Option<ResMut<WorldData>>,
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

    for (_entity, game_entity, sight) in &fovs {
        let mut fov = FovRecursiveShadowCasting::new();

        map.data.clear_fov();
        fov.compute_fov(
            &mut map.data,
            (game_entity.position.x + grid.size.x / 2 + 1) as usize,
            (game_entity.position.y + grid.size.y / 2 + 1) as usize,
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

            if map.data.is_in_fov(
                (pos.x + grid.size.x / 2 + 1) as usize,
                (pos.y + grid.size.y / 2 + 1) as usize,
            ) {
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
    }
}
