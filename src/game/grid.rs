use bevy::{
    app::Plugin,
    asset::Handle,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        schedule::{NextState, OnEnter, OnExit},
        system::{Commands, Res, ResMut, Resource},
    },
    math::{IVec2, Vec3},
    render::view::{RenderLayers, Visibility},
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    transform::components::Transform,
    utils::{HashMap, HashSet},
};
use doryen_fov::MapData;

use crate::game::{GameAssets, GameStates};

#[cfg(feature = "debug_mode")]
use bevy::{
    app::Update,
    ecs::schedule::{common_conditions::in_state, IntoSystemConfigs},
    system::Query,
};

#[cfg(feature = "debug_mode")]
use super::feel::Random;

#[derive(Component)]
pub struct GameEntityMarker;

#[derive(Component)]
pub struct GameEntity {
    pub position: IVec2,
    pub index: usize,
}

#[derive(Component)]
pub struct FOV;

#[derive(Bundle)]
pub struct GameEntityBundle {
    pub entity: GameEntity,
    pub sprite: SpriteSheetBundle,
    pub marker: GameEntityMarker,
    pub layer: RenderLayers,
    pub fov: FOV,
}

impl GameEntityBundle {
    pub fn new(grid: &Res<Grid>, pos: IVec2, index: usize) -> Self {
        GameEntityBundle {
            entity: GameEntity {
                position: pos,
                index,
            },
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(index),
                texture_atlas: grid.atlas.clone_weak(),
                transform: grid.get_tile_position(pos),
                ..Default::default()
            },
            marker: GameEntityMarker,
            layer: RenderLayers::layer(1),
            fov: FOV,
        }
    }
}

#[derive(Resource)]
pub struct Grid {
    pub size: IVec2,
    pub tile: IVec2,
    pub atlas: Handle<TextureAtlas>,
    pub entities: HashMap<IVec2, Entity>,
}

#[derive(Resource)]
pub struct WorldData {
    pub data: MapData,
    pub solid: HashSet<IVec2>,
    pub memory: HashSet<IVec2>,
}

#[derive(Component, Default, Clone, Copy, PartialEq)]
pub enum Passability {
    #[default]
    Passable,
    Blocking,
    SightBlocking,
}

impl Grid {
    pub fn get_tile_position(&self, position: IVec2) -> Transform {
        Transform::from_translation(Vec3::new(
            (self.tile.x * position.x) as f32,
            (self.tile.y * position.y) as f32,
            0.0,
        ))
    }

    pub fn spawn(&self, commands: &mut Commands, index: usize, position: IVec2) -> Entity {
        commands
            .spawn((
                SpriteSheetBundle {
                    transform: self.get_tile_position(position),
                    sprite: TextureAtlasSprite::new(index),
                    texture_atlas: self.atlas.clone_weak(),
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                Passability::Passable,
            ))
            .id()
    }

    pub fn get(&self, position: IVec2) -> Option<&Entity> {
        self.entities.get(&position)
    }
}

fn create_grid_resource(mut commands: Commands, assets: Res<GameAssets>) {
    commands.insert_resource(Grid {
        size: IVec2::new(120, 62),
        tile: IVec2::new(16, 16),
        atlas: assets.atlas.clone_weak(),
        entities: Default::default(),
    });

    commands.insert_resource(WorldData {
        data: MapData::new(122, 64),
        solid: Default::default(),
        memory: Default::default(),
    });
}

fn initialize_grid(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    mut next_state: ResMut<NextState<GameStates>>,
) {
    let size = grid.size;
    for i in (0..=size.y as usize).rev() {
        for j in 0..=size.x as usize {
            let position = IVec2::new(j as i32 - grid.size.x / 2, i as i32 - grid.size.y / 2);
            let spawned = grid.spawn(&mut commands, 0, position);
            grid.entities.insert(position, spawned);
        }
    }

    next_state.set(GameStates::Game);
}

pub struct SvarogGridPlugin;

impl Plugin for SvarogGridPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.add_systems(OnExit(GameStates::AssetLoading), create_grid_resource)
            .add_systems(OnEnter(GameStates::Setup), initialize_grid);
    }
}
