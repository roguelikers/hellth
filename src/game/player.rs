use bevy::{
    app::AppExit,
    prelude::*,
    render::{camera::CameraUpdateSystem, view::RenderLayers},
    transform::TransformSystem,
};
use bevy_kira_audio::Audio;

use crate::game::actions::{a_drop, a_move};

use super::{
    actions::{
        a_consume, a_descend, a_equip, a_focus, a_fortune, a_pickup, a_throw, a_unequip, a_wait, play_sfx, ActionEvent
    }, ai::PendingActions, character::Character, feel::{Random, Targeting, TweenSize}, grid::{Grid, WorldData, WorldEntity}, health::Health, history::HistoryLog, inventory::{
        CarriedItems, CarriedMarker, CurrentlySelectedItem, EquippedItems, Item, ItemActions,
        ItemType,
    }, music::{SfxCommand, SfxRevCommand}, procgen::{LevelDepth, PlayerMarker, ProcGenEvent}, sprites::{OCTOPUS, TARGET}, turns::{TurnCounter, TurnOrder}, GameStates
};

#[derive(Resource, Default, Debug, PartialEq)]
pub enum PlayerState {
    Idle,
    Dead,
    ItemSelected {
        index: usize,
    },
    PreparingToThrow {
        entity: Entity,
        item_entity: Entity,
    },
    #[default]
    Help,
    SacrificeWarning,
    Sacrifice,
    Descended,
    Ascended,
    Exiting,
    Shutdown,
    Reading(Entity),
}

fn try_item_keys(keys: &Res<Input<KeyCode>>) -> Option<usize> {
    if keys.just_pressed(KeyCode::Key1) {
        Some(1)
    } else if keys.just_pressed(KeyCode::Key2) {
        Some(2)
    } else if keys.just_pressed(KeyCode::Key3) {
        Some(3)
    } else if keys.just_pressed(KeyCode::Key4) {
        Some(4)
    } else if keys.just_pressed(KeyCode::Key5) {
        Some(5)
    } else if keys.just_pressed(KeyCode::Key6) {
        Some(6)
    } else if keys.just_pressed(KeyCode::Key7) {
        Some(7)
    } else if keys.just_pressed(KeyCode::Key8) {
        Some(8)
    } else if keys.just_pressed(KeyCode::Key9) {
        Some(9)
    } else {
        None
    }
}

fn try_direction_keys(keys: &Res<Input<KeyCode>>) -> Option<IVec2> {
    let shift = keys.pressed(KeyCode::ShiftLeft);
    if (shift && (keys.pressed(KeyCode::Numpad8) || keys.pressed(KeyCode::W))) || keys.just_pressed(KeyCode::W) || keys.just_pressed(KeyCode::Numpad8) {
        Some(IVec2::new(0, 1))
    } else if (shift && (keys.pressed(KeyCode::Numpad2) || keys.pressed(KeyCode::S))) || keys.just_pressed(KeyCode::S) || keys.just_pressed(KeyCode::Numpad2) {
        Some(IVec2::new(0, -1))
    } else if (shift && (keys.pressed(KeyCode::Numpad4) || keys.pressed(KeyCode::A))) || keys.just_pressed(KeyCode::A) || keys.just_pressed(KeyCode::Numpad4) {
        Some(IVec2::new(-1, 0))
    } else if (shift && (keys.pressed(KeyCode::Numpad6) || keys.pressed(KeyCode::D))) || keys.just_pressed(KeyCode::D) || keys.just_pressed(KeyCode::Numpad6) {
        Some(IVec2::new(1, 0))
    } else if (shift && (keys.pressed(KeyCode::Numpad7) || keys.pressed(KeyCode::Q))) || keys.just_pressed(KeyCode::Q) || keys.just_pressed(KeyCode::Numpad7) {
        Some(IVec2::new(-1, 1))
    } else if (shift && (keys.pressed(KeyCode::Numpad9) || keys.pressed(KeyCode::E))) || keys.just_pressed(KeyCode::E) || keys.just_pressed(KeyCode::Numpad9) {
        Some(IVec2::new(1, 1))
    } else if (shift && (keys.pressed(KeyCode::Numpad1) || keys.pressed(KeyCode::Z))) || keys.just_pressed(KeyCode::Z) || keys.just_pressed(KeyCode::Numpad1) {
        Some(IVec2::new(-1, -1))
    } else if (shift && (keys.pressed(KeyCode::Numpad3) || keys.pressed(KeyCode::C))) || keys.just_pressed(KeyCode::C) || keys.just_pressed(KeyCode::Numpad3) {
        Some(IVec2::new(1, -1))
    } else if keys.just_pressed(KeyCode::Period) || keys.just_pressed(KeyCode::X) || keys.just_pressed(KeyCode::Numpad5) {
        Some(IVec2::ZERO)
    } else {
        None
    }
}

pub fn on_shutdown(player_state: Res<PlayerState>, mut exit: EventWriter<AppExit>) {
    if matches!(*player_state, PlayerState::Shutdown) {
        exit.send(AppExit);
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
#[allow(unused_assignments)]
pub fn character_controls(
    mut procgen_events: EventWriter<ProcGenEvent>,
    mut turn_counter: ResMut<TurnCounter>,
    mut turn_order: ResMut<TurnOrder>,
    grid: Res<Grid>,
    map: Res<WorldData>,
    keys: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut targeting: Query<(Entity, &mut Transform, &mut Targeting), Without<PlayerMarker>>,
    mut player_query: Query<
        (
            Entity,
            &WorldEntity,
            &mut Health,
            &mut Character,
            &mut CarriedItems,
            &mut EquippedItems,
            &mut PendingActions,
        ),
        With<PlayerMarker>,
    >,
    free_item_query: Query<
        (Entity, &WorldEntity, &Item),
        (Without<PlayerMarker>, Without<CarriedMarker>),
    >,
    carried_item_query: Query<&Item, With<CarriedMarker>>,
    mut actions: EventWriter<ActionEvent>,
    mut history: ResMut<HistoryLog>,
    mut depth: ResMut<LevelDepth>,
    mut currently_selected_item: ResMut<CurrentlySelectedItem>,
    mut player_state: ResMut<PlayerState>,
) {
    if matches!(*player_state, PlayerState::Dead) && keys.just_pressed(KeyCode::Space) {
        *player_state = PlayerState::Help;
        procgen_events.send(ProcGenEvent::RestartWorld);
        turn_counter.0 = 0;
    }

    if let Some(e) = turn_order.peek() {
        if !player_query.contains(e) {
            //println!("#1");
            return;
        }
    }

    let Ok((
        entity,
        player_game_entity,
        health,
        character,
        inventory,
        equipped,
        mut pending_actions,
    )) = player_query.get_single_mut()
    else {
        //println!("#2");
        return;
    };

    let mut taken_action: Option<ActionEvent> = None;

    if let Some(next_action) = pending_actions.0.pop_front() {
        taken_action = Some(ActionEvent(next_action));
    } else {
        match player_state.as_ref() {
            PlayerState::Shutdown => {}

            PlayerState::Exiting => {
                if keys.just_pressed(KeyCode::Return) || keys.just_pressed(KeyCode::Y) {
                    *player_state = PlayerState::Shutdown;
                } else if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::N) {
                    *player_state = PlayerState::Idle;
                }
            }
            PlayerState::Ascended => {
                if keys.just_pressed(KeyCode::Space) {
                    *player_state = PlayerState::Help;
                    procgen_events.send(ProcGenEvent::RestartWorld);
                    turn_counter.0 = 0;
                }
            }

            PlayerState::Idle if currently_selected_item.0.is_some() => {
                currently_selected_item.0 = None;
            }

            PlayerState::Idle => {
                if health.hitpoints.is_empty() {
                    taken_action = Some(ActionEvent(a_wait()));

                    *player_state = PlayerState::Dead;
                    return;
                }

                let maybe_move = try_direction_keys(&keys);
                if let Some(direction) = maybe_move {
                    if direction == IVec2::ZERO {
                        taken_action = Some(ActionEvent(a_wait()));
                    }
                    if !map
                        .solid
                        .contains(&(player_game_entity.position + direction))
                    {
                        taken_action = Some(ActionEvent(a_move(entity, direction)));
                    }
                } else if keys.just_pressed(KeyCode::Escape) {
                    commands.add(SfxCommand { name: "ui_hover".to_string() });
                    *player_state = PlayerState::Exiting;
                } else if keys.just_pressed(KeyCode::Comma) || keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::G) {
                    let items = free_item_query
                        .iter()
                        .filter(|(_e, w, _i)| w.position == player_game_entity.position)
                        .collect::<Vec<_>>();

                    #[allow(clippy::comparison_chain)]
                    if !items.is_empty() {
                        taken_action = Some(ActionEvent(a_pickup(
                            entity,
                            items.iter().map(|i| i.0).collect::<Vec<_>>(),
                        )));
                    } else {
                        history.add("Nothing to pick up");
                    }
                } else if let Some(item_key) = try_item_keys(&keys) {
                    commands.add(SfxCommand { name: "ui_hover".to_string() });
                    *player_state = PlayerState::ItemSelected { index: item_key };
                } else if keys.just_pressed(KeyCode::H) {
                    commands.add(SfxCommand { name: "ui_hover".to_string() });
                    *player_state = PlayerState::Help;
                } else if keys.just_pressed(KeyCode::F) {
                    taken_action = Some(ActionEvent(a_focus(entity)));
                } else if keys.just_pressed(KeyCode::M) && depth.0 < 5 {
                    commands.add(SfxCommand { name: "ui_hover".to_string() });
                    *player_state = PlayerState::SacrificeWarning;
                }
            }

            PlayerState::SacrificeWarning => {
                if keys.just_pressed(KeyCode::Y) {
                    *player_state = PlayerState::Sacrifice;
                } else if keys.just_pressed(KeyCode::N) {
                    *player_state = PlayerState::Idle;
                }
            }

            PlayerState::Sacrifice => {
                history.add("You descend...");
                history.add("---------------------------------");
                procgen_events.send(ProcGenEvent::NextLevel);

                taken_action = Some(ActionEvent(a_descend()));
                depth.0 += 1;
                *player_state = PlayerState::Descended;
            }

            PlayerState::Descended => {
                if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Escape) {
                    *player_state = PlayerState::Idle;
                }
            }

            PlayerState::Dead => {
                if keys.just_pressed(KeyCode::Space) {
                    *player_state = PlayerState::Help;
                    procgen_events.send(ProcGenEvent::RestartWorld);
                    turn_counter.0 = 0;
                }

                taken_action = Some(ActionEvent(a_wait()));
                turn_order.pushback(100);
            }

            PlayerState::ItemSelected { index } if *index > inventory.0.len() => {
                *player_state = PlayerState::Idle;
            }

            PlayerState::ItemSelected { index } if currently_selected_item.0.is_none() => {
                let Some(item_entity) = inventory.0.get(*index - 1) else {
                    panic!("Item should be here, but is not found!");
                };

                currently_selected_item.0 = Some(*item_entity);
            }

            PlayerState::ItemSelected { index: _ } => {
                if let Some(item_key) = try_item_keys(&keys) {
                    commands.add(SfxCommand { name: "ui_hover".to_string() });
                    currently_selected_item.0 = None;
                    *player_state = PlayerState::ItemSelected { index: item_key };
                    return;
                }

                if keys.just_pressed(KeyCode::Escape) {
                    commands.add(SfxRevCommand { name: "ui_select".to_string() });
                    *player_state = PlayerState::Idle;
                    return;
                }
                let item_entity = currently_selected_item.0.unwrap();

                let Ok(item) = carried_item_query.get(item_entity) else {
                    return;
                };

                for action in item.available_actions() {
                    let action_key = match action {
                        ItemActions::Drop => Some(KeyCode::D),
                        ItemActions::Focus => Some(KeyCode::F),
                        ItemActions::Equip if !equipped.0.contains(&item_entity) => {
                            Some(KeyCode::E)
                        }
                        ItemActions::Unequip if equipped.0.contains(&item_entity) => {
                            Some(KeyCode::E)
                        }
                        ItemActions::Throw => Some(KeyCode::T),
                        ItemActions::Consume => Some(KeyCode::C),
                        ItemActions::Examine => Some(KeyCode::X),
                        _ => None,
                    };

                    let Some(action_key) = action_key else {
                        continue;
                    };

                    if keys.just_pressed(action_key) {
                        match action {
                            ItemActions::Drop => {
                                commands.add(SfxCommand { name: "ui_select".to_string() });
                                taken_action = Some(ActionEvent(a_drop(entity, vec![item_entity])));
                            }
                            ItemActions::Focus => {
                                commands.add(SfxCommand { name: "ui_select".to_string() });
                                taken_action = Some(ActionEvent(a_focus(entity)));
                            }
                            ItemActions::Equip => {
                                commands.add(SfxCommand { name: "ui_select".to_string() });
                                taken_action = Some(ActionEvent(a_equip(entity, item_entity)));
                            }
                            ItemActions::Unequip => {
                                commands.add(SfxCommand { name: "ui_select".to_string() });
                                taken_action = Some(ActionEvent(a_unequip(entity, item_entity)));
                            }
                            ItemActions::Throw => {
                                commands.add(SfxCommand { name: "ui_select".to_string() });
                                commands.spawn((
                                    SpriteSheetBundle {
                                        sprite: TextureAtlasSprite::new(TARGET.into()),
                                        texture_atlas: grid.atlas.clone_weak(),
                                        transform: grid
                                            .get_tile_position(player_game_entity.position)
                                            .with_scale(Vec3::new(1.25, 1.25, 1.25)),
                                        ..Default::default()
                                    },
                                    RenderLayers::layer(1),
                                    TweenSize {
                                        baseline: 1.25,
                                        max: 0.25,
                                    },
                                    Targeting(player_game_entity.position),
                                ));

                                *player_state = PlayerState::PreparingToThrow {
                                    entity,
                                    item_entity,
                                };
                                break;
                            }

                            ItemActions::Consume => {
                                commands.add(SfxCommand { name: "ui_select".to_string() });
                                taken_action = Some(ActionEvent(a_consume(entity, item_entity)));
                            }

                            ItemActions::Examine => {
                                commands.add(SfxCommand { name: "ui_select".to_string() });
                                *player_state = PlayerState::Reading(item_entity);
                                return;
                            }
                        }

                        if !matches!(action, ItemActions::Focus) {
                            currently_selected_item.0 = None;
                            *player_state = PlayerState::Idle;
                        }
                        break;
                    }
                }
            }

            PlayerState::PreparingToThrow {
                entity,
                item_entity,
            } => {
                if let Some(dir) = try_direction_keys(&keys) {
                    let (_, mut target_transform, mut targeting) = targeting.single_mut();
                    targeting.0 += dir;
                    *target_transform = grid.get_tile_position(targeting.0);
                } else if keys.just_pressed(KeyCode::Escape) {
                    let (target_entity, _, _) = targeting.single();
                    commands.entity(target_entity).despawn_recursive();
                    *player_state = PlayerState::Idle;
                    return;
                } else if keys.just_pressed(KeyCode::Space) {
                    let (target_entity, _, targeting) = targeting.single();
                    taken_action = Some(ActionEvent(a_throw(*entity, *item_entity, targeting.0)));
                    commands.entity(target_entity).despawn_recursive();
                    *player_state = PlayerState::Idle;
                }
                //
            }

            PlayerState::Reading(item) => {
                if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Escape) {
                    taken_action = Some(ActionEvent(a_fortune(*item)));
                    *player_state = PlayerState::Idle;
                }
            }
            PlayerState::Help => {
                if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Escape) {
                    *player_state = PlayerState::Idle;
                }
            }
        }
    }

    if let Some(action) = taken_action {
        let cost = character.calculate_cost(action.0.get_affiliated_stat());
        turn_order.pushback(cost);
        actions.send(action);
    }
}

#[derive(Resource)]
pub struct Achievements {
    pub octopus_mode: bool,
    pub messages: Vec<String>,
}

impl Default for Achievements {
    fn default() -> Self {
        Self { octopus_mode: false, messages: vec![
            "(from the lost book of Agustin the Mage)\n\n [1/5] We start through LORE, like stories of old, ...".to_string(),
            "(from the lost book of Agustin the Mage)\n\n [2/5]   ...through hardships up the knife, to the EDGE...".to_string(),
            "(from the lost book of Agustin the Mage)\n\n [3/5]...to slice and fall, ourselves into PRISON cast...".to_string(),
            "(from the lost book of Agustin the Mage)\n\n [4/5]   ...our REGALIA taken and thrown to the wolves...".to_string(),
            "(from the lost book of Agustin the Mage)\n\n [5/5]...until we become DUST in someone else's cough.".to_string(),
            "(from the Tome of Nhub)\n\nThrowing staffs is pretty inefficient...".to_string(),
            "(from the Tome of Nhub)\n\nSacrifices all go to the HEALER...".to_string(),
            "(from the Tome of Nhub)\n\nOf all the stats, only INT and WIL affect your sight...".to_string(),
            "(from the Tome of Nhub)\n\nYour combat moves are faster if you have higher STR,\n and you walk faster if you have higher AGI!".to_string(),
            "(from a tomb clad in leather)\n\n...be wary of sacrifices as they will undo ye.\n To travel, thou arth undone and then redone yet again.\n Thy vessel remade. Thy greatest strength turned against you.\n Nine they take.".to_string(),
            "(from a quickly scribbled note)\n\nI see them now, the thaumaturg litanists of the Healer.\n I see them, and hear them too. I gave too much to stop,\n yet turn back I do as I understand the truth...".to_string(),
            "(from a tomb clad in leather)\n\nat the ...scribble...demy of arts spiritual, they tell us\n to enscribe into our bones the chants of our enemies.\n From your bones to...".to_string(),
            "(from the Tome of Nhub)\n\nUse FOCUS ('F' key) to move the effects of consumed bones deeper into\n your health bar, making them harder to remove.".to_string(),
            "(from an empty page, a bodiless voice emanates)\n...THE BODY: the certain rejection of one's thaums\n is as inevitable as daylight after night. If you consume,\n it will spill out. So focus and consume deep.".to_string(),
            "(from a crumbling piece of papyrus)\n\nFocus takes time. Focus means life. If you take other's bones, cast them not\n onto thyself without meaning and reason.\n Do so at the right moment, when thy bones dry out.".to_string(),
            "(from the Tome of Nhub)\n\nIf you have high STR, your body will expel enchantments good or bad,\n pushing them from your deeper health points to the weaker\n ones on the right, and disappearing over time.".to_string(),
            "(from the Tome of Nhub)\n\nIf your carpal tunnel is acting up, use SHIFT to run.\n It's not too precise but it gets you places.".to_string(),
            "(from the Tome of Nhub)\n\nIf you have at least 8 STR, you will recover health over time.".to_string(),
            "(from the Tome of Nhub)\n\nRaise WIS and ARC to start seeing auras - colors\n on items and monsters depicting their STRONGEST STAT.".to_string(),
            "(a sad, crumpled, hacked up note)\n\nPlease disregard previous message.".to_string(),
            "(a sad, crumpled, hacked up note)\n\nWizard needs food badly.".to_string(),
            "(from the Tome of Nhub)\n\nThey don't see you if you don't see them, but they remember and they follow.".to_string(),
            "(a disembodied voice escapes from a page of otherwise bland poetry)\n\nEr bones are not only good for sourcing one's thaums,\n but also fer cursing them with each other's thaums!\n Toss away and relish in their feeble state!".to_string(),
            "(from the Tome of Nhub)\n\nEnchanters will curse you. Your health will show you colors.\n These are your stat colors and every 'v' symbol\n there means you have -1 of that stat. '^' means you have +1,\n and you can get that by focusing at the\n right spot and consuming bones".to_string(),
        ] }
    }
}

fn achievement_restart(mut procgen_events: EventReader<ProcGenEvent>, mut achievements: ResMut<Achievements>, mut rng: ResMut<Random>) {
    for procgen in procgen_events.read() {
        if *procgen == ProcGenEvent::RestartWorld {
            *achievements = Achievements::default();
            achievements.messages = rng.shuffle(achievements.messages.clone());
        }
    }
}

fn octopus_tracker(
    mut equipped_query: Query<(&mut TextureAtlasSprite, &EquippedItems), With<PlayerMarker>>,
    items: Query<&Item>,
    mut log: ResMut<HistoryLog>,
    mut achievements: ResMut<Achievements>,
) {
    if achievements.octopus_mode {
        return;
    }

    let Ok((mut sprite, equipped)) = equipped_query.get_single_mut() else {
        return;
    };

    let count = equipped
        .0
        .iter()
        .filter(|&item_entity| {
            if let Ok(item) = items.get(*item_entity) {
                item.item_type == ItemType::Weapon
            } else {
                false
            }
        })
        .count();

    if count > 2 {
        log.add("You wielded more than two weapons at once. Must be an octopus. Try doing a run without this for an achievement.");
        achievements.octopus_mode = true;
        sprite.index = OCTOPUS.into();
    }
}

pub struct SvarogPlayerPlugin;
impl Plugin for SvarogPlayerPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.init_resource::<PlayerState>();
        bevy.init_resource::<Achievements>();
        bevy.add_systems(
            Update,
            character_controls
                .before(TransformSystem::TransformPropagate)
                .before(CameraUpdateSystem)
                .run_if(in_state(GameStates::Game)),
        );
        bevy.add_systems(PostUpdate, (octopus_tracker, on_shutdown));
        bevy.add_systems(Update, achievement_restart.run_if(on_event::<ProcGenEvent>()));
    }
}
