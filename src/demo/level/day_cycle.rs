use bevy::prelude::*;
use bevy_ecs_tilemap::{
    anchor::TilemapAnchor,
    map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType},
    tiles::TilePos,
};
use leafwing_input_manager::prelude::ActionState;

use crate::demo::{
    ai::PickNextTarget,
    input::Action,
    level::{LevelAssets, enemies::basic_enemy},
    player::Player,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, trigger_night);
    app.add_observer(on_night_start_spawn_enemies);
}

#[derive(Event)]
pub struct NightStart;

pub fn on_night_start_spawn_enemies(
    _trigger: On<NightStart>,
    mut commands: Commands,
    assets: Res<LevelAssets>,
    tilemap_q: Single<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    )>,
) {
    let (map_size, grid_size, tile_size, map_type, anchor) = *tilemap_q;
    info!("Spawn bad guys");
    let enemy_tile_position = TilePos::new(20, 20);

    let enemy_world_position =
        enemy_tile_position.center_in_world(map_size, grid_size, tile_size, map_type, anchor);

    commands
        .spawn((
            basic_enemy(&assets),
            Transform::from_translation(enemy_world_position.extend(0.1)),
        ))
        .trigger(|e| PickNextTarget { entity: e });
}

pub fn trigger_night(mut commands: Commands, player: Single<&ActionState<Action>, With<Player>>) {
    if player.just_pressed(&Action::SpawnEnemies) {
        info!("Night Start triggered manually!");
        commands.trigger(NightStart);
    }
}
