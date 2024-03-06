pub mod ai_think_action;
pub mod cast_spell_action;
pub mod death_action;
pub mod flee_action;
pub mod hit_action;
pub mod melee_attack_action;
pub mod move_action;
pub mod random_walk_action;
pub mod switch_behaviour_action;
pub mod track_action;
pub mod wait_action;

use std::collections::VecDeque;
use std::fmt::Debug;

pub use {
    ai_think_action::a_think, cast_spell_action::a_cast_spell, death_action::a_death,
    flee_action::a_flee, hit_action::a_hit, melee_attack_action::a_melee, move_action::a_move,
    random_walk_action::a_random_walk, switch_behaviour_action::a_behave, track_action::a_track,
    wait_action::a_wait,
};

use bevy::prelude::*;

pub type AbstractAction = Box<dyn Action>;

pub trait Action: Send + Sync + Debug {
    fn do_action(&self, world: &mut World) -> Vec<AbstractAction>;
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
        reactions.extend(ev.0.do_action(world));
        while !reactions.is_empty() {
            let more_reactions = reactions.pop_front().unwrap().do_action(world);
            reactions.extend(more_reactions);
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
