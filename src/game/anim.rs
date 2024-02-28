use bevy::app::Plugin;
use bevy_tweening::TweeningPlugin;

pub struct SvarogAnimationPlugin;

impl Plugin for SvarogAnimationPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.add_plugins(TweeningPlugin);
    }
}
