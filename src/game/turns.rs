use bevy::prelude::*;
use bevy_mod_imgui::ImguiContext;
use priority_queue::PriorityQueue;

use super::grid::GameEntity;

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
) {
    if turn_order.peek().is_some() && turn_order.is_turn_done() {
        turn_order.restart_turn();
    } else if let Some(ping_target) = turn_order.should_ping {
        start_turn_events.send(StartTurnEvent(ping_target));
        turn_order.should_ping = None;
    }
}

fn debug_turn_order(
    mut context: NonSendMut<ImguiContext>,
    turn_order: Res<TurnOrder>,
    entities: Query<&GameEntity>,
) {
    let ui = context.ui();
    let window = ui.window("Turn Order");

    window
        .size([100.0, 300.0], imgui::Condition::FirstUseEver)
        .save_settings(true)
        .build(|| {
            for (turn_taker, energy) in &turn_order.order {
                ui.button(format!(
                    "{} ({})",
                    entities.get(turn_taker.entity).unwrap().name,
                    energy.0
                ));
            }
        });
}

pub struct SvarogTurnPlugin;
impl Plugin for SvarogTurnPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_event::<TurnOrderProgressEvent>()
            .add_event::<StartTurnEvent>()
            .insert_resource(TurnOrder::default())
            .add_systems(
                Update,
                (add_entity_to_turn_queue, turn_order_progress).chain(),
            )
            .add_systems(Update, debug_turn_order);
    }
}
