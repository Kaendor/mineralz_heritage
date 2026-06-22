use std::collections::VecDeque;

use bevy::prelude::*;

use crate::demo::commands::{mining::AttackOrder, path_following::FollowPath};

pub mod mining;
pub mod path_following;

pub fn plugin(app: &mut App) {
    app.add_observer(process_command_queue);
    app.add_plugins((mining::plugin, path_following::plugin));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub enum EntityCommand {
    GoTo(FollowPath),
    Attack(AttackOrder),
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CommandQueue(pub VecDeque<EntityCommand>);

impl CommandQueue {
    pub fn new(commands: Vec<EntityCommand>) -> Self {
        Self(VecDeque::from_iter(commands))
    }

    pub fn add(&mut self, command: EntityCommand) {
        self.0.push_back(command);
    }

    pub fn is_empty(&mut self) -> bool {
        self.0.is_empty()
    }

    pub fn has_attack_order(&self) -> bool {
        self.0
            .iter()
            .any(|c| matches!(c, EntityCommand::Attack(..)))
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
        EntityCommand::GoTo(follow_path) => {
            info!("Start new path: {follow_path:?}");
            commands.entity(on.event_target()).insert(follow_path);
        }
        EntityCommand::Attack(mine_order) => {
            commands.entity(on.event_target()).insert(mine_order);
            info!("Mine rock");
        }
    }
}
