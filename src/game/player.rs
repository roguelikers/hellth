use bevy::{prelude::*, render::camera::CameraUpdateSystem, transform::TransformSystem};

use super::{
    actions::{move_action::MoveAction, wait_action::WaitAction, ActionEvent},
    grid::{WorldData, WorldEntity},
    health::Health,
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
    if keys.just_pressed(KeyCode::Up) {
        Some(IVec2::new(0, 1))
    } else if keys.just_pressed(KeyCode::Down) {
        Some(IVec2::new(0, -1))
    } else if keys.just_pressed(KeyCode::Left) {
        Some(IVec2::new(-1, 0))
    } else if keys.just_pressed(KeyCode::Right) {
        Some(IVec2::new(1, 0))
    } else {
        None
    }
}

pub fn character_controls(
    mut turn_order: ResMut<TurnOrder>,
    map: Res<WorldData>,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<(Entity, &WorldEntity, &Health, &mut PlayerState), With<PlayerMarker>>,
    mut actions: EventWriter<ActionEvent>,
) {
    if let Some(e) = turn_order.peek() {
        if !player_query.contains(e) {
            return;
        }
    }

    let Ok((entity, player_game_entity, health, mut player_state)) = player_query.get_single_mut()
    else {
        return;
    };

    match player_state.as_ref() {
        PlayerState::Idle => {
            if health.hitpoints.is_empty() {
                actions.send(ActionEvent(Box::new(WaitAction)));
                turn_order.pushback(100);
                *player_state = PlayerState::Dead;
                return;
            }

            let maybe_move = try_direction_keys(&keys);
            if let Some(direction) = maybe_move {
                if !map
                    .solid
                    .contains(&(player_game_entity.position + direction))
                {
                    actions.send(ActionEvent(Box::new(MoveAction { entity, direction })));
                    turn_order.pushback(100);
                }
            } else if keys.just_pressed(KeyCode::C) {
                println!("Choose your spell");
                *player_state = PlayerState::Spell(CastSpellState::default());
            }
        }

        PlayerState::Dead => {
            actions.send(ActionEvent(Box::new(WaitAction)));
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
