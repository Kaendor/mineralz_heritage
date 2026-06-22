use bevy::prelude::*;
use vleue_navigator::{NavMesh, prelude::ManagedNavMesh};

use crate::demo::{
    commands::{
        CommandQueue, EntityCommand, NextCommand, mining::AttackOrder, path_following::FollowPath,
    },
    player::Player,
};

#[derive(Component)]
pub struct Ai;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, stalk_player);
}

fn stalk_player(
    mut commands: Commands,
    stalkers: Query<(Entity, &Transform), With<Ai>>,
    players: Query<(Entity, Ref<Transform>), With<Player>>,
    navmeshes: Res<Assets<NavMesh>>,
    navmesh: Query<&ManagedNavMesh>,
) {
    let Ok(navmesh) = navmesh.single() else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh) else {
        return;
    };

    for (s_entity, s_transform) in stalkers {
        let Some((t_entity, t_transform)) = players.iter().next() else {
            return;
        };

        if t_transform.is_changed() {
            let mut command_queue = CommandQueue::new(vec![]);

            if let Some(path) =
                navmesh.transformed_path(s_transform.translation, t_transform.translation)
            {
                command_queue.add(EntityCommand::GoTo(FollowPath::new(path.path)));
                command_queue.add(EntityCommand::Attack(AttackOrder { target: t_entity }));
            }

            if !command_queue.0.is_empty() {
                commands
                    .entity(s_entity)
                    .insert(command_queue)
                    .trigger(NextCommand::from);
            }
        }
    }
}
