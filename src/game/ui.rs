use bevy::prelude::*;

use bevy_mod_imgui::ImguiContext;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::ListenerInput,
};
use imgui::{DrawListMut, ImColor32};

use super::{
    character::{Character, CharacterStat},
    grid::{Grid, WorldData, WorldEntity},
    health::Health,
    inventory::{CarriedItems, EquippedItems, Item},
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

#[derive(Component)]
pub struct DetailWindowMarker;

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

#[derive(Resource)]
pub struct CharacterSettings {
    pub text_offset: Vec2,
    pub padding: f32,
    pub width: f32,
    pub height: f32,
    pub offset: Vec2,
    pub inside_padding: f32,
    pub stat_left: f32,
    pub stat_top: f32,
}

impl Default for CharacterSettings {
    fn default() -> Self {
        Self {
            text_offset: Vec2::new(-80.0, 3.0),
            padding: 0.0,
            width: 15.0,
            height: 20.0,
            offset: Vec2::new(97.0, 40.0),
            inside_padding: 2.0,
            stat_left: 100.0,
            stat_top: 2.0,
        }
    }
}

fn draw_health_settings(
    mut imgui: NonSendMut<ImguiContext>,
    mut char_settings: ResMut<CharacterSettings>,
) {
    let ui = imgui.ui();
    let window = ui.window("Health Settings");

    window
        .size([300.0, 300.0], imgui::Condition::FirstUseEver)
        .save_settings(true)
        .build(|| {
            let ox = char_settings.text_offset.x;
            let oy = char_settings.text_offset.y;
            let mut o = [ox, oy];
            ui.input_float2("Text Offset", &mut o).build();
            char_settings.text_offset.x = o[0];
            char_settings.text_offset.y = o[1];

            ui.input_float("Padding", &mut char_settings.padding)
                .build();
            ui.input_float("Width", &mut char_settings.width).build();
            ui.input_float("Height", &mut char_settings.height).build();

            let ox = char_settings.offset.x;
            let oy = char_settings.offset.y;
            let mut o = [ox, oy];
            ui.input_float2("Offset", &mut o).build();
            char_settings.offset.x = o[0];
            char_settings.offset.y = o[1];
            ui.input_float("Inside", &mut char_settings.inside_padding)
                .build();

            ui.separator();

            ui.input_float("Stats Left", &mut char_settings.stat_left)
                .build();
            ui.input_float("Stats Top", &mut char_settings.stat_top)
                .build();
        });
}

fn draw_stats(draw: &DrawListMut, p: Vec2, char: &Character) {
    let mut p = Vec2::new(p[0], p[1]);

    fn draw_stat(draw: &DrawListMut, p: Vec2, char: CharacterStat, val: i32) {
        let c = match char {
            CharacterStat::STR => ImColor32::from_rgb(221, 0, 120),
            CharacterStat::ARC => ImColor32::from_rgb(0, 137, 78),
            CharacterStat::INT => ImColor32::from_rgb(0, 132, 172),
            CharacterStat::WIS => ImColor32::from_rgb(144, 60, 255),
            CharacterStat::WIL => ImColor32::from_rgb(147, 122, 0),
            CharacterStat::AGI => ImColor32::from_rgb(194, 82, 0),
        };
        let p1 = [p.x, p.y];
        let p2 = [p.x + 4., p.y + 20.0];

        let o1 = [p.x - 2.0, p.y - 2.0];
        let o2 = [p.x + 6.0, p.y + 22.0];

        let t1 = [p.x + 10.0, p.y - 2.0];
        let t2 = [p.x + 10.0, p.y + 10.0];

        let w = ImColor32::from_rgb(207, 198, 184);
        draw.add_rect(p1, p2, c).filled(true).build();
        draw.add_rect(p1, p2, c).filled(true).build();
        draw.add_rect(o1, o2, w).filled(false).build();
        draw.add_rect(o1, o2, w).filled(false).build();
        draw.add_text(t1, w, format!("{:?}", char));
        draw.add_text(t2, w, format!("{}", val));
    }

    for stat in [
        CharacterStat::STR,
        CharacterStat::ARC,
        CharacterStat::INT,
        CharacterStat::WIS,
        CharacterStat::WIL,
        CharacterStat::AGI,
    ] {
        let val = char[stat];
        draw_stat(draw, p, stat, val);
        p.x += 50.0;
    }
}

fn draw_hp_bar(
    draw: &DrawListMut,
    p: Vec2,
    health: &Health,
    char_settings: &Res<CharacterSettings>,
) {
    let padding = char_settings.padding;
    let width = char_settings.width;
    let height = char_settings.height;
    let offset = char_settings.offset;

    for i in 0..health.size {
        let p1 = [
            p[0] + i as f32 * (width + padding) + offset.x,
            p[1] + offset.y,
        ];

        let p2 = [
            p[0] + i as f32 * (width + padding) + offset.x + width,
            p[1] + offset.y + height,
        ];

        let pi1 = [
            p[0] + i as f32 * (width + padding) + offset.x + char_settings.inside_padding,
            p[1] + offset.y + char_settings.inside_padding,
        ];

        let pi2 = [
            p[0] + i as f32 * (width + padding) + offset.x + width - char_settings.inside_padding,
            p[1] + offset.y + height - char_settings.inside_padding,
        ];

        let lite = ImColor32::from_rgb(227, 18, 45);
        draw.add_text(
            [
                p[0] + offset.x + char_settings.text_offset.x,
                p[1] + char_settings.text_offset.y + offset.y,
            ],
            ImColor32::from_rgb(207, 198, 184),
            "Health:",
        );
        if let Some(_hp) = health.hitpoints.get(i) {
            draw.add_rect(pi1, pi2, lite).filled(true).build();
        } else {
            draw.add_rect(pi1, pi2, ImColor32::from_rgb(0, 0, 0))
                .filled(true)
                .build();
        }

        draw.add_rect(p1, p2, ImColor32::from_rgb(207, 198, 184))
            .filled(false)
            .build();
    }
}

fn show_inventory(
    mut context: NonSendMut<ImguiContext>,
    player_entity: Query<
        (&WorldEntity, &Character, &CarriedItems, &EquippedItems),
        With<PlayerMarker>,
    >,
    items: Query<&Item>,
) {
    let ui = context.ui();
    let Ok((player, player_char, carried_items, equipped_items)) = player_entity.get_single()
    else {
        return;
    };

    ui.window("Inventory")
        .position_pivot([0.0, 0.0])
        .position([10.0, 90.0], imgui::Condition::Always)
        .size([200.0, 500.0], imgui::Condition::Always)
        .resizable(false)
        .collapsible(false)
        .no_decoration()
        .bg_alpha(1.0)
        .build(|| {
            ui.text("Inventory");
            ui.separator();
            for (id, item_id) in carried_items
                .0
                .iter()
                .enumerate()
                .map(|(i, id)| (i + 1, id))
            {
                let Ok(item) = items.get(*item_id) else {
                    continue;
                };

                ui.text(format!("{}: {} ({:?})", id, item.name, item.item_type));
            }
        });
}

fn show_status_for_world_entities(
    player_entity: Query<(&WorldEntity, &Character, &Health), With<PlayerMarker>>,
    world_entities: Query<(&WorldEntity, &Character, &Health), Without<PlayerMarker>>,
    grid: Option<Res<Grid>>,
    world: Res<WorldData>,
    health_settings: Res<CharacterSettings>,
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
        .position_pivot([0.0, 0.0])
        .position([10.0, 10.0], imgui::Condition::Always)
        .size([400.0, 75.0], imgui::Condition::Always)
        .resizable(false)
        .collapsible(false)
        .no_decoration()
        .bg_alpha(1.0)
        .build(|| {
            let draw = ui.get_window_draw_list();
            ui.text(&player.name);
            ui.separator();
            let p: Vec2 = ui.window_pos().into();

            draw_hp_bar(&draw, p, player_health, &health_settings);
            draw_stats(
                &draw,
                p + Vec2::new(health_settings.stat_left, health_settings.stat_top),
                player_char,
            );
        });

    let mut window_y = 10.0f32;
    for (other_entity, other_char, other_health) in &world_entities {
        let (x, y) = grid.norm(other_entity.position);
        if world.data.is_in_fov(x, y) {
            ui.window(&other_entity.name)
                .position_pivot([1.0, 0.0])
                .position([width - 10.0, window_y], imgui::Condition::Always)
                .size([400.0, 75.0], imgui::Condition::Always)
                .resizable(false)
                .collapsible(false)
                .no_decoration()
                .build(|| {
                    let draw = ui.get_window_draw_list();
                    ui.text(&other_entity.name);
                    let p: Vec2 = ui.window_pos().into();

                    draw_hp_bar(&draw, p, other_health, &health_settings);
                    draw_stats(
                        &draw,
                        p + Vec2::new(health_settings.stat_left, health_settings.stat_top),
                        other_char,
                    );
                });

            window_y += 78.0;
        }
    }
}

pub struct SvarogUIPlugin;
impl Plugin for SvarogUIPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_event::<ShowEntityDetails>();
        bevy.init_resource::<CharacterSettings>();
        bevy.add_systems(
            Update,
            (
                show_status_for_world_entities,
                show_inventory,
                draw_health_settings,
            )
                .run_if(in_state(GameStates::Game)),
        );

        bevy.add_systems(
            Update,
            on_show_details.run_if(on_event::<ShowEntityDetails>()),
        );

        #[cfg(feature = "debug_mode")]
        bevy.add_systems(Update, debug_ui_window);
    }
}
