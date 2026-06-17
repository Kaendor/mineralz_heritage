use std::iter;

use bevy::{camera::primitives::Aabb, color::palettes, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use vleue_navigator::{VleueNavigatorPlugin, prelude::NavmeshUpdaterPlugin};

use crate::{AppSystems, PausableSystems, demo::commands::NextCommand};

pub fn plugin(app: &mut App) {
    app.add_plugins((
        VleueNavigatorPlugin,
        NavmeshUpdaterPlugin::<Aabb, Obstacle>::default(),
    ));
    app.add_systems(
        Update,
        (follow_path, apply_movement)
            .chain()
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

const ARRIVAL_THRESHOLD: f32 = 0.5;
fn follow_path(
    mut commands: Commands,
    // tilemap_q: Query<(
    //     &TilemapSize,
    //     &TilemapGridSize,
    //     &TilemapTileSize,
    //     &TilemapType,
    //     &TilemapAnchor,
    // )>,
    mut player: Query<(
        Entity,
        &mut FollowPath,
        &mut MovementController,
        &mut TilePos,
        &mut Transform,
    )>,
) {
    // let Ok((map_size, grid_size, tile_size, map_type, anchor)) = tilemap_q.single() else {
    //     return;
    // };

    for (entity, mut follow, mut controller, mut tile_pos, mut transform) in &mut player {
        let Some(target_world) = follow.next().copied() else {
            controller.intent = Vec2::ZERO;
            commands.entity(entity).remove::<FollowPath>();
            commands.entity(entity).trigger(NextCommand::from);
            continue;
        };

        // let target_world =
        //     target_tile.center_in_world(map_size, grid_size, tile_size, map_type, anchor);
        let to_target = target_world - transform.translation;

        if to_target.length() <= ARRIVAL_THRESHOLD {
            // Snap to avoid drift, update the logical tile, advance.
            transform.translation = target_world.with_z(transform.translation.z);
            // *tile_pos = target_tile;
            follow.increment();
            controller.intent = Vec2::ZERO;
        } else {
            controller.intent = to_target.xy().normalize_or_zero();
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &mut Transform)>,
) {
    for (controller, mut transform) in &mut movement_query {
        let velocity = controller.max_speed * controller.intent;
        transform.translation += velocity.extend(0.0) * time.delta_secs();
    }
}

pub fn display_paths(
    paths: Query<(&Transform, &FollowPath)>,
    mut gizmos: Gizmos,
    // tilemap_q: Query<(
    //     &TilemapSize,
    //     &TilemapGridSize,
    //     &TilemapTileSize,
    //     &TilemapType,
    //     &TilemapAnchor,
    // )>,
) {
    // let Ok((map_size, grid_size, tile_size, map_type, anchor)) = tilemap_q.single() else {
    //     return;
    // };

    for (position, path) in &paths {
        gizmos.linestrip_2d(
            iter::once(position.translation.xy()).chain(path.path.iter().map(|n| {
                // let translation =
                //     n.center_in_world(map_size, grid_size, tile_size, map_type, anchor);

                n.xy()
            })),
            palettes::tailwind::PURPLE_500,
        );
    }
}
