use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{prelude::{EventReader, ResMut, Query, Commands, EventWriter, Entity, BuildChildren, Transform, default, SpatialBundle, IVec2, DespawnRecursiveExt, OrthographicProjection, Changed, GlobalTransform, With, Res}, math::Vec3Swizzles};
use bevy_ecs_tilemap::{tiles::{TilePos, TileTexture, TileStorage, TileBundle}, prelude::{TilemapType, get_neighboring_pos, TilemapGridSize, TilemapTexture, TilemapTileSize, TilemapSpacing, TilemapId}, TilemapBundle};
use iyes_loopless::state::NextState;

use crate::{util::{self, IRect, FRect}, block::Block, world_generator::{Tile, WORLD_SIZE_X, WORLD_SIZE_Y, generate, Wall}, plugins::{inventory::Inventory, world::{CHUNK_SIZE, TILE_SIZE}, assets::{BlockAssets, WallAssets}, camera::MainCamera}, state::GameState};

use super::{get_chunk_position, CHUNK_SIZE_U, MAP_SIZE, TileChunk, UpdateNeighborsEvent, WorldData, BlockPlaceEvent, get_tile_sprite_index, WallChunk, WALL_SIZE, CHUNKMAP_SIZE, Chunk, get_camera_fov, ChunkManager, get_wall_sprite_index};

pub fn spawn_terrain(mut commands: Commands) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    // let seed = current_time.as_millis() as u32;
    let seed = 2404226870;

    println!("The world's seed is {}", seed);

    println!("Generating world...");
    let tiles = generate(seed);

    commands.insert_resource(WorldData {
        width: tiles.ncols() as u16,
        height: tiles.nrows() as u16,
        tiles,
    });

    commands.insert_resource(NextState(GameState::InGame));
}

pub fn spawn_tile(
    commands: &mut Commands,
    tile: Tile,
    tile_pos: TilePos,
    tilemap_entity: Entity
) -> Entity {
    let index = util::get_tile_start_index(tile.tile_type) + get_tile_sprite_index(tile.neighbors);

    commands
        .spawn()
        .insert_bundle(TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            texture: TileTexture(index),
            ..default()
        })
        .insert(tile.tile_type)
        .id()
}

pub fn spawn_wall(
    commands: &mut Commands,
    wall: Wall,
    wall_pos: TilePos,
    wallmap_entity: Entity
) -> Entity {
    let index = util::get_wall_start_index(wall.wall_type) + get_wall_sprite_index(wall.neighbors);

    commands
        .spawn()
        .insert_bundle(TileBundle {
            position: wall_pos,
            tilemap_id: TilemapId(wallmap_entity),
            texture: TileTexture(index),
            ..default()
        })
        .id()
}

pub fn spawn_chunks(
    mut commands: Commands,
    block_assets: Res<BlockAssets>,
    wall_assets: Res<WallAssets>,
    world_data: Res<WorldData>,
    mut chunk_manager: ResMut<ChunkManager>,
    camera_query: Query<
        (&GlobalTransform, &OrthographicProjection),
        (With<MainCamera>, Changed<GlobalTransform>),
    >,
) {
    if let Ok((camera_transform, projection)) = camera_query.get_single() {
        let camera_fov = get_camera_fov(camera_transform.translation().xy(), projection);
        let camera_chunk_pos = get_chunk_position_by_camera_fov(camera_fov);

        for y in camera_chunk_pos.bottom..=camera_chunk_pos.top {
            for x in camera_chunk_pos.left..=camera_chunk_pos.right {
                let chunk_pos = IVec2::new(x, y);

                if !chunk_manager.spawned_chunks.contains(&chunk_pos) {
                    chunk_manager.spawned_chunks.insert(chunk_pos);
                    spawn_chunk(&mut commands, &block_assets, &wall_assets, &world_data, chunk_pos);
                }
            }
        }
    }
}

pub fn despawn_chunks(
    mut commands: Commands,
    chunks: Query<(Entity, &Chunk)>,
    mut chunk_manager: ResMut<ChunkManager>,
    camera_query: Query<
        (&GlobalTransform, &OrthographicProjection),
        (With<MainCamera>, Changed<GlobalTransform>),
    >,
) {
    if let Ok((camera_transform, projection)) = camera_query.get_single() {
        let camera_fov = get_camera_fov(camera_transform.translation().xy(), projection);
        let camera_chunk_pos = get_chunk_position_by_camera_fov(camera_fov);

        for (entity, Chunk { pos: chunk_pos }) in chunks.iter() {
            if (chunk_pos.x < camera_chunk_pos.left || chunk_pos.x > camera_chunk_pos.right) ||
               (chunk_pos.y < camera_chunk_pos.bottom || chunk_pos.y > camera_chunk_pos.top) 
            {
                chunk_manager.spawned_chunks.remove(&chunk_pos);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub fn spawn_chunk(
    commands: &mut Commands,
    block_assets: &BlockAssets,
    wall_assets: &WallAssets,
    world_data: &WorldData,
    chunk_pos: IVec2,
) { 
    let chunk = commands.spawn()
        .insert(Chunk { pos: chunk_pos })
        .insert_bundle(SpatialBundle {
            transform: Transform::from_xyz(chunk_pos.x as f32 * CHUNK_SIZE * TILE_SIZE, chunk_pos.y as f32 * CHUNK_SIZE * TILE_SIZE, 0.),
            ..default()
        })
        .id();

    let tilemap_entity = commands.spawn().id();
    let mut tile_storage = TileStorage::empty(CHUNKMAP_SIZE);

    let wallmap_entity = commands.spawn().id();
    let mut wall_storage = TileStorage::empty(CHUNKMAP_SIZE);

    for y in 0..CHUNK_SIZE_U {
        for x in 0..CHUNK_SIZE_U {
            let tile_pos = TilePos { 
                x, 
                y: y
            };

            let map_tile_pos = TilePos {
                x: (chunk_pos.x as f32 * CHUNK_SIZE) as u32 + x,
                y: WORLD_SIZE_Y as u32 - (chunk_pos.y as f32 * CHUNK_SIZE) as u32 - y - 1
            };

            if let Some(cell) = world_data.get_cell(map_tile_pos) {
                if let Some(tile) = cell.tile {
                    let tile_entity = spawn_tile(commands, tile, tile_pos, tilemap_entity);

                    commands.entity(tilemap_entity).add_child(tile_entity);
                    tile_storage.set(&tile_pos, tile_entity);
                }

                if let Some(wall) = cell.wall {
                    let wall_entity = spawn_wall(commands, wall, tile_pos, wallmap_entity);

                    commands.entity(wallmap_entity).add_child(wall_entity);
                    wall_storage.set(&tile_pos, wall_entity);
                }
            }
        }
    }

    commands
        .entity(tilemap_entity)
        .insert(TileChunk { pos: chunk_pos })
        .insert_bundle(TilemapBundle {
            grid_size: TilemapGridSize {
                x: TILE_SIZE,
                y: TILE_SIZE,
            },
            size: CHUNKMAP_SIZE,
            storage: tile_storage,
            texture: TilemapTexture::Single(block_assets.tiles.clone()),
            tile_size: TilemapTileSize {
                x: TILE_SIZE,
                y: TILE_SIZE,
            },
            spacing: TilemapSpacing {
                x: 2.,
                y: 2.
            },
            transform: Transform::from_xyz(0., 0., 2.),
            ..Default::default()
        });

    commands
        .entity(wallmap_entity)
        .insert(WallChunk { pos: chunk_pos })
        .insert_bundle(TilemapBundle {
            grid_size: TilemapGridSize {
                x: TILE_SIZE,
                y: TILE_SIZE,
            },
            size: CHUNKMAP_SIZE,
            storage: wall_storage,
            texture: TilemapTexture::Single(wall_assets.walls.clone()),
            tile_size: TilemapTileSize {
                x: WALL_SIZE,
                y: WALL_SIZE,
            },
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        });

    commands.entity(chunk).push_children(&[tilemap_entity, wallmap_entity]);
}

pub fn get_chunk_position_by_camera_fov(camera_fov: FRect) -> IRect {
    let mut rect = IRect { 
        left: (camera_fov.left / (CHUNK_SIZE * TILE_SIZE)).floor() as i32, 
        right: (camera_fov.right / (CHUNK_SIZE * TILE_SIZE)).ceil() as i32, 
        bottom: (camera_fov.bottom / (CHUNK_SIZE * TILE_SIZE)).floor() as i32, 
        top: (camera_fov.top / (CHUNK_SIZE * TILE_SIZE)).ceil() as i32,
    };

    const MAX_CHUNK_X: i32 = (WORLD_SIZE_X as f32 / CHUNK_SIZE) as i32;
    const MAX_CHUNK_Y: i32 = (WORLD_SIZE_Y as f32 / CHUNK_SIZE) as i32;

    if rect.left < 0 {
        rect.left = 0;
    }

    if rect.right > MAX_CHUNK_X {
        rect.right = MAX_CHUNK_X;
    }

    if rect.top > MAX_CHUNK_Y {
        rect.top = MAX_CHUNK_Y;
    }
    
    if rect.bottom < 0 {
        rect.bottom = 0;
    }

    rect
}

pub fn handle_block_place(
    mut commands: Commands,
    mut world_data: ResMut<WorldData>,
    mut events: EventReader<BlockPlaceEvent>,
    mut update_neighbors_ew: EventWriter<UpdateNeighborsEvent>,
    mut inventory: ResMut<Inventory>,
    mut chunks: Query<(&TileChunk, &mut TileStorage, Entity)>
) {
    for event in events.iter() {
        let map_tile_pos = TilePos { x: event.tile_pos.x as u32, y: (WORLD_SIZE_Y as u32).checked_sub(event.tile_pos.y as u32 + 1).unwrap_or(0) };

        let neighbors = world_data.get_neighbours(map_tile_pos);
        let tile = Tile { tile_type: event.block, neighbors };

        if world_data.get_tile(map_tile_pos).is_none() {
            let cell = world_data.get_cell_mut(map_tile_pos).unwrap();
            cell.tile = Some(tile);

            let chunk_pos = get_chunk_position(map_tile_pos);

            let chunk_tile_pos = TilePos {
                x: (map_tile_pos.x % 25),
                y: CHUNK_SIZE_U - 1 - (map_tile_pos.y % 25),
            };

            chunks.for_each_mut(|(chunk, mut tile_storage, tilemap_entity)| {
                if chunk.pos == chunk_pos {
                    let tile_entity = spawn_tile(&mut commands, tile, chunk_tile_pos, tilemap_entity);

                    commands.entity(tilemap_entity).add_child(tile_entity);
                    tile_storage.set(&chunk_tile_pos, tile_entity);
                }
            });

            inventory.consume_item(event.inventory_item_index);

            update_neighbors_ew.send(UpdateNeighborsEvent { 
                tile_pos: map_tile_pos,
                chunk_tile_pos,
                chunk_pos, 
                tile 
            });
        }
    }
}

pub fn update_neighbors(
    mut world_data: ResMut<WorldData>,
    mut events: EventReader<UpdateNeighborsEvent>,
    mut tiles: Query<(&mut TileTexture, &Block)>,
    mut chunks: Query<(&TileChunk, &TileStorage)>
) {
    let map_type = TilemapType::Square { diagonal_neighbors: false };

    for event in events.iter() {
        let tile_pos = event.tile_pos;
        let neighbor_positions = get_neighboring_pos(&tile_pos, &MAP_SIZE, &map_type);

        for pos in neighbor_positions.into_iter() {
            let neighbors = world_data.get_neighbours(pos);

            if let Some(mut tile) = world_data.get_tile_mut(pos) {
                tile.neighbors = tile.neighbors.or(neighbors);
            }

            let chunk_pos = get_chunk_position(pos);
            let chunk_tile_pos = TilePos {
                x: (pos.x % 25),
                y: CHUNK_SIZE_U - 1 - (pos.y % 25),
            };

            chunks.for_each_mut(|(chunk, tile_storage)| {
                if chunk.pos == chunk_pos {
                    if let Some(entity) = tile_storage.get(&chunk_tile_pos) {
                        if let Ok((mut tile_texture, block)) = tiles.get_mut(entity) {
                            tile_texture.0 = util::get_tile_start_index(*block) + get_tile_sprite_index(neighbors);
                        }
                    }
                }
            });
        }
    }
}
