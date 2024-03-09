#[derive(Clone, Copy)]
pub struct Tile(pub u32, pub u32);

impl From<Tile> for usize {
    fn from(val: Tile) -> Self {
        (val.0 * 49 + val.1) as usize
    }
}

pub const VOID: Tile = Tile(4, 0);

pub const EMPTY_FLOOR: Tile = Tile(0, 0);
pub const EXTERIOR_FLOOR1: Tile = Tile(0, 1);
pub const EXTERIOR_FLOOR2: Tile = Tile(0, 2);
pub const EXTERIOR_FLOOR3: Tile = Tile(0, 3);
pub const EXTERIOR_FLOOR4: Tile = Tile(0, 4);

pub const FOREST1: Tile = Tile(1, 0);
pub const FOREST2: Tile = Tile(1, 1);
pub const FOREST3: Tile = Tile(1, 2);
pub const FOREST4: Tile = Tile(1, 3);

pub const WALL1: Tile = Tile(13, 0);
pub const WALL2: Tile = Tile(11, 1);
pub const WALL3: Tile = Tile(11, 2);
pub const WALL4: Tile = Tile(17, 10);
pub const WALL5: Tile = Tile(18, 10);
pub const WALL6: Tile = Tile(18, 11);

pub const BONES: Tile = Tile(15, 0);

pub const INTERIOR_FLOOR1: Tile = Tile(0, 16);
pub const INTERIOR_FLOOR2: Tile = Tile(0, 17);

pub const OLD_MAGE: Tile = Tile(2, 24);
pub const EMO_MAGE: Tile = Tile(1, 26);

pub const SCROLL1: Tile = Tile(15, 34);
pub const SCROLL2: Tile = Tile(15, 33);

pub const HP_EMPTY: Tile = Tile(14, 39);
pub const HP_FULL: Tile = Tile(15, 39);

pub const SELECTION: Tile = Tile(12, 37);

#[derive(Default)]
pub struct Tiles(Vec<Vec<Tile>>);

impl Tiles {
    pub fn add_one(mut self, tile: Tile) -> Self {
        self.0.push(vec![tile]);
        self
    }

    pub fn add_bunch(mut self, tiles: &[Tile]) -> Self {
        self.0.push(tiles.to_owned());
        self
    }

    pub fn add_more(mut self, tile: Tile, n: usize) -> Self {
        self.0.push(vec![tile; n]);
        self
    }

    pub fn done(self) -> Vec<usize> {
        self.0
            .into_iter()
            .flatten()
            .map(|t| {
                let t_as_int: usize = t.into();
                t_as_int
            })
            .collect()
    }
}
