use bevy::{ecs::system::Command, prelude::*};
use bevy_ecs_tilemap::prelude::*;

use crate::cursor::{WorldCursor, WorldCursorSet};
use crate::util::box_lines;

use self::access::TileUpdateEvent;
use self::layer::Layer;

pub mod access;
pub mod layer;
pub mod serialization;

pub struct TileCursorPlugin;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TileCursor(pub Option<TilePos>);

impl Plugin for TileCursorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TilemapPlugin>() {
            app.add_plugins(TilemapPlugin);
        }
        app.insert_resource(TileCursor::default());
        app.add_systems(
            Update,
            update_tile_cursor.run_if(map_exists).after(WorldCursorSet),
        );
        app.add_event::<TileUpdateEvent>();
    }
}

pub fn map_exists(map_q: Query<&TileStorage>) -> bool {
    map_q.iter().len() > 0
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TileProperties {
    pub id: TileTextureIndex,
    pub flip: TileFlip,
}

pub struct SpawnMapCommand {
    size: UVec2,
    tile_size: u32,
    layer: Layer,
}

impl SpawnMapCommand {
    pub fn new(size: UVec2, tile_size: u32, layer: Layer) -> Self {
        Self {
            size,
            tile_size,
            layer,
        }
    }
}

impl Command for SpawnMapCommand {
    fn apply(self, world: &mut World) {
        let assets_server = world.resource::<AssetServer>();
        let tiles: Handle<Image> = assets_server.load("tiles.png");

        let size = TilemapSize::from(self.size);
        let storage = TileStorage::empty(size);
        let tilemap_entity = world.spawn_empty().id();

        let tile_size = TilemapTileSize::from(Vec2::splat(self.tile_size as f32));
        let grid_size = tile_size.into();
        let map_type = TilemapType::Square;

        world.entity_mut(tilemap_entity).insert((
            TilemapBundle {
                grid_size,
                map_type,
                size,
                storage,
                texture: TilemapTexture::Single(tiles),
                tile_size,
                transform: Transform::from_xyz(0., 0., self.layer.z_index()),
                ..default()
            },
            self.layer,
            Name::new(self.layer.name()),
        ));
    }
}

pub fn update_tile_cursor(
    world_cursor: Res<WorldCursor>,
    mut tile_cursor: ResMut<TileCursor>,
    tile_storage_q: Query<(&Transform, &TilemapSize, &TilemapGridSize)>,
) {
    // FIXME We should only query the currently focused layer,
    // this is especially important if at some point layers have different transforms
    if let Some((map_transform, map_size, grid_size)) = tile_storage_q.iter().next() {
        if world_cursor.is_changed() {
            let cursor_pos = **world_cursor;
            let cursor_in_map_pos: Vec2 = {
                let cursor_pos = Vec4::from((cursor_pos.extend(0.0), 1.0));
                let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
                cursor_in_map_pos.truncate().truncate()
            };

            **tile_cursor = from_world_pos(&cursor_in_map_pos, map_size, grid_size);
        }
    }
}

pub fn world_to_tile_pos(
    pos: Vec2,
    map_transform: &Transform,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
) -> Option<TilePos> {
    let in_map_pos: Vec2 = {
        let pos = Vec4::from((pos.extend(0.0), 1.0));
        let in_map_pos = map_transform.compute_matrix().inverse() * pos;
        in_map_pos.truncate().truncate()
    };

    from_world_pos(&in_map_pos, map_size, grid_size)
}

// Simplified version of TilePos;:from_world_pos with assumptions about tile and grid size
pub fn from_world_pos(
    world_pos: &Vec2,
    size: &TilemapSize,
    grid_size: &TilemapGridSize,
) -> Option<TilePos> {
    let x = ((world_pos.x / grid_size.x) + 0.5).floor() as i32;
    let y = ((world_pos.y / grid_size.y) + 0.5).floor() as i32;

    TilePos::from_i32_pair(x, y, size)
}

pub fn tile_to_world_pos(tpos: &TilePos, grid_size: &TilemapGridSize) -> Vec2 {
    Vec2::new(grid_size.x * (tpos.x as f32), grid_size.y * (tpos.y as f32))
}

pub fn draw_tile_outline(
    tile_cursor: Res<TileCursor>,
    grid_size: Query<&TilemapGridSize>,
    mut gizmos: Gizmos,
) {
    let Ok(grid_size) = grid_size.get_single() else {
        return;
    };
    if let Some(tile_cursor) = **tile_cursor {
        let wpos = tile_to_world_pos(&tile_cursor, grid_size);

        for (start, end) in box_lines(wpos, Vec2::new(16., 16.)) {
            gizmos.line_2d(start, end, Color::RED);
        }
    }
}
