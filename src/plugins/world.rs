use std::{
    collections::HashMap,
    ops::Mul,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{
    core::Name,
    prelude::{
        default, App, Changed, Commands, Entity, GlobalTransform, Handle, OrthographicProjection,
        Plugin, Query, Res, ResMut, Transform, Vec3, With, EventReader, UVec2, IVec2, Component, BuildChildren, Visibility, VisibilityBundle, DespawnRecursiveExt,
    },
    render::view::NoFrustumCulling,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite}, utils::HashSet,
};
use bevy_rapier2d::prelude::{Collider, Friction, Restitution, RigidBody};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet},
    state::NextState,
};
use ndarray::{Array2, ArrayView2};
use rand::{thread_rng, Rng};

use crate::{
    block::Block,
    state::GameState,
    util::FRect,
    world_generator::{generate, Cell, Neighbours, Tile, Wall},
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct URect {
    left: usize,
    right: usize,
    top: usize,
    bottom: usize,
}

impl URect {
    fn to_frect(&self) -> FRect {
        FRect {
            left: self.left as f32,
            right: self.right as f32,
            top: self.top as f32,
            bottom: self.bottom as f32,
        }
    }
}

impl FRect {
    fn intersect(&self, rect: FRect) -> bool {
        self.left < rect.right
            && self.right > rect.left
            && self.bottom > rect.top
            && self.top > -rect.bottom.abs()
    }
}

impl Mul<f32> for FRect {
    type Output = FRect;

    fn mul(self, rhs: f32) -> Self::Output {
        FRect {
            left: self.left * rhs,
            right: self.right * rhs,
            top: self.top * rhs,
            bottom: self.bottom * rhs,
        }
    }
}

#[derive(Component)]
struct Chunk {
    chunk_pos: IVec2
}

#[derive(Clone, Copy, Default)]
pub struct ColliderData {
    rect: FRect,
    entity: Option<Entity>,
}

pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub tiles: Array2<Cell>,
    pub colliders: Vec<ColliderData>,
}

pub struct BlockBreakEvent {
    pub coords: UVec2,
}

pub struct BlockPlaceEvent {
    pub coords: UVec2,
    pub block: Block,
}

fn spawn_terrain(mut commands: Commands) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    // let seed = current_time.as_millis() as u32;
    let seed = 4289917359;

    println!("The world's seed is {}", seed);

    println!("Generating world...");
    let tiles = generate(seed);

    let colliders = get_colliders(&tiles.view());

    commands.insert_resource(WorldData {
        width: tiles.ncols() as u16,
        height: tiles.nrows() as u16,
        tiles,
        colliders,
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
            sprite: TextureAtlasSprite { index, ..default() },
            texture_atlas,
            transform: Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(1.05)),
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
    ix: usize,
    x: f32,
    iy: usize,
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

fn spawn_collider(commands: &mut Commands, rect: FRect) -> Entity {
    commands
        .spawn()
        .insert(Collider::cuboid(
            (rect.right - rect.left + 1.) * TILE_SIZE / 2.,
            (rect.top - rect.bottom + 1.) * TILE_SIZE / 2.,
        ))
        .insert(RigidBody::Fixed)
        .insert(Friction::new(0.))
        .insert(Restitution::new(0.))
        .insert(Transform::from_xyz(
            (rect.left + rect.right) * TILE_SIZE / 2.,
            -(rect.bottom + rect.top) * TILE_SIZE / 2.,
            0.,
        ))
        .insert(GlobalTransform::default())
        .insert(Name::new("Terrain Collider"))
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

fn get_colliders(chunk: &ArrayView2<Cell>) -> Vec<ColliderData> {
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: usize,
        right: usize,
    }

    let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

    for y in 0..chunk.nrows() - 1 {
        let mut row_plates: Vec<Plate> = Vec::new();
        let mut plate_start = None;

        for x in 0..chunk.ncols() + 1 {
            let is_solid = chunk.get((y, x)).and_then(|cell| cell.tile).is_some();

            match (plate_start, is_solid) {
                (Some(s), false) => {
                    row_plates.push(Plate {
                        left: s,
                        right: x - 1,
                    });
                    plate_start = None;
                }
                (None, true) => plate_start = Some(x),
                _ => (),
            }
        }

        plate_stack.push(row_plates);
    }

    let mut tile_rects: Vec<URect> = Vec::new();
    let mut previous_rects: HashMap<Plate, URect> = HashMap::new();

    // an extra empty row so the algorithm "terminates" the rects that touch the top
    // edge
    plate_stack.push(Vec::new());

    for (y, row) in plate_stack.iter().enumerate() {
        let mut current_rects: HashMap<Plate, URect> = HashMap::new();

        for plate in row {
            if let Some(previous_rect) = previous_rects.remove(plate) {
                current_rects.insert(
                    *plate,
                    URect {
                        top: previous_rect.top + 1,
                        ..previous_rect
                    },
                );
            } else {
                current_rects.insert(
                    *plate,
                    URect {
                        bottom: y,
                        top: y,
                        left: plate.left,
                        right: plate.right,
                    },
                );
            }
        }

        // Any plates that weren't removed above have terminated
        tile_rects.append(&mut previous_rects.values().copied().collect());
        previous_rects = current_rects;
    }

    tile_rects
        .iter()
        .map(|rect| ColliderData {
            rect: rect.to_frect(),
            ..default()
        })
        .collect()
}

#[derive(Default)]
struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>
}

fn spawn_tiles(
    mut commands: Commands,
    block_assets: Res<BlockAssets>,
    wall_assets: Res<WallAssets>,
    mut world_data: ResMut<WorldData>,
    mut chunk_manager: ResMut<ChunkManager>,
    camera_query: Query<
        (&GlobalTransform, &OrthographicProjection),
        (With<MainCamera>, Changed<GlobalTransform>),
    >,
) {
    if let Ok((camera_transform, projection)) = camera_query.get_single() {
        let camera_x = camera_transform.translation().x;
        let camera_y = camera_transform.translation().y;

        let camera_fov = FRect {
            left: camera_x + projection.left * projection.scale,
            right: camera_x + projection.right * projection.scale,
            top: camera_y - projection.top * projection.scale,
            bottom: camera_y - projection.bottom * projection.scale,
        };

        let chunk_pos_left = (camera_fov.left / (CHUNK_SIZE * TILE_SIZE)) as i32;
        let chunk_pos_right = (camera_fov.right / (CHUNK_SIZE * TILE_SIZE)) as i32;

        let chunk_pos_top = (camera_fov.top / (CHUNK_SIZE * TILE_SIZE)) as i32;
        let chunk_pos_bottom = (camera_fov.bottom / (CHUNK_SIZE * TILE_SIZE)) as i32;

        for y in (chunk_pos_top.abs() - 1).max(0)..(chunk_pos_bottom.abs() + 1) {
            for x in (chunk_pos_left - 1)..(chunk_pos_right + 1) {
                let chunk_pos = IVec2::new(x, y);

                if !chunk_manager.spawned_chunks.contains(&chunk_pos) {
                    chunk_manager.spawned_chunks.insert(chunk_pos);
                    spawn_chunk(&mut commands, &block_assets, &wall_assets, &world_data, chunk_pos);
                }
            }
        }

        for collider in world_data.colliders.iter_mut() {
            let inside = (collider.rect * (TILE_SIZE)).intersect(camera_fov);

            match collider.entity {
                None if inside => {
                    let entity = spawn_collider(&mut commands, collider.rect);
                    collider.entity = Some(entity);
                }
                Some(entity) if !inside => {
                    commands.entity(entity).despawn();
                    collider.entity = None;
                }
                _ => (),
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
        let camera_x = camera_transform.translation().x;
        let camera_y = camera_transform.translation().y;

        let camera_fov = FRect {
            left: camera_x + projection.left * projection.scale,
            right: camera_x + projection.right * projection.scale,
            top: camera_y - projection.top * projection.scale,
            bottom: camera_y - projection.bottom * projection.scale,
        };

        let camera_chunk_pos_left = (camera_fov.left / (CHUNK_SIZE * TILE_SIZE)) as i32 - 1;
        let camera_chunk_pos_right = (camera_fov.right / (CHUNK_SIZE * TILE_SIZE)) as i32 + 1;

        let camera_chunk_pos_top = (camera_fov.top / (CHUNK_SIZE * TILE_SIZE)) as i32 + 1;
        let camera_chunk_pos_bottom = (camera_fov.bottom / (CHUNK_SIZE * TILE_SIZE)) as i32 - 1;

        for (entity, Chunk { chunk_pos }) in chunks.iter() {

            if (chunk_pos.x < camera_chunk_pos_left || chunk_pos.x > camera_chunk_pos_right) &&
               (chunk_pos.y < camera_chunk_pos_bottom || chunk_pos.y > camera_chunk_pos_top) 
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
        .insert(Transform::from_xyz(chunk_pos.x as f32 * CHUNK_SIZE * TILE_SIZE, -chunk_pos.y as f32 * CHUNK_SIZE * TILE_SIZE, 0.1))
        .insert(GlobalTransform::default())
        .insert(Name::new(format!("Chunk (x: {}, y: {})", chunk_pos.x, chunk_pos.y)))
        .id();

    for y in 0..CHUNK_SIZE as usize {
        for x in 0..CHUNK_SIZE as usize {
            if let Some(cell) = world_data.tiles.get((
                (chunk_pos.y as f32 * CHUNK_SIZE) as usize + y, 
                (chunk_pos.x as f32 * CHUNK_SIZE) as usize + x
            )) {
                if let Some(tile) = cell.tile {
                    if let Some(texture_atlas) = block_assets.get_by_block(tile.tile_type) {
                        let index = get_tile_sprite_index(tile.neighbours);

                        let tile_entity = commands
                            .spawn_bundle(SpriteSheetBundle {
                                sprite: TextureAtlasSprite { index, ..default() },
                                texture_atlas,
                                transform: Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 0.1).with_scale(Vec3::splat(1.05)),
                                ..default()
                            })
                            .insert(tile.tile_type)
                            .insert(Name::new(format!("Block Tile {} {}", (chunk_pos.x as f32 * CHUNK_SIZE) as u32 + x as u32, (chunk_pos.y as f32 * CHUNK_SIZE) as u32 + y as u32)))
                            .id();

                        commands.entity(chunk).add_child(tile_entity);
                    }
                }

                if let Some(wall) = cell.wall {
                    if let Some(texture_atlas) = wall_assets.get_by_wall(wall.wall_type) {
                        let index = get_wall_sprite_index(wall.neighbours);

                        let tile_entity = commands
                            .spawn_bundle(SpriteSheetBundle {
                                sprite: TextureAtlasSprite { index, ..default() },
                                texture_atlas,
                                transform: Transform::from_xyz(x as f32 * TILE_SIZE, -(y as f32) * TILE_SIZE, 0.0).with_scale(Vec3::splat(1.05)),
                                ..default()
                            })
                            .insert(Name::new(format!("Block Tile {} {}", (chunk_pos.x as f32 * CHUNK_SIZE) as u32 + x as u32, (chunk_pos.y as f32 * CHUNK_SIZE) as u32 + y as u32)))
                            .id();

                        commands.entity(chunk).add_child(tile_entity);
                    }
                }
            }
        }
    }
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
            event.coords.x, 
            event.coords.x as f32 * 16.,
            event.coords.y, 
            event.coords.y as f32 * 16.,
        );
    }
}
