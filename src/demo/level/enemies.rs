use std::time::Duration;

use bevy::prelude::*;

use crate::demo::{
    ai::Ai,
    commands::{mining::AttackStats, path_following::MovementController},
    level::LevelAssets,
    player::Faction,
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
        AttackStats::new(2.0, 18.0, Duration::from_millis(700)),
        Ai,
        Faction::monster(),
    )
}
