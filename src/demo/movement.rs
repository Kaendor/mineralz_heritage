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

use std::collections::VecDeque;

use bevy::{ecs::system::entity_command::insert, prelude::*, window::PrimaryWindow};
use bevy_ecs_tilemap::{
    anchor::TilemapAnchor,
    map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType},
    tiles::TilePos,
};

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (follow_path, apply_movement, apply_screen_wrap)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_observer(process_command_queue);
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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FollowPath {
    path: Vec<TilePos>,
    current_index: usize,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MineOrder {
    pub target: Entity,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub enum PlayerCommand {
    GoTo(FollowPath),
    Mine(MineOrder),
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CommandQueue(pub VecDeque<PlayerCommand>);

impl CommandQueue {
    pub fn new(commands: Vec<PlayerCommand>) -> Self {
        Self(VecDeque::from_iter(commands))
    }
}

#[derive(EntityEvent)]
pub struct NextCommand(Entity);

impl From<Entity> for NextCommand {
    fn from(value: Entity) -> Self {
        NextCommand(value)
    }
}

// Process on event NextCommand
fn process_command_queue(
    on: On<NextCommand>,
    mut commands: Commands,
    mut commanded: Query<&mut CommandQueue>,
) {
    // If a command is running, do nothing
    // if the queue is not empty and no command is running, apply next command

    let Ok(mut command_queue) = commanded.get_mut(on.event_target()) else {
        return;
    };
    let next_command = command_queue.0.pop_front();

    let Some(next_command) = next_command else {
        info!("No more commands");
        return;
    };

    match next_command {
        PlayerCommand::GoTo(follow_path) => {
            commands.entity(on.event_target()).insert(follow_path);
            info!("Start new path");
        }
        PlayerCommand::Mine(mine_order) => {
            commands.entity(on.event_target()).insert(mine_order);
            info!("Mine rock");
        }
    }
}

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
        let Some(target_tile) = follow.path.get(follow.current_index).copied() else {
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
            follow.current_index += 1;
            controller.intent = Vec2::ZERO;
        } else {
            controller.intent = to_target.normalize_or_zero();
        }
    }
}

impl FollowPath {
    pub fn new(path: Vec<TilePos>) -> Self {
        Self {
            path,
            current_index: 0,
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
