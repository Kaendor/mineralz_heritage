use std::time::Duration;

use bevy::{prelude::*, sprite::Anchor};
use bevy_tweening::{Tween, TweenAnim, lens::SpriteColorLens};

use crate::demo::{commands::mining::AttackStats, level::LevelAssets};

pub fn plugin(app: &mut App) {
    app.add_observer(sync_healthbar_with_health)
        .add_observer(visual_feedback_on_damages);
}

#[derive(Component, Clone, Copy, Debug)]
/// Used for rocks for the moment
pub struct Health {
    current: f32,
}

#[derive(Component)]
/// Marker component for UI
pub struct HealthDisplay;

#[derive(EntityEvent)]
pub struct TakeDamage {
    #[event_target]
    from: Entity,
    // amount: f32,
}

impl TakeDamage {
    pub fn new(from: Entity, _damages: &AttackStats) -> Self {
        Self {
            from,
            // amount: damages.amount(),
        }
    }
}

#[derive(EntityEvent)]
pub struct Die {
    #[event_target]
    target: Entity,
    // killed_by: Entity,
}

impl Die {
    pub fn new(target: Entity, _killer: Entity) -> Self {
        Self {
            target,
            // killed_by: killer,
        }
    }
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn take_damage(&mut self, damages: &AttackStats) {
        self.current -= damages.amount();

        if self.current < 0.0 {
            self.current = 0.0;
        }
    }
}

pub fn healthbar(assets: &LevelAssets, health: Health) -> impl Bundle {
    (
        Name::new("Healthbar"),
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
            Transform::from_translation(Vec3::Z.with_x(health.current * -8.0 / 2.0)),
            HealthDisplay,
            Anchor::CENTER_LEFT,
        )],
    )
}

fn sync_healthbar_with_health(
    trigger: On<TakeDamage>,
    healths: Query<&Health>,
    mut sprites: Query<&mut Sprite, With<HealthDisplay>>,
    healthbars: Query<&Children>,
) {
    let Ok(health) = healths.get(trigger.event_target()) else {
        return;
    };
    info!("Current health: {health:?}");
    // I need to get the sprite component showing health data
    // It is a child of the entity receiving this event

    for child in healthbars.iter_descendants(trigger.event_target()) {
        let Ok(mut sprite) = sprites.get_mut(child) else {
            continue;
        };

        sprite.custom_size = Some(Vec2::new(health.current * 8.0, 8.0));
    }
}

fn visual_feedback_on_damages(trigger: On<TakeDamage>, mut commands: Commands) {
    let color_tween = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(200),
        SpriteColorLens {
            start: Color::BLACK,
            end: Color::WHITE,
        },
    );

    commands
        .entity(trigger.event_target())
        .insert(TweenAnim::new(color_tween));
}
