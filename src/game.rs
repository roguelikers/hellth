use bevy::{prelude::*, transform::commands};
use bevy_asset_loader::prelude::*;

use crate::game::sprite::ChangeSprite;

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

fn procgen(
    mut commands: Commands,
    grid: Res<Grid>,
    mut sprites: Query<&mut TextureAtlasSprite>,
    mut rng: ResMut<Random>,
) {
    fn clear_grid(grid: &Res<Grid>, sprites: &mut Query<&mut TextureAtlasSprite>) {
        grid.entities.iter().for_each(|(_, e)| {
            sprites.get_mut(*e).unwrap().index = 0;
        });
    }

    fn make_ground_layer(commands: &mut Commands, size: IVec2, rng: &mut ResMut<Random>) {
        for i in -size.x..=size.x {
            for j in -size.y..=size.y {
                let symbols = [0, 0, 0, 0, 1, 2, 3, 4, 5];

                if rng.gen(0..100) < 20 {
                    commands.add(ChangeSprite {
                        position: IVec2::new(i, j),
                        index: symbols[rng.gen(0..symbols.len() as i32) as usize],
                    });
                }
            }
        }
    }

    fn make_obstructions(
        commands: &mut Commands,
        count: usize,
        size: IVec2,
        rng: &mut ResMut<Random>,
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

            let tiles = if rng.coin() {
                forest_tiles.as_slice()
            } else {
                ruin_tiles.as_slice()
            };

            let IVec2 { x, y } = rng.gen2d(3..6, 4..7);
            for i in -x..=x {
                for j in -y..=y {
                    let pos = middle + IVec2::new(i, j);
                    let dist = middle.distance_squared(pos);
                    if rng.percent(3 * dist as u32) {
                        commands.add(ChangeSprite {
                            position: pos,
                            index: rng.from(tiles),
                        });
                    }
                }
            }
        }
    }

    fn make_houses(commands: &mut Commands, count: usize, size: IVec2, rng: &mut ResMut<Random>) {
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
                    let dist = 110 - middle.distance_squared(pos) * 2;

                    commands.add(ChangeSprite {
                        position: pos,
                        index: rng.from(&[13 * 49 + 0]),
                    });
                }
            }

            for i in -room_size.x + 1..room_size.x {
                for j in -room_size.y + 1..room_size.y {
                    let ij = IVec2::new(i, j);
                    let pos = middle + ij;

                    commands.add(ChangeSprite {
                        position: pos,
                        index: rng.from(&[17, 17, 17, 17, 17, 17, 17, 17, 17, 0, 1, 2, 3, 4, 16]),
                    });
                }
            }
        }
    }

    let size = grid.size;
    clear_grid(&grid, &mut sprites);
    make_ground_layer(&mut commands, size, &mut rng);
    make_obstructions(&mut commands, 20, size, &mut rng);
    make_houses(&mut commands, 30, size, &mut rng);
}

fn start_game(mut commands: Commands, mut procgen_events: EventWriter<ProcGenEvent>) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            far: 1000.,
            near: -1000.,
            scale: 1.0, //0.5,
            ..Default::default()
        },
        ..Default::default()
    });

    procgen_events.send(ProcGenEvent);
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
            .add_systems(OnEnter(GameStates::Game), start_game)
            .add_systems(Update, procgen.run_if(on_event::<ProcGenEvent>()))
            .add_systems(Update, debug_procgen);
    }
}
