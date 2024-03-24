use bevy::{ecs::component::Component, utils::HashMap};
use std::collections::VecDeque;

use super::character::CharacterStat;

#[derive(Component, Default)]
pub struct RecoveryCounter(pub u32);

#[derive(Default, Clone)]
pub struct HitPoint {
    pub stat: Option<(CharacterStat, i32)>,
}

impl HitPoint {
    pub fn enchant(&mut self, chant: (CharacterStat, i32)) -> HashMap<CharacterStat, i32> {
        let mut hash = HashMap::new();

        if let Some((old, val)) = self.stat {
            hash.insert(old, -val);
        }
        self.stat = Some(chant);
        if hash.contains_key(&chant.0) {
            hash.entry(chant.0).and_modify(|v| *v += chant.1);
        } else {
            hash.insert(chant.0, chant.1);
        }
        hash
    }
}

#[derive(Component, Clone)]
pub struct Health {
    pub size: usize,
    pub hitpoints: VecDeque<HitPoint>,
}

impl Health {
    pub fn new(size: usize) -> Health {
        Health {
            size,
            hitpoints: VecDeque::from_iter(vec![HitPoint::default(); size]),
        }
    }

    pub fn normal_damage(&mut self, n: usize) -> HashMap<CharacterStat, i32> {
        let mut result = HashMap::new();
        for _ in 0..n {
            if let Some(rightmost) = self.hitpoints.pop_back() {
                if let Some((stat, val)) = rightmost.stat {
                    if result.contains_key(&stat) {
                        *result.get_mut(&stat).unwrap() -= val;
                    } else {
                        result.insert(stat, -val);
                    }
                }
            }
        }

        result
    }

    pub fn normal_heal(&mut self, n: usize) {
        for _ in 0..n {
            if self.hitpoints.len() < self.size {
                self.hitpoints.push_front(HitPoint::default());
            }
        }
    }

    pub fn enchanted_heal(&mut self, n: usize, typ: CharacterStat) {
        for _ in 0..n {
            if self.hitpoints.len() < self.size {
                self.hitpoints.push_front(HitPoint { stat: Some((typ, -1)) });
            }
        }
    }
}
