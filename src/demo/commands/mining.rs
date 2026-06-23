use std::time::Duration;

use bevy::{color::palettes, prelude::*};
use vleue_navigator::{NavMesh, prelude::ManagedNavMesh};

use crate::demo::{
    ai::{Ai, PickNextTarget},
    commands::{CommandQueue, EntityCommand, NextCommand, path_following::FollowPath},
    level::map::Occupancy,
    player::Player,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        process_attack_order.run_if(resource_exists::<Occupancy>),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AttackOrder {
    pub target: Entity,
}

#[derive(Component, Debug)]
/// in HP by Seconds
pub struct AttackStats {
    power: f32,
    range: f32,
    timer: Timer,
}

impl AttackStats {
    /// Attack timer start complete in order to be able to attack instantly the first time
    pub fn new(power: f32, range: f32, rate: Duration) -> Self {
        let mut timer = Timer::new(rate, TimerMode::Repeating);
        timer.finish();

        Self {
            power,
            range,
            timer,
        }
    }

    pub fn range(&self) -> f32 {
        self.range
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

    pub fn take_damage(&mut self, damages: &AttackStats) {
        self.current -= damages.power
    }
}

impl From<Entity> for AttackOrder {
    fn from(value: Entity) -> Self {
        Self { target: value }
    }
}

pub fn on_right_click_request_mining(
    trigger: On<Pointer<Press>>,
    mut commands: Commands,
    player: Query<(Entity, &Transform, &AttackStats), With<Player>>,
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
    let Ok((p_entity, p_transform, mining_stats)) = player.single() else {
        return;
    };
    let Ok(r_transform) = transforms.get(trigger.event_target()) else {
        return;
    };

    let mut command_queue = CommandQueue::new(vec![]);
    // FIXME: If already in range, mine
    if let Some(path) = navmesh
        .get()
        .get_closest_point_towards(r_transform.translation.xy(), p_transform.translation.xy())
        .and_then(|p| navmesh.transformed_path(p_transform.translation, p.position().extend(1.0)))
    {
        command_queue.add(EntityCommand::GoTo(FollowPath::new(path.path)));
    } else {
        warn!("No path found");
        // TODO: add sound and/or visual cue
    }

    if p_transform
        .translation
        .xy()
        .distance(r_transform.translation.xy())
        <= mining_stats.range()
    {
        command_queue.add(EntityCommand::Attack(AttackOrder::from(
            trigger.event_target(),
        )));
    }

    if !command_queue.is_empty() {
        commands
            .entity(p_entity)
            .insert(command_queue)
            .trigger(NextCommand::from);
    }
}

pub(crate) fn surface_distance(miner: Vec2, rock_center: Vec2, footprint: Vec2) -> f32 {
    let d = (miner - rock_center).abs() - footprint / 2.0; // (16,16) for a 2x2
    d.max(Vec2::ZERO).length()
}

fn process_attack_order(
    mut commands: Commands,
    mut miners: Query<(
        Entity,
        &mut AttackOrder,
        &mut AttackStats,
        &Transform,
        Option<&Ai>,
    )>,
    mut targets: Query<(&mut Health, &Transform)>,
    mut occupancy: ResMut<Occupancy>,
    time: Res<Time>,
) {
    for (miner, order, mut stats, m_transform, is_ai) in &mut miners {
        stats.timer.tick(time.delta());

        if !stats.timer.just_finished() {
            continue;
        }

        let Ok((mut target, t_transform)) = targets.get_mut(order.target) else {
            continue;
        };

        let distance_to_edge = surface_distance(
            m_transform.translation.xy(),
            t_transform.translation.xy(),
            Vec2::splat(16.0),
        );

        let target_in_range = distance_to_edge <= stats.range;

        if target_in_range {
            info!("Damage rock for {stats:?}");
            target.take_damage(&stats);

            if target.is_dead() {
                info!("Target is dead");
                commands
                    .entity(miner)
                    .remove::<AttackOrder>()
                    .trigger(NextCommand);
                commands.entity(order.target).despawn();
                occupancy.free(order.target);

                if is_ai.is_some() {
                    info!("AI must pick another target");

                    commands
                        .entity(miner)
                        .trigger(|e| PickNextTarget { entity: e });
                }
            }
        }
    }
}

pub fn display_mining_range(miners: Query<(&Transform, &AttackStats)>, mut gizmos: Gizmos) {
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
