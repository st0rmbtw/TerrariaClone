use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform, StartupStage}, sprite::{SpriteSheetBundle, TextureAtlasSprite}, transform::TransformBundle, core::Name};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, Friction};
use rand::Rng;

use super::{BlockAssets, TILE_SIZE};

// const WORLD_SIZE: (i32, i32) = (1750, 900);
const WORLD_SIZE: (i32, i32) = (1750, 5);

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
    let mut rng = rand::thread_rng();

    for y in 0..WORLD_SIZE.1 {
        for x in (-WORLD_SIZE.0 / 2)..(WORLD_SIZE.0 / 2) {
            let (texture_atlas, index) = match y {
                0 => (block_assets.grass.clone(), rng.gen_range(1..3)),
                _ => (block_assets.dirt.clone(), rng.gen_range(1..3) + 21)
            };

            commands
                .spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite { 
                        index,
                        ..default()
                    },
                    texture_atlas,
                    transform: Transform::from_xyz((x as f32) * TILE_SIZE, -((y as f32) * TILE_SIZE), 0.),
                    ..default()
                })
                .insert(Name::new("Block Tile"));
        }
    }

    commands.spawn()
        .insert(Collider::cuboid((WORLD_SIZE.0 as f32 * TILE_SIZE) / 2., TILE_SIZE / 2. - 1.))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Friction::coefficient(0.))
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(0., -2., 0.)))
        .insert(Name::new("Terrain Collider"));
}