use crate::{
    parallax::{LayerData, ParallaxResource},
    state::GameState, util::screen_to_world,
};
use bevy::{
    prelude::{default, App, Commands, Plugin, Res, ResMut, Vec2, Transform, Component, Query, Camera, GlobalTransform, With},
    window::Windows, sprite::SpriteBundle,
};
use iyes_loopless::prelude::*;
use rand::{thread_rng, Rng, seq::SliceRandom};

use super::{assets::BackgroundAssets, camera::MainCamera};

// region: Plugin
pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::MainMenu, setup_main_menu_background)
            .add_enter_system(GameState::InGame, despawn_background)
            .add_exit_system(GameState::InGame, despawn_background)
            .add_enter_system(GameState::MainMenu, spawn_stars)
            .add_system(move_stars.run_in_state(GameState::MainMenu));
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
    windows: Res<Windows>,
    background_assets: Res<BackgroundAssets>
) {
    let mut rng = thread_rng();
    let window = windows.primary();

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
    windows: Res<Windows>,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query_stars: Query<(&mut Transform, &Star)>
) {
    let window = windows.primary();
    let window_size = Vec2::new(window.width(), window.height());

    let (camera, camera_transform) = query_camera.single();

    for (mut star_transform, star) in &mut query_stars {
        let star_world_position = screen_to_world(star.screen_position, window_size, camera, camera_transform);

        star_transform.translation.x = star_world_position.x;
        star_transform.translation.y = star_world_position.y;
    }
}

// region: Main menu background

fn setup_main_menu_background(
    wnds: Res<Windows>,
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
) {
    let window = wnds.get_primary().unwrap();

    // 600 is the background image height
    let height = window.height() - 600.;

    commands.insert_resource(ParallaxResource {
        layer_data: vec![
            LayerData {
                speed: 1.,
                scale: 1.,
                z: 0.0,
                image: backgrounds.background_0.clone(),
                fill_screen_height: true,
                // position: Vec2::NEG_Y * (window.height() - 1400. / 2.),
                ..default()
            },
            LayerData {
                speed: 0.9,
                image: backgrounds.background_112.clone_weak(),
                z: 0.0,
                transition_factor: 1.,
                scale: 2.,
                position: Vec2::NEG_Y * height + 400.
            },
            LayerData {
                speed: 0.9,
                image: backgrounds.background_7.clone_weak(),
                z: 0.1,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height,
                scale: 1.5
            },
            LayerData {
                speed: 0.8,
                image: backgrounds.background_90.clone_weak(),
                z: 1.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 200.,
                scale: 1.5
            },
            LayerData {
                speed: 0.7,
                image: backgrounds.background_91.clone_weak(),
                z: 2.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 300.,
                scale: 1.5
            },
            LayerData {
                speed: 0.6,
                image: backgrounds.background_92.clone_weak(),
                z: 3.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 400.,
                scale: 1.5
            },
            LayerData {
                speed: 0.7,
                image: backgrounds.background_112.clone_weak(),
                z: 4.0,
                transition_factor: 1.,
                scale: 1.2,
                position: Vec2::NEG_Y * height + 200.
            },
        ],
        ..default()
    });
}

// endregion
