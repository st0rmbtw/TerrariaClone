use bevy::{prelude::{Plugin, Commands, App, AssetServer, Res, ResMut, Assets, default, Transform}, sprite::{TextureAtlas, SpriteSheetBundle, TextureAtlasSprite}, math::Vec2, hierarchy::BuildChildren};
use bevy_rapier2d::prelude::{Collider, ActiveEvents, RigidBody};


const TILE_WIDTH: f32 = 16.;
const TILE_HEIGHT: f32 = 16.;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(init_sprite_sheet);
    }
}

fn init_sprite_sheet(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    let texture_handle = assets.load("sprites/Tiles_0.png");
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle, Vec2::new(TILE_WIDTH, TILE_HEIGHT), 16, 15, Vec2::new(0., 3.)
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let blocks_count = (60 * 16) / 2;

    for x in (-blocks_count..=blocks_count).step_by(16) {
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    index: 5,
                    ..default()
                },
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_xyz(x as f32, -20., 0.),
                ..default()
            })
            .insert(RigidBody::Fixed)
            .with_children(|children| {
                // Collider
                children.spawn()
                    .insert(Collider::cuboid(TILE_WIDTH / 2., TILE_HEIGHT / 2.))
                    .insert(ActiveEvents::COLLISION_EVENTS);
            });
    }
}