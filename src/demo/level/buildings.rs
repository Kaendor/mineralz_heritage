use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::demo::{
    input::Action,
    level::LevelAssets,
    player::{Cursor, Player},
};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Building {
    Rock,
    Wall,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct PreparedBuilding {
    current: Option<Building>,
}

impl PreparedBuilding {
    pub fn current(&self) -> Option<Building> {
        self.current
    }

    pub fn next(&mut self) {
        self.current = match self.current {
            Some(building) => match building {
                Building::Rock => None,
                Building::Wall => Some(Building::Rock),
            },
            None => Some(Building::Wall),
        }
    }
}

pub fn change_prepared_building(
    mut player: Single<(&ActionState<Action>, &mut PreparedBuilding), With<Player>>,
) {
    if player.0.just_pressed(&Action::ChangePreparedBuilding) {
        player.1.next();
    }
}

pub fn display_prepared_building(
    mut commands: Commands,
    player: Single<Ref<PreparedBuilding>, With<Player>>,
    cursor: Single<Entity, With<Cursor>>,
    assets: Res<LevelAssets>,
) {
    if player.is_changed() {
        let mut e_cmd = commands.entity(*cursor);
        match player.current() {
            Some(building) => match building {
                Building::Rock => {
                    e_cmd.insert((Sprite {
                        image: assets.rock.clone(),
                        color: Color::WHITE.with_alpha(0.30),
                        ..default()
                    },));
                }
                Building::Wall => {
                    e_cmd.insert((Sprite {
                        image: assets.wall.clone(),
                        color: Color::WHITE.with_alpha(0.30),
                        ..default()
                    },));
                }
            },
            None => {
                e_cmd.remove::<Sprite>();
            }
        }
    }
}
