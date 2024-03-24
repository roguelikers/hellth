use crate::game::{
    actions::*,
    character::CharacterStat,
    feel::Random,
    inventory::{ItemBuilder, ItemType},
    sprites::BONES,
};
use bevy::prelude::*;

use self::inflict_action::a_inflict;

use super::{get_player, get_positions_and_health, get_the_healer, AIBehaviour, AbstractAIBehaviour};

#[derive(Debug)]
pub struct CasterThinking;

pub fn ai_caster() -> AbstractAIBehaviour {
    Box::new(CasterThinking)
}

impl AIBehaviour for CasterThinking {
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

        let distance = (player_pos.distance_squared(enemy_pos) as f32).sqrt();

        if enemy_hp.hitpoints.len() + 1 >= player_hp.hitpoints.len() && distance < 1.45 {
            return vec![a_track(entity, player)];
        }
        
        if let Some(healer) = get_the_healer(world) {
            if let Some((healer_pos, _healer_hp)) = stats.get(&healer).cloned().unwrap_or_default() {
                let distance_to_healer = (healer_pos.distance_squared(enemy_pos) as f32).sqrt();
                if distance_to_healer < 6.0 {
                    a_heal(entity).do_action(world);
                    a_heal(healer).do_action(world);
                }
            }
        }
        if distance > 10.0 {
            vec![a_random_walk(entity), a_track(entity, player), a_focus(entity)]
        } else {
            let mut rng = world.get_resource_mut::<Random>().unwrap();

            let mut actions = vec![];

            for _ in 0..rng.gen(1..3) {
                let artifact = {
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
                    for _ in 0..rng.gen(2..5) {
                        let stat = rng.from(&stats);
                        let power = rng.gen(-3..-1);
                        builder = builder.with_stat(stat, power);
                    }
    
                    builder.to_item()
                };
                for _ in 0..rng.gen(2..4) {
                    if rng.coin() {
                        actions.push(a_track(entity, player));    
                    } else {
                        actions.push(a_random_walk(entity));
                    }
                }
                let distance = (player_pos.distance_squared(enemy_pos) as f32).sqrt();
                if distance <= 7.0 {
                    actions.push(a_inflict(entity, player, artifact));
                }
                for _ in 0..rng.gen(2..4) {
                    if rng.coin() {
                        actions.push(a_track(entity, player));    
                    } else {
                        actions.push(a_random_walk(entity));
                    }
                }
            }
            actions        
        }
    }
}
