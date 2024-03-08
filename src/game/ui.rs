use bevy::ecs::system::{Command, SystemState};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_mod_imgui::ImguiContext;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::ListenerInput,
};

use super::{
    character::Character,
    grid::{Grid, WorldData, WorldEntity},
    health::Health,
    procgen::PlayerMarker,
    GameStates,
};

#[derive(Event, Debug)]
pub struct ShowEntityDetails(Entity, f32);

impl From<ListenerInput<Pointer<Click>>> for ShowEntityDetails {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        ShowEntityDetails(event.target, event.hit.depth)
    }
}

pub fn on_show_details(
    mut show_details: EventReader<ShowEntityDetails>,
    world_entities: Query<&WorldEntity>,
) {
    for detail in show_details.read() {
        if let Ok(world_entity) = world_entities.get(detail.0) {
            println!(
                "Show Detail for {:?} at {:?}: {}",
                detail, world_entity.position, world_entity.name
            );
        }
    }
}

fn show_status_for_world_entities(
    player_entity: Query<(&WorldEntity, &Character, &Health), With<PlayerMarker>>,
    world_entities: Query<(&WorldEntity, &Character, &Health), Without<PlayerMarker>>,
    grid: Option<Res<Grid>>,
    world: Res<WorldData>,
    mut context: NonSendMut<ImguiContext>,
) {
    let Some(grid) = grid else {
        return;
    };

    let ui = context.ui();

    let [width, _height] = ui.io().display_size;

    let Ok((player, player_char, player_health)) = player_entity.get_single() else {
        return;
    };

    ui.window(&player.name)
        .position_pivot([1.0, 0.0])
        .position([width - 10.0, 10.0], imgui::Condition::Always)
        .size([400.0, 75.0], imgui::Condition::Always)
        .resizable(false)
        .collapsible(false)
        .focused(false)
        .build(|| {
            ui.text(format!("{:?}", player_char));
            ui.text(format!("{:?}", player_health));
        });

    let mut window_y = 85.0f32;
    for (other_entity, other_char, other_health) in &world_entities {
        let (x, y) = grid.norm(other_entity.position);
        if world.data.is_in_fov(x, y) {
            ui.window(&other_entity.name)
                .position_pivot([1.0, 0.0])
                .position([width - 10.0, window_y], imgui::Condition::Always)
                .size([400.0, 75.0], imgui::Condition::Always)
                .resizable(false)
                .collapsible(false)
                .focused(false)
                .build(|| {
                    ui.text(format!("{:?}", other_char));
                    ui.text(format!("{:?}", other_health));
                });

            window_y += 75.0;
        }
    }
}

#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub enum HorizontalAlign {
    #[default]
    Left,
    Right,
}

#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub enum VerticalAlign {
    #[default]
    Up,
    Down,
}

fn debug_ui_window(
    mut commands: Commands,
    mut context: NonSendMut<ImguiContext>,
    mut xy: Local<(f32, f32, f32, f32, HorizontalAlign, VerticalAlign)>,
) {
    let ui = context.ui();

    let window = ui.window("Debug UI");

    let mut is_left = xy.4 == HorizontalAlign::Left;
    let mut is_up = xy.5 == VerticalAlign::Up;

    window
        .size([300.0, 300.0], imgui::Condition::FirstUseEver)
        .save_settings(true)
        .build(|| {
            ui.input_float("X", &mut xy.0).build();
            ui.input_float("Y", &mut xy.1).build();
            ui.input_float("W", &mut xy.2).build();
            ui.input_float("H", &mut xy.3).build();

            let _ = ui.checkbox("Horizontal Left", &mut is_left);
            let _ = ui.checkbox("Vertical Up", &mut is_up);

            xy.4 = if is_left {
                HorizontalAlign::Left
            } else {
                HorizontalAlign::Right
            };

            xy.5 = if is_up {
                VerticalAlign::Up
            } else {
                VerticalAlign::Down
            };

            if ui.button("CREATE!") {
                commands.add(NewUIWindow {
                    x: xy.0,
                    y: xy.1,
                    w: xy.2,
                    h: xy.3,
                    hor: xy.4,
                    ver: xy.5,
                });
            }
        });
}

pub struct NewUIWindow {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub hor: HorizontalAlign,
    pub ver: VerticalAlign,
}

impl NewUIWindow {
    pub fn top_left(offset: Vec2, size: Vec2) -> Self {
        NewUIWindow {
            x: offset.x,
            y: offset.y,
            w: size.x,
            h: size.y,
            hor: HorizontalAlign::Left,
            ver: VerticalAlign::Up,
        }
    }

    pub fn top_right(offset: Vec2, size: Vec2) -> Self {
        NewUIWindow {
            x: offset.x,
            y: offset.y,
            w: size.x,
            h: size.y,
            hor: HorizontalAlign::Right,
            ver: VerticalAlign::Up,
        }
    }

    pub fn bottom_left(offset: Vec2, size: Vec2) -> Self {
        NewUIWindow {
            x: offset.x,
            y: offset.y,
            w: size.x,
            h: size.y,
            hor: HorizontalAlign::Left,
            ver: VerticalAlign::Down,
        }
    }

    pub fn bottom_right(offset: Vec2, size: Vec2) -> Self {
        NewUIWindow {
            x: offset.x,
            y: offset.y,
            w: size.x,
            h: size.y,
            hor: HorizontalAlign::Right,
            ver: VerticalAlign::Down,
        }
    }
}

impl Command for NewUIWindow {
    fn apply(self, world: &mut World) {
        let (x, y, texture) = {
            let mut world_state =
                SystemState::<(Res<AssetServer>, Query<&Window, With<PrimaryWindow>>)>::new(world);
            let (asset_server, windows) = world_state.get_mut(world);

            let window = windows.single();
            let size = (window.width(), window.height());

            let half_size = Vec2::new((size.0 - self.w) / 2.0, -(size.1 - self.h) / 2.0);
            let left_top = -half_size;

            let hor_align_offset = if self.hor == HorizontalAlign::Left {
                0.0f32
            } else {
                2.0 * half_size.x
            };

            let ver_align_offset = if self.ver == VerticalAlign::Up {
                0.0f32
            } else {
                2.0 * half_size.y
            };
            let x = hor_align_offset + self.x + left_top.x;
            let y = ver_align_offset + -self.y + left_top.y;

            (x, y, asset_server.load("black.png"))
        };

        world.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_translation(Vec3::new(x, y, 0.0))
                    .with_scale(Vec3::new(self.w, self.h, 1.0)),
                ..Default::default()
            },
            RenderLayers::layer(2),
        ));
    }
}

pub struct SvarogUIPlugin;
impl Plugin for SvarogUIPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_event::<ShowEntityDetails>();

        bevy.add_systems(
            Update,
            show_status_for_world_entities.run_if(in_state(GameStates::Game)),
        );

        bevy.add_systems(
            Update,
            on_show_details.run_if(on_event::<ShowEntityDetails>()),
        );

        bevy.add_systems(Update, debug_ui_window);
    }
}
