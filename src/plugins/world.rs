use std::time::{UNIX_EPOCH, SystemTime};

use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform, StartupStage}, sprite::{SpriteSheetBundle, TextureAtlasSprite}, transform::TransformBundle, core::Name};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, Friction};
use ndarray::{Array2, s};
use rand::Rng;

use crate::world_generator::generate;

use super::{BlockAssets, TILE_SIZE};

const WORLD_SIZE: (i32, i32) = (1750, 900);
// const WORLD_SIZE: (i32, i32) = (1750, 5);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system_to_stage(StartupStage::PreStartup, spawn_terrain);
    }
}

fn spawn_terrain(
    mut commands: Commands,
    block_assets: Res<BlockAssets>
) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    println!("Generating world...");
    let tiles = generate(current_time.as_millis() as u32);

    println!("{}", &tiles);

    println!("Loading chunk...");
    load_chunk(&mut commands, block_assets, &tiles, (200, 200), (tiles.ncols() / 2 - 200, 0));

    commands.spawn()
        .insert(Collider::cuboid((WORLD_SIZE.0 as f32 * TILE_SIZE) / 2., TILE_SIZE / 2. - 1.))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Friction::coefficient(0.))
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(0., -2., 0.)))
        .insert(Name::new("Terrain Collider"));
}

// size (width, height)
// offset (offset_width, offset_height)
fn load_chunk(commands: &mut Commands, block_assets: Res<BlockAssets>, tiles: &Array2<u32>, size: (usize, usize), offset: (usize, usize)) {
    for ((y, x), tile) in tiles.slice(s![(offset.1)..(offset.1 + size.1), (offset.0)..(offset.0 + size.0)]).indexed_iter() {
        if let Some(texture_atlas) = block_assets.get_by_id(*tile) {
            commands
                .spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite { 
                        index: rand::thread_rng().gen_range(1..3),
                        ..default()
                    },
                    texture_atlas,
                    transform: Transform::from_xyz((x as f32) * TILE_SIZE, -((y as f32) * TILE_SIZE), 0.),
                    ..default()
                })
                .insert(Name::new("Block Tile"));
        }
    }
}