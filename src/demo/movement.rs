use bevy::prelude::*;

pub(super) fn plugin(_app: &mut App) {}

#[derive(Component, Reflect)]
pub struct Footprint(pub UVec2);
