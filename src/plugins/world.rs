use std::{time::{UNIX_EPOCH, SystemTime}, collections::HashMap};

use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform, Component, Vec3, Handle, GlobalTransform, With, Query, Changed, OrthographicProjection, ResMut, Entity}, sprite::{SpriteSheetBundle, TextureAtlasSprite, TextureAtlas}, core::Name};
use bevy_rapier2d::prelude::{Collider, Friction, RigidBody, Restitution};
use iyes_loopless::{prelude::{AppLooplessStateExt, ConditionSet}, state::NextState};
use ndarray::{Array2, s, ArrayView2};
use rand::{Rng, thread_rng};

use crate::{world_generator::{generate, Slope, Cell}, state::GameState};

use super::{BlockAssets, WallAssets, MainCamera};

pub const TILE_SIZE: f32 = 16.;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::WorldLoading, spawn_terrain)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(update)
                    .into()
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
            bottom: self.bottom as f32
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct FRect {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

#[derive(Clone, Copy, Default)]
pub struct ColliderData {
    rect: URect,
    entity: Option<Entity>
}

pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub cells: Box<Array2<Cell>>,
    pub colliders: Vec<ColliderData>
}

#[derive(Component)]
pub struct BlockMarker;

fn spawn_terrain(
    mut commands: Commands,
    block_assets: Res<BlockAssets>,
    wall_assets: Res<WallAssets>
) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let seed = current_time.as_millis() as u32;
    // let seed = 4118821582;

    println!("The world's seed is {}", seed);

    println!("Generating world...");
    let tiles = generate(seed);
    let colliders = get_colliders(&tiles.view());

    commands.insert_resource(WorldData {
        width: tiles.ncols() as u16,
        height: tiles.nrows() as u16,
        cells: Box::new(tiles),
        colliders
    });

    println!("Loading chunk...");
    // load_chunk(&mut commands, &block_assets, &wall_assets, &tiles, (150, 50), (0, 0));

    commands.insert_resource(NextState(GameState::InGame));
}

fn spawn_tile(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    index: usize,
    ix: usize,
    x: f32,
    iy: usize,
    y: f32
) -> Entity {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index,
                ..default()
            },
            texture_atlas,
            transform: Transform::from_xyz(x, -y, 0.1).with_scale(Vec3::splat(1.05)),
            ..default()
        })
        .insert(BlockMarker)
        .insert(Name::new(format!("Block Tile {} {}", ix, iy)))
        .insert(RigidBody::Fixed)
        .id()
}

fn spawn_wall(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    index: usize,
    ix: usize,
    x: f32,
    iy: usize,
    y: f32
) -> Entity {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index,
                ..default()
            },
            texture_atlas,
            transform: Transform::from_xyz(x, -y, 0.).with_scale(Vec3::splat(1.05)),
            ..default()
        })
        .insert(Name::new(format!("Wall {} {}", ix, iy)))
        .id()
}

fn get_tile_sprite_index(slope: Slope) -> usize {
    let rand: usize = thread_rng().gen_range(1..3);

    match slope {
        // All
        Slope { top: true,  bottom: true,  left: true,  right: true  } => rand + 16,
        // None
        Slope { top: false, bottom: false, left: false, right: false } => 16 * 3 + rand + 8,
        // Bottom
        Slope { top: false, bottom: true,  left: false, right: false } => rand + 6,
        // Top
        Slope { top: true,  bottom: false, left: false, right: false } => 16 * 3 + rand + 5,
        // Left
        Slope { top: false, bottom: false, left: true,  right: false } => (rand - 1) * 16 + 12,
        // Right
        Slope { top: false, bottom: false, left: false, right: true  } => (rand - 1) * 16 + 9,
        // Top Bottom
        Slope { top: true,  bottom: true,  left: false, right: false } => (rand - 1) * 16 + 5,
        // Bottom Left Right
        Slope { top: false, bottom: true,  left: true,  right: true  } => rand,
        // Left Right
        Slope { top: false, bottom: false, left: true,  right: true  } => 4 * 16 + 5 + rand,
        // Top Left Right
        Slope { top: true,  bottom: false, left: true,  right: true  } => 16 * 2 + rand + 1,
        // Bottom Right
        Slope { top: false, bottom: true,  left: false, right: true  } => 16 * 3 + (rand - 1) * 2,
        // Bottom Left
        Slope { top: false, bottom: true,  left: true,  right: false } => 16 * 3 + 1 + (rand - 1) * 2,
        // Top Right
        Slope { top: true,  bottom: false, left: false, right: true  } => 16 * 4 + (rand - 1) * 2,
        // Top Left
        Slope { top: true,  bottom: false, left: true,  right: false } => 16 * 4 + 1 + (rand - 1) * 2,
        // Top Bottom Right
        Slope { top: true,  bottom: true,  left: false, right: true  } => (rand - 1) * 16,
        // Top Bottom Left
        Slope { top: true,  bottom: true,  left: true,  right: false } => (rand - 1) * 16 + 4
    }
}

fn get_wall_sprite_index(slope: Slope) -> usize {
    let rand: usize = thread_rng().gen_range(1..3);

    match slope {
        // All
        Slope { top: true,  bottom: true,  left: true,  right: true  } => 13 + rand,
        // None
        Slope { top: false, bottom: false, left: false, right: false } => 13 * 3 + 8 + rand,
        // Bottom
        Slope { top: false, bottom: true,  left: false, right: false } => 6 + rand,
        // Top
        Slope { top: true,  bottom: false, left: false, right: false } => 13 * 2 + rand,
        // Top Bottom
        Slope { top: true,  bottom: true,  left: false, right: false } => (rand - 1) * 13 + 5,
        // Bottom Right
        Slope { top: false, bottom: true,  left: false, right: true  } => 13 * 3 + (rand - 1) * 2,
        // Bottom Left
        Slope { top: false, bottom: true,  left: true,  right: false } => 13 * 3 + 1 + (rand - 1) * 2,
        // Top Right
        Slope { top: true,  bottom: false, left: false, right: true  } => 13 * 4 + (rand - 1) * 2,
        // Top Left
        Slope { top: true,  bottom: false, left: true,  right: false } => 13 * 4 + 1 + (rand - 1) * 2,
        // Left Right
        Slope { top: false, bottom: false, left: true,  right: true  } => 13 * 4 + 5 + rand,
        // Bottom Left Right
        Slope { top: false, bottom: true,  left: true,  right: true  } => 1 + rand,
        // Top Bottom Right
        Slope { top: true,  bottom: true,  left: false, right: true  } => 13 * (rand - 1),
        // Top Bottom Left
        Slope { top: true,  bottom: true,  left: true, right: false  } => 13 * (rand - 1) + 4,
        // Top Left Right
        Slope { top: true,  bottom: false, left: true,  right: true  } => 13 * 2 + rand,
        _ => panic!("{:#?}", slope)
    }
}

fn get_colliders(
    chunk: &ArrayView2<Cell>
) -> Vec<ColliderData> {
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
                        right: x - 1
                    });
                    plate_start = None;
                },
                (None, true) => plate_start = Some(x),
                _ => ()
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

    tile_rects.iter().map(|rect| ColliderData {
        rect: *rect,
        ..default()
    }).collect()
}

fn spawn_collider(
    commands: &mut Commands,
    rect: URect
) -> Entity{
    commands
        .spawn()
        .insert(Collider::cuboid(
            (rect.right as f32 - rect.left as f32 + 1.) * TILE_SIZE / 2.,
            (rect.top as f32 - rect.bottom as f32 + 1.) * TILE_SIZE / 2.,
        ))
        .insert(RigidBody::Fixed)
        .insert(Friction::new(0.))
        .insert(Restitution::new(0.))
        .insert(Transform::from_xyz(
            (rect.left + rect.right) as f32 * TILE_SIZE / 2., 
            -((rect.bottom + rect.top) as f32 * TILE_SIZE / 2.), 
            0.
        ))
        .insert(GlobalTransform::default())
        .insert(Name::new("Terrain Collider"))
        .id()
}

fn update(
    mut commands: Commands,
    block_assets: Res<BlockAssets>,
    wall_assets: Res<WallAssets>,
    mut world_data: ResMut<WorldData>,
    camera_query: Query<(&GlobalTransform, &OrthographicProjection), (With<MainCamera>, Changed<GlobalTransform>)>
) {
    if let Ok((camera_transform, projection)) = camera_query.get_single() {
        let camera_x = camera_transform.translation().x;
        let camera_y = camera_transform.translation().y;

        let camera_fov = FRect {
            left: camera_x + projection.left,
            right: camera_x + projection.right,
            top: camera_y.abs() - projection.top,
            bottom: camera_y.abs() - projection.bottom
        };

        let start_x = ((camera_fov.left / TILE_SIZE) - 5.).clamp(0., world_data.cells.ncols() as f32) as usize;
        let end_x = ((camera_fov.right / TILE_SIZE) + 5.).clamp(0., world_data.cells.ncols() as f32) as usize;
        let start_y = ((camera_fov.top / TILE_SIZE) - 5.).clamp(0., world_data.cells.nrows() as f32) as usize;
        let end_y = ((camera_fov.bottom / TILE_SIZE) + 5.).clamp(0., world_data.cells.nrows() as f32) as usize;
        
        // Despawn all tiles and walls that are not in the camera fov
        for ((y, x), cell) in world_data.cells.indexed_iter_mut() {
            if (x < start_x || x > end_x) || (y < start_y || y > end_y) {
                if let Some(entity) = cell.tile_entity {
                    commands.entity(entity).despawn();
                }

                if let Some(entity) = cell.wall_entity {
                    commands.entity(entity).despawn();
                }

                cell.tile_entity = None;
                cell.wall_entity = None;
            }   
        }

        // Spawn all tiles and walls that are in the camera fov
        for ((iy, ix), cell) in world_data.cells.slice_mut(s![start_y..end_y, start_x..end_x]).indexed_iter_mut() {
            let x = start_x as f32 * TILE_SIZE + ix as f32 * TILE_SIZE;
            let y = start_y as f32 * TILE_SIZE + iy as f32 * TILE_SIZE;

            if let Some(tile) = cell.tile {
                if let Some(texture_atlas) = block_assets.get_by_block(tile.tile_type) {
                    if cell.tile_entity.is_none() {
                        let index = get_tile_sprite_index(tile.slope);

                        let entity = spawn_tile(&mut commands, texture_atlas, index, ix, x, iy, y);
                        
                        cell.tile_entity = Some(entity);
                    }
                }
            }
    
            if let Some(wall) = cell.wall {
                if let Some(texture_atlas) = wall_assets.get_by_wall(wall.wall_type) {
                    if cell.wall_entity.is_none() {
                        let index = get_wall_sprite_index(wall.slope);

                        let entity = spawn_wall(&mut commands, texture_atlas, index, ix, x, iy, y);

                        cell.wall_entity = Some(entity);
                    }
                }
            }
        }
        
        // Spawn colliders for tiles that are in the camera fov
        for collider in world_data.colliders.iter_mut() {
            let rect = collider.rect.to_frect();

            let inside = inside((rect.bottom * TILE_SIZE, rect.left * TILE_SIZE), camera_fov) || 
                    inside((rect.top * TILE_SIZE, rect.right * TILE_SIZE), camera_fov);

            if inside && collider.entity.is_none() {
                let entity = spawn_collider(&mut commands, collider.rect);
                collider.entity = Some(entity);
            };
        }
    }
}

fn inside(p: (f32, f32), rect: FRect) -> bool {
    p.0 < rect.bottom && p.0 > rect.top && p.1 > rect.left && p.1 < rect.right
}