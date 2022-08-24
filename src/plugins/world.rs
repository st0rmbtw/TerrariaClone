use std::time::{UNIX_EPOCH, SystemTime};

use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform, Vec2, Component}, sprite::{SpriteSheetBundle, TextureAtlasSprite}, core::Name};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, Friction, RigidBody, Restitution};
use iyes_loopless::{prelude::AppLooplessStateExt, state::NextState};
use ndarray::{Array2, s};
use rand::Rng;

use crate::{world_generator::generate, block::{BLOCK_DIRT_ID, BlockId}, state::GameState};

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

    println!("Generating world...");
    let tiles = generate(current_time.as_millis() as u32);
    
    commands.insert_resource(WorldSettings {
        width: tiles.ncols() as u16,
        height: tiles.nrows() as u16
    });

    println!("Loading chunk...");
    load_chunk(&mut commands, block_assets, &tiles, (150, 100), ((tiles.ncols()) / 2, 0));

    commands.insert_resource(NextState(GameState::InGame));
}

// size (width, height)
// offset (width, height)
fn load_chunk(commands: &mut Commands, block_assets: Res<BlockAssets>, tiles: &Array2<BlockId>, size: (usize, usize), offset: (usize, usize)) {
    let half_width = (size.0 / 2) as f32;

    let chunk = tiles.slice(s![(offset.1)..(offset.1 + size.1), (offset.0)..(offset.0 + size.0)]);

    for ((iy, ix), tile) in chunk.indexed_iter() {
        let mut x = (-half_width * TILE_SIZE) + ix as f32 * TILE_SIZE;

        x = offset.0 as f32 + x;
        let y = offset.1 as f32 + iy as f32;
        
        if let Some(texture_atlas) = block_assets.get_by_id(*tile) {
            let index = rand::thread_rng().gen_range(1..3) + match *tile {
                BLOCK_DIRT_ID => 16,
                _ => 0
            };

            if iy == 0 {
                commands.spawn()
                    .insert(Transform::from_xyz(x, 0., 0.),)
                    .insert(Collider::cuboid(TILE_SIZE * 200., 2. * TILE_SIZE / 2.))
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(Friction::coefficient(0.))
                    .insert(Restitution::coefficient(0.))
                    .insert(Name::new("Terrain Collider"));
            }

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
                .insert(BlockMarker)
                .insert(Name::new("Block Tile"))
                .insert(RigidBody::Fixed);
                // .with_children(|cmd| {
                //     if iy <= 1 {
 
                //         cmd.spawn()
                //             .insert(Collider::cuboid(TILE_SIZE / 2., TILE_SIZE / 2.))
                //             .insert(ActiveEvents::COLLISION_EVENTS)
                //             .insert(Friction::coefficient(0.))
                //             .insert(Restitution::coefficient(0.))
                //             .insert(Name::new("Terrain Collider"));

                //     }
                // });
        }
    }
}