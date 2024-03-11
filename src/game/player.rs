use bevy::{
    app::AppExit,
    prelude::*,
    render::{camera::CameraUpdateSystem, view::RenderLayers},
    transform::TransformSystem,
};

use crate::game::actions::{a_drop, a_move};

use super::{
    actions::{
        a_consume, a_descend, a_equip, a_focus, a_pickup, a_throw, a_unequip, a_wait, ActionEvent,
    },
    ai::PendingActions,
    character::Character,
    feel::{Targeting, TweenSize},
    grid::{Grid, WorldData, WorldEntity},
    health::Health,
    history::HistoryLog,
    inventory::{
        CarriedItems, CarriedMarker, CurrentlySelectedItem, EquippedItems, Item, ItemActions,
    },
    procgen::{LevelDepth, PlayerMarker, ProcGenEvent},
    sprites::TARGET,
    turns::{TurnCounter, TurnOrder},
    GameStates,
};

#[derive(Resource, Default, Debug)]
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
    if (shift && keys.pressed(KeyCode::W)) || keys.just_pressed(KeyCode::W) {
        Some(IVec2::new(0, 1))
    } else if (shift && keys.pressed(KeyCode::S)) || keys.just_pressed(KeyCode::S) {
        Some(IVec2::new(0, -1))
    } else if (shift && keys.pressed(KeyCode::A)) || keys.just_pressed(KeyCode::A) {
        Some(IVec2::new(-1, 0))
    } else if (shift && keys.pressed(KeyCode::D)) || keys.just_pressed(KeyCode::D) {
        Some(IVec2::new(1, 0))
    } else if (shift && keys.pressed(KeyCode::Q)) || keys.just_pressed(KeyCode::Q) {
        Some(IVec2::new(-1, 1))
    } else if (shift && keys.pressed(KeyCode::E)) || keys.just_pressed(KeyCode::E) {
        Some(IVec2::new(1, 1))
    } else if (shift && keys.pressed(KeyCode::Z)) || keys.just_pressed(KeyCode::Z) {
        Some(IVec2::new(-1, -1))
    } else if (shift && keys.pressed(KeyCode::C)) || keys.just_pressed(KeyCode::C) {
        Some(IVec2::new(1, -1))
    } else if keys.just_pressed(KeyCode::Period) || keys.just_pressed(KeyCode::X) {
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
                if keys.just_pressed(KeyCode::Return) {
                    *player_state = PlayerState::Shutdown;
                } else if keys.just_pressed(KeyCode::Escape) {
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
                    *player_state = PlayerState::Exiting;
                } else if keys.just_pressed(KeyCode::Comma) || keys.just_pressed(KeyCode::Space) {
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
                    *player_state = PlayerState::ItemSelected { index: item_key };
                } else if keys.just_pressed(KeyCode::H) {
                    *player_state = PlayerState::Help;
                } else if keys.just_pressed(KeyCode::F) {
                    taken_action = Some(ActionEvent(a_focus(entity)));
                } else if keys.just_pressed(KeyCode::M) && depth.0 < 5 {
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
                    currently_selected_item.0 = None;
                    *player_state = PlayerState::ItemSelected { index: item_key };
                    return;
                }

                if keys.just_pressed(KeyCode::Escape) {
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
                                taken_action = Some(ActionEvent(a_drop(entity, vec![item_entity])));
                            }
                            ItemActions::Equip => {
                                taken_action = Some(ActionEvent(a_equip(entity, item_entity)));
                            }
                            ItemActions::Unequip => {
                                taken_action = Some(ActionEvent(a_unequip(entity, item_entity)));
                            }
                            ItemActions::Throw => {
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
                                taken_action = Some(ActionEvent(a_consume(entity, item_entity)));
                            }

                            ItemActions::Examine => todo!(),
                        }
                        currently_selected_item.0 = None;
                        *player_state = PlayerState::Idle;
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

pub struct SvarogPlayerPlugin;
impl Plugin for SvarogPlayerPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.init_resource::<PlayerState>();
        bevy.add_systems(
            Update,
            character_controls
                .before(TransformSystem::TransformPropagate)
                .before(CameraUpdateSystem)
                .run_if(in_state(GameStates::Game)),
        );
        bevy.add_systems(PostUpdate, on_shutdown);
    }
}
