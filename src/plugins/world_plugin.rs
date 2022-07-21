use bevy::{prelude::{Plugin, Commands, App, AssetServer, Res, ResMut, Assets, default, Transform}, sprite::{TextureAtlas, SpriteSheetBundle, TextureAtlasSprite}, math::{Vec2}, transform::TransformBundle, core::Name};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, RigidBody, Ccd};
use rand::Rng;

const TILE_WIDTH: f32 = 16.;
const TILE_HEIGHT: f32 = 16.;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(spawn_terrain);
    }
}

fn spawn_terrain(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    let texture_handle = assets.load("sprites/Tiles_477.png");
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle, Vec2::new(TILE_WIDTH, TILE_HEIGHT), 16, 15, Vec2::new(2., 2.)
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let blocks_count_half = (60 * 16) / 2;

    let mut rng = rand::thread_rng();

    for x in (-blocks_count_half..=blocks_count_half).step_by(TILE_WIDTH as usize) {
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    index: rng.gen_range(1..3),
                    ..default()
                },
                texture_atlas: texture_atlas_handle.clone(),
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