use crate::{
    parallax::{LayerData, ParallaxResource, LayerSpeed, follow_camera_system},
    state::GameState, util::in_menu_state,
};
use bevy::{
    prelude::{default, App, Commands, Plugin, Res, ResMut, Vec2, Transform, Component, Query, Camera, GlobalTransform, With, OnEnter, OnExit, IntoSystemAppConfig, OnUpdate, IntoSystemConfig, IntoSystemConfigs, IntoSystemAppConfigs},
    sprite::{SpriteBundle, Anchor}, window::{Window, PrimaryWindow},
};
use rand::{thread_rng, Rng, seq::SliceRandom};

use super::{assets::BackgroundAssets, camera::MainCamera, world::{WorldData, TILE_SIZE}};

// region: Plugin
pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_main_menu_background.in_schedule(OnExit(GameState::AssetLoading)));
        app.add_system(spawn_stars.in_schedule(OnExit(GameState::AssetLoading)));

        app.add_systems(
            (
                despawn_background,
                setup_forest_background
            )
            .chain()
            .in_schedule(OnEnter(GameState::InGame))
        );

        app.add_system(despawn_background.in_schedule(OnExit(GameState::InGame)));

        app.add_system(move_stars.run_if(in_menu_state));
        app.add_system(follow_camera_system.in_set(OnUpdate(GameState::InGame)));
    }
}
// endregion

#[derive(Component)]
pub struct Star {
    screen_position: Vec2
}

fn despawn_background(mut commands: Commands, mut parallax: ResMut<ParallaxResource>) {
    parallax.despawn_layers(&mut commands);
    commands.remove_resource::<ParallaxResource>();
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

    for _ in 0..100 {
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
            }
        ));
    }
}

fn move_stars(
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query_stars: Query<(&mut Transform, &Star)>
) {
    let (camera, camera_transform) = query_camera.single();

    for (mut star_transform, star) in &mut query_stars {
        if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, star.screen_position) {
            star_transform.translation.x = world_position.x;
            star_transform.translation.y = world_position.y;
        }
    }
}

// region: Main menu background

fn setup_main_menu_background(
    query_windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
) {
    let window = query_windows.single();

    // 600 is the background image height
    let height = window.height() - 600.;

    commands.insert_resource(ParallaxResource {
        layer_data: vec![
            LayerData {
                speed: LayerSpeed::Horizontal(1.),
                scale: 1.,
                z: 0.0,
                image: backgrounds.background_0.clone(),
                fill_screen_height: true,
                // position: Vec2::NEG_Y * (window.height() - 1400. / 2.),
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.9),
                image: backgrounds.background_112.clone_weak(),
                z: 0.0,
                transition_factor: 1.,
                scale: 2.,
                position: Vec2::NEG_Y * height + 400.,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.9),
                image: backgrounds.background_7.clone_weak(),
                z: 0.1,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.8),
                image: backgrounds.background_90.clone_weak(),
                z: 1.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 200.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.7),
                image: backgrounds.background_91.clone_weak(),
                z: 2.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 300.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.6),
                image: backgrounds.background_92.clone_weak(),
                z: 3.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 400.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.7),
                image: backgrounds.background_112.clone_weak(),
                z: 4.0,
                transition_factor: 1.,
                scale: 1.2,
                position: Vec2::NEG_Y * height + 200.,
                ..default()
            },
        ],
        ..default()
    });
}

// fn setup_ingame_background(
//     mut commands: Commands,
//     backgrounds: Res<BackgroundAssets>,
//     world_data: Res<WorldData>
// ) {
//     let underground_level = world_data.layer.underground as f32 * TILE_SIZE;
//     let cavern_level = world_data.layer.cavern as f32 * TILE_SIZE;

//     let tiles_count = (cavern_level - underground_level) / 96.;

//     let layer_entity = commands.spawn_empty();

//     for y in 0..tiles_count as i32 {
//         layer_entity.insert(SpriteBundle {
//             transform: Transform::from_translation(),
//             texture: backgrounds.background_74.clone_weak(),
//             ..default()
//         })
//     }
// }

fn setup_forest_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
    world_data: Res<WorldData>
) {
    let cavern_layer = world_data.layer.cavern as f32 * TILE_SIZE;

    commands.insert_resource(ParallaxResource {
        layer_data: vec![
            LayerData {
                speed: LayerSpeed::Bidirectional(0.9, 0.6),
                image: backgrounds.background_55.clone_weak(),
                z: 0.4,
                transition_factor: 1.,
                scale: 2.5,
                position: Vec2::NEG_Y * cavern_layer - 100.,
                anchor: Anchor::BottomCenter,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.6, 0.5),
                image: backgrounds.background_114.clone_weak(),
                z: 0.3,
                transition_factor: 1.,
                scale: 2.,
                position: Vec2::NEG_Y * cavern_layer,
                anchor: Anchor::BottomCenter,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.4, 0.4),
                image: backgrounds.background_93.clone_weak(),
                z: 0.2,
                transition_factor: 1.,
                scale: 2.,
                position: Vec2::NEG_Y * cavern_layer,
                anchor: Anchor::BottomCenter,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(1., 1.),
                image: backgrounds.background_0.clone_weak(),
                z: 0.0,
                transition_factor: 1.,
                scale: 1.5,
                position: Vec2::splat(TILE_SIZE / 2.),
                anchor: Anchor::TopCenter,
                ..default()
            },
        ],
        ..default()
    });
}

// endregion
