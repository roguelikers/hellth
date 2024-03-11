use bevy::prelude::*;

use bevy_mod_imgui::ImguiContext;
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::ListenerInput,
};
use imgui::{DrawListMut, ImColor32, StyleColor};

use super::{
    character::{ Character, CharacterStat},
    grid::{Grid, WorldData, WorldEntity, WorldEntityColor},
    health::Health,
    history::HistoryLog,
    inventory::{CarriedItems, CurrentlySelectedItem, EquippedItems, Item, ItemActions},
    magic::Magic,
    player::{Achievements, PlayerState},
    procgen::PlayerMarker,
    turns::TurnCounter,
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

fn draw_player_stats(draw: &DrawListMut, magic: &Res<Magic>, p: Vec2, char: &mut Character) {
    let mut p = Vec2::new(p[0], p[1]);

    fn draw_stat(
        draw: &DrawListMut,
        magic: &Res<Magic>,
        p: Vec2,
        stat: CharacterStat,
        val: i32,
        learned: bool,
    ) {
        let c = {
            let c = magic[stat];
            if learned {
                ImColor32::from_rgb_f32s(c.r(), c.g(), c.b())
            } else {
                let g = (c.r() + c.g() + c.b()) / 3.0;
                ImColor32::from_rgb_f32s(g, g, g)
            }
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
        draw.add_text(t1, w, format!("{:?}", stat));
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

        let learned = if char.learned.contains(&stat) {
            true
        } else {
            let count = {
                if char.counters.contains_key(&stat) {
                    *char.counters.get(&stat).unwrap()
                } else {
                    0
                }
            };

            let mut limit = 3;
            if char[CharacterStat::WIS] < 3 {
                limit = 3 + (3 - char[CharacterStat::WIS]);
            }

            let learned = count > limit as u32;
            if learned {
                char.learned.insert(stat);
            }
            learned
        };

        draw_stat(draw, magic, p, stat, val, learned);
        p.x += 50.0;
    }
}

fn draw_npc_stats(
    draw: &DrawListMut,
    magic: &Res<Magic>,
    p: Vec2,
    char: &Character,
    player: &Character,
) {
    let mut p = Vec2::new(p[0], p[1]);

    fn draw_stat(
        draw: &DrawListMut,
        magic: &Res<Magic>,
        p: Vec2,
        stat: CharacterStat,
        val: i32,
        player: &Character,
    ) {
        if player.learned.contains(&stat) {
            let c = {
                let c = magic[stat];
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
            draw.add_text(t1, w, format!("{:?}", stat));
            draw.add_text(t2, w, format!("{}", val));
        }
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
        draw_stat(draw, magic, p, stat, val, player);
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
        // let de = Vec2::new(pi2[0], pi2[1]) + Vec2::new(0., 1.33);
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
                            ItemActions::Drop => Some("[D]rop"),
                            ItemActions::Equip if !equipped => Some("[E]quip"),
                            ItemActions::Unequip if equipped => Some("Un[E]quip"),
                            ItemActions::Throw => Some("[T]hrow"),
                            ItemActions::Consume => Some("[C]onsume"),
                            ItemActions::Examine => Some("E[x]amine"),
                            _ => None,
                        };

                        if action_text.is_some() {
                            ui.text(format!("    {}", action_text.unwrap()));
                        }
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

fn show_throw_tip(mut context: NonSendMut<ImguiContext>, player_state: Res<PlayerState>) {
    let ui = context.ui();

    if matches!(
        *player_state,
        PlayerState::PreparingToThrow {
            entity: _,
            item_entity: _
        }
    ) {
        let [w, _] = ui.io().display_size;

        ui.window("Tip")
            .position_pivot([0.5, 0.0])
            .position([w / 2.0, 100.0], imgui::Condition::Always)
            .size([400.0, 20.0], imgui::Condition::Always)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .bg_alpha(1.0)
            .build(|| {
                let text = "MOVE to target, ESCAPE to cancel, SPACE to commit";
                let [w, _] = ui.calc_text_size(text);
                ui.set_cursor_pos([(400.0 - w) * 0.5, 10.0]);
                ui.text(text);
            });
    }
}

fn show_sacrifice_warning(
    mut context: NonSendMut<ImguiContext>,
    player_character: Query<&Character, With<PlayerMarker>>,
    player_state: Res<PlayerState>,
) {
    let Ok(player_character) = player_character.get_single() else {
        return;
    };

    let ui = context.ui();

    if matches!(*player_state, PlayerState::SacrificeWarning) {
        let [w, _] = ui.io().display_size;

        ui.window("Sacrifice")
            .position_pivot([0.5, 0.0])
            .position([w / 2.0, 100.0], imgui::Condition::Always)
            .size([600.0, 200.0], imgui::Condition::Always)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .bg_alpha(1.0)
            .build(|| {
                let [w, _] = ui.calc_text_size("MAKE SACRIFICE?");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 10.0]);
                ui.text("MAKE SACRIFICE?");

                ui.set_cursor_pos([25.0, 40.0]);

                let mut message = vec![];
                message.push("Making a sacrifice exhudes a heavy toll. NINE is the number".to_string());
                message.push("by which you pay, or the bounty is collected from thy undying bones.".to_string());

                let (stat, val) = player_character.get_strongest_stat();
                
                if val < 9 {
                    message.push("This body of thine will suffer if you attempt this now.".to_string());

                    if val > 7 {
                        let stat_name = match stat {
                            CharacterStat::STR => "strength",
                            CharacterStat::ARC => "arcana",
                            CharacterStat::INT => "intelligence",
                            CharacterStat::WIS => "wisdom",
                            CharacterStat::WIL => "willpower",
                            CharacterStat::AGI => "agility",
                        };
                        message.push(format!("There are slivers of greatness in your {}, however. Look deeper into it.", stat_name).to_string());
                        message.push("Let others sacrifice onto you before you approach again.".to_string());
                    }
                }

                let mut y = 30.0;
                for msg in message {
                    let [w, _] = ui.calc_text_size(&msg);
                    ui.set_cursor_pos([(600.0 - w) * 0.5, y]);
                    ui.text_wrapped(&msg);    
                    y += 15.0;
                }

                let [w, _] = ui.calc_text_size("Are you sure you want to proceed?");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 100.0]);
                ui.text_wrapped("Are you sure you want to proceed?");


                let [w, _] = ui.calc_text_size("[Y]es");
                ui.set_cursor_pos([200.0 - w * 0.5, 150.0]);
                ui.text("[Y]es");

                let [w, _] = ui.calc_text_size("[N]o");
                ui.set_cursor_pos([400.0 - w * 0.5, 150.0]);
                ui.text("[N]o");

            });
    }
}

fn show_exit(
    mut context: NonSendMut<ImguiContext>,
    player_state: Res<PlayerState>,
) {
    let ui = context.ui();

    if matches!(*player_state, PlayerState::Exiting) {
        let [w, _] = ui.io().display_size;

        ui.window("Exit")
            .position_pivot([0.5, 0.0])
            .position([w / 2.0, 100.0], imgui::Condition::Always)
            .size([600.0, 200.0], imgui::Condition::Always)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .bg_alpha(1.0)
            .build(|| {
                let [w, _] = ui.calc_text_size("Exit game?");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 10.0]);
                ui.text("Exit game?");

                let [w, _] = ui.calc_text_size(
                    "You will lose all progress if you quit. Do you want to proceed?",
                );
                ui.set_cursor_pos([(600.0 - w) * 0.5, 40.0]);
                ui.text_wrapped("You will lose all progress if you quit. Do you want to proceed?");

                let [w, _] = ui.calc_text_size("[Return/Enter] Yes");
                ui.set_cursor_pos([200.0 - w * 0.5, 150.0]);
                ui.text("[Return/Enter] Yes");

                let [w, _] = ui.calc_text_size("[Escape again] No");
                ui.set_cursor_pos([400.0 - w * 0.5, 150.0]);
                ui.text("[Escape again] No");
            });
    }
}

use crate::game::procgen::LevelDepth;

fn show_descend_info(
    mut context: NonSendMut<ImguiContext>,
    player_state: Res<PlayerState>,
    depth: Res<LevelDepth>,
) {

    let ui = context.ui();

    if matches!(*player_state, PlayerState::Descended) {
        let [w, _] = ui.io().display_size;

        ui.window("Descent")
            .position_pivot([0.5, 0.0])
            .position([w / 2.0, 100.0], imgui::Condition::Always)
            .size([600.0, 110.0], imgui::Condition::Always)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .bg_alpha(1.0)
            .build(|| {
                let [w, _] = ui.calc_text_size("YOU HAVE DESCENDED.");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 10.0]);
                ui.text("YOU HAVE DESCENDED.");

                let text = format!(
                    "You have descended into level {}. Stand proud, if you can stand.",
                    depth.0
                );
                let [w, _] = ui.calc_text_size(&text);
                ui.set_cursor_pos([(600.0 - w) * 0.5, 40.0]);
                ui.text(&text);

                let [w, _] = ui.calc_text_size("Press SPACE to continue.");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 80.0]);
                ui.text("Press SPACE to continue.");
            });
    }
}

fn show_ascended_status(
    mut context: NonSendMut<ImguiContext>,
    player_state: Res<PlayerState>,
    turn_counter: Res<TurnCounter>,
    achievements: Res<Achievements>,
) {
    
    let ui = context.ui();

    if matches!(*player_state, PlayerState::Ascended) {
        let [w, _] = ui.io().display_size;

        ui.window("CONGRATULATIONS")
            .position_pivot([0.5, 0.0])
            .position([w / 2.0, 100.0], imgui::Condition::Always)
            .size([600.0, 110.0], imgui::Condition::Always)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .bg_alpha(1.0)
            .build(|| {
                let [w, _] = ui.calc_text_size("CONGRATULATIONS!");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 10.0]);
                ui.text("CONGRATULATIONS");

                let octo = achievements.octopus_mode;
                let text = format!("You have beaten the Healer in {} turns{}.", turn_counter.0, 
                    if octo { " and sacrificed your humanity along the way" } else { " without sacrificing your bipedal nature" });

                let [w, _] = ui.calc_text_size(&text);
                ui.set_cursor_pos([(600.0 - w) * 0.5, 40.0]);
                ui.text(&text);
                
                let [w, _] = ui.calc_text_size("Press SPACE to restart.");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 80.0]);
                ui.text("Press SPACE to restart.");
            });
    }
}

fn show_dead_screen(
    mut context: NonSendMut<ImguiContext>,
    player_state: Res<PlayerState>,
    depth: Res<LevelDepth>,
    turn_counter: Res<TurnCounter>,
) {
    let ui = context.ui();

    if matches!(*player_state, PlayerState::Dead) {
        let [w, _] = ui.io().display_size;

        ui.window("Ded")
            .position_pivot([0.5, 0.0])
            .position([w / 2.0, 100.0], imgui::Condition::Always)
            .size([600.0, 110.0], imgui::Condition::Always)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .bg_alpha(1.0)
            .build(|| {
                let [w, _] = ui.calc_text_size("You died.");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 10.0]);
                ui.text("You died.");

                let text = format!(
                    "You have been killed after {} turns on level {}.",
                    turn_counter.0, depth.0
                );

                let [w, _] = ui.calc_text_size(&text);
                ui.set_cursor_pos([(600.0 - w) * 0.5, 40.0]);
                ui.text(&text);

                let [w, _] = ui.calc_text_size("Press SPACE to restart.");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 80.0]);
                ui.text("Press SPACE to restart.");
            });
    }
}

fn show_help(mut context: NonSendMut<ImguiContext>, player_state: Res<PlayerState>) {
    let ui = context.ui();

    if matches!(*player_state, PlayerState::Help) {
        let [w, _] = ui.io().display_size;

        ui.window("Tip")
            .position_pivot([0.5, 0.0])
            .position([w / 2.0, 100.0], imgui::Condition::Always)
            .size([600.0, 370.0], imgui::Condition::Always)
            .resizable(false)
            .collapsible(false)
            .no_decoration()
            .bg_alpha(1.0)
            .build(|| {
                let [w, _] = ui.calc_text_size("HOW TO");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 10.0]);
                ui.text("HOW TO");

                ui.text("Hark thee!");
                ui.spacing();
                let mut message = vec!["You are the latest in a long line of acolytes sent to venture into the Ruins of the World in the hopes of slaying the Healer.".to_string()];                
                message.push("Let not yourself be fooled by that name -- this person brought ruin; he remade the world by collapsing the Spiritual Divide.".to_string());
                message.push("Going down doesn't require only time, but sacrifice...".to_string());
                ui.text_wrapped(message.join(" "));
                ui.text_wrapped("Staircases going down don't exist. Consume. Grow. Sacrifice. Find a way.");
                ui.separator();
                ui.text_wrapped("Help (this screen): H");
                ui.text_wrapped("Movement: ASDW + QEZC (diagonal)");
                ui.text_wrapped("Make Sacrifice (attempt to descend): M");
                ui.text_wrapped("Focus Thaumaturgy: F");
                ui.text_wrapped("Wait Turn: X");
                ui.text_wrapped("Cancel: Escape");
                ui.text_wrapped("Pickup: Space");
                ui.text_wrapped("Items: 1-9 to start interaction");
                ui.separator();
                let mut message = vec!["This challenge will see you use and consume artifacts that inflict upon your health strange glyphs to affect your state.".to_string()];
                message.push("It shall change how you perceive things. It shall make you wonder about your choices. Remember the shape of your soul and track it carefully.".to_string());
                message.push("Remember that in this place NINE is the strength with which devotion accepts a sacrifice. Be wary of what you ask for. Remember it in your BONES.".to_string());
                ui.text_wrapped(message.join(" "));
                ui.spacing();

                let [w, _] = ui.calc_text_size("Press SPACE to continue.");
                ui.set_cursor_pos([(600.0 - w) * 0.5, 350.0]);
                ui.text("Press SPACE to continue.");
            });
    }
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
    mut player_entity: Query<(&WorldEntity, &mut Character, &Health), With<PlayerMarker>>,
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

    let Ok((player, mut player_char, player_health)) = player_entity.get_single_mut() else {
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
            draw_player_stats(
                &draw,
                &magic,
                p + Vec2::new(health_settings.stat_left, health_settings.stat_top),
                &mut player_char,
            );
        });

    let mut window_y = 10.0f32;
    for (other_entity, other_char, other_health) in &world_entities {
        let (x, y) = grid.norm(other_entity.position);
        if world.data.is_in_fov(x, y) {
            ui.window(&format!("{}{}", other_entity.name, window_y))
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
                    if player_char.wisdom >= 5 && player_char.arcana >= 5 {
                        draw_npc_stats(
                            &draw,
                            &magic,
                            p + Vec2::new(health_settings.stat_left, health_settings.stat_top),
                            other_char,
                            player_char.as_ref(),
                        );
                    }
                });

            window_y += 78.0;
        }
    }
}

pub fn show_progress_status(mut context: NonSendMut<ImguiContext>, level_depth: Res<LevelDepth>, turn_counter: Res<TurnCounter>) {
    let ui = context.ui();

    let [width, height] = ui.io().display_size;

    ui.window("PROGRESS")
        .position_pivot([1.0, 1.0])
        .position([width - 20.0, height - 20.0], imgui::Condition::Always)
        .size([100.0, 50.0], imgui::Condition::Always)
        .resizable(false)
        .collapsible(false)
        .no_decoration()
        .bg_alpha(1.0)
        .build(|| {
            ui.text(format!("Depth: {}", level_depth.0));
            ui.text(format!("Turns: {}", turn_counter.0));
        });
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
                show_throw_tip,
                show_sacrifice_warning,
                show_descend_info,
                show_dead_screen,
                show_ascended_status,
                show_exit,
                draw_health_settings,
                show_progress_status,
                show_help,
            )
                .chain()
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
