use bevy::prelude::*;
use bevy_mod_imgui::ImguiContext;

use super::{
    grid::{Grid, WorldData, WorldEntity},
    health::Health,
    procgen::PlayerMarker,
    GameStates,
};

pub struct SvarogUIPlugin;

fn show_status_for_world_entities(
    player_entity: Query<(&WorldEntity, &Health), With<PlayerMarker>>,
    world_entities: Query<(&WorldEntity, &Health), Without<PlayerMarker>>,
    grid: Option<Res<Grid>>,
    world: Res<WorldData>,
    mut context: NonSendMut<ImguiContext>,
) {
    let Some(grid) = grid else {
        return;
    };

    let ui = context.ui();

    let [width, _height] = ui.io().display_size;

    let Ok((player, player_health)) = player_entity.get_single() else {
        return;
    };

    ui.window(&player.name)
        .position_pivot([1.0, 0.0])
        .position([width - 10.0, 10.0], imgui::Condition::Always)
        .size([300.0, 50.0], imgui::Condition::Always)
        .build(|| {
            ui.text(format!("{:?}", player_health));
        });

    let mut window_y = 65.0f32;
    for (other_entity, other_health) in &world_entities {
        let (x, y) = grid.norm(other_entity.position);
        if world.data.is_in_fov(x, y) {
            ui.window(&other_entity.name)
                .position_pivot([1.0, 0.0])
                .position([width - 10.0, window_y], imgui::Condition::Always)
                .size([300.0, 50.0], imgui::Condition::Always)
                .build(|| {
                    ui.text(format!("{:?}", other_health));
                });

            window_y += 55.0;
        }
    }
}

impl Plugin for SvarogUIPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_systems(
            Update,
            show_status_for_world_entities.run_if(in_state(GameStates::Game)),
        );
    }
}
