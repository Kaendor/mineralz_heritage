use bevy::{color::palettes, prelude::*};
use vleue_navigator::{NavMesh, prelude::ManagedNavMesh};

use crate::demo::{
    commands::{CommandQueue, NextCommand, PlayerCommand, path_following::FollowPath},
    level::map::Occupancy,
    player::Player,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        process_mining_order.run_if(resource_exists::<Occupancy>),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
/// TODO: check is rock is near before performing mining operation
pub struct MiningOrder {
    pub target: Entity,
}

#[derive(Component, Debug)]
/// in HP by Seconds
pub struct MiningStats {
    power: f32,
    range: f32,
}

impl MiningStats {
    pub fn new(power: f32, range: f32) -> Self {
        Self { power, range }
    }
}

#[derive(Component)]
/// Used for rocks for the moment
pub struct Health {
    current: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn take_damage(&mut self, damages: &MiningStats) {
        self.current -= damages.power
    }
}

impl From<Entity> for MiningOrder {
    fn from(value: Entity) -> Self {
        Self { target: value }
    }
}

pub fn on_right_click_request_mining(
    trigger: On<Pointer<Press>>,
    mut commands: Commands,
    player: Query<(Entity, &Transform), With<Player>>,
    navmeshes: Res<Assets<NavMesh>>,
    navmesh: Query<&ManagedNavMesh>,
    transforms: Query<&Transform>,
) {
    let PointerButton::Secondary = trigger.event().button else {
        return;
    };
    let Ok(navmesh) = navmesh.single() else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh) else {
        return;
    };
    let Ok((p_entity, p_transform)) = player.single() else {
        return;
    };
    let Ok(rock_position) = transforms.get(trigger.event_target()) else {
        return;
    };

    let Some(path) = navmesh
        .get()
        .get_closest_point_towards(rock_position.translation.xy(), p_transform.translation.xy())
        .and_then(|p| navmesh.transformed_path(p_transform.translation, p.position().extend(1.0)))
    else {
        warn!("No path found");
        // TODO: add sound and/or visual cue
        return;
    };

    // TODO: Check mining range in order to pick closest point
    // TODO: check tile position to have closest_tile
    // Use the rock surface and not the tile clicked. Use an observer on rocks

    let mut command_queue = CommandQueue::new(vec![]);
    command_queue.add(PlayerCommand::GoTo(FollowPath::new(path.path)));
    command_queue.add(PlayerCommand::Mine(MiningOrder::from(
        trigger.event_target(),
    )));
    commands
        .entity(p_entity)
        .insert(command_queue)
        .trigger(NextCommand::from);
}

fn surface_distance(miner: Vec2, rock_center: Vec2, footprint: Vec2) -> f32 {
    let d = (miner - rock_center).abs() - footprint / 2.0; // (16,16) for a 2x2
    d.max(Vec2::ZERO).length()
}

fn process_mining_order(
    mut commands: Commands,
    miners: Query<(Entity, &mut MiningOrder, &MiningStats, &Transform)>,
    mut targets: Query<(&mut Health, &Transform)>,
    mut occupancy: ResMut<Occupancy>,
) {
    for (miner, order, power, m_transform) in &miners {
        let Ok((mut target, t_transform)) = targets.get_mut(order.target) else {
            continue;
        };

        let distance_to_edge = surface_distance(
            m_transform.translation.xy(),
            t_transform.translation.xy(),
            Vec2::splat(16.0),
        );

        let target_in_range = distance_to_edge <= power.range;

        if target_in_range {
            info!("Damage rock for {power:?}");
            target.take_damage(power);

            if target.is_dead() {
                info!("Target is dead");
                commands
                    .entity(miner)
                    .remove::<MiningOrder>()
                    .trigger(NextCommand);
                commands.entity(order.target).despawn();
                occupancy.free(order.target);
            }
        }
    }
}

pub fn display_mining_range(miners: Query<(&Transform, &MiningStats)>, mut gizmos: Gizmos) {
    for (position, stats) in &miners {
        gizmos.circle_2d(
            position.translation.xy(),
            stats.range,
            palettes::tailwind::RED_500,
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use crate::demo::commands::mining::surface_distance;

    #[test]
    fn distance_to_surface() {
        let agent_pos = Vec2::X;
        let target_pos = Vec2::NEG_X;
        let target_footprint = Vec2::ONE;

        let distance = surface_distance(agent_pos, target_pos, target_footprint);

        assert_eq!(distance, 1.5);
    }
}
