/*

   level 1 spell
   level 2 spell = level 1 + focus | level 1 + chant
   level 3 spell = level 1 + focus + chant
   level 4 spell = level 1 + 2 fokus | level 1 + 2 chant < 4 kruga

           [=======BBB] 10  -1 hp po krugu
           [=======BB-] 9
           [=======B--] 8
           [=======---] 7

       FOCUS

           [=====BBB==] 10  -1 hp po krugu, fokus
           [=====BBB=-] 9
           [=====BBB--] 8
           [=====BB---] 7
           [=====B----] 6
           [=====-----] 5

           [=====BBB==] 10  -1 hp po krugu
           [=====-----] 10
           [==HHH-----]
           [===HHH----] <<<

       CHANT

           [=====B=B=B]
           fokus
           bacis spel 3
           [=====HHH=-]
           [=====B=HHH]

           bacis AGGRO na sve oko sebe
           [====AAA===]

       MOVE

           [=====BBB==] 10
           tri koraka
           [======BBB=] 10
           tri koraka
           [=======BBB] 10

       TRIGGER

           [=T========] 10
*/

use bevy::ecs::{component::Component, entity::Entity};
use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Default, Clone)]
pub struct HitPoint {
    pub spell: Option<Entity>,
}

#[derive(Component, Clone)]
pub struct Health {
    pub size: usize,
    pub hitpoints: VecDeque<HitPoint>,
}

impl Debug for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hps = self
            .hitpoints
            .iter()
            .map(|h| if h.spell.is_none() { "=" } else { "S" })
            .collect::<Vec<_>>()
            .join("");

        if hps.len() < self.size {
            hps += &"-".repeat(self.size - hps.len());
        }
        f.write_fmt(format_args!("[{}]", hps))
    }
}

impl Health {
    pub fn new(size: usize) -> Health {
        Health {
            size,
            hitpoints: VecDeque::from_iter(vec![HitPoint::default(); size]),
        }
    }

    pub fn normal_damage(&mut self, n: usize) {
        for _ in 0..n {
            let _ = self.hitpoints.pop_back();
        }
    }

    pub fn normal_heal(&mut self, n: usize) {
        for _ in 0..n {
            if self.hitpoints.len() < self.size {
                self.hitpoints.push_front(HitPoint::default());
            }
        }
    }
}
