use bevy::{app::Plugin, ecs::system::Resource};

#[derive(Resource, Default)]
pub struct History(pub Vec<String>);

pub type HistoryLog = History;

impl History {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn add(&mut self, s: &str) {
        self.0.push(s.to_string());
    }
}

pub struct SvarogHistoryPlugin;
impl Plugin for SvarogHistoryPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<History>();
    }
}
