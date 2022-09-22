use std::{
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{
    core::Name,
    prelude::{
        default, App, Changed, Commands, Entity, GlobalTransform, Handle, OrthographicProjection,
        Plugin, Query, Res, ResMut, Transform, Vec3, With, EventReader, UVec2, IVec2, Component, 
        BuildChildren, Visibility, VisibilityBundle, DespawnRecursiveExt, Vec2,
    },
    render::view::NoFrustumCulling,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite}, utils::HashSet, math::Vec3Swizzles,
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
    util::{FRect, IRect},
    world_generator::{generate, Cell, Neighbours, Tile, Wall, WORLD_SIZE_Y, WORLD_SIZE_X},
};

use super::{BlockAssets, MainCamera, WallAssets};

pub const TILE_SIZE: f32 = 16.;

const CHUNK_SIZE: f32 = 25.;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkManager>()
            .add_event::<BlockPlaceEvent>()
            .add_enter_system(GameState::WorldLoading, spawn_terrain)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(spawn_tiles)
                    .with_system(despawn_tiles)
                    .with_system(handle_block_place)
                    .into(),
            );
    }
}

#[derive(Component)]
pub struct Chunk {
    pub chunk_pos: IVec2
}

pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub tiles: Array2<Cell>,
}

#[derive(Default)]
struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>
}

pub struct BlockBreakEvent {
    pub coords: UVec2,
}

pub struct BlockPlaceEvent {
    pub coords: Vec2,
    pub block: Block,
}

fn spawn_terrain(mut commands: Commands) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    // let seed = current_time.as_millis() as u32;
    let seed = 4289917359;

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
    texture_atlas: Handle<TextureAtlas>,
    tile: Tile,
    ix: u32,
    x: f32,
    iy: u32,
    y: f32,
) -> Entity {
    let index = get_tile_sprite_index(tile.neighbours);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::new(16., 16.)),
                index,
                ..default()
            },
            texture_atlas,
            transform: Transform::from_xyz(x, y, 0.1).with_scale(Vec3::splat(1.0)),
            ..default()
        })
        .insert(tile.tile_type)
        .insert(Name::new(format!("Block Tile {} {}", ix, iy)))
        .id()
}

fn spawn_wall(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    wall: Wall,
    ix: u32,
    x: f32,
    iy: u32,
    y: f32,
) -> Entity {
    let index = get_wall_sprite_index(wall.neighbours);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite { index, ..default() },
            texture_atlas,
            transform: Transform::from_xyz(x, y, 0.).with_scale(Vec3::splat(1.05)),
            ..default()
        })
        .insert(Name::new(format!("Wall {} {}", ix, iy)))
        .insert(NoFrustumCulling)
        .id()
}

fn get_tile_sprite_index(slope: Neighbours) -> usize {
    let rand: usize = thread_rng().gen_range(1..3);

    match slope {
        // All
        Neighbours::ALL => rand + 16,
        // None
        Neighbours::NONE => 16 * 3 + rand + 8,
        // Top
        Neighbours::TOP => 16 * 3 + rand + 5,
        // Bottom
        Neighbours::BOTTOM => rand + 6,
        // Left
        Neighbours::LEFT => (rand - 1) * 16 + 12,
        // Right
        Neighbours::RIGHT => (rand - 1) * 16 + 9,
        // Top Bottom
        Neighbours::TOP_BOTTOM => (rand - 1) * 16 + 5,
        // Top Left Right
        Neighbours::TOP_LEFT_RIGHT => 16 * 2 + rand + 1,
        // Bottom Left Right
        Neighbours::BOTTOM_LEFT_RIGHT => rand,
        // Left Right
        Neighbours::LEFT_RIGHT => 4 * 16 + 5 + rand,
        // Bottom Left
        Neighbours::BOTTOM_LEFT => 16 * 3 + 1 + (rand - 1) * 2,
        // Bottom Right
        Neighbours::BOTTOM_RIGHT => 16 * 3 + (rand - 1) * 2,
        // Top Left
        Neighbours::TOP_LEFT => 16 * 4 + 1 + (rand - 1) * 2,
        // Top Right
        Neighbours::TOP_RIGHT => 16 * 4 + (rand - 1) * 2,
        // Top Bottom Left
        Neighbours::TOP_BOTTOM_LEFT => (rand - 1) * 16 + 4,
        // Top Bottom Right
        Neighbours::TOP_BOTTOM_RIGHT => (rand - 1) * 16,
    }
}

fn get_wall_sprite_index(slope: Neighbours) -> usize {
    let rand: usize = thread_rng().gen_range(1..3);

    match slope {
        // All
        Neighbours::ALL => 13 + rand,
        // None
        Neighbours::NONE => 13 * 3 + 8 + rand,
        // Top
        Neighbours::TOP => 13 * 2 + rand,
        // Bottom
        Neighbours::BOTTOM => 6 + rand,
        // Top Bottom
        Neighbours::TOP_BOTTOM => (rand - 1) * 13 + 5,
        // Bottom Right
        Neighbours::BOTTOM_RIGHT => 13 * 3 + (rand - 1) * 2,
        // Bottom Left
        Neighbours::BOTTOM_LEFT => 13 * 3 + 1 + (rand - 1) * 2,
        // Top Right
        Neighbours::TOP_RIGHT => 13 * 4 + (rand - 1) * 2,
        // Top Left
        Neighbours::TOP_LEFT => 13 * 4 + 1 + (rand - 1) * 2,
        // Left Right
        Neighbours::LEFT_RIGHT => 13 * 4 + 5 + rand,
        // Bottom Left Right
        Neighbours::BOTTOM_LEFT_RIGHT => 1 + rand,
        // Top Bottom Right
        Neighbours::TOP_BOTTOM_RIGHT => 13 * (rand - 1),
        // Top Bottom Left
        Neighbours::TOP_BOTTOM_LEFT => 13 * (rand - 1) + 4,
        // Top Left Right
        Neighbours::TOP_LEFT_RIGHT => 13 * 2 + rand,
        _ => panic!("{:#?}", slope),
    }
}

fn spawn_tiles(
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

fn despawn_tiles(
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

        for (entity, Chunk { chunk_pos }) in chunks.iter() {
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
    let chunk = commands
        .spawn()
        .insert(Chunk {
            chunk_pos
        })
        .insert_bundle(VisibilityBundle {
            visibility: Visibility::visible(),
            ..default()
        })
        .insert(Transform::from_xyz(chunk_pos.x as f32 * CHUNK_SIZE * TILE_SIZE, -chunk_pos.y as f32 * CHUNK_SIZE * TILE_SIZE, 0.0))
        .insert(GlobalTransform::default())
        .insert(Name::new(format!("Chunk (x: {}, y: {})", chunk_pos.x, chunk_pos.y)))
        .id();

    for y in 0..CHUNK_SIZE as usize {
        for x in 0..CHUNK_SIZE as usize {
            let tile_x = x as f32 * TILE_SIZE;
            let tile_y = -(y as f32) * TILE_SIZE;

            let tile_ix = (chunk_pos.x as f32 * CHUNK_SIZE) as u32 + x as u32;
            let tile_iy = (chunk_pos.y as f32 * CHUNK_SIZE) as u32 + y as u32;

            let cell_option = world_data.tiles.get((tile_iy as usize, tile_ix as usize));

            if let Some(cell) = cell_option {
                if let Some(tile) = cell.tile {
                    if let Some(texture_atlas) = block_assets.get_by_block(tile.tile_type) {
                        let tile_entity = spawn_tile(
                            commands, 
                            texture_atlas, 
                            tile, 
                            tile_ix, 
                            tile_x, 
                            tile_iy, 
                            tile_y
                        );

                        commands.entity(chunk).add_child(tile_entity);
                    }
                }

                if let Some(wall) = cell.wall {
                    if let Some(texture_atlas) = wall_assets.get_by_wall(wall.wall_type) {
                        let wall_entity = spawn_wall(
                            commands, 
                            texture_atlas, 
                            wall, 
                            tile_ix, 
                            tile_x, 
                            tile_iy, 
                            tile_y
                        );

                        commands.entity(chunk).add_child(wall_entity);
                    }
                }
            }
        }
    }
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

pub fn get_chunk_position(
    pos: Vec2
) -> IVec2 {
    (pos / (CHUNK_SIZE * TILE_SIZE)).as_ivec2()
}

fn handle_block_place(
    mut commands: Commands,
    mut events: EventReader<BlockPlaceEvent>,
    block_assets: Res<BlockAssets>
) {
    for event in events.iter() {
        spawn_tile(
            &mut commands, 
            block_assets.get_by_block(event.block).unwrap(), 
            Tile { tile_type: event.block, neighbours: Neighbours::NONE }, 
            event.coords.x as u32, 
            event.coords.x as f32 * 16.,
            event.coords.y as u32, 
            event.coords.y as f32 * 16.,
        );
    }
}
