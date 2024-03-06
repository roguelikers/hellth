pub mod death_action;
pub mod hit_action;
pub mod melee_attack_action;
pub mod move_action;
pub mod wait_action;

use std::collections::VecDeque;

use bevy::prelude::*;

pub trait Action: Send + Sync {
    fn do_action(&self, world: &mut World) -> Vec<Box<dyn Action>>;
}

#[derive(Event)]
pub struct ActionEvent(pub Box<dyn Action>);

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
