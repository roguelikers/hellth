use bevy::{ecs::system::SystemState, prelude::*};

use crate::game::{
    character::{Character, CharacterStat},
    grid::{Grid, WorldData, WorldEntity},
    health::Health,
    history::HistoryLog,
    inventory::{Item, ItemType},
};

use super::{a_death, AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct BreakAction {
    pub what: Entity,
}

pub fn a_break(what: Entity) -> AbstractAction {
    println!("BREAK {:?}", what);
    Box::new(BreakAction { what })
}

impl Action for BreakAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::WIL
    }

    fn do_action(&self, world: &mut World) -> ActionResult {
        let mut read_system_state = SystemState::<(
            Query<(&mut WorldEntity, &mut Transform, Option<&mut Character>)>,
            Query<&mut Health>,
            Query<(&Item, &mut Visibility)>,
            ResMut<HistoryLog>,
            Res<Grid>,
            Res<WorldData>,
        )>::new(world);

        let (mut transforms, mut healths, mut items, mut log, grid, world_data) =
            read_system_state.get_mut(world);

        let Ok((item, vis)) = items.get_mut(self.what) else {
            log.add("ERR: No item found.");
            return vec![];
        };

        let Ok((item_world_entity, mut transform, _)) = transforms.get_mut(self.what) else {
            log.add("ERR: No world item found.");
            return vec![];
        };

        let mut result = vec![];

        let breaks = match item.item_type {
            ItemType::Artifact => {
                log.add("The thrown artifact breaks on impact!");
                if let Some(e) = world_data.blocking.get(&item_world_entity.position) {
                    if let Ok((hit_entity, _, Some(mut hit_char))) = transforms.get_mut(*e) {
                        let mut message: Vec<String> =
                            vec![format!("The broken artifact affects {}.", hit_entity.name)];

                        if let Ok(mut health) = healths.get_mut(*e) {
                            for (pos, (stat, val)) in item.equip_stat_changes.iter().enumerate() {
                                if let Some(hp) = health.hitpoints.get_mut(pos) {
                                    for (effect, val) in hp.enchant((*stat, *val)) {
                                        hit_char[effect] += val;

                                        {
                                            let e = hit_char.counters.entry(effect).or_insert(0);
                                            *e += 1;
                                        }

                                        message.push(format!(
                                            "{} has its {} {} by {}.",
                                            hit_entity.name,
                                            format!("{:?}", effect).to_uppercase(),
                                            if val > 0 { "raised" } else { "lowered" },
                                            val.abs()
                                        ));
                                    }
                                }
                            }
                        }
                        log.add(&message.join(" "));
                    }
                } else {
                    log.add(&format!("Err: NO HIT at {:?}!", item_world_entity.position));
                }
                true
            }

            ItemType::Weapon => {
                //
                if let Some(e) = world_data.blocking.get(&item_world_entity.position) {
                    let Ok((hit_entity, _, Some(mut hit_char))) = transforms.get_mut(*e) else {
                        return vec![];
                    };

                    log.add(&format!(
                        "The {} hits {} for {} damage.",
                        format!("{:?}", item.item_type).to_lowercase(),
                        hit_entity.name,
                        1 + item.equip_stat_changes.len()
                    ));

                    if let Ok(mut health) = healths.get_mut(*e) {
                        let diff = health.normal_damage(1 + item.equip_stat_changes.len());
                        for (stat, val) in diff {
                            hit_char[stat] += val;
                            {
                                let e = hit_char.counters.entry(stat).or_insert(0);
                                *e += 1;
                            }
                        }

                        if health.hitpoints.is_empty() {
                            result.push(a_death(*e));
                        }
                    }
                }
                true
            }

            _ => false,
        };

        if breaks {
            let mut to_remove = vec![self.what];
            if let Some(ch) = world.get::<Children>(self.what) {
                for c in ch.iter() {
                    to_remove.push(*c);
                }
            }

            for rem in to_remove {
                world.despawn(rem);
            }
        }

        result
    }
}
