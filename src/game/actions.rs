pub mod ai_think_action;
pub mod break_action;
pub mod consume_action;
pub mod death_action;
pub mod descend_action;
pub mod destroy_action;
pub mod drop_action;
pub mod equip_action;
pub mod flee_action;
pub mod fly_action;
pub mod focus_action;
pub mod hit_action;
pub mod inflict_action;
pub mod leave_bones_action;
pub mod melee_attack_action;
pub mod move_action;
pub mod pickup_action;
pub mod random_walk_action;
pub mod switch_behaviour_action;
pub mod throw_action;
pub mod track_action;
pub mod unequip_action;
pub mod wait_action;
pub mod yell_action;
pub mod fortune_action;
pub mod heal_action;

use std::collections::VecDeque;
use std::fmt::Debug;

use super::{character::CharacterStat, music::GameAudioSettings};

pub use {
    ai_think_action::a_think, break_action::a_break, 
    consume_action::a_consume, death_action::a_death, descend_action::a_descend,
    destroy_action::a_destroy, drop_action::a_drop, equip_action::a_equip, flee_action::a_flee,
    fly_action::a_fly, focus_action::a_focus, hit_action::a_hit, inflict_action::a_inflict,
    leave_bones_action::a_leave_bones, melee_attack_action::a_melee, move_action::a_move,
    pickup_action::a_pickup, random_walk_action::a_random_walk, switch_behaviour_action::a_behave,
    throw_action::a_throw, track_action::a_track, unequip_action::a_unequip, wait_action::a_wait,
    yell_action::a_yell, fortune_action::a_fortune, heal_action::a_heal
};

use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};

pub type AbstractAction = Box<dyn Action>;

pub type ActionResult = Vec<AbstractAction>;

pub trait Action: Send + Sync + Debug {
    fn get_affiliated_stat(&self) -> CharacterStat;
    fn do_action(&self, world: &mut World) -> ActionResult;
}

#[derive(Event)]
pub struct ActionEvent(pub AbstractAction);

fn handle_gameplay_action(world: &mut World) {
    let events = if let Some(mut res) = world.get_resource_mut::<Events<ActionEvent>>() {
        res.drain().collect::<Vec<_>>()
    } else {
        return;
    };

    let mut reactions = VecDeque::new();
    for ev in events {
        {
            reactions.extend(ev.0.do_action(world));
            while !reactions.is_empty() {
                let reaction = reactions.pop_front().unwrap();
                let more_reactions = reaction.do_action(world);
                reactions.extend(more_reactions);
            }
        }
    }
}

pub fn play_sfx(name: &str, world: &mut World) {
    if let Some(settings) = world.get_resource::<GameAudioSettings>() {
        if let Some(asset_server) = world.get_resource::<AssetServer>() {
            if let Some(audio) = world.get_resource::<Audio>() {
                audio.play(asset_server.load(format!("sounds/{}.ogg", name))).with_volume(settings.sfx_volume);
            }
        }
    }
}

pub struct SvarogActionsPlugin;
impl Plugin for SvarogActionsPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_event::<ActionEvent>();
        bevy.add_systems(
            Update,
            handle_gameplay_action.run_if(on_event::<ActionEvent>()),
        );
    }
}
