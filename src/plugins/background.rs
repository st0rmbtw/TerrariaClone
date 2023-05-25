use crate::{
    parallax::{LayerData, LayerSpeed, follow_camera_system, ParallaxSet, ParallaxContainer},
    common::{state::GameState, conditions::in_menu_state}, world::{WorldData, generator::DIRT_HILL_HEIGHT},
};
use bevy::{
    prelude::{default, App, Commands, Plugin, Res, Vec2, Transform, Component, Query, Camera, GlobalTransform, With, OnEnter, OnExit, IntoSystemAppConfig, OnUpdate, IntoSystemConfig, IntoSystemConfigs, IntoSystemAppConfigs, Name, Entity, DespawnRecursiveExt, Assets, Image},
    sprite::{SpriteBundle, Anchor}, window::{Window, PrimaryWindow},
};
use rand::{thread_rng, Rng, seq::SliceRandom};

use super::{assets::BackgroundAssets, camera::MainCamera, world::TILE_SIZE};

// region: Plugin
pub(crate) struct BackgroundPlugin;
impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_main_menu_background.in_schedule(OnExit(GameState::AssetLoading)));
        app.add_system(spawn_stars.in_schedule(OnExit(GameState::AssetLoading)));

        app.add_systems(
            (
                despawn_menu_background,
                spawn_sky_background,
                spawn_ingame_background,
                spawn_forest_background,
            )
            .chain()
            .in_schedule(OnEnter(GameState::InGame))
        );

        app.add_system(despawn_menu_background.in_schedule(OnExit(GameState::InGame)));

        app.add_system(
            move_stars
                .run_if(in_menu_state)
                .before(ParallaxSet::FollowCamera)
        );
        app.add_system(
            follow_camera_system
                .in_set(OnUpdate(GameState::InGame))
        );
    }
}
// endregion

#[derive(Component)]
pub(crate) struct Star {
    screen_position: Vec2
}

#[derive(Component)]
pub(crate) struct MenuParallaxContainer;

#[derive(Component)]
pub(crate) struct BiomeParallaxContainer;

#[derive(Component)]
pub(crate) struct InGameParallaxContainer;

fn despawn_menu_background(
    mut commands: Commands,
    query_menu_parallax_container: Query<Entity, With<MenuParallaxContainer>>
) {
    let entity = query_menu_parallax_container.single();
    commands.entity(entity).despawn_recursive();
}

fn spawn_stars(
    mut commands: Commands,
    query_windows: Query<&Window, With<PrimaryWindow>>,
    background_assets: Res<BackgroundAssets>
) {
    let mut rng = thread_rng();
    let window = query_windows.single();

    let star_images = [
        background_assets.star_0.clone_weak(),
        background_assets.star_1.clone_weak(),
        background_assets.star_2.clone_weak(),
        background_assets.star_3.clone_weak(),
        background_assets.star_4.clone_weak(),
    ];

    for i in 0..100 {
        let x = rng.gen_range(0f32..window.width());
        let y = rng.gen_range(0f32..window.height());

        let star_image = star_images.choose(&mut rng).unwrap();

        commands.spawn((
            SpriteBundle {
                texture: star_image.clone_weak(),
                ..default()
            },
            Star {
                screen_position: Vec2::new(x, y)
            },
            Name::new(format!("Star {i}")),
        ));
    }
}

fn move_stars(
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query_stars: Query<(&mut Transform, &Star)>
) {
    let (camera, camera_transform) = query_camera.single();

    for (mut star_transform, star) in &mut query_stars {
        let world_position = camera.viewport_to_world_2d(camera_transform, star.screen_position).unwrap();
        star_transform.translation.x = world_position.x;
        star_transform.translation.y = world_position.y;
    }
}

fn setup_main_menu_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
) {
    let pos = 150.;

    commands.spawn((
        Name::new("Menu Parallax Container"),
        MenuParallaxContainer,
        ParallaxContainer::new(vec![
            LayerData {
                speed: LayerSpeed::Horizontal(1.),
                scale: 1.,
                z: 0.0,
                image: backgrounds.background_0.clone_weak(),
                fill_screen_height: true,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.9),
                image: backgrounds.background_7.clone_weak(),
                z: 0.1,
                transition_factor: 1.,
                position: Vec2::NEG_Y * pos,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.8),
                image: backgrounds.background_90.clone_weak(),
                z: 1.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * pos - 200.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.7),
                image: backgrounds.background_91.clone_weak(),
                z: 2.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * pos - 300.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.6),
                image: backgrounds.background_92.clone_weak(),
                z: 3.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * pos - 400.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.7),
                image: backgrounds.background_112.clone_weak(),
                z: 4.0,
                transition_factor: 1.,
                scale: 1.2,
                position: Vec2::NEG_Y * pos + 200.,
                ..default()
            },
        ])
    ));
}

fn spawn_sky_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
) {
    commands.spawn((
        Name::new("Sky Parallax Container"),
        ParallaxContainer::new(vec![
            LayerData {
                speed: LayerSpeed::Bidirectional(1., 1.),
                image: backgrounds.background_0.clone_weak(),
                z: 0.0,
                transition_factor: 1.,
                scale: 6.,
                position: Vec2::splat(TILE_SIZE / 2.),
                anchor: Anchor::TopCenter,
                ..default()
            },
        ])
    ));
}

fn spawn_ingame_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
    world_data: Res<WorldData>,
    images: Res<Assets<Image>>
) {
    let underground_level = world_data.layer.underground as f32 * TILE_SIZE;
    let world_height = world_data.size.height as f32 * TILE_SIZE;

    let image = images.get(&backgrounds.background_78).unwrap();
    let image_height = image.size().y;

    let mut layers = Vec::new();

    let layer_options = LayerData {
        speed: LayerSpeed::Horizontal(0.8),
        z: 0.5,
        transition_factor: 1.2,
        scale: 1.,
        ..default()
    };

    layers.push(LayerData {
        image: backgrounds.background_77.clone_weak(),
        position: Vec2::NEG_Y * underground_level,
        anchor: Anchor::BottomCenter,
        ..layer_options.clone()
    });

    let mut position = underground_level;
    while position < world_height {
        layers.push(LayerData {
            image: backgrounds.background_78.clone_weak(),
            position: Vec2::NEG_Y * position,
            anchor: Anchor::TopCenter,
            ..layer_options.clone()
        });
        
        position += image_height;
    }

    commands.spawn((
        Name::new("InGame Parallax Container"),
        InGameParallaxContainer,
        ParallaxContainer::new(layers)
    ));
}

fn spawn_forest_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
    world_data: Res<WorldData>
) {
    commands.spawn((
        Name::new("Biome Parallax Container"),
        BiomeParallaxContainer,
        ParallaxContainer::new(vec![
            LayerData {
                speed: LayerSpeed::Bidirectional(0.9, 0.6),
                image: backgrounds.background_55.clone_weak(),
                z: 0.4,
                transition_factor: 1.,
                scale: 2.,
                position: (world_data.layer.underground - world_data.layer.dirt_height + DIRT_HILL_HEIGHT) as f32 * TILE_SIZE * Vec2::NEG_Y,
                anchor: Anchor::Center,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.6, 0.5),
                image: backgrounds.background_114.clone_weak(),
                z: 0.3,
                transition_factor: 1.,
                scale: 2.,
                position: (world_data.layer.underground - world_data.layer.dirt_height) as f32 * TILE_SIZE * Vec2::NEG_Y,
                anchor: Anchor::Center,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.4, 0.4),
                image: backgrounds.background_93.clone_weak(),
                z: 0.2,
                transition_factor: 1.,
                scale: 1.5,
                position: (world_data.layer.underground - world_data.layer.dirt_height) as f32 * TILE_SIZE * Vec2::NEG_Y,
                anchor: Anchor::BottomCenter,
                ..default()
            },
        ])
    ));
}
