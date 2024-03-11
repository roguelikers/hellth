use super::{
    character::CharacterStat,
    grid::{Grid, WorldEntityBundle, WorldEntityKind},
    magic::Magic,
    sprites::Tile,
    ui::ShowEntityDetails,
};
use bevy::{prelude::*, render::view::RenderLayers, utils::HashMap};
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

#[derive(PartialEq, Eq, Debug)]
pub enum ItemActions {
    Drop,
    Equip,
    Unequip,
    Throw,
    Consume,
    Examine,
}

impl Item {
    pub fn available_actions(&self) -> Vec<ItemActions> {
        match self.item_type {
            ItemType::Unknown => vec![ItemActions::Drop],
            ItemType::Artifact => vec![ItemActions::Drop, ItemActions::Throw, ItemActions::Consume],
            ItemType::Weapon => vec![
                ItemActions::Drop,
                ItemActions::Throw,
                ItemActions::Equip,
                ItemActions::Unequip,
            ],
            ItemType::Armor => vec![ItemActions::Drop, ItemActions::Equip, ItemActions::Unequip],
            ItemType::Potion => vec![ItemActions::Drop, ItemActions::Throw, ItemActions::Consume],
            ItemType::Scroll => vec![ItemActions::Drop, ItemActions::Examine],
        }
    }
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

        f.write_fmt(format_args!("[{}]", desc.join(", ")))
    }
}

#[derive(Component, Default)]
pub struct EquippedItems(pub Vec<Entity>);

#[derive(Component, Default)]
pub struct CarriedItems(pub Vec<Entity>);

#[derive(Resource, Default)]
pub struct CurrentlySelectedItem(pub Option<Entity>);

#[derive(Component)]
pub struct CarriedMarker;

#[derive(Default, Debug, PartialEq, Eq)]
pub enum ItemType {
    #[default]
    Unknown,
    Artifact,
    Weapon,
    Armor,
    Potion,
    Scroll,
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

    pub fn to_item(self) -> Item {
        Item {
            name: self.name,
            image: self.tile,
            item_type: self.item_type,
            equip_stat_changes: self.stats.into_iter().collect(),
        }
    }

    pub fn create_at_raw(
        self,
        pos: IVec2,
        world: &mut World,
        transform: Transform,
        atlas: Handle<TextureAtlas>,
    ) {
        let color = {
            if let Some(magic) = world.get_resource::<Magic>() {
                self.stats
                    .clone()
                    .into_iter()
                    .max_by(|(_, v1), (_, v2)| v1.cmp(v2))
                    .map(|k| magic[k.0])
                    .unwrap_or(Color::WHITE)
            } else {
                Color::WHITE
            }
        };

        let child_atlas = atlas.clone();
        world
            .spawn(WorldEntityBundle::new_raw(
                transform,
                atlas,
                &self.name,
                pos,
                self.tile,
                false,
                WorldEntityKind::Item,
                Some(color),
            ))
            .with_children(|f| {
                f.spawn(((
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(0),
                        texture_atlas: child_atlas.clone_weak(),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                        ..Default::default()
                    },
                    RenderLayers::layer(1),
                ),));
            })
            .insert((
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

    pub fn create_at(
        self,
        pos: IVec2,
        commands: &mut Commands,
        grid: &Res<Grid>,
        magic: &ResMut<Magic>,
    ) {
        let color = {
            self.stats
                .clone()
                .into_iter()
                .max_by(|(_, v1), (_, v2)| v1.cmp(v2))
                .map(|k| magic[k.0])
                .unwrap_or(Color::WHITE)
        };

        commands
            .spawn(WorldEntityBundle::new(
                grid,
                &self.name,
                pos,
                self.tile,
                false,
                WorldEntityKind::Item,
                Some(color),
            ))
            .with_children(|f| {
                f.spawn(((
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(0),
                        texture_atlas: grid.atlas.clone_weak(),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                        ..Default::default()
                    },
                    RenderLayers::layer(1),
                ),));
            })
            .insert((
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

pub struct SvarogInventoryPlugin;

impl Plugin for SvarogInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentlySelectedItem>();
    }
}
