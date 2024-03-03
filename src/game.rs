use std::collections::HashSet;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*, render::camera::ScalingMode,
    utils::HashMap,
};
use bevy_asset_loader::prelude::*;

use crate::game::{
    grid::Passability,
    sprite::{ChangePassability, ChangeSprite},
};

use self::{
    feel::{Random, SvarogFeelPlugin},
    grid::{Grid, SvarogGridPlugin},
    loading::SvarogLoadingPlugin,
    window::SvarogWindowPlugins,
};

pub mod feel;
pub mod grid;
pub mod loading;
pub mod sprite;
pub mod window;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    AssetLoading,
    Setup,
    Game,
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "atlas")]
    pub atlas: Handle<TextureAtlas>,
}

#[derive(Event)]
pub struct StartGameEvent;

#[derive(Event)]
pub struct ProcGenEvent;

#[derive(Resource)]
pub struct MapRadius(pub i32);

fn procgen(
    mut commands: Commands,
    grid: Res<Grid>,
    mut sprites: Query<(&mut TextureAtlasSprite, &mut Passability)>,
    mut rng: ResMut<Random>,
    radius: Res<MapRadius>,
) {
    fn clear_grid(
        grid: &Res<Grid>,
        rng: &mut ResMut<Random>,
        radius: &Res<MapRadius>,
        sprites: &mut Query<(&mut TextureAtlasSprite, &mut Passability)>,
    ) -> HashSet<IVec2> {
        let symbols = [0, 0, 0, 0, 1, 2, 3, 4, 5];
        let mut okay = HashSet::new();

        grid.entities.iter().for_each(|(pos, e)| {
            if let Ok((mut sprite, mut passable)) = sprites.get_mut(*e) {
                let dist = pos.distance_squared(IVec2::ZERO);
                let r = radius.0;
                if dist < r || rng.gen(0..(r * 3 / 2)) > dist {
                    sprite.index = symbols[rng.gen(0..symbols.len() as i32) as usize];
                    sprite.color = Color::WHITE;
                    *passable = Passability::Passable;
                    okay.insert(*pos);
                } else {
                    sprite.index = 4 * 49 + 0;
                    sprite.color = Color::WHITE;
                    *passable = Passability::Blocking;
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
        okay: &HashSet<IVec2>,
    ) {
        let forest_tiles = [
            0,
            1 * 49 + 0,
            1 * 49 + 1,
            1 * 49 + 2,
            1 * 49 + 3,
            2 * 49 + 0,
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
                (forest_tiles.as_slice(), Passability::SeethruBlocking)
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

                        walls.insert(pos, index);
                    }
                }
            }

            for (pos, wall) in &walls {
                if okay.contains(&pos) {
                    commands.add(ChangePassability {
                        position: *pos,
                        passable: if *wall != WALL_TILES[0] {
                            Passability::Passable
                        } else {
                            Passability::Blocking
                        },
                    });
                }
            }
        }
    }

    let size = grid.size;
    let okay = clear_grid(&grid, &mut rng, &radius, &mut sprites);
    make_obstructions(&mut commands, 20, size, &mut rng, &okay);
    make_houses(&mut commands, 50, size, &mut rng, &okay);
}

fn start_game(mut commands: Commands, mut procgen_events: EventWriter<ProcGenEvent>) {
    commands.spawn((Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::rgb(71. / 255., 45. / 255., 60. / 255.)),
        },
        projection: OrthographicProjection {
            far: 1000.,
            near: -1000.,
            scale: 1.0,
            scaling_mode: ScalingMode::WindowSize(2.0),
            ..Default::default()
        },
        ..Default::default()
    },));

    procgen_events.send(ProcGenEvent);
}

fn debug_camera(mut camera_query: Query<&mut OrthographicProjection>, keys: Res<Input<KeyCode>>) {
    let Ok(mut projection) = camera_query.get_single_mut() else {
        return;
    };

    if let ScalingMode::WindowSize(size) = projection.scaling_mode {
        let mut new_size = size;
        if keys.just_pressed(KeyCode::F1) {
            new_size = 1.0;
        } else if keys.just_pressed(KeyCode::F2) {
            new_size = 2.0;
        } else if keys.just_pressed(KeyCode::F3) {
            new_size = 3.0;
        } else if keys.just_pressed(KeyCode::F4) {
            new_size = 4.0;
        }

        projection.scaling_mode = ScalingMode::WindowSize(new_size);
    }
}

fn debug_radius(mut map_radius: ResMut<MapRadius>, keys: Res<Input<KeyCode>>) {
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

fn debug_procgen(mut procgen_events: EventWriter<ProcGenEvent>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::F5) {
        procgen_events.send(ProcGenEvent);
    }
}

pub struct SvarogGamePlugin;

impl Plugin for SvarogGamePlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_plugins(SvarogWindowPlugins)
            .add_plugins(SvarogLoadingPlugin)
            .add_plugins(SvarogGridPlugin)
            .add_plugins(SvarogFeelPlugin)
            .add_event::<ProcGenEvent>()
            .insert_resource(MapRadius(800))
            .insert_resource(ClearColor(Color::rgb(71. / 255., 45. / 255., 60. / 255.)))
            .insert_resource(Msaa::Off)
            .add_systems(OnEnter(GameStates::Game), start_game)
            .add_systems(Update, procgen.run_if(on_event::<ProcGenEvent>()))
            .add_systems(Update, debug_procgen)
            .add_systems(PostUpdate, (debug_camera, debug_radius));
    }
}
