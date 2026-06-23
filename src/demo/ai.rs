use bevy::prelude::*;
use vleue_navigator::{NavMesh, prelude::ManagedNavMesh};

use crate::demo::{
    commands::{
        CommandQueue, EntityCommand, NextCommand, mining::AttackOrder, path_following::FollowPath,
    },
    level::map::SpatialIndex,
    player::Faction,
};

#[derive(Component)]
pub struct Ai;

#[derive(EntityEvent)]
pub struct PickNextTarget {
    pub entity: Entity,
}

pub fn plugin(app: &mut App) {
    app.add_observer(on_pick_next_target);
}

fn on_pick_next_target(
    trigger: On<PickNextTarget>,
    mut commands: Commands,
    stalkers: Query<(Entity, Ref<Transform>, &Faction), With<Ai>>,
    other_factions_entities: Query<(Entity, Ref<Transform>, &Faction)>,
    navmeshes: Res<Assets<NavMesh>>,
    navmesh: Query<&ManagedNavMesh>,
    index: Res<SpatialIndex>,
) {
    let Ok(navmesh) = navmesh.single() else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh) else {
        return;
    };

    let Ok((s_entity, s_transform, s_faction)) = stalkers.get(trigger.event_target()) else {
        return;
    };

    // Walk candidates nearest-first and commit to the first one we can actually
    // path to. Committing to the single closest target and giving up if it is
    // unreachable leaves the AI idle when, e.g., it just killed a wall and the next
    // closest target is boxed in.
    let target = index
        .0
        .nearest_neighbor_iter(s_transform.translation.xy().to_array())
        .filter_map(|n| other_factions_entities.get(n.data).ok())
        .filter(|(_, _, f)| *f != s_faction)
        .find_map(|(t_entity, t_transform, _)| {
            // Movers (e.g. the player) sit inside the navmesh and can be pathed to
            // directly. Obstacle targets (e.g. a wall) have their center outside the
            // navmesh, so we path to the closest reachable point towards them instead.
            let destination = if navmesh.transformed_is_in_mesh(t_transform.translation) {
                Some(t_transform.translation)
            } else {
                navmesh
                    .get()
                    .get_closest_point_towards(
                        t_transform.translation.xy(),
                        s_transform.translation.xy(),
                    )
                    .map(|p| p.position().extend(1.0))
            };

            let path = destination
                .and_then(|dest| navmesh.transformed_path(s_transform.translation, dest))?;

            Some((t_entity, path))
        });

    let Some((t_entity, path)) = target else {
        warn!("No reachable target found");
        return;
    };

    let mut command_queue = CommandQueue::new(vec![]);
    command_queue.add(EntityCommand::GoTo(FollowPath::new(path.path)));
    command_queue.add(EntityCommand::Attack(AttackOrder { target: t_entity }));

    commands
        .entity(s_entity)
        .insert(command_queue)
        .trigger(NextCommand::from);
}
