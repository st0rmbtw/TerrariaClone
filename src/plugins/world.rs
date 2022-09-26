use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{
    prelude::{
        default, App, Changed, Commands, Entity, GlobalTransform, OrthographicProjection,
        Plugin, Query, Res, ResMut, Transform, With, EventReader, UVec2, IVec2, Component, 
        BuildChildren, DespawnRecursiveExt, Vec2, EventWriter,
    },
    utils::HashSet, math::Vec3Swizzles,
};
use bevy_ecs_tilemap::{
    tiles::{TileStorage, TilePos, TileBundle, TileTexture}, 
    prelude::{TilemapSize, TilemapId, TilemapTexture, TilemapTileSize, TilemapGridSize, TilemapSpacing, TilemapType, get_neighboring_pos},
    TilemapBundle, 
};  
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet},
    state::NextState,
};
use ndarray::Array2;
use rand::{thread_rng, Rng};

use crate::{
    block::Block,
    state::GameState,
    util::{FRect, IRect, self},
    world_generator::{generate, Cell, Neighbors, Tile, Wall, WORLD_SIZE_Y, WORLD_SIZE_X}, DefaultBundle,
};

use super::{assets::{BlockAssets, WallAssets}, inventory::Inventory, camera::MainCamera};

pub const TILE_SIZE: f32 = 16.;
pub const WALL_SIZE: f32 = 32.;

const CHUNK_SIZE: f32 = 25.;
const CHUNK_SIZE_U: u32 = CHUNK_SIZE as u32;

const CHUNKMAP_SIZE: TilemapSize = TilemapSize {
    x: CHUNK_SIZE as u32,
    y: CHUNK_SIZE as u32,
};

const MAP_SIZE: TilemapSize = TilemapSize {
    x: WORLD_SIZE_X as u32,
    y: WORLD_SIZE_Y as u32
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkManager>()
            .add_event::<BlockPlaceEvent>()
            .add_event::<UpdateNeighborsEvent>()
            .add_enter_system(GameState::WorldLoading, spawn_terrain)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(spawn_chunks)
                    .with_system(despawn_chunks)
                    .with_system(handle_block_place)
                    .with_system(update_neighbors)
                    .into(),
            );
    }
}

#[derive(Component)]
pub struct Chunk {
    pub pos: IVec2
}

#[derive(Component)]
pub struct TileChunk {
    pub pos: IVec2
}

#[derive(Component)]
pub struct WallChunk {
    pub pos: IVec2
}

pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub tiles: Array2<Cell>,
}

impl WorldData {
    pub fn get_cell_mut(&mut self, pos: TilePos) -> Option<&mut Cell> {
        self.tiles.get_mut((pos.y as usize, pos.x as usize))
    }
    
    pub fn get_tile(&self, pos: TilePos) -> Option<&Tile> {
        self.tiles.get((pos.y as usize, pos.x as usize)).and_then(|cell| cell.tile.as_ref())
    }

    pub fn get_tile_mut(&mut self, pos: TilePos) -> Option<&mut Tile> {
        self.tiles.get_mut((pos.y as usize, pos.x as usize)).and_then(|cell| cell.tile.as_mut())
    }

    pub fn tile_exists(&self, pos: TilePos) -> bool {
        self.tiles.get((pos.y as usize, pos.x as usize)).and_then(|cell| cell.tile).is_some()
    }

    pub fn get_neighbours(&self, pos: TilePos) -> Neighbors {
        Neighbors { 
            left: pos.square_west().and_then(|p| self.get_tile(p)).is_some(),
            right: pos.square_east(&MAP_SIZE).and_then(|p| self.get_tile(p)).is_some(),
            top: pos.square_south().and_then(|p| self.get_tile(p)).is_some(),
            bottom: pos.square_north(&MAP_SIZE).and_then(|p| self.get_tile(p)).is_some()
        }
    }
}

#[derive(Default)]
struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>
}

pub struct BlockBreakEvent {
    pub coords: UVec2,
}

pub struct BlockPlaceEvent {
    pub tile_pos: Vec2,
    pub block: Block,
    pub inventory_item_index: usize
}

pub struct UpdateNeighborsEvent {
    pub tile_pos: TilePos,
    pub chunk_tile_pos: TilePos,
    pub chunk_pos: IVec2,
    pub tile: Tile,
}

fn spawn_terrain(mut commands: Commands) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let seed = current_time.as_millis() as u32;
    // let seed = 4289917359;

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

fn spawn_tile(
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

fn spawn_wall(
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

fn get_tile_sprite_index(slope: Neighbors) -> u32 {
    let rand: u32 = thread_rng().gen_range(1..3);

    match slope {
        // All
        Neighbors::ALL => rand + 16,
        // None
        Neighbors::NONE => 16 * 3 + rand + 8,
        // Top
        Neighbors::TOP => 16 * 3 + rand + 5,
        // Bottom
        Neighbors::BOTTOM => rand + 6,
        // Left
        Neighbors::LEFT => (rand - 1) * 16 + 12,
        // Right
        Neighbors::RIGHT => (rand - 1) * 16 + 9,
        // Top Bottom
        Neighbors::TOP_BOTTOM => (rand - 1) * 16 + 5,
        // Top Left Right
        Neighbors::TOP_LEFT_RIGHT => 16 * 2 + rand + 1,
        // Bottom Left Right
        Neighbors::BOTTOM_LEFT_RIGHT => rand,
        // Left Right
        Neighbors::LEFT_RIGHT => 4 * 16 + 5 + rand,
        // Bottom Left
        Neighbors::BOTTOM_LEFT => 16 * 3 + 1 + (rand - 1) * 2,
        // Bottom Right
        Neighbors::BOTTOM_RIGHT => 16 * 3 + (rand - 1) * 2,
        // Top Left
        Neighbors::TOP_LEFT => 16 * 4 + 1 + (rand - 1) * 2,
        // Top Right
        Neighbors::TOP_RIGHT => 16 * 4 + (rand - 1) * 2,
        // Top Bottom Left
        Neighbors::TOP_BOTTOM_LEFT => (rand - 1) * 16 + 4,
        // Top Bottom Right
        Neighbors::TOP_BOTTOM_RIGHT => (rand - 1) * 16,
    }
}

fn get_wall_sprite_index(slope: Neighbors) -> u32 {
    let rand: u32 = thread_rng().gen_range(1..3);

    match slope {
        // All
        Neighbors::ALL => 13 + rand,
        // None
        Neighbors::NONE => 13 * 3 + 8 + rand,
        // Top
        Neighbors::TOP => 13 * 2 + rand,
        // Bottom
        Neighbors::BOTTOM => 6 + rand,
        // Top Bottom
        Neighbors::TOP_BOTTOM => (rand - 1) * 13 + 5,
        // Bottom Right
        Neighbors::BOTTOM_RIGHT => 13 * 3 + (rand - 1) * 2,
        // Bottom Left
        Neighbors::BOTTOM_LEFT => 13 * 3 + 1 + (rand - 1) * 2,
        // Top Right
        Neighbors::TOP_RIGHT => 13 * 4 + (rand - 1) * 2,
        // Top Left
        Neighbors::TOP_LEFT => 13 * 4 + 1 + (rand - 1) * 2,
        // Left Right
        Neighbors::LEFT_RIGHT => 13 * 4 + 5 + rand,
        // Bottom Left Right
        Neighbors::BOTTOM_LEFT_RIGHT => 1 + rand,
        // Top Bottom Right
        Neighbors::TOP_BOTTOM_RIGHT => 13 * (rand - 1),
        // Top Bottom Left
        Neighbors::TOP_BOTTOM_LEFT => 13 * (rand - 1) + 4,
        // Top Left Right
        Neighbors::TOP_LEFT_RIGHT => 13 * 2 + rand,
        _ => panic!("{:#?}", slope),
    }
}

fn spawn_chunks(
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

        for y in camera_chunk_pos.top..=camera_chunk_pos.bottom {
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

fn despawn_chunks(
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
               (chunk_pos.y > camera_chunk_pos.bottom || chunk_pos.y < camera_chunk_pos.top) 
            {
                chunk_manager.spawned_chunks.remove(&chunk_pos);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn spawn_chunk(
    commands: &mut Commands,
    block_assets: &BlockAssets,
    wall_assets: &WallAssets,
    world_data: &WorldData,
    chunk_pos: IVec2,
) { 
    let chunk = commands.spawn()
        .insert(Chunk { pos: chunk_pos })
        .insert_bundle(DefaultBundle {
            transform: Transform::from_xyz(chunk_pos.x as f32 * CHUNK_SIZE * TILE_SIZE, -(chunk_pos.y + 1) as f32 * CHUNK_SIZE * TILE_SIZE + TILE_SIZE, 0.),
            ..default()  
        })
        .id();

    let tilemap_entity = commands.spawn().id();
    let mut tile_storage = TileStorage::empty(CHUNKMAP_SIZE);

    let wallmap_entity = commands.spawn().id();
    let mut wall_storage = TileStorage::empty(CHUNKMAP_SIZE);

    for y in 0..CHUNK_SIZE as usize {
        for x in 0..CHUNK_SIZE as usize {
            let tile_pos = TilePos {
                x: x as u32,
                y: CHUNK_SIZE as u32 - 1 - y as u32,
            };

            let tile_x = (chunk_pos.x as f32 * CHUNK_SIZE) as u32 + x as u32;
            let tile_y = (chunk_pos.y as f32 * CHUNK_SIZE) as u32 + y as u32;

            let cell_option = world_data.tiles.get((tile_y as usize, tile_x as usize));

            if let Some(cell) = cell_option {
                if let Some(tile) = cell.tile {
                    let tile_entity = spawn_tile(commands, tile, tile_pos, tilemap_entity);

                    commands.entity(tilemap_entity).add_child(tile_entity);
                    tile_storage.set(&tile_pos, Some(tile_entity));
                }

                if let Some(wall) = cell.wall {
                    let wall_entity = spawn_wall(commands, wall, tile_pos, wallmap_entity);

                    commands.entity(wallmap_entity).add_child(wall_entity);
                    wall_storage.set(&tile_pos, Some(wall_entity));
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
            texture: TilemapTexture(block_assets.tiles.clone()),
            tile_size: TilemapTileSize {
                x: TILE_SIZE,
                y: TILE_SIZE,
            },
            spacing: TilemapSpacing {
                x: 2.,
                y: 2.
            },
            transform: Transform::from_xyz(0., 0., 1.),
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
            texture: TilemapTexture(wall_assets.walls.clone()),
            tile_size: TilemapTileSize {
                x: WALL_SIZE,
                y: WALL_SIZE,
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..Default::default()
        });

    commands.entity(chunk).push_children(&[tilemap_entity, wallmap_entity]);
}

fn get_camera_fov(camera_pos: Vec2, projection: &OrthographicProjection) -> FRect {
    FRect {
        left: camera_pos.x + projection.left * projection.scale,
        right: camera_pos.x + projection.right * projection.scale,
        top: camera_pos.y - projection.top * projection.scale,
        bottom: camera_pos.y - projection.bottom * projection.scale,
    }
}

fn get_chunk_position_by_camera_fov(camera_fov: FRect) -> IRect {
    let mut rect = IRect { 
        left: (camera_fov.left / (CHUNK_SIZE * TILE_SIZE)).floor() as i32, 
        right: (camera_fov.right / (CHUNK_SIZE * TILE_SIZE)).ceil() as i32, 
        bottom: (camera_fov.top / (CHUNK_SIZE * TILE_SIZE)).abs().ceil() as i32, 
        top: (camera_fov.bottom / (CHUNK_SIZE * TILE_SIZE)).abs() as i32 - 1
    };

    const MAX_CHUNK_X: i32 = (WORLD_SIZE_X as f32 / CHUNK_SIZE) as i32;
    const MAX_CHUNK_Y: i32 = (WORLD_SIZE_Y as f32 / CHUNK_SIZE) as i32;

    if rect.top < 0 {
        rect.top = 0;
    }

    if rect.left < 0 {
        rect.left = 0;
    }

    if rect.right > MAX_CHUNK_X {
        rect.right = MAX_CHUNK_X;
    }
    
    if rect.bottom > MAX_CHUNK_Y {
        rect.bottom = MAX_CHUNK_Y;
    }

    rect
}

pub fn get_chunk_position(pos: TilePos) -> IVec2 {
    IVec2 { 
        x: (pos.x / CHUNK_SIZE as u32) as i32,
        y: (pos.y / CHUNK_SIZE as u32) as i32
    }
}

fn handle_block_place(
    mut commands: Commands,
    mut world_data: ResMut<WorldData>,
    mut events: EventReader<BlockPlaceEvent>,
    mut update_neighbors_ew: EventWriter<UpdateNeighborsEvent>,
    mut inventory: ResMut<Inventory>,
    mut chunks: Query<(&TileChunk, &mut TileStorage, Entity)>
) {
    for event in events.iter() {
        let map_tile_pos = TilePos { x: event.tile_pos.x as u32, y: event.tile_pos.y.abs() as u32 };
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
                    tile_storage.set(&chunk_tile_pos, Some(tile_entity));
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

fn update_neighbors(
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
                        let (mut tile_texture, block) = tiles.get_mut(entity).unwrap();

                        tile_texture.0 = util::get_tile_start_index(*block) + get_tile_sprite_index(neighbors);
                    }
                }
            });
        }
    }
}
