use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Resource)]
pub struct Occupancy {
    size: TilemapSize,
    cells: Vec<Option<Entity>>,
}

impl Occupancy {
    pub fn new(size: TilemapSize) -> Self {
        Self {
            size,
            cells: vec![None; (size.x * size.y) as usize],
        }
    }

    fn index(&self, p: &TilePos) -> usize {
        (p.y * self.size.x + p.x) as usize
    }

    fn is_free(&self, p: &TilePos) -> bool {
        p.x < self.size.x && p.y < self.size.y && self.cells[self.index(p)].is_none()
    }

    pub fn is_free_at(&self, pos: TilePos, size: UVec2) -> bool {
        (0..size.x)
            .all(|dx| (0..size.y).all(|dy| self.is_free(&TilePos::new(pos.x + dx, pos.y + dy))))
    }

    pub fn free(&mut self, entity: Entity) {
        for cell in self.cells.iter_mut() {
            if let Some(to_free) = cell
                && *to_free == entity
            {
                cell.take();
            }
        }
    }

    pub fn occupy(&mut self, pos: TilePos, size: UVec2, entity: Entity) {
        for dx in 0..size.x {
            for dy in 0..size.y {
                let occupied_tile = TilePos {
                    x: pos.x + dx,
                    y: pos.y + dy,
                };
                let insertion_index = self.index(&occupied_tile);
                self.cells[insertion_index] = Some(entity);
            }
        }
    }
}
