use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{camera::ScalingMode, view::RenderLayers},
};
use bevy_asset_loader::prelude::*;
use bevy_mouse_tracking_plugin::{
    mouse_pos::{InitMouseTracking, InitWorldTracking},
    MainCamera,
};
use bevy_trauma_shake::{Shake, ShakeSettings, TraumaPlugin};

use self::{
    actions::SvarogActionsPlugin,
    ai::SvarogAIPlugin,
    camera::{FollowCameraMarker, MainCameraMarker, SvarogCameraPlugin},
    feel::SvarogFeelPlugin,
    grid::SvarogGridPlugin,
    loading::SvarogLoadingPlugin,
    player::SvarogPlayerPlugin,
    procgen::{ProcGenEvent, SvarogProcgenPlugin},
    turns::SvarogTurnPlugin,
    ui::SvarogUIPlugin,
    window::SvarogWindowPlugins,
};

pub mod actions;
pub mod ai;
pub mod camera;
pub mod character;
pub mod feel;
pub mod fov;
pub mod grid;
pub mod health;
pub mod inventory;
pub mod loading;
pub mod player;
pub mod procgen;
pub mod spells;
pub mod sprite;
pub mod sprites;
pub mod turns;
pub mod ui;
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
pub enum CharacterIntent {
    Move(Entity, IVec2),
}

#[derive(Event)]
pub struct StartGameEvent;

pub struct SvarogGamePlugin;

impl Plugin for SvarogGamePlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_plugins(SvarogWindowPlugins)
            .add_plugins(SvarogLoadingPlugin)
            .add_plugins(SvarogActionsPlugin)
            .add_plugins(SvarogGridPlugin)
            .add_plugins(SvarogFeelPlugin)
            .add_plugins(SvarogProcgenPlugin)
            .add_plugins(SvarogCameraPlugin)
            .add_plugins(SvarogTurnPlugin)
            .add_plugins(SvarogPlayerPlugin)
            .add_plugins(SvarogAIPlugin)
            .add_plugins(SvarogUIPlugin)
            .add_plugins(TraumaPlugin);
    }
}
