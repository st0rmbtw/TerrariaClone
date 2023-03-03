use crate::{
    parallax::{LayerData, ParallaxResource},
    state::GameState,
};
use bevy::{
    prelude::{default, App, Commands, Plugin, Res, ResMut, Vec2},
    window::Windows,
};
use iyes_loopless::prelude::*;

use super::assets::BackgroundAssets;

// region: Plugin
pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::MainMenu, setup_main_menu_background)
            .add_exit_system(GameState::MainMenu, despawn_background)
            .add_enter_system(GameState::InGame, setup_game_background)
            .add_exit_system(GameState::InGame, despawn_background);
    }
}
// endregion

fn despawn_background(mut commands: Commands, mut parallax: ResMut<ParallaxResource>) {
    parallax.despawn_layers(&mut commands);
    commands.remove_resource::<ParallaxResource>();
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
                speed: 0.9,
                image: backgrounds.background_112.clone(),
                z: 0.05,
                transition_factor: 1.,
                scale: 2.,
                position: Vec2::NEG_Y * height + 400.,
                ..default()
            },
            LayerData {
                speed: 0.9,
                image: backgrounds.background_7.clone(),
                z: 0.1,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: 0.8,
                image: backgrounds.background_90.clone(),
                z: 1.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 200.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: 0.7,
                image: backgrounds.background_91.clone(),
                z: 2.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 300.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: 0.6,
                image: backgrounds.background_92.clone(),
                z: 3.0,
                transition_factor: 1.,
                position: Vec2::NEG_Y * height - 400.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: 0.7,
                image: backgrounds.background_112.clone(),
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

// endregion

// region: Game background

fn setup_game_background(mut commands: Commands, backgrounds: Res<BackgroundAssets>) {
    commands.insert_resource(ParallaxResource {
        layer_data: vec![LayerData {
            speed: 1.,
            image: backgrounds.background_0.clone(),
            z: 0.0,
            scale: 1.,
            ..default()
        }],
        ..default()
    })
}

// endregion
