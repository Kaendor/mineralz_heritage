//! Spawn the main level.

use bevy::{color::palettes, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_ecs_tilemap::{
    TilemapBundle,
    anchor::TilemapAnchor,
    map::{TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TileColor, TilePos, TileStorage},
};
use vleue_navigator::{
    NavMeshDebug, Triangulation,
    prelude::{NavMeshSettings, NavMeshUpdateMode},
};

use crate::{
    demo::{
        input::{on_left_click_spawn_rock, on_right_click_request_actions},
        level::map::Occupancy,
        player::{PlayerAssets, player},
    },
    screens::Screen,
};

pub mod enemies;
pub mod map;

pub(super) fn plugin(_app: &mut App) {}

#[derive(Resource, Clone, Reflect, AssetCollection)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[asset(path = "audio/music/Fluffing A Duck.ogg")]
    music: Handle<AudioSource>,

    #[asset(path = "images/tiles.png")]
    #[asset(image(sampler(filter = nearest)))]
    tiles: Handle<Image>,

    #[asset(path = "images/rock.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub rock: Handle<Image>,

    #[asset(path = "images/enemy_sprite.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub enemy: Handle<Image>,
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
) {
    let map_size = TilemapSize { x: 32, y: 32 };
    let tilemap = commands.spawn_empty().id();

    let mut storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let pos = TilePos { x, y };
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: pos,
                        tilemap_id: bevy_ecs_tilemap::map::TilemapId(tilemap),
                        ..default()
                    },
                    Pickable::default(),
                ))
                .observe(on_left_click_spawn_rock)
                .observe(on_right_click_request_actions)
                .observe(recolor_on::<Pointer<Over>>(Color::BLACK))
                .observe(recolor_on::<Pointer<Out>>(Color::WHITE))
                .observe(recolor_on::<Pointer<Release>>(Color::WHITE))
                .id();

            storage.set(&pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    let player_tile_position = TilePos::new(0, 0);

    let player_world_position = player_tile_position.center_in_world(
        &map_size,
        &grid_size,
        &tile_size,
        &map_type,
        &TilemapAnchor::Center,
    );
    let mut occupancy = Occupancy::new(map_size);

    let level = commands
        .spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    let player = commands
        .spawn((
            player(&player_assets),
            player_tile_position,
            Transform::from_translation(player_world_position.extend(0.1))
                .with_scale(Vec3::splat(10.0)),
            ChildOf(level),
        ))
        .id();

    occupancy.occupy(player_tile_position, UVec2::ONE, player);
    commands.insert_resource(occupancy);

    // Outer boundary of the navmesh: the tilemap's AABB in navmesh-local space.
    // `center_in_world` returns tile *centers* in the tilemap's local space (which
    // is the navmesh space, since the tilemap transform isn't applied here), so
    // push the two opposite corner tiles out by half a tile to reach the borders.
    let half_tile = Vec2::new(tile_size.x, tile_size.y) / 2.0;
    let min = TilePos::new(0, 0).center_in_world(
        &map_size,
        &grid_size,
        &tile_size,
        &map_type,
        &TilemapAnchor::Center,
    ) - half_tile;
    let max = TilePos::new(map_size.x - 1, map_size.y - 1).center_in_world(
        &map_size,
        &grid_size,
        &tile_size,
        &map_type,
        &TilemapAnchor::Center,
    ) + half_tile;
    // Counter-clockwise winding for the outer boundary.
    let edges = vec![
        Vec2::new(min.x, min.y),
        Vec2::new(max.x, min.y),
        Vec2::new(max.x, max.y),
        Vec2::new(min.x, max.y),
    ];

    commands.entity(tilemap).insert((
        ChildOf(level),
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage,
            texture: TilemapTexture::Single(level_assets.tiles.clone()),
            tile_size,
            anchor: TilemapAnchor::Center,
            ..default()
        },
        NavMeshSettings {
            fixed: Triangulation::from_outer_edges(&edges),
            agent_radius: tile_size.x / 3.0,
            simplify: 0.02,
            merge_steps: 2,
            ..default()
        },
        NavMeshUpdateMode::Direct,
        NavMeshDebug(palettes::tailwind::CYAN_500.into()),
    ));
}

fn recolor_on<E: EntityEvent + Clone + Reflect>(
    color: Color,
) -> impl Fn(On<E>, Query<&mut TileColor>) {
    move |ev, mut tile_q| {
        if let Ok(mut tile_color) = tile_q.get_mut(ev.event_target()) {
            tile_color.0 = color;
        }
    }
}
