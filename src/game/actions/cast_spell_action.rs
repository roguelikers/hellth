use bevy::prelude::*;

use crate::game::{character::CharacterStat, spells::*};

use super::{AbstractAction, Action, ActionResult};

#[derive(Debug)]
pub struct CastSpellAction {
    pub caster: Entity,
    pub target: SpellTarget,
    pub radius: u32,
    pub effects: Vec<SpellEffect>,
}

pub fn a_cast_spell(
    caster: Entity,
    target: SpellTarget,
    radius: u32,
    effects: &[SpellEffect],
) -> AbstractAction {
    Box::new(CastSpellAction {
        caster,
        target,
        radius,
        effects: effects.to_vec(),
    })
}

impl Action for CastSpellAction {
    fn get_affiliated_stat(&self) -> CharacterStat {
        CharacterStat::INT
    }

    fn do_action(&self, _world: &mut World) -> ActionResult {
        vec![]
    }
}
