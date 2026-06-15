//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_ecs_tilemap::{
    anchor::TilemapAnchor,
    map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType},
    tiles::TilePos,
};

use crate::{
    AppSystems, PausableSystems,
    demo::commands::{FollowPath, NextCommand},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (follow_path, apply_movement, apply_screen_wrap)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

#[derive(Component, Reflect)]
pub struct Footprint(pub UVec2);

const ARRIVAL_THRESHOLD: f32 = 0.5;

fn follow_path(
    mut commands: Commands,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    )>,
    mut player: Query<(
        Entity,
        &mut FollowPath,
        &mut MovementController,
        &mut TilePos,
        &mut Transform,
    )>,
) {
    let Ok((map_size, grid_size, tile_size, map_type, anchor)) = tilemap_q.single() else {
        return;
    };

    for (entity, mut follow, mut controller, mut tile_pos, mut transform) in &mut player {
        let Some(target_tile) = follow.next().copied() else {
            controller.intent = Vec2::ZERO;
            commands.entity(entity).remove::<FollowPath>();
            commands.entity(entity).trigger(NextCommand::from);
            continue;
        };

        let target_world =
            target_tile.center_in_world(map_size, grid_size, tile_size, map_type, anchor);
        let to_target = target_world - transform.translation.xy();

        if to_target.length() <= ARRIVAL_THRESHOLD {
            // Snap to avoid drift, update the logical tile, advance.
            transform.translation = target_world.extend(transform.translation.z);
            *tile_pos = target_tile;
            follow.increment();
            controller.intent = Vec2::ZERO;
        } else {
            controller.intent = to_target.normalize_or_zero();
        }
    }
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

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&MovementController, &mut Transform)>,
) {
    for (controller, mut transform) in &mut movement_query {
        let velocity = controller.max_speed * controller.intent;
        transform.translation += velocity.extend(0.0) * time.delta_secs();
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenWrap;

fn apply_screen_wrap(
    window: Single<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform, With<ScreenWrap>>,
) {
    let size = window.size() + 256.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}
