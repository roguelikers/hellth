use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_trauma_shake::TraumaPlugin;

use self::{
    actions::SvarogActionsPlugin, ai::SvarogAIPlugin, camera::SvarogCameraPlugin,
    feel::SvarogFeelPlugin, grid::SvarogGridPlugin, history::SvarogHistoryPlugin,
    inventory::SvarogInventoryPlugin, loading::SvarogLoadingPlugin, magic::SvarogMagicPlugin,
    player::SvarogPlayerPlugin, procgen::SvarogProcgenPlugin, turns::SvarogTurnPlugin,
    ui::SvarogUIPlugin, window::SvarogWindowPlugins,
};

pub mod actions;
pub mod ai;
pub mod camera;
pub mod character;
pub mod feel;
pub mod fov;
pub mod grid;
pub mod health;
pub mod history;
pub mod inventory;
pub mod loading;
pub mod magic;
pub mod mobs;
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

#[derive(Resource, Default)]
pub struct DebugFlag(pub bool);

impl Plugin for SvarogGamePlugin {
    fn build(&self, bevy: &mut App) {
        bevy.insert_resource::<DebugFlag>(DebugFlag(false))
            .add_plugins(SvarogWindowPlugins)
            .add_plugins(SvarogMagicPlugin)
            .add_plugins(SvarogHistoryPlugin)
            .add_plugins(SvarogLoadingPlugin)
            .add_plugins(SvarogActionsPlugin)
            .add_plugins(SvarogGridPlugin)
            .add_plugins(SvarogFeelPlugin)
            .add_plugins(SvarogProcgenPlugin)
            .add_plugins(SvarogCameraPlugin)
            .add_plugins(SvarogTurnPlugin)
            .add_plugins(SvarogPlayerPlugin)
            .add_plugins(SvarogAIPlugin)
            .add_plugins(SvarogInventoryPlugin)
            .add_plugins(SvarogUIPlugin)
            .add_plugins(TraumaPlugin);
    }
}
