//! Spawn the main level.

use bevy::{color::palettes, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_ecs_tilemap::{
    TilemapBundle,
    anchor::TilemapAnchor,
    map::{TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TileColor, TilePos, TileStorage},
};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {}

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
pub fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
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
                .observe(recolor_on::<Pointer<Release>>(Color::WHITE))
                .observe(on_right_click)
                .observe(on_left_click)
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

fn on_left_click(trigger: On<Pointer<Press>>, mut tiles: Query<&mut TileColor>) {
    let pointer = trigger.event();
    match pointer.button {
        PointerButton::Primary => {
            if let Ok(mut tile_color) = tiles.get_mut(trigger.event_target()) {
                tile_color.0 = palettes::tailwind::TEAL_500.into();
            }
        }
        PointerButton::Middle => {}
        PointerButton::Secondary => {}
    }
}
fn on_right_click(trigger: On<Pointer<Press>>, mut tiles: Query<&mut TileColor>) {
    let pointer = trigger.event();
    match pointer.button {
        PointerButton::Secondary => {
            if let Ok(mut tile_color) = tiles.get_mut(trigger.event_target()) {
                tile_color.0 = palettes::tailwind::RED_500.into();
            }
        }
        PointerButton::Primary => {}
        PointerButton::Middle => {}
    }
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
