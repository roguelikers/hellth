use std::ops::Index;

use bevy::{
    app::Plugin,
    ecs::system::{ResMut, Resource},
    render::color::Color,
    utils::hashbrown::HashMap,
};

use super::{character::CharacterStat, feel::Random};

const STATS: [CharacterStat; 6] = [
    CharacterStat::STR,
    CharacterStat::ARC,
    CharacterStat::INT,
    CharacterStat::WIS,
    CharacterStat::WIL,
    CharacterStat::AGI,
];

#[derive(Resource, Default)]
pub struct Magic {
    pub color_bindings: HashMap<CharacterStat, Color>,
}

impl Magic {
    pub fn new(rng: &mut ResMut<Random>) -> Self {
        let colors = [
            Color::rgb_u8(221, 0, 120),
            Color::rgb_u8(0, 137, 78),
            Color::rgb_u8(0, 132, 172),
            Color::rgb_u8(144, 60, 255),
            Color::rgb_u8(147, 122, 0),
            Color::rgb_u8(194, 82, 0),
        ];

        let colors = rng.shuffle(Vec::from_iter(colors));
        let stats = rng.shuffle(Vec::from_iter(STATS));

        let mapping: Vec<(CharacterStat, Color)> =
            stats.iter().zip(colors).map(|(c, s)| (*c, s)).collect();

        Self {
            color_bindings: HashMap::from_iter(mapping),
        }
    }

    pub fn reset(&mut self, rng: &mut ResMut<Random>) {
        *self = Self::new(rng);
    }
}

impl Index<CharacterStat> for Magic {
    type Output = Color;

    fn index(&self, index: CharacterStat) -> &Self::Output {
        self.color_bindings
            .get(&index)
            .expect("Expecting all stats to have colors")
    }
}

pub struct SvarogMagicPlugin;

impl Plugin for SvarogMagicPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.init_resource::<Magic>();
    }
}
