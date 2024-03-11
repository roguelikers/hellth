use super::{
    ai::{AIAgent, AIStrategy, PendingActions},
    character::Character,
    feel::Random,
    grid::{Grid, WorldEntityBundle, WorldEntityKind},
    health::{Health, RecoveryCounter},
    inventory::{CarriedItems, EquippedItems},
    magic::Focus,
    sprites::*,
    turns::TurnTaker,
    ui::ShowEntityDetails,
};
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
    PickableBundle,
};

pub fn make_goblin(commands: &mut Commands, grid: &Res<Grid>, place: IVec2) {
    commands
        .spawn(WorldEntityBundle::new(
            grid,
            "Goblin",
            place,
            GOBLIN.into(),
            true,
            WorldEntityKind::NPC,
            None,
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
            TurnTaker,
            Character::default(),
            Focus(0),
            AIAgent::default(),
            CarriedItems::default(),
            EquippedItems::default(),
            PendingActions::default(),
            PickableBundle::default(),
            RecoveryCounter::default(),
            On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
            Health::new(2),
        ));
}

pub fn make_orc(commands: &mut Commands, grid: &Res<Grid>, place: IVec2, aggro: bool) {
    let mut char = Character::default();
    char.agility += 2;
    char.strength += 2;

    commands
        .spawn(WorldEntityBundle::new(
            grid,
            "Orc",
            place,
            ORC.into(),
            true,
            WorldEntityKind::NPC,
            None,
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
            TurnTaker,
            char,
            Focus(0),
            AIAgent(if aggro {
                AIStrategy::Aggro
            } else {
                AIStrategy::Standard
            }),
            CarriedItems::default(),
            EquippedItems::default(),
            PendingActions::default(),
            PickableBundle::default(),
            RecoveryCounter::default(),
            On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
            Health::new(3),
        ));
}

pub fn make_acolyte(
    commands: &mut Commands,
    rng: &mut ResMut<Random>,
    grid: &Res<Grid>,
    place: IVec2,
) {
    let mut char = Character::default();
    char.arcana += 2;

    char.intelligence += rng.gen(0..3);

    if rng.coin() {
        char.strength += rng.gen(-2..1);
    }

    if rng.coin() {
        char.wisdom += rng.gen(-1..2);
    }

    commands
        .spawn(WorldEntityBundle::new(
            grid,
            "Evoker",
            place,
            EVOKER.into(),
            true,
            WorldEntityKind::NPC,
            None,
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
            TurnTaker,
            char,
            Focus(0),
            AIAgent(if rng.coin() {
                AIStrategy::Aggro
            } else {
                AIStrategy::Standard
            }),
            CarriedItems::default(),
            EquippedItems::default(),
            PendingActions::default(),
            PickableBundle::default(),
            RecoveryCounter::default(),
            On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
            Health::new(5),
        ));
}

pub fn make_thaumaturge(
    commands: &mut Commands,
    rng: &mut ResMut<Random>,
    grid: &Res<Grid>,
    place: IVec2,
) {
    let mut char = Character::default();
    char.arcana += 3;
    char.wisdom += 2;

    char.intelligence += rng.gen(2..3);

    if rng.coin() {
        char.strength += rng.gen(-1..1);
    }

    commands
        .spawn(WorldEntityBundle::new(
            grid,
            "Thaumaturge",
            place,
            THAUMATURGE.into(),
            true,
            WorldEntityKind::NPC,
            None,
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
            TurnTaker,
            char,
            Focus(4),
            if rng.coin() {
                AIAgent::default()
            } else {
                AIAgent(AIStrategy::RandomMove)
            },
            CarriedItems::default(),
            EquippedItems::default(),
            PendingActions::default(),
            PickableBundle::default(),
            RecoveryCounter::default(),
            On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
            Health::new(8),
        ));
}

#[derive(Component)]
pub struct TheHealer;

pub fn make_healer(
    commands: &mut Commands,
    rng: &mut ResMut<Random>,
    grid: &Res<Grid>,
    place: IVec2,
) {
    let mut char = Character {
        strength: 6,
        arcana: 5,
        intelligence: rng.gen(8..9),
        wisdom: 6,
        willpower: 10,
        agility: rng.gen(8..9),
        ..Default::default()
    };

    commands
        .spawn(WorldEntityBundle::new(
            grid,
            "The Healer",
            place,
            HEALER.into(),
            true,
            WorldEntityKind::NPC,
            None,
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
            TurnTaker,
            char,
            Focus(5),
            AIAgent::default(),
            CarriedItems::default(),
            EquippedItems::default(),
            PendingActions::default(),
            PickableBundle::default(),
            RecoveryCounter::default(),
            On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
            Health::new(14),
            TheHealer,
        ));
}

pub fn make_bat(commands: &mut Commands, rng: &mut ResMut<Random>, grid: &Res<Grid>, place: IVec2) {
    let char = Character {
        agility: rng.gen(8..10),
        ..Default::default()
    };

    commands
        .spawn(WorldEntityBundle::new(
            grid,
            "Bat",
            place,
            BAT.into(),
            true,
            WorldEntityKind::NPC,
            None,
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
            TurnTaker,
            char,
            Focus(0),
            AIAgent(AIStrategy::RandomMove),
            CarriedItems::default(),
            EquippedItems::default(),
            PendingActions::default(),
            PickableBundle::default(),
            RecoveryCounter::default(),
            On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
            Health::new(1),
        ));
}
