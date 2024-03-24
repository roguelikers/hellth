use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::{Character, CharacterStat}, grid::WorldEntity, health::{Health, HitPoint}, history::HistoryLog, inventory::Item, procgen::PlayerMarker
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
            ResMut<LevelDepth>,
            ResMut<HistoryLog>,
            ResMut<Random>,
        )>::new(world);

        let (mut player_query, item_query, mut depth, mut log, mut rng) =
            read_system_state.get_mut(world);
        let (mut char, mut health, _, carried, mut equipped) = player_query.single_mut();

        let (stat, val) = char.get_strongest_stat();
        if val < 9 {
            log.add(&format!("You feel {} health wither away and go to the Healer.", 9 - val));

            let dval = (9 - val) as usize;
            if dval >= health.size {
                health.size = 0;
                health.hitpoints.clear();
            } else if dval > 0 {
                depth.1 += dval as i32;
                let diff = health.normal_damage(dval);
                for (stat, val) in diff {
                    char[stat] += val;
                    {
                        let e = char.counters.entry(stat).or_insert(0);
                        *e += 1;
                    }
                }

                health.size -= dval - 1;
                let h = health.size;
                health.normal_heal(h);

                for hp in health.hitpoints.iter_mut() {
                    for (stat, val) in hp.enchant((stat, -1)) {
                        char[stat] += val;
                        {
                            let e = char.counters.entry(stat).or_insert(0);
                            *e += 1;
                        }
                    }
                }
            }
        } else {
            let h = health.size;
            health.normal_heal(h);

            let stat = char.get_weakest_stat().0;
            if let Some(hp) = health.hitpoints.iter_mut().next() {
                for (stat, val) in hp.enchant((stat, 1)) {
                    char[stat] += val;
                    {
                        let e = char.counters.entry(stat).or_insert(0);
                        *e += 1;
                    }
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
        log.add("");
        item_destruction
    }
}
