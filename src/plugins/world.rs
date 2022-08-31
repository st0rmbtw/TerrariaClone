use std::{time::{UNIX_EPOCH, SystemTime}, collections::HashMap};

use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform, Component, Vec3, Handle, GlobalTransform}, sprite::{SpriteSheetBundle, TextureAtlasSprite, TextureAtlas}, core::Name};
use bevy_rapier2d::prelude::{Collider, Friction, RigidBody, Restitution};
use iyes_loopless::{prelude::AppLooplessStateExt, state::NextState};
use ndarray::{Array2, s, ArrayView2};
use rand::{Rng, thread_rng};

use crate::{world_generator::{generate, Slope, Cell}, state::GameState};

use super::{BlockAssets, WallAssets};

pub const TILE_SIZE: f32 = 16.;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::WorldLoading, spawn_terrain);
    }
}

pub struct WorldSettings {
    pub width: u16,
    pub height: u16
}

#[derive(Component)]
pub struct BlockMarker;

fn spawn_terrain(
    mut commands: Commands,
    block_assets: Res<BlockAssets>,
    wall_assets: Res<WallAssets>
) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    // let seed = current_time.as_millis() as u32;
    let seed = 3921763872;

    println!("The world's seed is {}", seed);

    println!("Generating world...");
    let tiles = generate(seed);
    
    commands.insert_resource(WorldSettings {
        width: tiles.ncols() as u16,
        height: tiles.nrows() as u16
    });

    println!("Loading chunk...");
    load_chunk(&mut commands, &block_assets, &wall_assets, &tiles, (150, 100), (0, 0));

    commands.insert_resource(NextState(GameState::InGame));
}

// size (width, height)
// offset (width, height)
fn load_chunk(
    commands: &mut Commands, 
    block_assets: &BlockAssets,
    wall_assets: &WallAssets,
    tiles: &Array2<Cell>,
    size: (usize, usize), 
    offset: (usize, usize)
) {
    let chunk = tiles.slice(s![(offset.1)..(offset.1 + size.1), (offset.0)..(offset.0 + size.0)]);

    let tiles_offset_x = offset.0 as f32 * TILE_SIZE;
    let tiles_offset_y = offset.1 as f32 * TILE_SIZE;

    for ((iy, ix), cell) in chunk.indexed_iter() {
        let x = tiles_offset_x + ix as f32 * TILE_SIZE;
        let y = tiles_offset_y + iy as f32 * TILE_SIZE;

        if let Some(tile) = cell.tile {
            if let Some(texture_atlas) = block_assets.get_by_block(tile.tile_type) {
                let index = get_tile_sprite_index(tile.slope);

                spawn_tile(commands, texture_atlas, index, ix, x, iy, y);
            }
        }

        if let Some(wall) = cell.wall {
            if let Some(texture_atlas) = wall_assets.get_by_wall(wall.wall_type) {
                let index = get_wall_sprite_index(wall.slope);

                spawn_wall(commands, texture_atlas, index, ix, x, iy, y);
            }
        }
    }

    spawn_colliders(commands, &chunk);
}

fn spawn_tile(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    index: usize,
    ix: usize,
    x: f32,
    iy: usize,
    y: f32
) {
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
        .insert(RigidBody::Fixed);
}

fn spawn_wall(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    index: usize,
    ix: usize,
    x: f32,
    iy: usize,
    y: f32
) {
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
    .insert(Name::new(format!("Wall {} {}", ix, iy)));
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
        // Top Left Right
        Slope { top: true,  bottom: false, left: true,  right: true  } => 13 * 2 + rand,
        _ => panic!("{:#?}", slope)
    }
}

fn spawn_colliders(
    commands: &mut Commands,
    chunk: &ArrayView2<Cell>
) {
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: usize,
        right: usize,
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Rect {
        left: usize,
        right: usize,
        top: usize,
        bottom: usize,
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

    let mut tile_rects: Vec<Rect> = Vec::new();
    let mut previous_rects: HashMap<Plate, Rect> = HashMap::new();

    // an extra empty row so the algorithm "terminates" the rects that touch the top
    // edge
    plate_stack.push(Vec::new());

    for (y, row) in plate_stack.iter().enumerate() {
        let mut current_rects: HashMap<Plate, Rect> = HashMap::new();

        for plate in row {
            if let Some(previous_rect) = previous_rects.remove(plate) {
                current_rects.insert(
                    *plate,
                    Rect {
                        top: previous_rect.top + 1,
                        ..previous_rect
                    },
                );
            } else {
                current_rects.insert(
                    *plate,
                    Rect {
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

    for tile_rect in tile_rects {
        commands
            .spawn()
            .insert(Collider::cuboid(
                (tile_rect.right as f32 - tile_rect.left as f32 + 1.) * TILE_SIZE / 2.,
                (tile_rect.top as f32 - tile_rect.bottom as f32 + 1.) * TILE_SIZE / 2.,
            ))
            .insert(RigidBody::Fixed)
            .insert(Friction::new(0.))
            .insert(Restitution::new(0.))
            .insert(Transform::from_xyz(
                (tile_rect.left + tile_rect.right) as f32 * TILE_SIZE / 2., 
                -((tile_rect.bottom + tile_rect.top) as f32 * TILE_SIZE / 2.), 
                0.
            ))
            .insert(GlobalTransform::default())
            .insert(Name::new("Terrain Collider"));
    }
}