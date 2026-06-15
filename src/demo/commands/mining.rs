use bevy::prelude::*;

pub fn plugin(_app: &mut App) {}

#[derive(Component, Reflect)]
#[reflect(Component)]
/// TODO: check is rock is near before performing mining operation
pub struct MineOrder {
    pub target: Entity,
}

impl From<Entity> for MineOrder {
    fn from(value: Entity) -> Self {
        Self { target: value }
    }
}
