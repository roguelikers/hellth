use super::{
    character::CharacterStat,
    grid::{Grid, WorldEntityBundle, WorldEntityKind},
    sprites::Tile,
    ui::ShowEntityDetails,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
    PickableBundle,
};
use std::fmt::Debug;

#[derive(Component)]
pub struct Item {
    pub name: String,
    pub image: usize,
    pub item_type: ItemType,
    pub equip_stat_changes: Vec<(CharacterStat, i32)>,
}

impl Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut desc = vec![];
        for (stat, val) in &self.equip_stat_changes {
            if *val == 0 {
                continue;
            }
            let sign = if *val > 0 { "+" } else { "-" };
            desc.push(format!("{}{} {:?}", sign, val.abs(), stat));
        }

        f.write_fmt(format_args!(
            "{} -- {:?} [{}]",
            self.name,
            self.item_type,
            desc.join(", ")
        ))
    }
}

#[derive(Component, Default)]
pub struct EquippedItems(pub Vec<Entity>);

#[derive(Component, Default)]
pub struct CarriedItems(pub Vec<Entity>);

#[derive(Component)]
pub struct CarriedMarker;

#[derive(Default, Debug)]
pub enum ItemType {
    #[default]
    Unknown,
    Spell,
    Artifact,
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

    pub fn with_stats(mut self, stats: &[(CharacterStat, i32)]) -> Self {
        for (stat, val) in stats {
            self.stats.insert(*stat, *val);
        }

        self
    }

    pub fn with_image(mut self, t: Tile) -> Self {
        self.tile = t.into();
        self
    }

    pub fn create_at_raw(
        self,
        pos: IVec2,
        world: &mut World,
        transform: Transform,
        atlas: Handle<TextureAtlas>,
    ) {
        let mut item = world.spawn(WorldEntityBundle::new_raw(
            transform,
            atlas,
            &self.name,
            pos,
            self.tile,
            false,
            WorldEntityKind::Item,
        ));
        item.insert((
            Item {
                name: self.name,
                image: self.tile,
                item_type: self.item_type,
                equip_stat_changes: self.stats.into_iter().collect(),
            },
            PickableBundle::default(),
            On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
        ));
    }

    pub fn create_at(self, pos: IVec2, commands: &mut Commands, grid: &Res<Grid>) {
        let mut item = commands.spawn(WorldEntityBundle::new(
            grid,
            &self.name,
            pos,
            self.tile,
            false,
            WorldEntityKind::Item,
        ));

        item.insert((
            Item {
                name: self.name,
                image: self.tile,
                item_type: self.item_type,
                equip_stat_changes: self.stats.into_iter().collect(),
            },
            PickableBundle::default(),
            On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
        ));
    }
}

pub fn item() -> ItemBuilder {
    ItemBuilder::default()
}
