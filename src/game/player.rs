use bevy::{
    prelude::*, render::camera::CameraUpdateSystem, transform::TransformSystem,
    window::PrimaryWindow,
};
use bevy_mouse_tracking_plugin::MousePosWorld;

use crate::game::{actions::a_move, turns::TurnOrderEntity};

use super::{
    actions::{a_pickup, a_wait, ActionEvent},
    ai::PendingActions,
    character::Character,
    grid::{WorldData, WorldEntity},
    health::Health,
    inventory::{CarriedMarker, Item},
    procgen::PlayerMarker,
    turns::TurnOrder,
    GameStates,
};

#[derive(Debug, Default)]
pub enum CastSpellState {
    #[default]
    ChooseSpell,
    ChooseTarget,
    CastingTime,
    // back to idle
}

#[derive(Debug, Default)]
pub enum ItemThrowState {
    #[default]
    ChooseItem,
    ChooseTarget,
    // back to idle
}

#[derive(Component, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    Dead,
    Spell(CastSpellState),
    Throw(ItemThrowState),
}

fn try_direction_keys(keys: &Res<Input<KeyCode>>) -> Option<IVec2> {
    if keys.just_pressed(KeyCode::W) {
        Some(IVec2::new(0, 1))
    } else if keys.just_pressed(KeyCode::S) {
        Some(IVec2::new(0, -1))
    } else if keys.just_pressed(KeyCode::A) {
        Some(IVec2::new(-1, 0))
    } else if keys.just_pressed(KeyCode::D) {
        Some(IVec2::new(1, 0))
    } else if keys.just_pressed(KeyCode::Q) {
        Some(IVec2::new(-1, 1))
    } else if keys.just_pressed(KeyCode::E) {
        Some(IVec2::new(1, 1))
    } else if keys.just_pressed(KeyCode::Z) {
        Some(IVec2::new(-1, -1))
    } else if keys.just_pressed(KeyCode::C) {
        Some(IVec2::new(1, -1))
    } else if keys.just_pressed(KeyCode::Period) || keys.just_pressed(KeyCode::X) {
        Some(IVec2::ZERO)
    } else {
        None
    }
}

#[allow(clippy::type_complexity)]
pub fn character_controls(
    mouse: Res<MousePosWorld>,
    mut turn_order: ResMut<TurnOrder>,
    map: Res<WorldData>,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<
        (
            Entity,
            &WorldEntity,
            &Health,
            &Character,
            &mut PendingActions,
            &mut PlayerState,
        ),
        With<PlayerMarker>,
    >,
    item_query: Query<
        (Entity, &WorldEntity, &Item),
        (Without<PlayerMarker>, Without<CarriedMarker>),
    >,
    mut actions: EventWriter<ActionEvent>,
) {
    if let Some(e) = turn_order.peek() {
        if !player_query.contains(e) {
            return;
        }
    }

    let Ok((entity, player_game_entity, health, character, mut pending_actions, mut player_state)) =
        player_query.get_single_mut()
    else {
        return;
    };

    //println!("{:?} {:?}", mouse, player_game_entity.position);
    let mut taken_action: Option<ActionEvent> = None;

    if let Some(next_action) = pending_actions.0.pop_front() {
        taken_action = Some(ActionEvent(next_action));
    } else {
        match player_state.as_ref() {
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
                } else if keys.just_pressed(KeyCode::Comma) || keys.just_pressed(KeyCode::Space) {
                    let items = item_query
                        .iter()
                        .filter(|(e, w, i)| w.position == player_game_entity.position)
                        .collect::<Vec<_>>();

                    #[allow(clippy::comparison_chain)]
                    if items.len() > 1 {
                        println!("Choose what to pick up: ");
                        for (n, item) in items.iter().enumerate() {
                            println!("  {}: {}", n + 1, item.2.name);
                        }
                    } else if items.len() == 1 {
                        taken_action = Some(ActionEvent(a_pickup(
                            entity,
                            items.iter().map(|i| i.0).collect::<Vec<_>>(),
                        )));
                    } else {
                        println!("Nothing to pick up");
                    }
                } else if keys.just_pressed(KeyCode::T) {
                    println!("Choose thy thaum");
                    *player_state = PlayerState::Spell(CastSpellState::default());
                }
            }

            PlayerState::Dead => {
                taken_action = Some(ActionEvent(a_wait()));
                turn_order.pushback(100);
            }

            PlayerState::Spell(spell_state) => match spell_state {
                CastSpellState::ChooseSpell => {
                    if keys.just_pressed(KeyCode::Escape) {
                        println!("Casting cancelled");
                        *player_state = PlayerState::Idle;
                    } else if keys.just_pressed(KeyCode::Return) {
                        println!("ZAPPING FIREBALL");
                        *player_state = PlayerState::Idle;
                    }
                }

                CastSpellState::ChooseTarget => todo!(),
                CastSpellState::CastingTime => todo!(),
            },

            PlayerState::Throw(_) => todo!(),
        }
    }

    if let Some(action) = taken_action {
        let cost = character.calculate_cost(action.0.get_affiliated_stat());
        // let current_energy = turn_order
        //     .order
        //     .get_priority(&TurnOrderEntity { entity })
        //     .unwrap();
        // println!(
        //     "{:?} ({} energy) decides to do {:?} for {} energy",
        //     "Player", current_energy.0, action.0, cost
        // );
        turn_order.pushback(cost);
        actions.send(action);
    }
}

pub struct SvarogPlayerPlugin;
impl Plugin for SvarogPlayerPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_systems(
            Update,
            character_controls
                .before(TransformSystem::TransformPropagate)
                .before(CameraUpdateSystem)
                .run_if(in_state(GameStates::Game)),
        );
    }
}
