use bevy::{prelude::*, utils::HashMap};

use super::{
    character::CharacterStat,
    grid::{Grid, WorldEntityBundle},
    health::Health,
    sprites::Tile,
};

#[derive(Component)]
pub struct Item {
    pub name: String,
    pub image: usize,
    pub item_type: ItemType,
    pub equip_stat_changes: Vec<(CharacterStat, i32)>,
}

#[derive(Component, Default)]
pub struct EquippedItems(pub Vec<Entity>);

#[derive(Default)]
pub enum ItemType {
    #[default]
    Unknown,
    Spell,
    Weapon,
    Armor,
    Food,
    Potion,
}

#[derive(Default)]
pub struct ItemBuilder {
    name: String,
    item_type: ItemType,
    tile: usize,
    stats: HashMap<CharacterStat, i32>,
}

impl ItemBuilder {
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_type(mut self, t: ItemType) -> Self {
        self.item_type = t;
        self
    }

    pub fn with_stat(mut self, s: CharacterStat, i: i32) -> Self {
        self.stats.insert(s, i);
        self
    }

    pub fn with_image(mut self, t: Tile) -> Self {
        self.tile = t.into();
        self
    }

    pub fn create_at(self, pos: IVec2, commands: &mut Commands, grid: &Res<Grid>) {
        let mut player = commands.spawn(WorldEntityBundle::new(
            grid, &self.name, pos, self.tile, false, false,
        ));
        player.insert((
            Item {
                name: self.name,
                image: self.tile,
                item_type: self.item_type,
                equip_stat_changes: self.stats.into_iter().collect(),
            },
            Health::new(1),
        ));
    }
}

pub fn item() -> ItemBuilder {
    ItemBuilder::default()
}
