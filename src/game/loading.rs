use bevy::app::Plugin;
use bevy_asset_loader::{
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};

use super::{GameAssets, GameStates};

pub struct SvarogLoadingPlugin;

impl Plugin for SvarogLoadingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state::<GameStates>().add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .load_collection::<GameAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("resources.assets.ron")
                .continue_to_state(GameStates::Setup),
        );
    }
}
