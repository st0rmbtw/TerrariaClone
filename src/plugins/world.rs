use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform, StartupStage, NodeBundle}, sprite::{SpriteSheetBundle, TextureAtlasSprite}, transform::TransformBundle, core::Name, hierarchy::BuildChildren};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, Friction};
use rand::Rng;

use super::{BlockAssets, TILE_SIZE};

// const WORLD_SIZE: (i32, i32) = (1750, 900);
const WORLD_SIZE: (i32, i32) = (1750, 1);

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
    // let blocks_count_half = (60 * 16) / 2;

    let mut rng = rand::thread_rng();

    for y in 0..WORLD_SIZE.1 {
        for x in (-WORLD_SIZE.0 / 2)..(WORLD_SIZE.0 / 2) {
            let texture_atlas = match y {
                0 => block_assets.grass.clone(),
                _ => block_assets.dirt.clone()
            };

            commands
                .spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite { 
                        index: rng.gen_range(1..3),
                        ..default()
                    },
                    texture_atlas,
                    transform: Transform::from_xyz((x as f32) * TILE_SIZE, -((y as f32) * TILE_SIZE) - 30., 0.),
                    ..default()
                })
                .insert(Name::new("Block Tile"));
        }
    }

    // for x in (-blocks_count_half..blocks_count_half).step_by(TILE_SIZE as usize) {
    //     commands
    //         .spawn_bundle(SpriteSheetBundle {
    //             sprite: TextureAtlasSprite { 
    //                 index: rng.gen_range(1..3),
    //                 ..default()
    //             },
    //             texture_atlas: block_assets.grass.clone(),
    //             transform: Transform::from_xyz(x as f32, -30., 0.),
    //             ..default()
    //         })
    //         .insert(Name::new("Block tile"));
    // }

    // for y in ((TILE_SIZE as i32)..=(6 * TILE_SIZE as i32)).step_by(TILE_SIZE as usize) {
    //     commands
    //         .spawn_bundle(SpriteSheetBundle {
    //             sprite: TextureAtlasSprite { 
    //                 index: 21,
    //                 ..default()
    //             }, 
    //             texture_atlas: block_assets.dirt.clone(),
    //             transform: Transform::from_xyz(blocks_count_half as f32 / 2., -30. + y as f32, 0.),
    //             ..default()
    //         })
    //         .insert(Name::new("Block tile"));
    // }

    commands.spawn()
        .insert(Collider::cuboid((WORLD_SIZE.0 as f32 * TILE_SIZE) / 2., TILE_SIZE / 2. - 1.))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Friction::coefficient(0.))
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(0., -32., 0.)))
        .insert(Name::new("Terrain Collider"));
}