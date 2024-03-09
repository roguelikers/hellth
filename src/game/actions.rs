pub mod ai_think_action;
pub mod cast_spell_action;
pub mod consume_action;
pub mod death_action;
pub mod destroy_action;
pub mod drop_action;
pub mod equip_action;
pub mod flee_action;
pub mod hit_action;
pub mod leave_bones_action;
pub mod melee_attack_action;
pub mod move_action;
pub mod pickup_action;
pub mod random_walk_action;
pub mod switch_behaviour_action;
pub mod track_action;
pub mod unequip_action;
pub mod wait_action;

use std::collections::VecDeque;
use std::fmt::Debug;

use super::character::CharacterStat;

pub use {
    ai_think_action::a_think, cast_spell_action::a_cast_spell, consume_action::a_consume,
    death_action::a_death, destroy_action::a_destroy, drop_action::a_drop, equip_action::a_equip,
    flee_action::a_flee, hit_action::a_hit, leave_bones_action::a_leave_bones,
    melee_attack_action::a_melee, move_action::a_move, pickup_action::a_pickup,
    random_walk_action::a_random_walk, switch_behaviour_action::a_behave, track_action::a_track,
    unequip_action::a_unequip, wait_action::a_wait,
};

use bevy::prelude::*;

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

    // println!("HANDLE =====================================================");
    let mut reactions = VecDeque::new();
    for ev in events {
        {
            //println!("  {:?}", ev.0);
            reactions.extend(ev.0.do_action(world));
            while !reactions.is_empty() {
                let reaction = reactions.pop_front().unwrap();
                //println!("    {:?}", reaction);
                let more_reactions = reaction.do_action(world);
                reactions.extend(more_reactions);
            }
        }
    }
    // println!("DONE =====================================================");
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
