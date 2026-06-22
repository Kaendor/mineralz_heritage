use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rstar::{RTree, primitives::GeomWithData};

use crate::{
    demo::commands::path_following::{MovementController, Obstacle},
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup_spatial_index);
    app.add_systems(
        Update,
        update_spatial_index
            .run_if(resource_exists::<SpatialIndex>)
            .run_if(spatial_index_dirty),
    );
}

pub type SpatialEntity = GeomWithData<[f32; 2], Entity>;

// TODO: Maybe there should be multiple spatial indices for each usages. When a monster needs the
// closest enemy for example...
#[derive(Resource, Default)]
pub struct SpatialIndex(pub rstar::RTree<SpatialEntity>);

fn setup_spatial_index(mut commands: Commands) {
    commands.init_resource::<SpatialIndex>();
}

fn spatial_index_dirty(
    changed: Query<
        (),
        (
            Changed<Transform>,
            Or<(With<Obstacle>, With<MovementController>)>,
        ),
    >,
    mut removed_obstacles: RemovedComponents<Obstacle>,
    mut removed_movers: RemovedComponents<MovementController>,
) -> bool {
    !changed.is_empty()
        || removed_obstacles.read().next().is_some()
        || removed_movers.read().next().is_some()
}

fn update_spatial_index(
    mut index: ResMut<SpatialIndex>,
    obstacles: Query<(Entity, &Transform), With<Obstacle>>,
    moving_entities: Query<(Entity, &Transform), With<MovementController>>,
) {
    let spatial_entities = obstacles
        .iter()
        .chain(moving_entities)
        .map(|(e, t)| GeomWithData::new(t.translation.xy().to_array(), e))
        .collect();
    let tree = RTree::bulk_load(spatial_entities);

    index.0 = tree;
}

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
