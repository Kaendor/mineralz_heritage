use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

pub fn plugin(app: &mut App) {
    app.add_observer(process_command_queue);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FollowPath {
    path: Vec<TilePos>,
    current_index: usize,
}

impl FollowPath {
    pub fn next(&self) -> Option<&TilePos> {
        self.path.get(self.current_index)
    }

    pub fn increment(&mut self) {
        self.current_index += 1;
    }

    pub fn new(path: Vec<TilePos>) -> Self {
        Self {
            path,
            current_index: 0,
        }
    }
}

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

    pub fn add(&mut self, command: PlayerCommand) {
        self.0.push_back(command);
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
