use bevy::{prelude::*, sprite::Anchor};

use crate::demo::{commands::mining::AttackStats, level::LevelAssets};

pub fn plugin(app: &mut App) {}

#[derive(Component, Clone, Copy)]
/// Used for rocks for the moment
pub struct Health {
    current: f32,
}

#[derive(Component)]
/// Marker component for UI
pub struct HealthDisplay;

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn take_damage(&mut self, damages: &AttackStats) {
        self.current -= damages.amount()
    }
}

pub fn healthbar(assets: &LevelAssets, health: &Health) -> impl Bundle {
    (
        Sprite {
            image: assets.empty_health.clone(),
            custom_size: Some(Vec2::new(health.current * 8.0, 8.0)),
            image_mode: SpriteImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(1.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Tile { stretch_value: 0.2 },
                ..default()
            }),
            ..default()
        },
        Transform::from_translation(Vec3::Y * 16.0),
        children![(
            Sprite {
                image: assets.full_health.clone(),
                custom_size: Some(Vec2::new(health.current * 8.0, 8.0)),
                image_mode: SpriteImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(1.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Tile { stretch_value: 0.2 },
                    ..default()
                }),
                ..default()
            },
            Transform::from_translation(Vec3::Z.with_x(-5.0 * 8.0 / 2.0)),
            HealthDisplay,
            Anchor::CENTER_LEFT,
        )],
    )
}
