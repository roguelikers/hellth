use bevy::{
    app::{Plugin, Update},
    asset::Handle,
    ecs::{
        entity::Entity,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter, OnExit},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    math::{IVec2, Vec3},
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    transform::components::Transform,
    utils::HashMap,
};
use bevy_rand::{prelude::WyRand, resource::GlobalEntropy};
use rand_core::RngCore;

use crate::game::{GameAssets, GameStates};

#[derive(Resource)]
pub struct Grid {
    pub size: IVec2,
    pub tile: IVec2,
    pub atlas: Handle<TextureAtlas>,
    pub entities: HashMap<IVec2, Entity>,
}

impl Grid {
    pub fn spawn(&self, commands: &mut Commands, index: usize, position: IVec2) -> Entity {
        commands
            .spawn(SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(
                    (self.tile.x * position.x) as f32,
                    (self.tile.y * position.y) as f32,
                    0.0,
                )),
                sprite: TextureAtlasSprite::new(index),
                texture_atlas: self.atlas.clone_weak(),
                ..Default::default()
            })
            .id()
    }

    pub fn get(&self, position: IVec2) -> Option<&Entity> {
        self.entities.get(&position)
    }
}

fn make_grid(mut commands: Commands, assets: Res<GameAssets>) {
    commands.insert_resource(Grid {
        size: IVec2::new(140, 70),
        tile: IVec2::new(16, 16),
        atlas: assets.atlas.clone_weak(),
        entities: Default::default(),
    });
}

fn initialize_grid(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    let size = grid.size;
    for i in (0..=size.y as usize).rev() {
        for j in 0..=size.x as usize {
            let position = IVec2::new(j as i32 - grid.size.x / 2, i as i32 - grid.size.y / 2);
            let spawned = grid.spawn(
                &mut commands,
                (rng.next_u32() % (49 * 22)) as usize,
                position,
            );
            grid.entities.insert(position, spawned);
        }
    }

    next_state.set(GameStates::Game);
}

fn update_grid_randomly(
    grid: ResMut<Grid>,
    mut sprites: Query<&mut TextureAtlasSprite>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    let size = grid.size / 2;
    for i in -size.x..=size.x {
        for j in -size.y..=size.y {
            let Some(e) = grid.get(IVec2::new(i, j)) else {
                continue;
            };

            let Ok(mut sprite) = sprites.get_mut(*e) else {
                continue;
            };

            sprite.index = (rng.next_u32() % (49 * 22)) as usize;
        }
    }
}

pub struct SvarogGridPlugin;

impl Plugin for SvarogGridPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.add_systems(OnExit(GameStates::AssetLoading), make_grid)
            .add_systems(OnEnter(GameStates::Setup), initialize_grid)
            .add_systems(
                Update,
                update_grid_randomly.run_if(in_state(GameStates::Game)),
            );
    }
}
