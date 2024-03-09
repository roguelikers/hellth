use bevy::prelude::*;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

use super::feel::Random;
use super::magic::Magic;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CharacterStat {
    STR,
    ARC,
    INT,
    WIS,
    WIL,
    AGI,
}

#[derive(Component)]
pub struct Character {
    pub strength: i32,
    pub arcana: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub willpower: i32,
    pub agility: i32,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            strength: 3,
            arcana: 3,
            intelligence: 3,
            wisdom: 3,
            willpower: 3,
            agility: 3,
        }
    }
}

impl Debug for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "STR[{}] ARC[{}] INT[{}] WIS[{}] WIL[{}] AGI[{}]",
            self.strength,
            self.arcana,
            self.intelligence,
            self.wisdom,
            self.willpower,
            self.agility
        ))
    }
}

impl IndexMut<CharacterStat> for Character {
    fn index_mut(&mut self, index: CharacterStat) -> &mut Self::Output {
        match index {
            CharacterStat::STR => &mut self.strength,
            CharacterStat::ARC => &mut self.arcana,
            CharacterStat::INT => &mut self.intelligence,
            CharacterStat::WIS => &mut self.wisdom,
            CharacterStat::WIL => &mut self.willpower,
            CharacterStat::AGI => &mut self.agility,
        }
    }
}

impl Index<CharacterStat> for Character {
    type Output = i32;

    fn index(&self, index: CharacterStat) -> &Self::Output {
        match index {
            CharacterStat::STR => &self.strength,
            CharacterStat::ARC => &self.arcana,
            CharacterStat::INT => &self.intelligence,
            CharacterStat::WIS => &self.wisdom,
            CharacterStat::WIL => &self.willpower,
            CharacterStat::AGI => &self.agility,
        }
    }
}

impl Character {
    pub fn random(rng: &mut ResMut<Random>) -> Self {
        let vals = [
            [3, 3, 3, 3, 3, 4],
            [3, 3, 3, 3, 3, 5],
            [2, 2, 3, 3, 3, 3],
            [1, 3, 3, 3, 3, 7],
            [3, 3, 3, 4, 5, 2],
            [3, 3, 5, 2, 2, 5],
            [6, 4, 2, 2, 3, 1],
            [1, 2, 3, 3, 4, 5],
        ];

        let vals = rng.from(&vals);
        let vals = rng.shuffle(Vec::from(vals));

        Self {
            strength: vals[0],
            arcana: vals[1],
            intelligence: vals[2],
            wisdom: vals[3],
            willpower: vals[4],
            agility: vals[5],
        }
    }

    pub fn calculate_cost(&self, stat: CharacterStat) -> i32 {
        match self[stat] {
            i32::MIN..=0_i32 => 200,
            1 => 150,
            2 => 125,
            3 => 100,
            4 => 75,
            5 => 60,
            6 => 50,
            7 => 40,
            8 => 30,
            9 => 25,
            10_i32..=i32::MAX => 20,
        }
    }

    pub fn get_strongest_stat(&self) -> (CharacterStat, i32) {
        let mut max = self[CharacterStat::STR];
        let mut strongest = CharacterStat::STR;

        for stat in [
            CharacterStat::ARC,
            CharacterStat::INT,
            CharacterStat::WIS,
            CharacterStat::WIL,
            CharacterStat::AGI,
        ] {
            if self[stat] > max {
                max = self[stat];
                strongest = stat;
            }
        }

        (strongest, max)
    }

    pub fn get_strongest_stat_color(&self, magic: &ResMut<Magic>) -> Color {
        let (stat, val) = self.get_strongest_stat();
        if val == 3 {
            Color::WHITE
        } else {
            magic[stat]
        }
    }

    pub fn get_weakest_stat(&self) -> (CharacterStat, i32) {
        let mut min = self[CharacterStat::STR];
        let mut weakest = CharacterStat::STR;

        for stat in [
            CharacterStat::ARC,
            CharacterStat::INT,
            CharacterStat::WIS,
            CharacterStat::WIL,
            CharacterStat::AGI,
        ] {
            if self[stat] < min {
                min = self[stat];
                weakest = stat;
            }
        }

        (weakest, min)
    }
}
