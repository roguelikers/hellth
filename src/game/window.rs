use bevy::{
    app::{Plugin, Startup},
    render::texture::ImagePlugin,
    window::{Window, WindowPlugin},
    winit::WinitWindows,
    DefaultPlugins,
};

fn set_window_icon(windows: NonSend<WinitWindows>) {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("icon.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}

use bevy::prelude::*;

use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_mouse_tracking_plugin::mouse_pos::MousePosPlugin;
use winit::window::Icon;

#[derive(Component)]
pub struct FpsRoot;

#[derive(Component)]
pub struct FpsText;

pub fn setup_fps_counter(mut commands: Commands) {
    let root = commands
        .spawn((
            FpsRoot,
            NodeBundle {
                background_color: BackgroundColor(Color::BLACK.with_a(0.5)),
                z_index: ZIndex::Global(i32::MAX),
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(1.),
                    top: Val::Percent(1.),
                    bottom: Val::Auto,
                    left: Val::Auto,
                    padding: UiRect::all(Val::Px(4.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .id();

    let text_fps = commands
        .spawn((
            FpsText,
            TextBundle {
                text: Text::from_sections([
                    TextSection {
                        value: "FPS: ".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    },
                    TextSection {
                        value: " N/A".into(),
                        style: TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    },
                ]),
                ..Default::default()
            },
        ))
        .id();
    commands.entity(root).push_children(&[text_fps]);
}

pub fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(value) = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.sections[1].value = format!("{value:>4.0}");

            text.sections[1].style.color = if value >= 60.0 {
                Color::rgb(0.0, 1.0, 0.0)
            } else if value >= 45.0 {
                Color::rgb((1.0 - (value - 60.0) / (120.0 - 60.0)) as f32, 1.0, 0.0)
            } else if value >= 30.0 {
                Color::rgb(1.0, ((value - 30.0) / (60.0 - 30.0)) as f32, 0.0)
            } else {
                Color::rgb(1.0, 0.0, 0.0)
            }
        } else {
            text.sections[1].value = " N/A".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}

pub fn fps_counter_showhide(
    mut q: Query<&mut Visibility, With<FpsRoot>>,
    kbd: Res<Input<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        let mut vis = q.single_mut();
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}

pub struct SvarogWindowPlugins;

impl Plugin for SvarogWindowPlugins {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Svarog".into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        );
        #[cfg(feature = "debug_mode")]
        bevy.edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        });
        bevy.add_plugins(MousePosPlugin)
            .add_plugins(DefaultPickingPlugins)
            .add_systems(Startup, set_window_icon)
            .add_plugins(bevy_mod_imgui::ImguiPlugin {
                ini_filename: Some("imgui.ini".into()),
                ..Default::default()
            });

        #[cfg(feature = "debug_mode")]
        bevy.add_plugins(FrameTimeDiagnosticsPlugin);
        #[cfg(feature = "debug_mode")]
        bevy.add_systems(Startup, setup_fps_counter);
        #[cfg(feature = "debug_mode")]
        bevy.add_systems(Update, (fps_text_update_system, fps_counter_showhide));
    }
}
