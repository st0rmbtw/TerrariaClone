use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform}, sprite::{SpriteSheetBundle, TextureAtlasSprite}, transform::TransformBundle, core::Name};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, Friction};
use rand::Rng;

use super::{BlockAssets, TILE_SIZE};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(spawn_terrain);
    }
}

fn spawn_terrain(
    mut commands: Commands,
    block_assets: Res<BlockAssets>
) {
    let blocks_count_half = (60 * 16) / 2;

    let mut rng = rand::thread_rng();

    for x in (-blocks_count_half..blocks_count_half).step_by(TILE_SIZE as usize) {
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    index: rng.gen_range(1..3),
                    ..default()
                },
                texture_atlas: block_assets.grass.clone(),
                transform: Transform::from_xyz(x as f32, -30., 0.),
                ..default()
            })
            .insert(Name::new("Block tile"));
    }

    for y in ((TILE_SIZE as i32)..=(6 * TILE_SIZE as i32)).step_by(TILE_SIZE as usize) {
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    index: 21,
                    ..default()
                }, 
                texture_atlas: block_assets.dirt.clone(),
                transform: Transform::from_xyz(blocks_count_half as f32 / 2., -30. + y as f32, 0.),
                ..default()
            })
            .insert(Name::new("Block tile"));
    }

    commands.spawn()
        .insert(Collider::cuboid(blocks_count_half as f32, TILE_SIZE / 2. - 1.))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Friction::coefficient(0.))
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(0., -32., 0.)));
}