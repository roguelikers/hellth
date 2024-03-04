use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::game::{
    fov::Sight,
    grid::GameEntityBundle,
    sprite::{ChangePassability, ChangeSprite},
};

use super::{
    feel::Random,
    fov::{on_new_fov_added, recalculate_fov, RecalculateFOVEvent},
    grid::{GameEntityMarker, Grid, Passability, WorldData},
    GameStates,
};

#[derive(Event)]
pub struct ProcGenEvent;

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Resource)]
pub struct MapRadius(pub i32);

#[allow(clippy::identity_op)]
#[allow(clippy::too_many_arguments)]
pub fn generate_level(
    game_entities: Query<Entity, With<GameEntityMarker>>,
    mut commands: Commands,
    mut map: ResMut<WorldData>,
    mut rng: ResMut<Random>,
    mut sprites: Query<(&mut TextureAtlasSprite, &mut Passability)>,
    mut visibility: Query<&mut Visibility>,
    grid: Res<Grid>,
    radius: Res<MapRadius>,
) {
    fn clear_grid(
        grid: &Res<Grid>,
        rng: &mut ResMut<Random>,
        map: &mut ResMut<WorldData>,
        radius: &Res<MapRadius>,
        visibility: &mut Query<&mut Visibility>,
        sprites: &mut Query<(&mut TextureAtlasSprite, &mut Passability)>,
    ) -> HashSet<IVec2> {
        let symbols = [0, 0, 0, 0, 1, 2, 3, 4, 5];
        let mut okay = HashSet::new();

        map.solid.clear();
        grid.entities.iter().for_each(|(pos, e)| {
            if let Ok(mut vis) = visibility.get_mut(*e) {
                *vis = Visibility::Hidden;
            }

            if let Ok((mut sprite, mut passable)) = sprites.get_mut(*e) {
                let dist = pos.distance_squared(IVec2::ZERO);
                let r = radius.0;
                if dist < r || rng.gen(0..(r * 3 / 2)) > dist {
                    sprite.index = symbols[rng.gen(0..symbols.len() as i32) as usize];
                    sprite.color = Color::WHITE;
                    *passable = Passability::Passable;
                    okay.insert(*pos);
                    map.data.set_transparent(
                        (pos.x + grid.size.x / 2 + 1) as usize,
                        (pos.y + grid.size.y / 2 + 1) as usize,
                        true,
                    );
                } else {
                    sprite.index = 4 * 49 + 0;
                    sprite.color = Color::WHITE;
                    *passable = Passability::Blocking;
                    map.data.set_transparent(
                        (pos.x + grid.size.x / 2 + 1) as usize,
                        (pos.y + grid.size.y / 2 + 1) as usize,
                        false,
                    );
                }
            }
        });

        okay
    }

    #[allow(clippy::identity_op)]
    fn make_obstructions(
        commands: &mut Commands,
        count: usize,
        size: IVec2,
        rng: &mut ResMut<Random>,
        grid: &Res<Grid>,
        map: &mut ResMut<WorldData>,
        okay: &HashSet<IVec2>,
    ) {
        let forest_tiles = [
            0,
            1 * 49 + 0,
            1 * 49 + 1,
            1 * 49 + 2,
            1 * 49 + 3,
            2 * 49 + 3,
        ];
        let ruin_tiles = [
            11 * 49 + 1,
            11 * 49 + 2,
            13 * 49 + 0,
            13 * 49 + 0,
            13 * 49 + 0,
            13 * 49 + 0,
            17 * 49 + 10,
            18 * 49 + 10,
            18 * 49 + 11,
        ];

        for _attempt in 0..count {
            let half = size / 2;
            let middle = IVec2::new(rng.gen(-half.x..half.x), rng.gen(-half.y..half.y));

            let (tiles, passability) = if rng.coin() {
                (forest_tiles.as_slice(), Passability::SightBlocking)
            } else {
                (ruin_tiles.as_slice(), Passability::Blocking)
            };

            let IVec2 { x, y } = rng.gen2d(3..6, 4..7);
            for i in -x..=x {
                for j in -y..=y {
                    let pos = middle + IVec2::new(i, j);
                    let dist = middle.distance_squared(pos);

                    let index = rng.from(tiles);

                    if okay.contains(&pos) && rng.percent(3 * dist as u32) {
                        commands.add(ChangeSprite {
                            position: pos,
                            index,
                        });

                        commands.add(ChangePassability {
                            position: pos,
                            passable: passability,
                        });

                        if passability == Passability::Blocking {
                            map.solid.insert(pos);
                        }

                        map.data.set_transparent(
                            (pos.x + grid.size.x / 2 + 1) as usize,
                            (pos.y + grid.size.y / 2 + 1) as usize,
                            passability == Passability::Passable,
                        );
                    }
                }
            }
        }
    }

    #[allow(clippy::identity_op)]
    fn make_houses(
        commands: &mut Commands,
        count: usize,
        size: IVec2,
        rng: &mut ResMut<Random>,
        grid: &Res<Grid>,
        map: &mut ResMut<WorldData>,
        okay: &HashSet<IVec2>,
    ) {
        const WALL_TILES: [usize; 1] = [13 * 49 + 0];
        const FLOOR_TILES: [usize; 14] = [17, 17, 17, 17, 17, 17, 17, 17, 17, 1, 2, 3, 4, 16];

        let mut walls = HashMap::new();
        for _attempt in 0..count {
            let half = size / 2;
            let dx = -half.x..half.x;
            let dy = -half.y..half.y;
            let middle = rng.gen2d(dx, dy);
            let room_size = rng.gen2d(3..7, 3..7);
            for i in -room_size.x..=room_size.x {
                for j in -room_size.y..=room_size.y {
                    if rng.gen(0..100) > 70 {
                        continue;
                    }

                    let ij = IVec2::new(i, j);
                    let pos = middle + ij;

                    if okay.contains(&pos) {
                        let index = rng.from(&WALL_TILES);
                        commands.add(ChangeSprite {
                            position: pos,
                            index,
                        });

                        walls.insert(pos, index);
                        map.data.set_transparent(
                            (pos.x + grid.size.x / 2 + 1) as usize,
                            (pos.y + grid.size.y / 2 + 1) as usize,
                            false,
                        );
                    }
                }
            }

            for i in -room_size.x + 1..room_size.x {
                for j in -room_size.y + 1..room_size.y {
                    let ij = IVec2::new(i, j);
                    let pos = middle + ij;

                    if okay.contains(&pos) {
                        let index = rng.from(&FLOOR_TILES);
                        commands.add(ChangeSprite {
                            position: pos,
                            index,
                        });

                        walls.remove(&pos);
                        map.data.set_transparent(
                            (pos.x + grid.size.x / 2 + 1) as usize,
                            (pos.y + grid.size.y / 2 + 1) as usize,
                            true,
                        );
                    }
                }
            }

            for (pos, wall) in &walls {
                if okay.contains(pos) {
                    commands.add(ChangePassability {
                        position: *pos,
                        passable: if *wall != WALL_TILES[0] {
                            Passability::Passable
                        } else {
                            Passability::Blocking
                        },
                    });
                    map.solid.insert(*pos);
                }
            }
        }
    }

    let size = grid.size;

    map.data.clear_fov();
    map.memory.clear();

    for entity in &game_entities {
        commands.entity(entity).despawn_recursive();
    }

    let okay = clear_grid(
        &grid,
        &mut rng,
        &mut map,
        &radius,
        &mut visibility,
        &mut sprites,
    );

    make_obstructions(&mut commands, 20, size, &mut rng, &grid, &mut map, &okay);
    make_houses(&mut commands, 40, size, &mut rng, &grid, &mut map, &okay);

    // add stuff
    // add people

    let mut player = commands.spawn(GameEntityBundle::new(
        &grid,
        rng.from(
            okay.into_iter()
                .filter(|f| f.distance_squared(IVec2::ZERO) <= 50)
                .collect::<Vec<_>>()
                .as_slice(),
        ),
        26 + 1 * 49,
    ));

    player.insert((PlayerMarker, Sight(12)));
}

pub fn debug_radius(mut map_radius: ResMut<MapRadius>, keys: Res<Input<KeyCode>>) {
    let mut radius = map_radius.0;

    if keys.just_pressed(KeyCode::F6) {
        radius -= 50;
    } else if keys.just_pressed(KeyCode::F7) {
        radius += 50;
    }

    if radius <= 50 {
        radius = 50;
    }

    map_radius.0 = radius;
}

pub fn debug_procgen(mut procgen_events: EventWriter<ProcGenEvent>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::F5) {
        procgen_events.send(ProcGenEvent);
    }
}

pub struct SvarogProcgenPlugin;

impl Plugin for SvarogProcgenPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_event::<ProcGenEvent>()
            .add_event::<RecalculateFOVEvent>()
            .insert_resource(MapRadius(800))
            .insert_resource(ClearColor(Color::BLACK))
            .insert_resource(Msaa::Off)
            .add_systems(Update, generate_level.run_if(on_event::<ProcGenEvent>()))
            .add_systems(Update, on_new_fov_added)
            .add_systems(
                PostUpdate,
                recalculate_fov
                    .run_if(on_event::<RecalculateFOVEvent>())
                    .run_if(in_state(GameStates::Game)),
            )
            .add_systems(Update, (debug_radius, debug_procgen));
    }
}
