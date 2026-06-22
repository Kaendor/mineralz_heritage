use bevy::prelude::*;
use vleue_navigator::{NavMesh, prelude::ManagedNavMesh};

use crate::demo::{
    commands::{
        CommandQueue, EntityCommand, NextCommand, mining::AttackOrder, path_following::FollowPath,
    },
    player::{Faction, Player},
};

#[derive(Component)]
pub struct Ai;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, stalk_other_factions);
}

fn stalk_other_factions(
    mut commands: Commands,
    stalkers: Query<(Entity, Ref<Transform>, &Faction), With<Ai>>,
    other_factions_entities: Query<(Entity, Ref<Transform>, &Faction)>,
    navmeshes: Res<Assets<NavMesh>>,
    navmesh: Query<&ManagedNavMesh>,
) {
    let Ok(navmesh) = navmesh.single() else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh) else {
        return;
    };

    for (s_entity, s_transform, s_faction) in stalkers {
        let Some((t_entity, t_transform, t_faction)) = other_factions_entities
            .iter()
            .filter(|(_, _, f)| *f != s_faction)
            .next()
        else {
            return;
        };

        if t_transform.is_changed() || s_transform.is_added() {
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
