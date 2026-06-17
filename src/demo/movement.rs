use bevy::prelude::*;
use vleue_navigator::prelude::{ObstacleSource, PrimitiveObstacle};

pub(super) fn plugin(_app: &mut App) {}

/// World size of a single tile, in pixels. Must match the tilemap's
/// `TilemapTileSize` (see `spawn_level`).
const TILE_SIZE: f32 = 16.0;
