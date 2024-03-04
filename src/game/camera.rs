use bevy::prelude::*;

use super::{
    grid::{GameEntity, Grid},
    procgen::PlayerMarker,
};

#[derive(Component)]
pub struct MovingCameraMarker;

#[derive(Component)]
pub struct FollowCameraMarker;

#[derive(Default)]
pub enum CameraMovingMode {
    #[default]
    Calm,
    Tracking,
}

#[derive(Resource)]
pub struct CameraSettings {
    pub tracking_speed: f32,
    pub tracking_distance: f32,
    pub stop_tracking_under: f32,
}

pub fn focus_camera(
    mut main_camera: Query<&mut Transform, With<MovingCameraMarker>>,
    mut follow_cameras: Query<
        &mut Transform,
        (Without<MovingCameraMarker>, With<FollowCameraMarker>),
    >,
    player: Query<&GameEntity, With<PlayerMarker>>,
    camera_settings: Res<CameraSettings>,
    grid: Option<Res<Grid>>,
    time: Res<Time>,
    mut mode: Local<CameraMovingMode>,
) {
    let Some(grid) = grid else {
        return;
    };

    let Ok(player_game_entity) = player.get_single() else {
        return;
    };

    for mut camera_transform in &mut main_camera {
        let target = grid
            .get_tile_position(player_game_entity.position)
            .translation;

        match *mode {
            CameraMovingMode::Calm => {
                let dist = camera_transform.translation.distance(target);
                if dist > camera_settings.tracking_distance {
                    *mode = CameraMovingMode::Tracking;
                }
            }
            CameraMovingMode::Tracking => {
                let direction = (target - camera_transform.translation).normalize_or_zero();
                camera_transform.translation +=
                    direction * camera_settings.tracking_speed * time.delta_seconds();

                let dist = camera_transform.translation.distance(target);
                if dist < camera_settings.stop_tracking_under {
                    *mode = CameraMovingMode::Calm;
                }
            }
        }

        for mut follow_camera in &mut follow_cameras {
            follow_camera.translation = camera_transform.translation;
        }
    }
}

pub struct SvarogCameraPlugin;
impl Plugin for SvarogCameraPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.insert_resource(CameraSettings {
            tracking_speed: 150.0,
            tracking_distance: 100.0,
            stop_tracking_under: 16.0,
        });
    }
}
