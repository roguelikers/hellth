use std::ops::Index;

use bevy::{
    app::{Plugin, Update},
    ecs::{
        component::Component,
        query::With,
        system::{Local, Query, ResMut, Resource},
    },
    render::color::Color,
    utils::hashbrown::HashMap,
};

use super::{
    character::{Character, CharacterStat},
    feel::Random,
    history::HistoryLog,
    procgen::PlayerMarker,
};

const STATS: [CharacterStat; 6] = [
    CharacterStat::STR,
    CharacterStat::ARC,
    CharacterStat::INT,
    CharacterStat::WIS,
    CharacterStat::WIL,
    CharacterStat::AGI,
];

#[derive(Component)]
pub struct Focus(pub u32);

pub type StatShorthand = String;

pub enum MagicAspect {
    Lore,       // summoning
    Edge,       // inflicts negatives around player
    Prison,     // inflicts burn around enemies
    Regalia,    // produces random artifacts around map -- all with magic aspects
    Dust,       // steals health from others
}

// pub const MAGIC_ASPECT_SONGS: [&str; 5] = [
//     "1/ We start through LORE, like stories of old,",
//     "2/    through hardships up the knife, to the EDGE",
//     "3/ to slice and fall, ourselves into PRISON cast",
//     "4/    our REGALIA taken and thrown to the wolves",
//     "5/ until we become DUST in someone else's cough.",
// ];

/* 
    1/ We start through LORE, like stories of old,
    2/    through hardships up the knife, to the EDGE
    3/ to slice and fall, ourselves into PRISON cast
    4/    our REGALIA taken and thrown to the wolves
    5/ until we become DUST in someone else's cough.
*/

#[derive(Resource, Default)]
pub struct Magic {
    pub color_bindings: HashMap<CharacterStat, Color>,
    pub aspects: HashMap<StatShorthand, MagicAspect>,
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
            ..Default::default()
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

fn knowledge_checker(
    mut player: Query<&mut Character, With<PlayerMarker>>,
    mut old: Local<Option<Character>>,
    mut log: ResMut<HistoryLog>,
) {
    let Ok(mut player) = player.get_single_mut() else {
        return;
    };

    if old.is_none() {
        *old = Some(player.clone());
    } else if let Some(old_state) = old.as_ref() {
        for stat in &player.learned {
            if !old_state.learned.contains(stat) {
                log.add(&format!("You learned the color of {}. Check in your stat bar to see which color it is.", format!("{:?}", stat).to_uppercase()));
            }
        }

        if (player.wisdom > 4 && player.arcana > 4)
            && (old_state.wisdom <= 4 || old_state.arcana <= 4)
        {
            log.add("Because of your higher wisdom and arcana, you can discern some enemy stats.");
        } else if (player.wisdom > 3 && player.arcana > 3)
            && (old_state.wisdom <= 3 || old_state.arcana <= 3)
        {
            player.learned.insert(CharacterStat::WIS);
            player.learned.insert(CharacterStat::ARC);
            log.add("Because of your high WIS and ARC score, you now see the color of the strongest stat's in items and enemies!");
        }

        if (old_state.wisdom > 3 && old_state.arcana > 3)
            && (player.wisdom <= 3 || player.arcana <= 3)
        {
            log.add("Your perception grows bleak again - you can no longer see stats as colors.");
        }

        *old = Some(player.clone());
    }
}

pub struct SvarogMagicPlugin;

impl Plugin for SvarogMagicPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.init_resource::<Magic>()
            .add_systems(Update, knowledge_checker);
    }
}
