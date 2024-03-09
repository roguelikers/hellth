use bevy::{
    prelude::*,
    render::view::RenderLayers,
    utils::{HashMap, HashSet},
};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::On,
    PickableBundle,
};
use doryen_fov::MapData;

use crate::game::{
    ai::{AIAgent, PendingActions},
    character::{Character, CharacterStat},
    fov::Sight,
    grid::{WorldEntityBundle, WorldEntityKind},
    health::Health,
    inventory::{CarriedItems, EquippedItems, ItemBuilder, ItemType},
    player::PlayerState,
    sprite::{ChangePassability, ChangeSprite},
    sprites::*,
    turns::TurnTaker,
    ui::ShowEntityDetails,
};

use super::{
    feel::Random,
    fov::{on_new_fov_added, recalculate_fov, RecalculateFOVEvent},
    grid::{Grid, Passability, WorldData},
    turns::{TurnOrder, TurnOrderProgressEvent},
    DebugFlag, GameStates,
};

#[derive(Event)]
pub struct ProcGenEvent;

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Resource)]
pub struct MapRadius(pub i32);

#[derive(Component)]
pub struct ClearLevel;

#[allow(clippy::identity_op)]
#[allow(clippy::too_many_arguments)]
pub fn generate_level(
    clear: Query<Entity, With<ClearLevel>>,

    mut commands: Commands,
    mut map: ResMut<WorldData>,
    mut rng: ResMut<Random>,
    mut turn_order: ResMut<TurnOrder>,
    mut sprites: Query<(&mut TextureAtlasSprite, &mut Passability)>,
    mut visibility: Query<&mut Visibility>,
    mut turn_order_progress: EventWriter<TurnOrderProgressEvent>,
    grid: Res<Grid>,
    radius: Res<MapRadius>,
) {
    for c in &clear {
        commands.entity(c).despawn_recursive();
    }

    fn clear_grid(
        grid: &Res<Grid>,
        rng: &mut ResMut<Random>,
        map: &mut ResMut<WorldData>,
        radius: &Res<MapRadius>,
        visibility: &mut Query<&mut Visibility>,
        sprites: &mut Query<(&mut TextureAtlasSprite, &mut Passability)>,
    ) -> HashSet<IVec2> {
        grid.entities.iter().for_each(|(pos, _)| {
            map.blocking.remove(pos);
            map.solid.remove(pos);
        });

        let symbols = Tiles::default()
            .add_more(EMPTY_FLOOR, 4)
            .add_bunch(&[
                EXTERIOR_FLOOR1,
                EXTERIOR_FLOOR2,
                EXTERIOR_FLOOR3,
                EXTERIOR_FLOOR4,
            ])
            .done();
        let mut okay = HashSet::new();

        map.solid.clear();
        grid.entities.iter().for_each(|(pos, e)| {
            if let Ok(mut vis) = visibility.get_mut(*e) {
                *vis = Visibility::Hidden;
            }

            if let Ok((mut sprite, mut passable)) = sprites.get_mut(*e) {
                let dist = pos.distance_squared(IVec2::ZERO);
                let r = radius.0;
                if dist < r || rng.gen(0..(r * 3 / 2)) > dist {
                    sprite.index = symbols[rng.gen(0..symbols.len() as i32) as usize];
                    sprite.color = Color::WHITE;
                    *passable = Passability::Passable;
                    okay.insert(*pos);
                    map.data.set_transparent(
                        (pos.x + grid.size.x / 2 + 1) as usize,
                        (pos.y + grid.size.y / 2 + 1) as usize,
                        true,
                    );
                } else {
                    sprite.index = VOID.into();
                    sprite.color = Color::WHITE;
                    *passable = Passability::Blocking;
                    map.data.set_transparent(
                        (pos.x + grid.size.x / 2 + 1) as usize,
                        (pos.y + grid.size.y / 2 + 1) as usize,
                        false,
                    );
                    map.solid.insert(*pos);
                }
            }
        });

        okay
    }

    #[allow(clippy::identity_op)]
    fn make_obstructions(
        commands: &mut Commands,
        count: usize,
        size: IVec2,
        rng: &mut ResMut<Random>,
        grid: &Res<Grid>,
        map: &mut ResMut<WorldData>,
        okay: &mut HashSet<IVec2>,
    ) {
        let forest_tiles = Tiles::default()
            .add_bunch(&[EMPTY_FLOOR, FOREST1, FOREST2, FOREST3])
            .add_more(FOREST4, 2)
            .done();

        let ruin_tiles = Tiles::default()
            .add_more(WALL1, 4)
            .add_bunch(&[WALL2, WALL3, WALL4, WALL5, WALL6])
            .done();

        for _attempt in 0..count {
            let half = size / 2;
            let middle = IVec2::new(rng.gen(-half.x..half.x), rng.gen(-half.y..half.y));

            let (tiles, passability) = if rng.coin() {
                (forest_tiles.as_slice(), Passability::SightBlocking)
            } else {
                (ruin_tiles.as_slice(), Passability::Blocking)
            };

            let IVec2 { x, y } = rng.gen2d(3..6, 4..7);
            for i in -x..=x {
                for j in -y..=y {
                    let pos = middle + IVec2::new(i, j);
                    let dist = middle.distance_squared(pos);

                    let index = rng.from(tiles);

                    if okay.contains(&pos) && rng.percent(3 * dist as u32) {
                        commands.add(ChangeSprite {
                            position: pos,
                            index,
                        });

                        commands.add(ChangePassability {
                            position: pos,
                            passable: passability,
                        });

                        if passability == Passability::Blocking {
                            map.solid.insert(pos);
                        }

                        okay.remove(&pos);

                        map.data.set_transparent(
                            (pos.x + grid.size.x / 2 + 1) as usize,
                            (pos.y + grid.size.y / 2 + 1) as usize,
                            passability == Passability::Passable,
                        );
                    }
                }
            }
        }
    }

    #[allow(clippy::identity_op)]
    fn make_houses(
        commands: &mut Commands,
        count: usize,
        size: IVec2,
        rng: &mut ResMut<Random>,
        grid: &Res<Grid>,
        map: &mut ResMut<WorldData>,
        okay: &mut HashSet<IVec2>,
    ) {
        let wall_tiles: Vec<usize> = Tiles::default().add_one(WALL1).done();
        let floor_tiles: Vec<usize> = Tiles::default()
            .add_more(INTERIOR_FLOOR2, 9)
            .add_bunch(&[
                EXTERIOR_FLOOR1,
                EXTERIOR_FLOOR2,
                EXTERIOR_FLOOR3,
                EXTERIOR_FLOOR4,
                INTERIOR_FLOOR1,
            ])
            .done();

        let mut walls = HashMap::new();
        for _attempt in 0..count {
            let half = size / 2;
            let dx = -half.x..half.x;
            let dy = -half.y..half.y;
            let middle = rng.gen2d(dx, dy);
            let room_size = rng.gen2d(3..7, 3..7);
            for i in -room_size.x..=room_size.x {
                for j in -room_size.y..=room_size.y {
                    if rng.gen(0..100) > 70 {
                        continue;
                    }

                    let ij = IVec2::new(i, j);
                    let pos = middle + ij;

                    if okay.contains(&pos) {
                        let index = rng.from(&wall_tiles);
                        commands.add(ChangeSprite {
                            position: pos,
                            index,
                        });

                        walls.insert(pos, index);
                        map.data.set_transparent(
                            (pos.x + grid.size.x / 2 + 1) as usize,
                            (pos.y + grid.size.y / 2 + 1) as usize,
                            false,
                        );
                    }
                }
            }

            for i in -room_size.x + 1..room_size.x {
                for j in -room_size.y + 1..room_size.y {
                    let ij = IVec2::new(i, j);
                    let pos = middle + ij;

                    if okay.contains(&pos) {
                        let index = rng.from(&floor_tiles);
                        commands.add(ChangeSprite {
                            position: pos,
                            index,
                        });

                        walls.remove(&pos);
                        map.data.set_transparent(
                            (pos.x + grid.size.x / 2 + 1) as usize,
                            (pos.y + grid.size.y / 2 + 1) as usize,
                            true,
                        );

                        if map.solid.contains(&pos) {
                            map.solid.remove(&pos);
                        }
                    }
                }
            }

            for (pos, wall) in &walls {
                if okay.contains(pos) {
                    commands.add(ChangePassability {
                        position: *pos,
                        passable: if *wall != wall_tiles[0] {
                            Passability::Passable
                        } else {
                            Passability::Blocking
                        },
                    });
                    okay.remove(pos);
                    map.solid.insert(*pos);
                }
            }
        }
    }

    let size = grid.size;

    map.data = MapData::new(122, 64);
    map.memory.clear();

    turn_order.clear();

    let mut okay = clear_grid(
        &grid,
        &mut rng,
        &mut map,
        &radius,
        &mut visibility,
        &mut sprites,
    );

    make_obstructions(
        &mut commands,
        20,
        size,
        &mut rng,
        &grid,
        &mut map,
        &mut okay,
    );
    make_houses(
        &mut commands,
        40,
        size,
        &mut rng,
        &grid,
        &mut map,
        &mut okay,
    );

    let stats = [
        CharacterStat::STR,
        CharacterStat::ARC,
        CharacterStat::INT,
        CharacterStat::WIS,
        CharacterStat::WIL,
        CharacterStat::AGI,
    ];

    let mut places = rng.shuffle(okay.into_iter().collect::<Vec<_>>());

    // add stuff
    for _ in 1..20 {
        let mut builder = ItemBuilder::default()
            .with_name("Arcane Scroll")
            .with_image(rng.from(&[SCROLL1, SCROLL2]))
            .with_type(ItemType::Spell);

        for _ in 0..rng.gen(1..3) {
            let mut power = 0;
            while power == 0 {
                power = rng.gen(-3..3);
            }

            builder = builder.with_stat(rng.from(&stats), power);
        }

        builder.create_at(places.pop().unwrap_or_default(), &mut commands, &grid)
    }

    // add player
    let mut player = commands.spawn(WorldEntityBundle::new(
        &grid,
        "Player",
        places.pop().unwrap_or_default(),
        EMO_MAGE.into(),
        true,
        WorldEntityKind::Player,
        None,
    ));
    player
        .with_children(|f| {
            f.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite::new(0),
                    texture_atlas: grid.atlas.clone_weak(),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                    ..Default::default()
                },
                RenderLayers::layer(1),
            ));
        })
        .insert((
            Character {
                agility: 6,
                ..Default::default()
            },
            CarriedItems::default(),
            EquippedItems::default(),
            PlayerMarker,
            PlayerState::default(),
            PendingActions::default(),
            Health::new(10),
            TurnTaker,
            PickableBundle::default(),
            On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
            Sight(6),
        ));

    // add mobs
    for i in 1..10 {
        let char = Character::random(&mut rng);
        let index: usize = OLD_MAGE.into();

        commands
            .spawn(WorldEntityBundle::new(
                &grid,
                format!("Mage {}", i).as_str(),
                places.pop().unwrap_or_default(),
                index + rng.gen(0..7) as usize,
                true,
                WorldEntityKind::NPC,
                Some(char.get_strongest_stat_color()),
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
                AIAgent::default(),
                CarriedItems::default(),
                EquippedItems::default(),
                PendingActions::default(),
                PickableBundle::default(),
                On::<Pointer<Click>>::send_event::<ShowEntityDetails>(),
                Health::new(5),
            ));
    }
    turn_order_progress.send(TurnOrderProgressEvent);
}

pub fn debug_radius(mut map_radius: ResMut<MapRadius>, keys: Res<Input<KeyCode>>) {
    let mut radius = map_radius.0;

    if keys.just_pressed(KeyCode::F6) {
        radius -= 50;
    } else if keys.just_pressed(KeyCode::F7) {
        radius += 50;
    }

    if radius <= 50 {
        radius = 50;
    }

    map_radius.0 = radius;
}

pub fn debug_procgen(
    mut procgen_events: EventWriter<ProcGenEvent>,
    keys: Res<Input<KeyCode>>,
    mut debug: ResMut<DebugFlag>,
) {
    if keys.just_pressed(KeyCode::F5) {
        procgen_events.send(ProcGenEvent);
    }

    if keys.just_pressed(KeyCode::F12) {
        debug.0 = !debug.0;
    }
}

pub struct SvarogProcgenPlugin;

impl Plugin for SvarogProcgenPlugin {
    fn build(&self, bevy: &mut App) {
        bevy.add_event::<ProcGenEvent>()
            .add_event::<RecalculateFOVEvent>()
            .insert_resource(MapRadius(800))
            .insert_resource(ClearColor(Color::BLACK))
            .insert_resource(Msaa::Off)
            .add_systems(Update, generate_level.run_if(on_event::<ProcGenEvent>()))
            .add_systems(Update, on_new_fov_added.run_if(in_state(GameStates::Game)))
            .add_systems(
                Last,
                recalculate_fov
                    .run_if(on_event::<RecalculateFOVEvent>())
                    .run_if(in_state(GameStates::Game)),
            )
            .add_systems(Update, (debug_radius, debug_procgen));
    }
}
