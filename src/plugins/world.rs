use std::{time::{UNIX_EPOCH, SystemTime}, collections::LinkedList};

use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform, StartupStage, Vec2, BuildChildren, Visibility}, sprite::{SpriteSheetBundle, TextureAtlasSprite}, core::Name, math::vec2};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, Friction, RigidBody, Restitution, Sleeping};
use ndarray::{Array2, s, ArrayView2};
use rand::Rng;

use crate::{world_generator::generate, block::{get_block_by_id, BLOCK_DIRT_ID, BlockId}};

use super::BlockAssets;

pub const TILE_SIZE: f32 = 16.;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system_to_stage(StartupStage::PreStartup, spawn_terrain);
    }
}

pub struct WorldSettings {
    pub width: u16,
    pub height: u16
}

fn spawn_terrain(
    mut commands: Commands,
    block_assets: Res<BlockAssets>
) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    
    println!("Generating world...");
    let tiles = generate(current_time.as_millis() as u32);
    
    commands.insert_resource(WorldSettings {
        width: tiles.ncols() as u16,
        height: tiles.nrows() as u16
    });


    println!("Loading chunk...");
    load_chunk(&mut commands, block_assets, &tiles, (150, 100), ((tiles.ncols()) / 2, 0), true);
}

// size (width, height)
// offset (width, height)
fn load_chunk(commands: &mut Commands, block_assets: Res<BlockAssets>, tiles: &Array2<BlockId>, size: (usize, usize), offset: (usize, usize), first_chunk: bool) {
    let half_width = (size.0 / 2) as f32;

    let chunk = tiles.slice(s![(offset.1)..(offset.1 + size.1), (offset.0)..(offset.0 + size.0)]);

    for ((y, x), tile) in chunk.indexed_iter() {
        let mut x = (-half_width * TILE_SIZE) + x as f32 * TILE_SIZE;

        x = offset.0 as f32 + x;
        let y = offset.1 as f32 + y as f32;
        
        if let Some(texture_atlas) = block_assets.get_by_id(*tile) {
            let index = rand::thread_rng().gen_range(1..3) + match *tile {
                BLOCK_DIRT_ID => 16,
                _ => 0
            };

            commands
                .spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index,
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..default()
                    },
                    texture_atlas,
                    transform: Transform::from_xyz(x, -y * TILE_SIZE, 0.),
                    ..default()
                })
                .insert(Name::new("Block Tile"))
                .insert(RigidBody::Fixed)
                .with_children(|cmd| {
                    cmd.spawn()
                        .insert(Collider::cuboid(TILE_SIZE / 2., TILE_SIZE / 2.))
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(Friction::coefficient(0.))
                        .insert(Restitution::coefficient(0.))
                        .insert(Name::new("Terrain Collider"));
                });
        }
    }
}

fn find_non_air_block(tiles: &ArrayView2<BlockId>) -> (u16, u16) {
    let mut coords: (u16, u16) = (0, 0);

    for ((y, x), tile) in tiles.indexed_iter() {
        let block = get_block_by_id(*tile);

        if block.is_some() {
            coords = (x as u16, y as u16);
            break;
        }
    }

    coords
}

fn get_colliders(tiles: &ArrayView2<BlockId>, start: (u16, u16)) -> Vec<Vec2> {
    let mut blocks: Vec<Vec2> = Vec::new();

    let check = |x: u16, y: u16| -> bool {
        tiles.get((y as usize, x as usize)).and_then(|tile| get_block_by_id(*tile)).is_some()
    };

    let mut q: LinkedList<(u16, u16)> = LinkedList::new();
    q.push_back(start);

    while !q.is_empty() {
        let (x, y) = q.pop_back().unwrap();

        blocks.push(vec2(x as f32, y as f32));

        if check(x + 1, y) {
            q.push_back((x + 1, y));
        }

        if check(x, y + 1) {
            q.push_back((x, y + 1));
        }

        if x == tiles.ncols() as u16 - 1 && y == tiles.nrows() as u16 - 1 {
            break;
        }
    }

    blocks
}