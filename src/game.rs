use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{camera::ScalingMode, view::RenderLayers},
};
use bevy_asset_loader::prelude::*;

use self::{
    camera::{focus_camera, FollowCameraMarker, MovingCameraMarker, SvarogCameraPlugin},
    feel::SvarogFeelPlugin,
    fov::RecalculateFOVEvent,
    grid::{GameEntity, Grid, SvarogGridPlugin, WorldData},
    loading::SvarogLoadingPlugin,
    procgen::{PlayerMarker, ProcGenEvent, SvarogProcgenPlugin},
    window::SvarogWindowPlugins,
};

pub mod camera;
pub mod feel;
pub mod fov;
pub mod grid;
pub mod loading;
pub mod procgen;
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

#[derive(Resource)]
pub struct TurnOrder {
    //pub order: PriorityQueue<Entity>,
}

#[derive(Event)]
pub enum CharacterIntent {
    Move(Entity, IVec2),
}

#[derive(Event)]
pub struct StartGameEvent;

fn start_game(mut commands: Commands, mut procgen_events: EventWriter<ProcGenEvent>) {
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
            },
            projection: OrthographicProjection {
                far: 1000.,
                near: -1000.,
                scale: 1.0,
                scaling_mode: ScalingMode::WindowSize(2.0),
                ..Default::default()
            },
            camera: Camera {
                order: 0,
                ..Default::default()
            },
            ..Default::default()
        },
        MovingCameraMarker,
        RenderLayers::layer(0),
    ));

    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            projection: OrthographicProjection {
                far: 1000.,
                near: -1000.,
                scale: 1.0,
                scaling_mode: ScalingMode::WindowSize(2.0),
                ..Default::default()
            },
            camera: Camera {
                order: 1,
                ..Default::default()
            },

            ..Default::default()
        },
        FollowCameraMarker,
        RenderLayers::layer(1),
    ));

    procgen_events.send(ProcGenEvent);
}

fn debug_camera(mut camera_query: Query<&mut OrthographicProjection>, keys: Res<Input<KeyCode>>) {
    for mut projection in &mut camera_query {
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
}

fn character_controls(
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

pub struct SvarogGamePlugin;

impl Plugin for SvarogGamePlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_plugins(SvarogWindowPlugins)
            .add_plugins(SvarogLoadingPlugin)
            .add_plugins(SvarogGridPlugin)
            .add_plugins(SvarogFeelPlugin)
            .add_plugins(SvarogProcgenPlugin)
            .add_plugins(SvarogCameraPlugin)
            .insert_resource(TurnOrder {})
            .add_systems(
                Update,
                character_controls.run_if(in_state(GameStates::Game)),
            )
            // .add_systems(
            //     Last,
            //     (focus_camera,).chain().run_if(in_state(GameStates::Game)),
            // )
            .add_systems(OnEnter(GameStates::Game), start_game)
            .add_systems(PostUpdate, debug_camera);
    }
}
