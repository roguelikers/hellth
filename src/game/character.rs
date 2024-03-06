use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct Character {
    pub strength: i32,
    pub arcane: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub willpower: i32,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            strength: 3,
            arcane: 3,
            intelligence: 3,
            wisdom: 3,
            willpower: 3,
        }
    }
}
