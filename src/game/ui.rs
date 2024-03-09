use bevy::prelude::*;

use bevy_mod_imgui::ImguiContext;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::ListenerInput,
};
use imgui::{DrawListMut, ImColor32, StyleColor};

use super::{
    character::{Character, CharacterStat},
    grid::{Grid, WorldData, WorldEntity, WorldEntityColor},
    health::Health,
    history::{History, HistoryLog},
    inventory::{CarriedItems, CurrentlySelectedItem, EquippedItems, Item, ItemActions},
    magic::Magic,
    procgen::PlayerMarker,
    DebugFlag, GameStates,
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
    mut log: ResMut<HistoryLog>,
    world_entities: Query<&WorldEntity>,
) {
    for detail in show_details.read() {
        if let Ok(world_entity) = world_entities.get(detail.0) {
            log.add(&format!(
                "Show Detail for {:?} at {:?}: {}",
                detail, world_entity.position, world_entity.name
            ));
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
    debug: Res<DebugFlag>,
) {
    if !debug.0 {
        return;
    }

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

fn draw_stats(draw: &DrawListMut, magic: &Res<Magic>, p: Vec2, char: &Character) {
    let mut p = Vec2::new(p[0], p[1]);

    fn draw_stat(draw: &DrawListMut, magic: &Res<Magic>, p: Vec2, char: CharacterStat, val: i32) {
        let c = {
            let c = magic[char];
            ImColor32::from_rgb_f32s(c.r(), c.g(), c.b())
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
        draw_stat(draw, magic, p, stat, val);
        p.x += 50.0;
    }
}

fn draw_hp_bar(
    draw: &DrawListMut,
    p: Vec2,
    health: &Health,
    magic: &Res<Magic>,
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
            p[0] + i as f32 * (width + padding) + offset.x - char_settings.inside_padding + width,
            p[1] + offset.y - char_settings.inside_padding + height,
        ];

        // let w = width - char_settings.inside_padding;
        // let h = height - char_settings.inside_padding;
        let al = Vec2::new(pi1[0], pi1[1]) - Vec2::new(0., 1.33);
        let de = Vec2::new(pi2[0], pi2[1]) + Vec2::new(0., 1.33);
        let ce = Vec2::new(al.x + width * 0.15, al.y + height * 0.25);
        // let be = al + Vec2::new(width - 2. * char_settings.inside_padding, 0.);
        // let ga = de - Vec2::new(width - 2. * char_settings.inside_padding, 0.);

        // //let ep = Vec2::new((al.x + be.x) * 0.5, al.y + h * 0.33);
        // //let pi = Vec2::new((al.x + be.x) * 0.5, ga.y - h * 0.33);
        // let c1 = Vec2::new((al.x + be.x) * 0.5, al.y);
        // let c2 = Vec2::new((ga.x + de.x) * 0.5, ga.y);
        // let d1 = Vec2::new(al.x, al.y + h * 0.33);
        // let d2 = Vec2::new(be.x, al.y + h * 0.33);
        // let e1 = Vec2::new(al.x, al.y + h * 0.66);
        // let e2 = Vec2::new(be.x, al.y + h * 0.66);

        let white = ImColor32::from_rgb(207, 198, 184);
        //let lite = ImColor32::from_rgb(227, 18, 45);
        let lite = ImColor32::from_rgb(122, 68, 74);
        draw.add_text(
            [
                p[0] + offset.x + char_settings.text_offset.x,
                p[1] + char_settings.text_offset.y + offset.y,
            ],
            ImColor32::from_rgb(207, 198, 184),
            "Health:",
        );
        if let Some(hp) = health.hitpoints.get(i) {
            if let Some((chant, val)) = hp.stat {
                let color = magic[chant];
                draw.add_rect(
                    pi1,
                    pi2,
                    ImColor32::from_rgb_f32s(color.r(), color.g(), color.b()),
                )
                .filled(true)
                .build();

                #[allow(clippy::comparison_chain)]
                if val < 0 {
                    draw.add_text([ce.x, ce.y], ImColor32::from_rgb(255, 255, 255), "-");
                    // draw.add_triangle([e1.x, e1.y], [ga.x, ga.y], [c2.x, c2.y], white)
                    //     .filled(true)
                    //     .build();
                    // draw.add_triangle([e2.x, e2.y], [de.x, de.y], [c2.x, c2.y], white)
                    //     .filled(true)
                    //     .build();
                } else if val > 0 {
                    draw.add_text([ce.x, ce.y], ImColor32::from_rgb(255, 255, 255), "+");
                    // draw.add_triangle([al.x, al.y], [c1.x, c1.y], [d1.x, d1.y], white)
                    //     .filled(true)
                    //     .build();
                    // draw.add_triangle([c1.x, c1.y], [be.x, be.y], [d2.x, d2.y], white)
                    //     .filled(true)
                    //     .build();
                }
            } else {
                draw.add_rect(pi1, pi2, lite).filled(true).build();
            }
        } else {
            draw.add_rect(pi1, pi2, ImColor32::from_rgb(0, 0, 0))
                .filled(true)
                .build();
        }

        draw.add_rect(p1, p2, white).filled(false).build();
    }
}

fn show_inventory(
    mut context: NonSendMut<ImguiContext>,
    player_entity: Query<
        (&WorldEntity, &Character, &CarriedItems, &EquippedItems),
        With<PlayerMarker>,
    >,
    items: Query<&Item>,
    colors: Query<&WorldEntityColor>,
    currently_selected_item: Res<CurrentlySelectedItem>,
) {
    let ui = context.ui();
    let Ok((_player, player_char, carried_items, equipped_items)) = player_entity.get_single()
    else {
        return;
    };

    ui.window("Inventory")
        .position_pivot([0.0, 0.0])
        .position([10.0, 90.0], imgui::Condition::Always)
        .size([400.0, 500.0], imgui::Condition::Always)
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

                let group = ui.begin_group();
                let selected_item = currently_selected_item.0 == Some(*item_id);

                let equipped = equipped_items.0.iter().any(|e| e == item_id);
                let eq = if equipped { "EQ" } else { "  " };

                if colors.contains(*item_id) && player_char.arcana > 3 && player_char.wisdom > 3 {
                    let color = colors.get(*item_id).unwrap().color;
                    let c = ui.push_style_color(
                        StyleColor::Text,
                        [color.r(), color.g(), color.b(), color.a()],
                    );
                    ui.text(format!(
                        "[{}] {}: {} ({:?})",
                        eq, id, item.name, item.item_type
                    ));
                    c.pop();
                } else {
                    ui.text(format!(
                        "[{}] {}: {} ({:?})",
                        eq, id, item.name, item.item_type
                    ));
                }

                let c = ui.push_style_color(
                    StyleColor::Text,
                    [207. / 255., 198. / 255., 184. / 255., 1.0],
                );
                ui.text_wrapped(format!("  {:?}", item));
                c.pop();

                if selected_item {
                    ui.text("  Actions:");
                    for action in item.available_actions() {
                        let action_text = match action {
                            ItemActions::Drop => "[D]rop",
                            ItemActions::Equip => "[E]quip",
                            ItemActions::Remove => "[R]emove",
                            ItemActions::Throw => "[T]hrow",
                            ItemActions::Consume => "[C]onsume",
                            ItemActions::Examine => "E[x]amine",
                        };
                        ui.text(format!("    {}", action_text));
                    }
                }

                ui.spacing();
                ui.separator();

                group.end();

                if selected_item {
                    let mut p1 = ui.item_rect_min();
                    p1[1] -= 2.0;
                    let mut p2 = ui.item_rect_max();
                    p2[0] = 400.0;
                    ui.get_window_draw_list()
                        .add_rect(p1, p2, ImColor32::from_rgba(255, 0, 0, 15))
                        .filled(true)
                        .build();
                }
            }
        });
}

fn show_log(mut context: NonSendMut<ImguiContext>, log: Res<HistoryLog>) {
    let ui = context.ui();

    ui.window("Log")
        .position_pivot([0.0, 0.0])
        .position([10.0, 600.0], imgui::Condition::Always)
        .size([400.0, 300.0], imgui::Condition::Always)
        .resizable(false)
        .collapsible(false)
        .no_decoration()
        .bg_alpha(1.0)
        .build(|| {
            ui.text("History");
            ui.separator();
            for line in log.0.iter().rev() {
                let c = ui.push_style_color(
                    StyleColor::Text,
                    [207. / 255., 198. / 255., 184. / 255., 1.0],
                );
                ui.text_wrapped(line);
                c.pop();
            }
        });
}

fn show_status_for_world_entities(
    player_entity: Query<(&WorldEntity, &Character, &Health), With<PlayerMarker>>,
    world_entities: Query<(&WorldEntity, &Character, &Health), Without<PlayerMarker>>,
    grid: Option<Res<Grid>>,
    world: Res<WorldData>,
    health_settings: Res<CharacterSettings>,
    magic: Res<Magic>,
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

            draw_hp_bar(&draw, p, player_health, &magic, &health_settings);
            draw_stats(
                &draw,
                &magic,
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

                    draw_hp_bar(&draw, p, other_health, &magic, &health_settings);
                    draw_stats(
                        &draw,
                        &magic,
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
                show_log,
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
