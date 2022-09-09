use std::{
    collections::HashMap,
    ops::Mul,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{
    core::Name,
    prelude::{
        default, App, Changed, Commands, Entity, GlobalTransform, Handle, OrthographicProjection,
        Plugin, Query, Res, ResMut, Transform, Vec2, Vec3, With, EventReader,
    },
    render::view::NoFrustumCulling,
    sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
};
use bevy_rapier2d::prelude::{Collider, Friction, Restitution, RigidBody, Sleeping};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet},
    state::NextState,
};
use ndarray::{s, Array2, ArrayView2};
use rand::{thread_rng, Rng};

use crate::{
    block::Block,
    state::GameState,
    util::{inside_f, FRect},
    world_generator::{generate, Cell, Slope, Tile, Wall},
};

use super::{BlockAssets, MainCamera, WallAssets};

pub const TILE_SIZE: f32 = 16.;

const CHUNK_WIDTH: usize = 25;
const CHUNK_HEIGHT: usize = 25;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BlockPlaceEvent>()
            .add_enter_system(GameState::WorldLoading, spawn_terrain)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(update)
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
    fn inside(&self, rect: FRect) -> bool {
        inside_f((self.bottom, self.left), rect)
            || inside_f((self.top, self.right), rect)
            || inside_f((self.bottom, self.right), rect)
            || inside_f((self.top, self.left), rect)
    }

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

#[derive(Clone, Copy, Default)]
pub struct ColliderData {
    rect: FRect,
    entity: Option<Entity>,
}

pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub chunks: Vec<Chunk>,
    pub colliders: Vec<ColliderData>,
}

impl WorldData {
    fn get_chunk_by_coords() {}
}

pub struct Chunk {
    pub bounds: FRect,
    pub cells: Array2<Cell>,
    pub spawned: bool,
}

impl Chunk {
    fn spawn(
        &mut self,
        commands: &mut Commands,
        block_assets: &BlockAssets,
        wall_assets: &WallAssets,
    ) {
        self.spawned = true;

        for ((iy, ix), cell) in self.cells.indexed_iter_mut() {
            let x = self.bounds.left + ix as f32 * TILE_SIZE;
            let y = self.bounds.top - iy as f32 * TILE_SIZE;

            if let Some(tile) = cell.tile {
                if let Some(texture_atlas) = block_assets.get_by_block(tile.tile_type) {
                    if cell.tile_entity.is_none() {
                        let entity = spawn_tile(commands, texture_atlas, tile, ix, x, iy, y);

                        cell.tile_entity = Some(entity);
                    }
                }
            }

            if let Some(wall) = cell.wall {
                if let Some(texture_atlas) = wall_assets.get_by_wall(wall.wall_type) {
                    if cell.wall_entity.is_none() {
                        let entity = spawn_wall(commands, texture_atlas, wall, ix, x, iy, y);

                        cell.wall_entity = Some(entity);
                    }
                }
            }
        }
    }

    fn despawn(&mut self, commands: &mut Commands) {
        self.spawned = false;

        for cell in self.cells.iter_mut() {
            if let Some(entity) = cell.tile_entity {
                commands.entity(entity).despawn();
                cell.tile_entity = None;
            }

            if let Some(entity) = cell.wall_entity {
                commands.entity(entity).despawn();
                cell.wall_entity = None;
            }
        }
    }
}

pub struct BlockBreakEvent {
    pub coords: Vec2,
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

    let colliders = get_colliders(&tiles.view());

    let chunks = get_chunks(&tiles.view());

    commands.insert_resource(WorldData {
        width: tiles.ncols() as u16,
        height: tiles.nrows() as u16,
        chunks,
        colliders,
    });

    commands.insert_resource(NextState(GameState::InGame));
}

fn get_chunks(world: &ArrayView2<Cell>) -> Vec<Chunk> {
    let mut chunks = vec![];

    for offset_y in (0..world.nrows()).step_by(CHUNK_HEIGHT) {
        for offset_x in (0..world.ncols()).step_by(CHUNK_WIDTH) {
            let cells: Array2<Cell> = world
                .slice(s![
                    offset_y..(offset_y + CHUNK_HEIGHT).clamp(0, world.nrows()),
                    offset_x..(offset_x + CHUNK_WIDTH).clamp(0, world.ncols())
                ])
                .to_owned();

            if cells.nrows() > 0 && cells.ncols() > 0 {
                chunks.push(Chunk {
                    bounds: FRect {
                        left: offset_x as f32 * TILE_SIZE,
                        right: (offset_x as f32 * TILE_SIZE + cells.ncols() as f32 * TILE_SIZE),
                        top: -(offset_y as f32) * TILE_SIZE,
                        bottom: -(offset_y as f32 * TILE_SIZE + cells.nrows() as f32 * TILE_SIZE),
                    },
                    cells,
                    spawned: false,
                });
            }
        }
    }

    chunks
}

fn spawn_tile(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    tile: Tile,
    ix: usize,
    x: f32,
    iy: usize,
    y: f32,
) -> Entity {
    let index = get_tile_sprite_index(tile.slope);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite { index, ..default() },
            texture_atlas,
            transform: Transform::from_xyz(x, y, 0.1).with_scale(Vec3::splat(1.05)),
            ..default()
        })
        .insert(tile.tile_type)
        .insert(Name::new(format!("Block Tile {} {}", ix, iy)))
        .insert(RigidBody::Fixed)
        .insert(NoFrustumCulling)
        .insert(Sleeping {
            sleeping: true,
            ..default()
        })
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
    let index = get_wall_sprite_index(wall.slope);

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

fn get_tile_sprite_index(slope: Slope) -> usize {
    let rand: usize = thread_rng().gen_range(1..3);

    match slope {
        // All
        Slope::ALL => rand + 16,
        // None
        Slope::NONE => 16 * 3 + rand + 8,
        // Top
        Slope::TOP => 16 * 3 + rand + 5,
        // Bottom
        Slope::BOTTOM => rand + 6,
        // Left
        Slope::LEFT => (rand - 1) * 16 + 12,
        // Right
        Slope::RIGHT => (rand - 1) * 16 + 9,
        // Top Bottom
        Slope::TOP_BOTTOM => (rand - 1) * 16 + 5,
        // Top Left Right
        Slope::TOP_LEFT_RIGHT => 16 * 2 + rand + 1,
        // Bottom Left Right
        Slope::BOTTOM_LEFT_RIGHT => rand,
        // Left Right
        Slope::LEFT_RIGHT => 4 * 16 + 5 + rand,
        // Bottom Left
        Slope::BOTTOM_LEFT => 16 * 3 + 1 + (rand - 1) * 2,
        // Bottom Right
        Slope::BOTTOM_RIGHT => 16 * 3 + (rand - 1) * 2,
        // Top Left
        Slope::TOP_LEFT => 16 * 4 + 1 + (rand - 1) * 2,
        // Top Right
        Slope::TOP_RIGHT => 16 * 4 + (rand - 1) * 2,
        // Top Bottom Left
        Slope::TOP_BOTTOM_LEFT => (rand - 1) * 16 + 4,
        // Top Bottom Right
        Slope::TOP_BOTTOM_RIGHT => (rand - 1) * 16,
    }
}

fn get_wall_sprite_index(slope: Slope) -> usize {
    let rand: usize = thread_rng().gen_range(1..3);

    match slope {
        // All
        Slope::ALL => 13 + rand,
        // None
        Slope::NONE => 13 * 3 + 8 + rand,
        // Top
        Slope::TOP => 13 * 2 + rand,
        // Bottom
        Slope::BOTTOM => 6 + rand,
        // Top Bottom
        Slope::TOP_BOTTOM => (rand - 1) * 13 + 5,
        // Bottom Right
        Slope::BOTTOM_RIGHT => 13 * 3 + (rand - 1) * 2,
        // Bottom Left
        Slope::BOTTOM_LEFT => 13 * 3 + 1 + (rand - 1) * 2,
        // Top Right
        Slope::TOP_RIGHT => 13 * 4 + (rand - 1) * 2,
        // Top Left
        Slope::TOP_LEFT => 13 * 4 + 1 + (rand - 1) * 2,
        // Left Right
        Slope::LEFT_RIGHT => 13 * 4 + 5 + rand,
        // Bottom Left Right
        Slope::BOTTOM_LEFT_RIGHT => 1 + rand,
        // Top Bottom Right
        Slope::TOP_BOTTOM_RIGHT => 13 * (rand - 1),
        // Top Bottom Left
        Slope::TOP_BOTTOM_LEFT => 13 * (rand - 1) + 4,
        // Top Left Right
        Slope::TOP_LEFT_RIGHT => 13 * 2 + rand,
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

fn update(
    mut commands: Commands,
    block_assets: Res<BlockAssets>,
    wall_assets: Res<WallAssets>,
    mut world_data: ResMut<WorldData>,
    camera_query: Query<
        (&GlobalTransform, &OrthographicProjection),
        (With<MainCamera>, Changed<GlobalTransform>),
    >,
) {
    if let Ok((camera_transform, projection)) = camera_query.get_single() {
        let camera_x = camera_transform.translation().x;
        let camera_y = camera_transform.translation().y;

        let camera_fov = FRect {
            left: camera_x + projection.left - 2. * TILE_SIZE,
            right: camera_x + projection.right + 2. * TILE_SIZE,
            top: camera_y - projection.top - 2. * TILE_SIZE,
            bottom: camera_y - projection.bottom + 2. * TILE_SIZE,
        };

        for chunk in world_data.chunks.iter_mut() {
            let inside = chunk.bounds.inside(camera_fov);

            match inside {
                true if !chunk.spawned => {
                    chunk.spawn(&mut commands, &block_assets, &wall_assets);
                }
                false if chunk.spawned => {
                    chunk.despawn(&mut commands);
                }
                _ => (),
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

fn handle_block_place(
    mut commands: Commands,
    mut events: EventReader<BlockPlaceEvent>,
    block_assets: Res<BlockAssets>
) {
    for event in events.iter() {
        spawn_tile(
            &mut commands, 
            block_assets.get_by_block(event.block).unwrap(), 
            Tile { tile_type: event.block, slope: Slope::NONE }, 
            event.coords.x as usize, 
            event.coords.x * 16.,
            event.coords.y as usize, 
            event.coords.y * 16.,
        );
    }
}
