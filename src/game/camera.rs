use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{camera::ScalingMode, view::RenderLayers},
};
use bevy_mod_imgui::prelude::*;
use bevy_mouse_tracking_plugin::mouse_pos::{InitMouseTracking, InitWorldTracking};
use bevy_trauma_shake::{Shake, ShakeSettings};

use super::{
    grid::{Grid, WorldEntity},
    procgen::{PlayerMarker, ProcGenEvent},
    GameStates,
};

#[derive(Component)]
pub struct MainCameraMarker;

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
    pub smooth_camera_track: bool,
}

pub fn track_camera(
    mut main_camera: Query<&mut Transform, With<MainCameraMarker>>,
    mut follow_cameras: Query<
        &mut Transform,
        (Without<MainCameraMarker>, With<FollowCameraMarker>),
    >,
    player: Query<&WorldEntity, With<PlayerMarker>>,
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
                    if camera_settings.smooth_camera_track {
                        *mode = CameraMovingMode::Tracking;
                    } else {
                        camera_transform.translation = target;
                    }
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

fn debug_camera(
    mut camera_query: Query<&mut OrthographicProjection>,
    keys: Res<Input<KeyCode>>,
    mut context: NonSendMut<ImguiContext>,
    mut camera_settings: ResMut<CameraSettings>,
) {
    for mut projection in &mut camera_query {
        if let ScalingMode::WindowSize(size) = projection.scaling_mode {
            let mut new_size = size;
            if keys.just_pressed(KeyCode::F1) {
                new_size = 1.0;
            } else if keys.just_pressed(KeyCode::F2) {
                new_size = 2.0;
            } else if keys.just_pressed(KeyCode::F3) {
                new_size = 3.0;
            } else if keys.just_pressed(KeyCode::F4) {
                new_size = 4.0;
            }

            projection.scaling_mode = ScalingMode::WindowSize(new_size);
        }
    }

    let ui = context.ui();
    let window = ui.window("Camera");
    window
        .size([300.0, 300.0], imgui::Condition::FirstUseEver)
        .save_settings(true)
        .build(|| {
            ui.input_float("Tracking speed", &mut camera_settings.tracking_speed)
                .build();
            ui.input_float("Tracking distance", &mut camera_settings.tracking_distance)
                .build();
            ui.input_float(
                "Stop tracking under",
                &mut camera_settings.stop_tracking_under,
            )
            .build();
            ui.checkbox("Smooth tracking", &mut camera_settings.smooth_camera_track);
        });
}

fn setup_cameras(mut commands: Commands, mut procgen_events: EventWriter<ProcGenEvent>) {
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
            },
            projection: OrthographicProjection {
                far: 1000.,
                near: -1000.,
                scale: 1.0,
                scaling_mode: ScalingMode::WindowSize(2.0),
                ..Default::default()
            },
            camera: Camera {
                order: 0,
                ..Default::default()
            },
            ..Default::default()
        },
        Shake::default(),
        ShakeSettings {
            trauma_power: 0.1,
            decay_per_second: 1.0,
            amplitude: 10.,
            frequency: 10.,
            octaves: 1,
        },
        MainCameraMarker,
        RenderLayers::layer(0),
    ));

    commands
        .spawn((
            Camera2dBundle {
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::None,
                },
                projection: OrthographicProjection {
                    far: 1000.,
                    near: -1000.,
                    scale: 1.0,
                    scaling_mode: ScalingMode::WindowSize(2.0),
                    ..Default::default()
                },
                camera: Camera {
                    order: 1,
                    ..Default::default()
                },

                ..Default::default()
            },
            FollowCameraMarker,
            RenderLayers::layer(1),
        ))
        .add(InitMouseTracking)
        .add(InitWorldTracking);

    procgen_events.send(ProcGenEvent);
}

pub struct SvarogCameraPlugin;
impl Plugin for SvarogCameraPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.insert_resource(CameraSettings {
            tracking_speed: 512.0,
            tracking_distance: 120.0,
            stop_tracking_under: 16.0,
            smooth_camera_track: true,
        })
        .add_systems(PostUpdate, track_camera.run_if(in_state(GameStates::Game)))
        .add_systems(OnEnter(GameStates::Game), setup_cameras)
        .add_systems(PostUpdate, debug_camera);
    }
}
