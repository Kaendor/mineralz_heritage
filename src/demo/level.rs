//! Spawn the main level.

use bevy::{
    color::palettes,
    picking::{
        PickingSystems,
        backend::{HitData, PointerHits},
        pointer::PointerId,
    },
    prelude::*,
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_ecs_tilemap::{
    TilemapBundle, TilemapPlugin,
    anchor::TilemapAnchor,
    map::{TilemapGridSize, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TileColor, TilePos, TileStorage},
};

use crate::{
    demo::player::{CursorPos, PlayerAssets},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PreUpdate,
        tilemap_picking_hits.in_set(PickingSystems::Backend),
    );
    app.add_plugins(TilemapPlugin);
}

#[derive(Resource, Clone, Reflect, AssetCollection)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[asset(path = "audio/music/Fluffing A Duck.ogg")]
    music: Handle<AudioSource>,

    #[asset(path = "images/tiles.png")]
    #[asset(image(sampler(filter = nearest)))]
    tiles: Handle<Image>,
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
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
                .observe(recolor_on::<Pointer<Over>>(Color::BLACK))
                .observe(recolor_on::<Pointer<Out>>(Color::WHITE))
                .id();

            storage.set(&pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    let level = commands
        .spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

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
    ));
}

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

        if let Some(tile_pos) = TilePos::from_world_pos(
            &cursor_in_map_pos,
            map_size,
            grid_size,
            tile_size,
            map_type,
            anchor,
        ) {
            if let Some(tile) = tile_storage.get(&tile_pos) {
                let hit_pos_world = cursor_pos.0.extend(map_transform.translation.z);

                // Transform the world-space hit into camera (view) space.
                let hit_pos_cam = cam_transform
                    .affine()
                    .inverse()
                    .transform_point3(hit_pos_world);

                // Depth measured from the camera's near plane to the hit, in world units.
                // The camera looks down -Z in view space, so hit_pos_cam.z is negative.
                let depth = -near_plane - hit_pos_cam.z;

                picks.push((
                    tile,
                    HitData::new(cam_entity, depth, Some(hit_pos_world), None),
                ));
            }
        }
    }
    pointer_hits_writer.write(PointerHits::new(
        PointerId::Mouse,
        picks,
        camera.order as f32,
    ));
}

fn recolor_on<E: EntityEvent + Clone + Reflect>(
    color: Color,
) -> impl Fn(On<E>, Query<&mut TileColor>) {
    move |ev, mut tile_q| {
        info!("picking event");

        if let Ok(mut tile_color) = tile_q.get_mut(ev.event_target()) {
            tile_color.0 = color;
        }
    }
}
