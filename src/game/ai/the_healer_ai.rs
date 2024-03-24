use crate::game::{actions::*, character::CharacterStat, feel::Random, history::HistoryLog, inventory::{ItemBuilder, ItemType}, sprites::BONES};
use bevy::prelude::*;

use super::{get_mobs, get_player, get_positions_and_health, AIBehaviour, AbstractAIBehaviour};

#[derive(Debug)]
pub struct TheHealerAIThinking;

pub fn ai_the_healer() -> AbstractAIBehaviour {
    Box::new(TheHealerAIThinking)
}

impl AIBehaviour for TheHealerAIThinking {
    fn do_thinking(&self, entity: Entity, world: &mut World) -> Vec<AbstractAction> {
        let player = {
            let Some(player) = get_player(world) else {
                return vec![a_random_walk(entity)];
            };

            player
        };

        let mut stats = { get_positions_and_health(world, &[entity, player]) };

        let Some((player_pos, player_hp)) = stats.get(&player).cloned().unwrap_or_default() else {
            return vec![a_random_walk(entity)];
        };

        let Some((enemy_pos, mut enemy_hp)) = stats.get_mut(&entity).cloned().unwrap_or_default() else {
            return vec![a_random_walk(entity)];
        };

        a_heal(entity).do_action(world);

        if enemy_hp.hitpoints.len() < enemy_hp.size / 4 * 3 {
            let artifact1 = {
                let mut builder = ItemBuilder::default()
                    .with_name("IMAGINARY ITEM")
                    .with_image(BONES)
                    .with_type(ItemType::Artifact);

                let stats = [
                    CharacterStat::STR,
                    CharacterStat::ARC,
                    CharacterStat::INT,
                    CharacterStat::WIS,
                    CharacterStat::WIL,
                    CharacterStat::AGI,
                ];
                let mut rng = world.get_resource_mut::<Random>().unwrap();
                for _ in 0..rng.gen(2..5) {
                    let stat = rng.from(&stats);
                    let power = rng.gen(-5..-3);
                    builder = builder.with_stat(stat, power);
                }

                builder.to_item()
            };

            let artifact2 = {
                let mut builder = ItemBuilder::default()
                    .with_name("IMAGINARY ITEM")
                    .with_image(BONES)
                    .with_type(ItemType::Artifact);

                let stats = [
                    CharacterStat::STR,
                    CharacterStat::ARC,
                    CharacterStat::INT,
                    CharacterStat::WIS,
                    CharacterStat::WIL,
                    CharacterStat::AGI,
                ];
                let mut rng = world.get_resource_mut::<Random>().unwrap();
                for _ in 0..rng.gen(2..5) {
                    let stat = rng.from(&stats);
                    let power = rng.gen(-5..-3);
                    builder = builder.with_stat(stat, power);
                }

                builder.to_item()
            };

            vec![ 
                a_yell(entity), 
                a_inflict(entity, player, artifact1),
                a_track(entity, player),
                a_inflict(entity, player, artifact2),
            ]
        } else if enemy_hp.hitpoints.len() <= enemy_hp.size / 2 {
            let mut results = vec![];
            if let Some(mobs) = get_mobs(world) {
                
                let sacrifice = { 
                    let mut rng = world.get_resource_mut::<Random>().unwrap();                    
                    mobs[rng.gen(0..mobs.len() as i32) as usize].clone()
                };
                
                if let Some(mut log) = world.get_resource_mut::<HistoryLog>() {
                    log.add(&format!("The healer glances at {}. Their skin starts to pale and wrinkle as they fall limp to the ground. The healer looks more powerful.", sacrifice.1.name));
                }

                for _ in 0..10 {
                    a_heal(entity).do_action(world);
                }
                a_destroy(sacrifice.0).do_action(world);
                results.extend(vec![a_yell(entity), a_wait()]);
            } else {
                results.extend(vec![
                    a_flee(entity, player),
                    a_flee(entity, player),
                    a_wait(),
                    a_wait(),
                    a_track(entity, player),
                    a_track(entity, player),
                    a_wait(),
                ]);
            }

            results
        } else {
            vec![
                a_track(entity, player),
                a_track(entity, player),
                a_wait(),
                a_track(entity, player),
            ]
        }
    }
}
