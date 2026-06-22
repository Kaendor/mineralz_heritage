use std::iter;

use bevy::{camera::primitives::Aabb, color::palettes, prelude::*};
use vleue_navigator::{VleueNavigatorPlugin, prelude::NavmeshUpdaterPlugin};

use crate::{
    AppSystems, PausableSystems,
    demo::commands::{CommandQueue, NextCommand, mining::AttackStats},
};

pub fn plugin(app: &mut App) {
    app.add_plugins((
        VleueNavigatorPlugin,
        NavmeshUpdaterPlugin::<Aabb, Obstacle>::default(),
    ));
    app.add_systems(
        Update,
        (follow_path)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Obstacle;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct FollowPath {
    path: Vec<Vec3>,
    current_index: usize,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            max_speed: 32.0,
        }
    }
}

impl FollowPath {
    pub fn next(&self) -> Option<&Vec3> {
        self.path.get(self.current_index)
    }

    pub fn destination(&self) -> Option<&Vec3> {
        self.path.last()
    }

    pub fn increment(&mut self) {
        self.current_index += 1;
    }

    pub fn new(path: Vec<Vec3>) -> Self {
        Self {
            path,
            current_index: 0,
        }
    }
}

fn follow_path(
    mut commands: Commands,
    mut player: Query<(
        Entity,
        &mut FollowPath,
        &mut MovementController,
        &mut Transform,
        &AttackStats,
        &CommandQueue,
    )>,
    time: Res<Time>,
) {
    for (entity, mut follow, controller, mut transform, mining_stat, command_queue) in &mut player {
        if let Some(destination) = follow.destination()
            && transform.translation.xy().distance(destination.xy()) <= mining_stat.range()
            && command_queue.has_attack_order()
        {
            commands.entity(entity).remove::<FollowPath>();
            commands.entity(entity).trigger(NextCommand::from);
            continue;
        }
        let Some(target_world) = follow.next().copied() else {
            commands.entity(entity).remove::<FollowPath>();
            commands.entity(entity).trigger(NextCommand::from);
            continue;
        };

        let direction = target_world - transform.translation;
        transform.translation += direction.normalize() * time.delta_secs() * controller.max_speed;

        if transform.translation.xy().distance(target_world.xy())
            < controller.max_speed * time.delta_secs()
        {
            // Snap to avoid drift
            follow.increment();
            transform.translation = target_world.with_z(1.0);
        }
    }
}

pub fn display_paths(paths: Query<(&Transform, &FollowPath)>, mut gizmos: Gizmos) {
    for (position, path) in &paths {
        gizmos.linestrip_2d(
            iter::once(position.translation.xy()).chain(path.path.iter().map(|n| n.xy())),
            palettes::tailwind::PURPLE_500,
        );
    }
}
