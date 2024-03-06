use std::ops::Range;

use bevy::{app::Plugin, ecs::system::Resource, math::IVec2};
use bevy_rand::{prelude::WyRand, resource::GlobalEntropy};

use funty::Unsigned;
use rand_core::RngCore;

#[derive(Resource, Default)]
pub struct Random(GlobalEntropy<WyRand>);

impl Random {
    pub fn from<T: Copy>(&mut self, arr: &[T]) -> T {
        arr[self.0.next_u32() as usize % arr.len()]
    }

    pub fn coin(&mut self) -> bool {
        self.percent(50usize)
    }

    pub fn percent<P: Unsigned>(&mut self, p: P) -> bool {
        (self.0.next_u32() as usize % 100) >= p.as_usize()
    }

    pub fn gen(&mut self, range: Range<i32>) -> i32 {
        range.start + (self.0.next_u32() % (range.end - range.start) as u32) as i32
    }

    pub fn gen2d(&mut self, x: Range<i32>, y: Range<i32>) -> IVec2 {
        IVec2::new(self.gen(x), self.gen(y))
    }

    pub fn shuffle<T>(&mut self, mut v: Vec<T>) -> Vec<T> {
        let l = v.len();
        for i in 0..l {
            v.swap(i, self.gen(0..l as i32) as usize);
        }

        v
    }
}

pub struct SvarogFeelPlugin;

impl Plugin for SvarogFeelPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        //bevy.add_plugins(TweeningPlugin);
        bevy.insert_resource(Random::default());
    }
}
