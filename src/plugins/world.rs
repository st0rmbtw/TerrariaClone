use std::time::{UNIX_EPOCH, SystemTime};

use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform, Component, BuildChildren}, sprite::{SpriteSheetBundle, TextureAtlasSprite}, core::Name};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, Friction, RigidBody, Restitution};
use iyes_loopless::{prelude::AppLooplessStateExt, state::NextState};
use ndarray::{Array2, s};
use rand::{Rng, thread_rng};

use crate::{world_generator::{generate, Tile}, state::GameState};

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
    let seed = 3700092736;

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
fn load_chunk(commands: &mut Commands, block_assets: &BlockAssets, tiles: &Array2<Tile>, size: (usize, usize), offset: (usize, usize)) {
    let chunk = tiles.slice(s![(offset.1)..(offset.1 + size.1), (offset.0)..(offset.0 + size.0)]);

    let tiles_offset_x = offset.0 as f32 * TILE_SIZE;
    let tiles_offset_y = offset.1 as f32 * TILE_SIZE;

    for ((iy, ix), tile) in chunk.indexed_iter() {
        let x = tiles_offset_x + ix as f32 * TILE_SIZE - ix as f32;
        let y = tiles_offset_y + iy as f32 * TILE_SIZE - iy as f32;
        
        if let Some(texture_atlas) = block_assets.get_by_id(tile.id) {
            let slope = tile.slope;

            let rand: usize = thread_rng().gen_range(1..3);

            // Yeah, i know this looks horrible, but i dont know how to write it in another way

            let index: usize = if slope.is_all() {
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
            };

            commands
                .spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index,
                        ..default()
                    },
                    texture_atlas,
                    transform: Transform::from_xyz(x, -y, 0.),
                    ..default()
                })
                .insert(BlockMarker)
                .insert(Name::new("Block Tile"))
                .insert(RigidBody::Fixed);
                // .with_children(|cmd| {
                //     cmd.spawn()
                //         .insert(Collider::cuboid(TILE_SIZE / 2., TILE_SIZE / 2.))
                //         .insert(ActiveEvents::COLLISION_EVENTS)
                //         .insert(Friction::coefficient(0.))
                //         .insert(Restitution::coefficient(0.))
                //         .insert(Name::new("Terrain Collider"));
                // });
        }
    }
    commands.spawn()
        .insert(Transform::from_xyz(0., 0., 0.),)
        .insert(Collider::cuboid(TILE_SIZE * 300., 2. * TILE_SIZE / 2.))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Friction::coefficient(0.))
        .insert(Restitution::coefficient(0.))
        .insert(Name::new("Terrain Collider"));
}