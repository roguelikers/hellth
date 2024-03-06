use bevy::{ecs::entity::Entity, math::IVec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellEffect {
    Push(u32),
    Pull(u32),
    HealPerTurn(u32),
    DamagePerTurn(u32),
    BurnArea(u32),
    ModifySight(u32),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SpellTarget {
    #[default]
    Caster,
    Person(Entity),
    Place(IVec2),
}
