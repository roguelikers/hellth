use std::ops::Range;

use bevy::{
    app::{Plugin, Update},
    ecs::{
        component::Component,
        schedule::{common_conditions::in_state, IntoSystemConfigs},
        system::{Local, Query, Res, Resource},
    },
    math::{IVec2, Vec3},
    time::Time,
    transform::components::Transform,
};
use bevy_rand::{prelude::WyRand, resource::GlobalEntropy};

use funty::Unsigned;
use rand_core::RngCore;

use super::GameStates;

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

#[derive(Component)]
pub struct Targeting(pub IVec2);

#[derive(Component)]
pub struct TweenSize {
    pub baseline: f32,
    pub max: f32,
}

pub fn tween_size(
    mut tweens: Query<(&mut Transform, &TweenSize)>,
    time: Res<Time>,
    mut sine: Local<f32>,
) {
    *sine += time.delta_seconds();

    for (mut transform, tween) in &mut tweens {
        let v = tween.baseline + tween.max * sine.sin();
        transform.scale = Vec3::new(v, v, v);
    }
}

pub struct SvarogFeelPlugin;

impl Plugin for SvarogFeelPlugin {
    fn build(&self, bevy: &mut bevy::prelude::App) {
        bevy.insert_resource(Random::default())
            .add_systems(Update, tween_size.run_if(in_state(GameStates::Game)));
    }
}
