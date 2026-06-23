use bevy::prelude::*;
use vleue_navigator::{NavMesh, prelude::ManagedNavMesh};

use crate::{
    demo::{
        commands::{
            CommandQueue, EntityCommand, NextCommand, mining::AttackOrder,
            path_following::FollowPath,
        },
        level::map::SpatialIndex,
        player::Faction,
    },
    screens::Screen,
};

#[derive(Component)]
pub struct Ai;

#[derive(EntityEvent)]
pub struct PickNextTarget {
    pub entity: Entity,
}

pub fn plugin(app: &mut App) {
    // app.add_systems(
    //     Update,
    //     stalk_other_factions.run_if(resource_exists::<SpatialIndex>),
    // );
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

    let target = index
        .0
        .nearest_neighbor_iter(s_transform.translation.xy().to_array())
        .filter_map(|n| other_factions_entities.get(n.data).ok())
        .find(|(_, _, f)| *f != s_faction);

    let Some((t_entity, t_transform, _faction)) = target else {
        warn!("No other faction found");
        return;
    };

    // if t_transform.is_changed() || s_transform.is_added() {
    let mut command_queue = CommandQueue::new(vec![]);

    if let Some(path) = navmesh.transformed_path(s_transform.translation, t_transform.translation) {
        command_queue.add(EntityCommand::GoTo(FollowPath::new(path.path)));
        command_queue.add(EntityCommand::Attack(AttackOrder { target: t_entity }));
    }

    if !command_queue.0.is_empty() {
        commands
            .entity(s_entity)
            .insert(command_queue)
            .trigger(NextCommand::from);
    }
    // }
}
