use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;

use self::{
    feel::SvarogFeelPlugin,
    grid::SvarogGridPlugin,
    loading::SvarogLoadingPlugin,
    procgen::{ProcGenEvent, SvarogProcgenPlugin},
    window::SvarogWindowPlugins,
};

pub mod feel;
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

#[derive(Event)]
pub struct StartGameEvent;

fn start_game(mut commands: Commands, mut procgen_events: EventWriter<ProcGenEvent>) {
    commands.spawn((Camera2dBundle {
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

pub struct SvarogGamePlugin;

impl Plugin for SvarogGamePlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_plugins(SvarogWindowPlugins)
            .add_plugins(SvarogLoadingPlugin)
            .add_plugins(SvarogGridPlugin)
            .add_plugins(SvarogFeelPlugin)
            .add_plugins(SvarogProcgenPlugin)
            .add_systems(OnEnter(GameStates::Game), start_game)
            .add_systems(PostUpdate, debug_camera);
    }
}
