use bevy::prelude::*;

use crate::demo::{commands::NextCommand, level::map::Occupancy};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, process_mining_order);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
/// TODO: check is rock is near before performing mining operation
pub struct MiningOrder {
    pub target: Entity,
}

#[derive(Component, Debug)]
/// in HP by Seconds
pub struct MiningPower(pub f32);

impl MiningPower {
    pub fn new(power: f32) -> Self {
        Self(power)
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

    pub fn take_damage(&mut self, damages: &MiningPower) {
        self.current -= damages.0
    }
}

impl From<Entity> for MiningOrder {
    fn from(value: Entity) -> Self {
        Self { target: value }
    }
}

fn process_mining_order(
    mut commands: Commands,
    miners: Query<(Entity, &mut MiningOrder, &MiningPower)>,
    mut targets: Query<&mut Health>,
    mut occupancy: ResMut<Occupancy>,
) {
    for (miner, order, power) in &miners {
        // TODO: The order is over when the target is gone
        // The target is gone, when its hp are equal or below 0
        let Ok(mut target) = targets.get_mut(order.target) else {
            continue;
        };

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
            // TODO: update occupancy
        }
    }
}
