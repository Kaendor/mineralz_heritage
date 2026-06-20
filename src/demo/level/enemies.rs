use bevy::prelude::*;

use crate::demo::{
    ai::Ai,
    commands::{mining::MiningStats, path_following::MovementController},
    level::LevelAssets,
};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Enemy;

pub fn basic_enemy(assets: &LevelAssets) -> impl Bundle {
    (
        Name::new("Enemy"),
        Enemy,
        Sprite {
            image: assets.enemy.clone(),
            ..default()
        },
        MovementController::default(),
        MiningStats::new(2.0, 26.0),
        Ai,
    )
}
