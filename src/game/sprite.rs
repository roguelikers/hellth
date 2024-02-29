use bevy::{ecs::system::Command, math::IVec2, sprite::TextureAtlasSprite};

use super::grid::Grid;

pub struct ChangeSprite {
    pub position: IVec2,
    pub index: usize,
}

impl Command for ChangeSprite {
    fn apply(self, world: &mut bevy::prelude::World) {
        let entity = {
            let Some(grid) = world.get_resource::<Grid>() else {
                return;
            };

            let Some(entity) = grid.get(self.position) else {
                return;
            };

            *entity
        };

        let mut query = world.query::<&mut TextureAtlasSprite>();
        let Ok(mut sprite) = query.get_mut(world, entity) else {
            return;
        };

        sprite.index = self.index;
    }
}
