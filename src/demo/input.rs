use bevy::{
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
    tiles::{TilePos, TileStorage},
};
use bevy_pancam::PanCamPlugin;
use leafwing_input_manager::Actionlike;

use crate::demo::player::CursorPos;

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
    app.add_plugins((TilemapPlugin, PanCamPlugin::default()));
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
