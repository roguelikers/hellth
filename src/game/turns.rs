use bevy::prelude::*;

#[derive(Resource)]
pub struct TurnOrder {
    //pub order: PriorityQueue<Entity>,
}

pub struct SvarogTurnPlugin;

impl Plugin for SvarogTurnPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.insert_resource(TurnOrder {});
    }
}
