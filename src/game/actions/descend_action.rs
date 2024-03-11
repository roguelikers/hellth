use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::{Character, CharacterStat},
    grid::WorldEntity,
    health::{Health, HitPoint},
    history::HistoryLog,
    inventory::Item,
    procgen::PlayerMarker,
};

use super::{AbstractAction, Action, ActionResult};
use crate::game::actions::a_destroy;
use crate::game::feel::Random;
use crate::game::inventory::CarriedItems;
use crate::game::inventory::EquippedItems;
use crate::game::procgen::LevelDepth;
#[derive(Debug)]
pub struct DescendAction;

pub fn a_descend() -> AbstractAction {
    Box::new(DescendAction)
}

impl Action for DescendAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            Query<
                (
                    &mut Character,
                    &mut Health,
                    &mut WorldEntity,
                    &mut CarriedItems,
                    &mut EquippedItems,
                ),
                With<PlayerMarker>,
            >,
            Query<&Item>,
            ResMut<HistoryLog>,
            Res<LevelDepth>,
            ResMut<Random>,
        )>::new(world);

        let (mut player_query, item_query, mut log, depth, mut rng) =
            read_system_state.get_mut(world);
        let (mut char, mut health, _, carried, mut equipped) = player_query.single_mut();

        let (stat, val) = char.get_strongest_stat();
        if val < 9 {
            log.add("Thine wishes were higher than thine skills - this sacrifice is untowards.");

            let dval = (9 - val) as usize;
            if dval >= health.size {
                health.size = 0;
                health.hitpoints.clear();
            } else {
                health.size -= dval;
                let mut shadow = false;
                for _ in 0..dval {
                    if let Some(hp) = health.hitpoints.pop_back() {
                        shadow |= hp.stat.is_some();
                    }
                }

                for _ in 0..(health.size - health.hitpoints.len()) {
                    health.hitpoints.push_front(HitPoint::default());
                }

                if shadow {
                    log.add(&format!("You lose {} health points, yet a shadow lingers behind them in your bones...", 9 - val));
                } else {
                    log.add(&format!("You lose {} health points.", 9 - val));
                }
            }
        }

        for hp in &mut health.hitpoints {
            for (effect, val) in hp.enchant((stat, -1)) {
                char[stat] += val;

                {
                    let e = char.counters.entry(effect).or_insert(0);
                    *e += 1;
                }
            }
        }

        let mut item_destruction = vec![];

        let mut items_lost = (0..carried.0.len()).collect::<Vec<_>>();
        items_lost = rng.shuffle(items_lost);
        items_lost.truncate(depth.0 as usize);
        for i in items_lost {
            if let Some(item_found) = carried.0.get(i) {
                if let Some(pos) = equipped.0.iter().position(|it| it == item_found) {
                    if let Ok(item) = item_query.get(*item_found) {
                        equipped.0.remove(pos);

                        for (stat, val) in &item.equip_stat_changes {
                            char[*stat] -= *val;
                        }
                    }
                }
                item_destruction.push(a_destroy(*item_found));
            }
        }
        item_destruction
    }
}
