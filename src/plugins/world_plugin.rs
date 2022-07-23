use bevy::{prelude::{Plugin, Commands, App, Res, default, Transform}, sprite::{SpriteSheetBundle, TextureAtlasSprite}, transform::TransformBundle, core::Name};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, RigidBody, Ccd};
use rand::Rng;

use super::BlockAssets;

pub const TILE_WIDTH: f32 = 16.;
pub const TILE_HEIGHT: f32 = 16.;

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

    for x in (-blocks_count_half..=blocks_count_half).step_by(TILE_WIDTH as usize) {
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    index: rng.gen_range(1..3),
                    ..default()
                },
                texture_atlas: block_assets.0.clone(),
                transform: Transform::from_xyz(x as f32, -30., 0.),
                ..default()
            })
            .insert(RigidBody::Fixed)
            .insert(Ccd::enabled())
            .insert(Name::new("Block tile"));
            // .with_children(|children| {
            //     // Collider
            //     children.spawn()
            //         .insert(Collider::cuboid(TILE_WIDTH / 2., TILE_HEIGHT / 2.))
            //         .insert(ActiveEvents::COLLISION_EVENTS);
            // });
    }

    commands.spawn()
        .insert(Collider::cuboid(blocks_count_half as f32, TILE_HEIGHT / 2.))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(0., -30., 0.)));
}