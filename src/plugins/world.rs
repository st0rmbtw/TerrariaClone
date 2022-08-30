use std::{time::{UNIX_EPOCH, SystemTime}, collections::HashMap};

use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform, Component, Vec3, Handle, GlobalTransform}, sprite::{SpriteSheetBundle, TextureAtlasSprite, TextureAtlas}, core::Name};
use bevy_rapier2d::prelude::{Collider, Friction, RigidBody, Restitution};
use iyes_loopless::{prelude::AppLooplessStateExt, state::NextState};
use ndarray::{Array2, s, ArrayView2};
use rand::{Rng, thread_rng};

use crate::{world_generator::{generate, Tile, Slope}, state::GameState, block::BLOCK_AIR};

use super::BlockAssets;

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
    block_assets: Res<BlockAssets>
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
    load_chunk(&mut commands, &block_assets, &tiles, (150, 100), (0, 0));

    commands.insert_resource(NextState(GameState::InGame));
}

// size (width, height)
// offset (width, height)
fn load_chunk(
    commands: &mut Commands, 
    block_assets: &BlockAssets, 
    tiles: &Array2<Tile>,
    size: (usize, usize), 
    offset: (usize, usize)
) {
    let chunk = tiles.slice(s![(offset.1)..(offset.1 + size.1), (offset.0)..(offset.0 + size.0)]);

    let tiles_offset_x = offset.0 as f32 * TILE_SIZE;
    let tiles_offset_y = offset.1 as f32 * TILE_SIZE;

    for ((iy, ix), tile) in chunk.indexed_iter() {
        let x = tiles_offset_x + ix as f32 * TILE_SIZE;
        let y = tiles_offset_y + iy as f32 * TILE_SIZE;
        
        if let Some(texture_atlas) = block_assets.get_by_id(tile.id) {
            let index = get_sprite_index_by_slope(tile.slope);

            spawn_tile(commands, texture_atlas, index, ix, x, iy, y);
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
            transform: Transform::from_xyz(x, -y, 0.).with_scale(Vec3::splat(1.05)),
            ..default()
        })
        .insert(BlockMarker)
        .insert(Name::new(format!("Block Tile {} {}", ix, iy)))
        .insert(RigidBody::Fixed);
}

fn get_sprite_index_by_slope(slope: Slope) -> usize {
    let rand: usize = thread_rng().gen_range(1..3);

    // Yeah, i know this looks horrible, but i dont know how to write it in another way

    if slope.is_all() {
        rand + 16
    } else if slope.is_none() {
        16 * 3 + rand + 8
    } else if slope.bottom && slope.left && slope.right && !slope.top {
        rand
    } else if slope.right && slope.left && !slope.bottom && !slope.top {
        4 * 16 + 5 + rand
    } else if slope.top && slope.left && slope.right && !slope.bottom {
        16 * 2 + rand + 1
    } else if slope.bottom && slope.right && !slope.top && !slope.left {
        16 * 3 + (rand - 1) * 2
    } else if slope.bottom && slope.left && !slope.top && !slope.right {
        16 * 3 + 1 + (rand - 1) * 2
    } else if slope.top && slope.right && !slope.bottom && !slope.left {
        16 * 4 + (rand - 1) * 2
    } else if slope.top && slope.left && !slope.bottom && !slope.right {
        16 * 4 + 1 + (rand - 1) * 2
    } else if slope.right && slope.bottom && slope.top && !slope.left {
        (rand - 1) * 16
    } else if slope.left && slope.bottom && slope.top && !slope.right {
        (rand - 1) * 16 + 4
    } else if slope.bottom && !slope.top && !slope.left && !slope.right {
        rand + 6
    } else if slope.top && !slope.bottom && !slope.left && !slope.right {
        16 * 3 + rand + 5
    } else if slope.right && !slope.left && !slope.top && !slope.bottom {
        (rand - 1) * 16 + 9
    } else if slope.left && !slope.right && !slope.top && !slope.bottom {
        (rand - 1) * 16 + 12
    } else {
        rand + 16
    }
}

fn spawn_colliders(
    commands: &mut Commands,
    chunk: &ArrayView2<Tile>
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
            let is_solid = matches!(chunk.get((y, x)), Some(tile) if tile.id != BLOCK_AIR); 

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