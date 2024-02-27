use bevy::prelude::*;
use game::SvarogGamePlugin;

pub mod game;

fn main() {
    App::new().add_plugins(SvarogGamePlugin).run();
}
