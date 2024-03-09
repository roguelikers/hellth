use bevy::{app::Plugin, ecs::system::Resource};

#[derive(Resource, Default)]
pub struct History(pub Vec<String>);

pub struct SvarogHistoryPlugin;
impl Plugin for SvarogHistoryPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<History>();
    }
}
