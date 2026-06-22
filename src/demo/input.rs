use bevy::{
    camera::primitives::Aabb,
    color::palettes,
    picking::{
        PickingSystems,
        backend::{HitData, PointerHits},
        pointer::PointerId,
    },
    prelude::*,
};
use bevy_ecs_tilemap::{
    TilemapPlugin,
    anchor::TilemapAnchor,
    map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType},
    tiles::{TileColor, TilePos, TileStorage},
};
use bevy_pancam::PanCamPlugin;
use leafwing_input_manager::Actionlike;
use vleue_navigator::{NavMesh, prelude::ManagedNavMesh};

use crate::demo::{
    commands::{
        CommandQueue, EntityCommand, NextCommand,
        mining::{Health, on_right_click_request_mining},
        path_following::{FollowPath, Obstacle},
    },
    level::{
        LevelAssets,
        buildings::{
            Building, PreparedBuilding, change_prepared_building, display_prepared_building,
        },
        map::Occupancy,
    },
    player::{CursorPos, Faction, Player},
};

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    GoTo,
    PickTile,
    SpawnEnemies,
    ChangePreparedBuilding,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        PreUpdate,
        tilemap_picking_hits.in_set(PickingSystems::Backend),
    );
    app.add_systems(
        Update,
        (change_prepared_building, display_prepared_building),
    );
    app.add_plugins((TilemapPlugin, PanCamPlugin));
}

/// Picking backend for tilemaps
pub fn tilemap_picking_hits(
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TileStorage,
        &Transform,
        &TilemapAnchor,
    )>,
    cursor_pos: Res<CursorPos>,
    mut pointer_hits_writer: MessageWriter<PointerHits>,
    camera_q: Query<(Entity, &Camera, &GlobalTransform, &Projection)>,
) {
    let mut picks = vec![];
    let Ok((cam_entity, camera, cam_transform, cam_projection)) = camera_q.single() else {
        return;
    };

    let hit_pos_cam = cam_transform.affine().inverse();
    let near_plane = match cam_projection {
        Projection::Perspective(p) => p.near,
        Projection::Orthographic(o) => o.near,
        Projection::Custom(_) => 0.1,
    };

    for (map_size, grid_size, tile_size, map_type, tile_storage, map_transform, anchor) in
        tilemap_q.iter()
    {
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos.0, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.to_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };

        let Some(tile_pos) = TilePos::from_world_pos(
            &cursor_in_map_pos,
            map_size,
            grid_size,
            tile_size,
            map_type,
            anchor,
        ) else {
            continue;
        };

        let Some(tile) = tile_storage.get(&tile_pos) else {
            continue;
        };
        let hit_pos_world = cursor_pos.0.extend(map_transform.translation.z);

        // Transform the world-space hit into camera (view) space.
        let hit_pos_cam = hit_pos_cam.transform_point3(hit_pos_world);

        // Depth measured from the camera's near plane to the hit, in world units.
        // The camera looks down -Z in view space, so hit_pos_cam.z is negative.
        let depth = -near_plane - hit_pos_cam.z;

        picks.push((
            tile,
            HitData::new(cam_entity, depth, Some(hit_pos_world), None),
        ));
    }

    if !picks.is_empty() {
        pointer_hits_writer.write(PointerHits::new(
            PointerId::Mouse,
            picks,
            camera.order as f32,
        ));
    }
}

pub fn on_left_click_spawn_prepared_building(
    trigger: On<Pointer<Press>>,
    mut commands: Commands,
    mut tiles: Query<(&mut TileColor, &TilePos)>,
    assets: Option<Res<LevelAssets>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    )>,
    mut occupancy: ResMut<Occupancy>,
    prepared_building: Single<&PreparedBuilding, With<Player>>,
) {
    let pointer = trigger.event();
    let Ok((map_size, grid_size, tile_size, map_type, map_anchor)) = tilemap_q.single() else {
        return;
    };
    let Some(assets) = assets else {
        return;
    };

    let PointerButton::Primary = pointer.button else {
        return;
    };

    let Ok((mut tile_color, tile_pos)) = tiles.get_mut(trigger.event_target()) else {
        return;
    };

    // FIXME: maybe off by one error
    let rock_footprint = UVec2::new(2, 2);

    if !occupancy.is_free_at(*tile_pos, rock_footprint) {
        warn!("Spot occupied: {tile_pos:?}");
        return;
    }

    tile_color.0 = palettes::tailwind::GREEN_500.into();

    let Some(prepared_building) = prepared_building.current() else {
        return;
    };

    let tile_center =
        tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, map_anchor);
    let building_world_position = tile_center + Vec2::new(grid_size.x, grid_size.y) / 2.0;

    match prepared_building {
        Building::Rock => {
            let rock_entity = commands
                .spawn((
                    Name::new("Rock"),
                    *tile_pos,
                    Sprite::from_image(assets.rock.clone()),
                    Transform::from_translation(building_world_position.extend(0.2)),
                    Health::new(5.0),
                    Obstacle,
                    Aabb::from_min_max(Vec3::ZERO, Vec3::splat(32.0).with_z(0.0)),
                    Pickable::default(),
                ))
                .observe(on_right_click_request_mining)
                .id();
            occupancy.occupy(*tile_pos, rock_footprint, rock_entity);
        }
        Building::Wall => {
            let wall_entity = commands
                .spawn((
                    Name::new("Wall"),
                    *tile_pos,
                    Sprite::from_image(assets.wall.clone()),
                    Transform::from_translation(building_world_position.extend(0.2)),
                    Health::new(5.0),
                    Obstacle,
                    Aabb::from_min_max(Vec3::ZERO, Vec3::splat(32.0).with_z(0.0)),
                    Pickable::default(),
                    Faction::player(),
                ))
                .id();
            occupancy.occupy(*tile_pos, rock_footprint, wall_entity);
        }
    }
    // The clicked tile is the bottom-left of the 2x2 footprint, covering
    // (x, y), (x+1, y), (x, y+1), (x+1, y+1). A 2x2 object is centered on the
    // corner shared by those tiles, i.e. half a grid cell up and to the right
    // of the clicked tile's center.
}

pub fn on_right_click_request_actions(
    trigger: On<Pointer<Press>>,
    mut commands: Commands,
    mut tiles: Query<(&mut TileColor, &TilePos)>,
    player: Query<(Entity, &Transform), With<Player>>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    )>,
    navmeshes: Res<Assets<NavMesh>>,
    navmesh: Query<&ManagedNavMesh>,
) {
    let pointer = trigger.event();
    let Ok((player_entity, from_world)) = player.single() else {
        return;
    };
    let Ok((map_size, grid_size, tile_size, map_type, anchor)) = tilemap_q.single() else {
        return;
    };
    let Ok(navmesh) = navmesh.single() else {
        return;
    };
    let Some(navmesh) = navmeshes.get(navmesh) else {
        return;
    };

    let PointerButton::Secondary = pointer.button else {
        return;
    };

    let Ok((mut tile_color, tile_pos)) = tiles.get_mut(trigger.event_target()) else {
        return;
    };

    let to = tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, anchor);

    // Check mining range in order to pick closest point
    // TODO: check tile position to have closest_tile
    // Use the rock surface and not the tile clicked. Use an observer on rocks
    let Some(a) = navmesh.get().get_closest_point(to) else {
        warn!("No closest point found");
        return;
    };
    // let from_world = from.center_in_world(map_size, grid_size, tile_size, map_type, anchor);

    tile_color.0 = palettes::tailwind::RED_500.into();

    let mut command_queue = CommandQueue::new(vec![]);

    if let Some(path) = navmesh.transformed_path(from_world.translation, a.position().extend(1.0)) {
        command_queue.add(EntityCommand::GoTo(FollowPath::new(path.path)));
    }

    if !command_queue.0.is_empty() {
        commands
            .entity(player_entity)
            .insert(command_queue)
            .trigger(NextCommand::from);
    }
}
