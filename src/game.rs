use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use self::{grid::SvarogGridPlugin, loading::SvarogLoadingPlugin, window::SvarogWindowPlugins};

pub mod grid;
pub mod loading;
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

fn start_game(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub struct SvarogGamePlugin;

impl Plugin for SvarogGamePlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_plugins(SvarogWindowPlugins)
            .add_plugins(SvarogLoadingPlugin)
            .add_plugins(SvarogGridPlugin)
            .add_systems(OnEnter(GameStates::Game), start_game);
    }
}
