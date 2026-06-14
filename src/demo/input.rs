use bevy::{
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
    helpers::square_grid::neighbors::Neighbors,
    map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType},
    tiles::{TileColor, TilePos, TileStorage},
};
use bevy_pancam::PanCamPlugin;
use leafwing_input_manager::Actionlike;
use pathfinding::prelude::astar;

use crate::demo::{
    level::{LevelAssets, map::Occupancy},
    movement::FollowPath,
    player::{CursorPos, Player},
};

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    GoTo,
    PickTile,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        PreUpdate,
        tilemap_picking_hits.in_set(PickingSystems::Backend),
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

pub fn on_left_click_spawn_rock(
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

    // The clicked tile is the bottom-left of the 2x2 footprint, covering
    // (x, y), (x+1, y), (x, y+1), (x+1, y+1). A 2x2 object is centered on the
    // corner shared by those tiles, i.e. half a grid cell up and to the right
    // of the clicked tile's center.
    let tile_center =
        tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, map_anchor);
    let rock_world_position = tile_center + Vec2::new(grid_size.x, grid_size.y) / 2.0;

    let rock_entity = commands
        .spawn((
            Name::new("Rock"),
            *tile_pos,
            Sprite::from_image(assets.rock.clone()),
            Transform::from_translation(rock_world_position.extend(0.2)),
        ))
        .id();
    occupancy.occupy(*tile_pos, rock_footprint, rock_entity);
}

pub fn on_right_click_move_player(
    trigger: On<Pointer<Press>>,
    mut commands: Commands,
    mut tiles: Query<(&mut TileColor, &TilePos)>,
    player: Query<(Entity, &TilePos), With<Player>>,
    tilemap_q: Query<&TilemapSize>,
    occupancy: Res<Occupancy>,
) {
    let pointer = trigger.event();
    let Ok((player_entity, from)) = player.single() else {
        return;
    };
    let Ok(map_size) = tilemap_q.single() else {
        return;
    };

    let PointerButton::Secondary = pointer.button else {
        return;
    };

    let Ok((mut tile_color, tile_pos)) = tiles.get_mut(trigger.event_target()) else {
        return;
    };

    tile_color.0 = palettes::tailwind::RED_500.into();

    let path = create_path(from, tile_pos, map_size, &occupancy);

    if let Some(path) = path {
        commands
            .entity(player_entity)
            .insert(FollowPath::new(path.0));
    }
}

fn create_path(
    from: &TilePos,
    target: &TilePos,
    map_size: &TilemapSize,
    occupancy: &Occupancy,
) -> Option<(Vec<TilePos>, u32)> {
    let is_tile_free = occupancy.is_free_at(*target, UVec2::ONE);

    let successors = |p: &TilePos| -> Vec<(TilePos, u32)> {
        Neighbors::get_square_neighboring_positions(p, map_size, false)
            .iter()
            .filter(|t| occupancy.is_free_at(**t, UVec2::ONE))
            .map(|a| (*a, 1))
            .collect()
    };
    let heuristic = |p: &TilePos| -> u32 {
        let dist = UVec2::from(*p).chebyshev_distance(UVec2::from(*target));
        if is_tile_free {
            dist
        } else {
            dist.saturating_sub(1)
        }
    };

    let success = |p: &TilePos| -> bool {
        if is_tile_free {
            *p == *target
        } else {
            UVec2::from(*p).chebyshev_distance(UVec2::from(*target)) == 1
        }
    };

    astar(from, successors, heuristic, success)
}
