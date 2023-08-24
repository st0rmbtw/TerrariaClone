use crate::{
    parallax::{LayerData, LayerSpeed, ParallaxContainer, ParallaxCameraComponent, LayerComponent, LayerDataComponent},
    common::{state::GameState, systems::despawn_with}, world::WorldData, BACKGROUND_LAYER
};
use bevy::{
    prelude::{default, App, Commands, Plugin, Res, Vec2, Query, Camera, With, OnExit, IntoSystemConfigs, Name, Assets, Image, Camera2dBundle, UiCameraConfig, PostUpdate, Transform, Without, Component, OnEnter},
    sprite::Anchor, render::view::RenderLayers,
};

use super::{assets::BackgroundAssets, camera::{components::BackgroundCamera, CameraSet}, world::constants::TILE_SIZE, InGameSystemSet, DespawnOnGameExit};

pub(crate) const BACKGROUND_RENDER_LAYER: RenderLayers = RenderLayers::layer(25);

// region: Plugin
pub(crate) struct BackgroundPlugin;
impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Menu),
            (
                spawn_background_camera,
                setup_main_menu_background
            )
        );

        app.add_systems(
            OnExit(GameState::WorldLoading),
            (
                despawn_with::<MenuParallaxContainer>,
                spawn_sky_background,
                spawn_ingame_background,
                spawn_forest_background,
            )
        );

        app.add_systems(
            PostUpdate,
            follow_camera_system
                .in_set(InGameSystemSet::PostUpdate)
                .after(CameraSet::MoveCamera),
        );
    }
}
// endregion

#[derive(Component)]
pub(crate) struct MenuParallaxContainer;

#[derive(Component)]
pub(crate) struct BiomeParallaxContainer;

#[derive(Component)]
pub(crate) struct InGameParallaxContainer;

fn follow_camera_system(
    query_parallax_camera: Query<&Transform, With<ParallaxCameraComponent>>,
    mut query_layer: Query<(&mut Transform, &LayerComponent, &LayerDataComponent), Without<ParallaxCameraComponent>>,
) {
    let Ok(camera_transform) = query_parallax_camera.get_single() else { return; };
    let camera_translation = camera_transform.translation.truncate();
    
    for (mut layer_transform, layer, layer_data) in &mut query_layer {
        let new_translation = camera_translation + (layer_data.position - camera_translation) * layer.speed;

        layer_transform.translation.x = new_translation.x;
        layer_transform.translation.y = new_translation.y;
    }
}

fn spawn_background_camera(
    mut commands: Commands
) {
    commands.spawn((
        Name::new("BackgroundCamera"),
        BackgroundCamera,
        ParallaxCameraComponent,
        BACKGROUND_RENDER_LAYER,
        UiCameraConfig { show_ui: false },
        DespawnOnGameExit,
        Camera2dBundle {
            camera: Camera {
                order: -1,
                ..default()
            },
            ..default()
        },
    ));
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
                z: BACKGROUND_LAYER,
                image: backgrounds.background_0.clone_weak(),
                fill_screen_height: true,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.9),
                image: backgrounds.background_7.clone_weak(),
                z: BACKGROUND_LAYER + 0.2,
                transition_factor: 1.,
                position: Vec2::NEG_Y * pos,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.8),
                image: backgrounds.background_90.clone_weak(),
                z: BACKGROUND_LAYER + 0.3,
                transition_factor: 1.,
                position: Vec2::NEG_Y * pos - 200.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.7),
                image: backgrounds.background_91.clone_weak(),
                z: BACKGROUND_LAYER + 0.4,
                transition_factor: 1.,
                position: Vec2::NEG_Y * pos - 300.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.6),
                image: backgrounds.background_92.clone_weak(),
                z: BACKGROUND_LAYER + 0.5,
                transition_factor: 1.,
                position: Vec2::NEG_Y * pos - 400.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.7),
                image: backgrounds.background_112.clone_weak(),
                z: BACKGROUND_LAYER + 0.6,
                transition_factor: 1.,
                scale: 1.2,
                position: Vec2::NEG_Y * pos + 200.,
                ..default()
            },
        ])
        .with_render_layer(BACKGROUND_RENDER_LAYER)
    ));
}

fn spawn_sky_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>
) { 
    commands.spawn((
        Name::new("Sky Parallax Container"),
        DespawnOnGameExit,
        ParallaxContainer::new(vec![
            LayerData {
                speed: LayerSpeed::Bidirectional(1., 0.),
                image: backgrounds.background_0.clone_weak(),
                z: BACKGROUND_LAYER,
                scale: 1.,
                position: Vec2::ZERO,
                anchor: Anchor::Center,
                ..default()
            },
        ])
        .with_render_layer(BACKGROUND_RENDER_LAYER)
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
        z: BACKGROUND_LAYER + 0.4,
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
        DespawnOnGameExit,
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
        DespawnOnGameExit,
        ParallaxContainer::new(vec![
            LayerData {
                speed: LayerSpeed::Bidirectional(0.8, 0.6),
                image: backgrounds.background_55.clone_weak(),
                z: BACKGROUND_LAYER + 0.3,
                transition_factor: 1.,
                scale: 2.,
                position: (world_data.layer.underground - world_data.layer.dirt_height) as f32 * TILE_SIZE * Vec2::NEG_Y,
                anchor: Anchor::Center,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.4, 0.5),
                image: backgrounds.background_114.clone_weak(),
                z: BACKGROUND_LAYER + 0.2,
                transition_factor: 1.,
                scale: 2.,
                position: (world_data.layer.underground - world_data.layer.dirt_height) as f32 * TILE_SIZE * Vec2::NEG_Y,
                anchor: Anchor::Center,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.2, 0.4),
                image: backgrounds.background_93.clone_weak(),
                z: BACKGROUND_LAYER + 0.1,
                transition_factor: 1.,
                scale: 2.,
                position: (world_data.layer.underground - world_data.layer.dirt_height) as f32 * TILE_SIZE * Vec2::NEG_Y,
                anchor: Anchor::Center,
                ..default()
            },
        ])
        .with_render_layer(BACKGROUND_RENDER_LAYER)
    ));
}
