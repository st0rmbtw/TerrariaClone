use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{
    prelude::{
        EventReader, ResMut, Query, Commands, EventWriter, Entity, BuildChildren, Transform, 
        default, SpatialBundle, DespawnRecursiveExt, OrthographicProjection, Changed, 
        GlobalTransform, With, Res, UVec2, Audio
    }, 
    math::Vec3Swizzles
};
use bevy_ecs_tilemap::{
    tiles::{
        TilePos, TileStorage, TileBundle, TileTextureIndex
    }, 
    prelude::{
        TilemapGridSize, TilemapTexture, TilemapTileSize, 
        TilemapSpacing, TilemapId, TilemapSize
    }, 
    TilemapBundle, helpers::square_grid::neighbors::Neighbors
};
use iyes_loopless::state::NextState;
use rand::thread_rng;

use crate::{util::{FRect, URect}, plugins::{inventory::Inventory, world::{CHUNK_SIZE, TILE_SIZE, LightMap, light::generate_light_map, WorldSize}, assets::{BlockAssets, WallAssets, SoundAssets}, camera::{MainCamera, UpdateLightEvent}}, state::GameState};

use super::{get_chunk_pos, CHUNK_SIZE_U, TileChunk, UpdateNeighborsEvent, WallChunk, WALL_SIZE, CHUNKMAP_SIZE, Chunk, get_camera_fov, ChunkManager, ChunkPos, get_chunk_tile_pos, world::WorldData, block::Block, Wall, Size, BreakBlockEvent, DigBlockEvent, PlaceBlockEvent};

pub fn spawn_terrain(mut commands: Commands) {
    let _current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    // let seed = current_time.as_millis() as u32;
    let seed = 2404226870;

    println!("The world's seed is {}", seed);

    let world = super::generator::generate(seed, WorldSize::Tiny);
    let light_map = generate_light_map(&world);

    commands.insert_resource(world);

    commands.insert_resource(LightMap {
        width: light_map.ncols() as u16,
        height: light_map.nrows() as u16,
        colors: light_map
    });

    commands.insert_resource(NextState(GameState::InGame));
}

pub fn spawn_block(
    commands: &mut Commands,
    block: Block,
    tile_pos: TilePos,
    tilemap_entity: Entity,
    index: u32
) -> Entity {
    commands
        .spawn(TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            texture_index: TileTextureIndex(index),
            ..default()
        })
        .insert(block)
        .id()
}

pub fn spawn_wall(
    commands: &mut Commands,
    wall_pos: TilePos,
    wallmap_entity: Entity,
    index: u32
) -> Entity {
    commands
        .spawn(TileBundle {
            position: wall_pos,
            tilemap_id: TilemapId(wallmap_entity),
            texture_index: TileTextureIndex(index),
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
    >
) {
    if let Ok((camera_transform, projection)) = camera_query.get_single() {
        let camera_fov = get_camera_fov(camera_transform.translation().xy(), projection);
        let camera_chunk_pos = get_chunk_position_by_camera_fov(camera_fov, world_data.size);

        for y in camera_chunk_pos.top..=camera_chunk_pos.bottom {
            for x in camera_chunk_pos.left..=camera_chunk_pos.right {
                let chunk_pos = UVec2::new(x, y);

                if chunk_manager.spawned_chunks.insert(chunk_pos) {
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
    world_data: Res<WorldData>
) {
    if let Ok((camera_transform, projection)) = camera_query.get_single() {
        let camera_fov = get_camera_fov(camera_transform.translation().xy(), projection);
        let camera_chunk_pos = get_chunk_position_by_camera_fov(camera_fov, world_data.size);

        for (entity, Chunk { pos: chunk_pos }) in chunks.iter() {
            if (chunk_pos.x < camera_chunk_pos.left || chunk_pos.x > camera_chunk_pos.right) ||
               (chunk_pos.y > camera_chunk_pos.bottom || chunk_pos.y < camera_chunk_pos.top) 
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
    chunk_pos: ChunkPos,
) { 
    let chunk = commands.spawn(SpatialBundle {
        transform: Transform::from_xyz(chunk_pos.x as f32 * CHUNK_SIZE * TILE_SIZE, -(chunk_pos.y as f32 + 1.) * CHUNK_SIZE * TILE_SIZE + TILE_SIZE, 0.),
        ..default()
        })
        .insert(Chunk { pos: chunk_pos })
        .id();

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNKMAP_SIZE);

    let wallmap_entity = commands.spawn_empty().id();
    let mut wall_storage = TileStorage::empty(CHUNKMAP_SIZE);

    for y in 0..CHUNK_SIZE_U {
        for x in 0..CHUNK_SIZE_U {
            let chunk_tile_pos = TilePos { 
                x, 
                y: CHUNK_SIZE_U - 1 - y
            };

            let map_tile_pos = TilePos {
                x: (chunk_pos.x as f32 * CHUNK_SIZE) as u32 + x,
                y: (chunk_pos.y as f32 * CHUNK_SIZE as f32 + y as f32) as u32
            };

            if let Some(block) = world_data.get_block(map_tile_pos) {
                let index = Block::get_sprite_index(
                    &world_data.get_block_neighbors(map_tile_pos).map_ref(|b| b.block_type), 
                    block.block_type
                );

                let tile_entity = spawn_block(commands, *block, chunk_tile_pos, tilemap_entity, index);

                commands.entity(tilemap_entity).add_child(tile_entity);
                tile_storage.set(&chunk_tile_pos, tile_entity);
            }

            if let Some(wall) = world_data.get_wall(map_tile_pos) {
                let index = Wall::get_sprite_index(
                    world_data.get_wall_neighbors(map_tile_pos),
                    *wall
                );
                let wall_entity = spawn_wall(commands, chunk_tile_pos, wallmap_entity, index);

                commands.entity(wallmap_entity).add_child(wall_entity);
                wall_storage.set(&chunk_tile_pos, wall_entity);
            }
        }
    }

    commands
        .entity(tilemap_entity)
        .insert(TileChunk { pos: chunk_pos })
        .insert(TilemapBundle {
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
        .insert(TilemapBundle {
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

pub fn get_chunk_position_by_camera_fov(camera_fov: FRect, world_size: Size) -> URect {
    let mut rect = URect { 
        left: (camera_fov.left / (CHUNK_SIZE * TILE_SIZE)).floor() as u32, 
        right: (camera_fov.right / (CHUNK_SIZE * TILE_SIZE)).ceil() as u32, 
        bottom: (camera_fov.bottom.abs() / (CHUNK_SIZE * TILE_SIZE).ceil()) as u32, 
        top: ((camera_fov.top.abs() / (CHUNK_SIZE * TILE_SIZE)).floor()) as u32,
    };

    let max_chunk_x: u32 = world_size.width as u32 / CHUNK_SIZE_U;
    let max_chunk_y: u32 = world_size.height as u32 / CHUNK_SIZE_U;

    if rect.right > max_chunk_x {
        rect.right = max_chunk_x;
    }
    
    if rect.bottom > max_chunk_y {
        rect.bottom = max_chunk_y;
    }

    rect
}

pub fn handle_break_block_event(
    mut commands: Commands,
    mut world_data: ResMut<WorldData>,
    mut break_block_events: EventReader<BreakBlockEvent>,
    mut update_light_events: EventWriter<UpdateLightEvent>,
    mut update_neighbors_ew: EventWriter<UpdateNeighborsEvent>,
    mut chunks: Query<(&TileChunk, &mut TileStorage)>,
) {
    for BreakBlockEvent { tile_pos } in break_block_events.iter() {
        let map_tile_pos = TilePos { x: tile_pos.x as u32, y: tile_pos.y as u32 };

        if world_data.block_exists(map_tile_pos) {
            world_data.remove_block(map_tile_pos);

            let chunk_pos = get_chunk_pos(map_tile_pos);
            let chunk_tile_pos = get_chunk_tile_pos(map_tile_pos);

            if let Some((_, mut tile_storage)) = chunks.iter_mut().find(|(chunk, _)| chunk.pos == chunk_pos) {
                if let Some(tile_entity) = tile_storage.get(&chunk_tile_pos) {
                    commands.entity(tile_entity).despawn_recursive();
                    tile_storage.remove(&chunk_tile_pos);
                }
            }

            update_neighbors_ew.send(UpdateNeighborsEvent { 
                tile_pos: map_tile_pos,
                chunk_tile_pos,
                chunk_pos
            });

            update_light_events.send(UpdateLightEvent {
                tile_pos: map_tile_pos,
                color: 0xFF
            });
        }
    }
}

pub fn handle_dig_block_event(
    mut world_data: ResMut<WorldData>,
    mut break_block_events: EventWriter<BreakBlockEvent>,
    mut dig_block_events: EventReader<DigBlockEvent>,
    sound_assets: Res<SoundAssets>,
    audio: Res<Audio>
) {
    let mut rng = thread_rng();

    for DigBlockEvent { tile_pos, pickaxe } in dig_block_events.iter() {
        let map_tile_pos = TilePos { x: tile_pos.x as u32, y: tile_pos.y as u32 };

        if let Some(block) = world_data.get_block_mut(map_tile_pos) {
            block.hp -= pickaxe.power();

            if block.hp <= 0 {
                break_block_events.send(BreakBlockEvent { tile_pos: *tile_pos });
            }

            audio.play(sound_assets.get_by_block(block.block_type, &mut rng));
        }
    }
}

pub fn handle_place_block_event(
    mut commands: Commands,
    mut world_data: ResMut<WorldData>,
    mut place_block_events: EventReader<PlaceBlockEvent>,
    mut update_light_events: EventWriter<UpdateLightEvent>,
    mut update_neighbors_ew: EventWriter<UpdateNeighborsEvent>,
    mut inventory: ResMut<Inventory>,
    mut chunks: Query<(&TileChunk, &mut TileStorage, Entity)>,
    sound_assets: Res<SoundAssets>,
    audio: Res<Audio>
) {
    let mut rng = thread_rng();

    for PlaceBlockEvent { tile_pos, block, inventory_item_index } in place_block_events.iter() {
        let map_tile_pos = TilePos { x: tile_pos.x as u32, y: tile_pos.y as u32 };

        if !world_data.block_exists(map_tile_pos) {
            let neighbors = world_data
                .get_block_neighbors(map_tile_pos)
                .map_ref(|b| b.block_type);

            world_data.set_block(map_tile_pos, block);

            let chunk_pos = get_chunk_pos(map_tile_pos);
            let chunk_tile_pos = get_chunk_tile_pos(map_tile_pos);

            if let Some((_, mut tile_storage, tilemap_entity)) = chunks.iter_mut().find(|(chunk, _, _)| chunk.pos == chunk_pos) {
                let index = Block::get_sprite_index(&neighbors, block.block_type);
                let tile_entity = spawn_block(&mut commands, *block, chunk_tile_pos, tilemap_entity, index);

                commands.entity(tilemap_entity).add_child(tile_entity);
                tile_storage.set(&chunk_tile_pos, tile_entity);
            }

            inventory.consume_item(*inventory_item_index);

            update_neighbors_ew.send(UpdateNeighborsEvent { 
                tile_pos: map_tile_pos,
                chunk_tile_pos,
                chunk_pos
            });

            update_light_events.send(UpdateLightEvent {
                tile_pos: map_tile_pos,
                color: 0
            });

            audio.play(sound_assets.get_by_block(block.block_type, &mut rng));
        }
    }
}

pub fn update_neighbors(
    world_data: Res<WorldData>,
    mut events: EventReader<UpdateNeighborsEvent>,
    mut tiles: Query<&mut TileTextureIndex>,
    chunks: Query<(&TileChunk, &TileStorage)>
) {
    for event in events.iter() {
        let tile_pos = event.tile_pos;
        let map_size = TilemapSize {
            x: world_data.size.width as u32,
            y: world_data.size.height as u32,
        };
        
        let neighbor_positions = Neighbors::get_square_neighboring_positions(&tile_pos, &map_size, false);

        for pos in neighbor_positions.iter() {
            if let Some(block) = world_data.get_block(*pos) {
                let neighbors = world_data
                    .get_block_neighbors(*pos)
                    .map_ref(|b| b.block_type);

                let index = Block::get_sprite_index(&neighbors, block.block_type);

                let chunk_pos = get_chunk_pos(*pos);
                let chunk_tile_pos = get_chunk_tile_pos(*pos);

                if let Some((_, tile_storage)) = chunks.iter().find(|(chunk, _)| chunk.pos == chunk_pos) {
                    if let Some(entity) = tile_storage.get(&chunk_tile_pos) {
                        if let Ok(mut tile_texture) = tiles.get_mut(entity) {
                            tile_texture.0 = index;   
                        }
                    }
                }
            }
        }
    }
}
