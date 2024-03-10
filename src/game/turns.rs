use bevy::prelude::*;
use bevy_mod_imgui::ImguiContext;
use priority_queue::PriorityQueue;

use super::{
    ai::{AIAgent, PendingActions},
    character::Character,
    feel::Random,
    grid::WorldEntity,
    health::{Health, HitPoint, RecoveryCounter},
    history::HistoryLog,
    DebugFlag,
};

#[derive(Component)]
pub struct TurnTaker;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct TurnOrderEntity {
    pub entity: Entity,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Energy(pub i32);

#[derive(Resource, Default)]
pub struct TurnOrder {
    pub order: PriorityQueue<TurnOrderEntity, Energy>,
    pub should_ping: Option<Entity>,
}

#[derive(Event)]
pub struct TurnOrderProgressEvent;

#[derive(Event)]
pub struct StartTurnEvent(pub Entity);

#[derive(Event)]
pub struct EndTurnEvent;

#[derive(Resource, Default)]
pub struct TurnCounter(pub u32);

impl TurnOrder {
    pub fn clear(&mut self) {
        self.order.clear();
    }

    pub fn peek(&self) -> Option<Entity> {
        self.order.peek().map(|(a, _)| a.entity)
    }

    pub fn pushback(&mut self, spend: i32) {
        if let Some((top, energy)) = self.order.pop() {
            self.order.push_decrease(top, Energy(energy.0 - spend));
            self.should_ping = Some(self.order.peek().unwrap().0.entity);
        }
    }

    pub fn is_turn_done(&self) -> bool {
        self.order
            .peek()
            .map(|(_, Energy(energy))| *energy <= 0)
            .unwrap_or(true)
    }

    pub fn restart_turn(&mut self) {
        let entities = self
            .order
            .iter()
            .map(|(a, b)| (*a, b.0))
            .collect::<Vec<_>>();
        for (entity, energy) in entities {
            self.order
                .change_priority(&entity, Energy((energy + 100).min(100)));
        }
    }
}

pub fn add_entity_to_turn_queue(
    turn_takers: Query<Entity, Added<TurnTaker>>,
    mut turn_order: ResMut<TurnOrder>,
) {
    for entity in &turn_takers {
        turn_order.order.push(TurnOrderEntity { entity }, Energy(0));
    }
}

pub fn turn_order_progress(
    mut turn_order: ResMut<TurnOrder>,
    mut start_turn_events: EventWriter<StartTurnEvent>,
    mut end_turn_events: EventWriter<EndTurnEvent>,
) {
    if turn_order.peek().is_some() && turn_order.is_turn_done() {
        turn_order.restart_turn();
        end_turn_events.send(EndTurnEvent);
    } else if let Some(ping_target) = turn_order.should_ping {
        start_turn_events.send(StartTurnEvent(ping_target));
        turn_order.should_ping = None;
    }
}

fn get_recovery_based_on_str(str: i32) -> u32 {
    match str {
        i32::MIN..=0_i32 => 24,
        1 => 23,
        2 => 22,
        3 => 21,
        4 => 20,
        5 => 19,
        6 => 18,
        7 => 17,
        8 => 16,
        9 => 15,
        10_i32..=i32::MAX => 14,
    }
}

fn on_turn_end(
    mut end_turn: EventReader<EndTurnEvent>,
    mut turn_counter: ResMut<TurnCounter>,
    mut health: Query<(
        &mut Character,
        &mut Health,
        &mut RecoveryCounter,
        &WorldEntity,
    )>,
    mut log: ResMut<HistoryLog>,
    mut rng: ResMut<Random>,
) {
    for _ in end_turn.read() {
        for (mut char, mut health, mut recovery, world_entity) in &mut health {
            if world_entity.is_player && !health.hitpoints.is_empty() {
                turn_counter.0 += 1;
            }

            let turns_needed = get_recovery_based_on_str(char.strength);
            recovery.0 += 1;
            if turns_needed <= recovery.0 {
                recovery.0 = 0;
                if let Some(rightmost) = health.hitpoints.pop_back() {
                    health.hitpoints.push_front(HitPoint::default());
                    if let Some((stat, val)) = rightmost.stat {
                        char[stat] -= val;
                        log.add(&format!(
                            "You can feel your body force thaum out. {} {} is dispelled.",
                            val,
                            format!("{:?}", stat).to_uppercase(),
                        ));
                    }

                    if char.strength >= 8 && health.hitpoints.len() < health.size && rng.coin() {
                        log.add("You can feel your wounds healing.");
                        health.normal_heal(1);
                    }
                }
            }
        }
    }
}

fn debug_turn_order(
    mut context: NonSendMut<ImguiContext>,
    turn_order: Res<TurnOrder>,
    living: Query<(&WorldEntity, Option<&AIAgent>, Option<&PendingActions>)>,
    debug: Res<DebugFlag>,
) {
    if !debug.0 {
        return;
    }

    let ui = context.ui();
    let window = ui.window("Turn Order");

    window
        .size([100.0, 300.0], imgui::Condition::FirstUseEver)
        .save_settings(true)
        .build(|| {
            for (turn_taker, energy) in &turn_order.order {
                let Ok((entity, agent, plan)) = living.get(turn_taker.entity) else {
                    continue;
                };

                let behaviour_name = if let Some(AIAgent(behaviour)) = agent {
                    format!("[{:?}]", behaviour)
                } else {
                    "".to_string()
                };

                let plan = if let Some(PendingActions(plan)) = plan {
                    format!("({:?})", plan)
                } else {
                    "".to_string()
                };

                ui.button(format!(
                    "{} ({}) {}{}",
                    entity.name, energy.0, behaviour_name, plan
                ));
            }
        });
}

fn debug_all_entities(
    mut context: NonSendMut<ImguiContext>,
    mut entities: Query<(&WorldEntity, &mut Transform)>,
    debug: Res<DebugFlag>,
) {
    if !debug.0 {
        return;
    }

    let ui = context.ui();
    let window = ui.window("All Entities");

    window
        .size([100.0, 300.0], imgui::Condition::FirstUseEver)
        .save_settings(true)
        .build(|| {
            for (entity, mut transform) in &mut entities {
                if ui.button(format!("{} at {:?}", &entity.name, transform)) {
                    transform.translation.z += 1.0;
                }
            }
        });
}

pub struct SvarogTurnPlugin;
impl Plugin for SvarogTurnPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_event::<TurnOrderProgressEvent>()
            .add_event::<StartTurnEvent>()
            .init_resource::<TurnCounter>()
            .add_event::<EndTurnEvent>()
            .insert_resource(TurnOrder::default())
            .add_systems(
                Update,
                (add_entity_to_turn_queue, turn_order_progress).chain(),
            )
            .add_systems(Update, on_turn_end.run_if(on_event::<EndTurnEvent>()))
            .add_systems(Update, (debug_turn_order, debug_all_entities));
    }
}
