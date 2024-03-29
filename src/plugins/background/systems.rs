use crate::{
    parallax::{
        LayerComponent, LayerData, LayerDataComponent, LayerSpeed, LayerTextureComponent,
        ParallaxCameraComponent, ParallaxContainer,
    },
    world::WorldData,
    BACKGROUND_LAYER
};
use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, tonemapping::Tonemapping},
    prelude::{
        default, Assets, Camera, Camera2d, Camera2dBundle, Commands, Image, Name, Query, Res,
        Transform, UiCameraConfig, Vec2, With, Without,
    },
    sprite::{Anchor, Sprite}
};

use crate::plugins::{
    assets::BackgroundAssets,
    camera::components::{BackgroundCamera, InGameBackgroundCamera, MoveCamera, ZoomableCamera},
    world::{constants::TILE_SIZE, time::GameTime},
    DespawnOnGameExit,
};

use super::{
    BiomeParallaxContainer, InGameParallaxContainer, MenuParallaxContainer,
    BACKGROUND_RENDER_LAYER, INGAME_BACKGROUND_RENDER_LAYER,
};

pub(super) fn follow_camera_system(
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

pub(super) fn spawn_background_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("BackgroundCamera"),
        BackgroundCamera,
        MoveCamera,
        ParallaxCameraComponent,
        BACKGROUND_RENDER_LAYER,
        UiCameraConfig { show_ui: false },
        DespawnOnGameExit,
        Camera2dBundle {
            camera: Camera {
                order: -2,
                msaa_writeback: false,
                ..default()
            },
            tonemapping: Tonemapping::None,
            ..default()
        },
    ));
}

pub(super) fn spawn_ingame_background_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("InGameBackgroundCamera"),
        InGameBackgroundCamera,
        MoveCamera,
        ZoomableCamera,
        INGAME_BACKGROUND_RENDER_LAYER,
        UiCameraConfig { show_ui: false },
        DespawnOnGameExit,
        Camera2dBundle {
            camera: Camera {
                order: -1,
                msaa_writeback: false,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            tonemapping: Tonemapping::None,
            ..default()
        },
    ));
}

pub(super) fn setup_main_menu_background(
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
                position: Vec2::NEG_Y * pos,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.8),
                image: backgrounds.background_90.clone_weak(),
                z: BACKGROUND_LAYER + 0.3,
                position: Vec2::NEG_Y * pos - 200.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.7),
                image: backgrounds.background_91.clone_weak(),
                z: BACKGROUND_LAYER + 0.4,
                position: Vec2::NEG_Y * pos - 300.,
                scale: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.6),
                image: backgrounds.background_92.clone_weak(),
                z: BACKGROUND_LAYER + 0.5,
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
        .with_render_layer(BACKGROUND_RENDER_LAYER),
    ));
}

pub(super) fn spawn_sky_background(mut commands: Commands, backgrounds: Res<BackgroundAssets>) {
    commands.spawn((
        Name::new("Sky Parallax Container"),
        DespawnOnGameExit,
        ParallaxContainer::new(vec![LayerData {
            speed: LayerSpeed::Bidirectional(1., 0.),
            image: backgrounds.background_0.clone_weak(),
            z: BACKGROUND_LAYER,
            scale: 1.,
            anchor: Anchor::Center,
            ..default()
        }])
        .with_render_layer(BACKGROUND_RENDER_LAYER),
    ));
}

pub(super) fn spawn_ingame_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
    world_data: Res<WorldData>,
    images: Res<Assets<Image>>,
) {
    let underground_level = world_data.layer.underground as f32 * TILE_SIZE;
    let world_height = world_data.height() as f32 * TILE_SIZE;

    let image = images.get(&backgrounds.background_78).unwrap();
    let image_height = image.size().y;

    let mut layers = Vec::new();

    let layer_options = LayerData {
        speed: LayerSpeed::Horizontal(0.8),
        z: BACKGROUND_LAYER + 0.5,
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
        ParallaxContainer::new(layers).with_render_layer(INGAME_BACKGROUND_RENDER_LAYER),
    ));
}

pub(super) fn spawn_forest_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
    world_data: Res<WorldData>,
) {
    commands.spawn((
        Name::new("Biome Parallax Container"),
        BiomeParallaxContainer,
        DespawnOnGameExit,
        ParallaxContainer::new(vec![
            LayerData {
                speed: LayerSpeed::Bidirectional(0.8, 0.6),
                image: backgrounds.background_55.clone_weak(),
                z: BACKGROUND_LAYER + 0.4,
                scale: 2.5,
                position: (world_data.layer.underground - world_data.layer.dirt_height / 2) as f32
                    * TILE_SIZE
                    * Vec2::NEG_Y,
                anchor: Anchor::Center,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.4, 0.5),
                image: backgrounds.background_114.clone_weak(),
                z: BACKGROUND_LAYER + 0.3,
                scale: 2.,
                position: (world_data.layer.underground - world_data.layer.dirt_height / 2) as f32
                    * TILE_SIZE
                    * Vec2::NEG_Y,
                anchor: Anchor::Center,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.2, 0.4),
                image: backgrounds.background_93.clone_weak(),
                z: BACKGROUND_LAYER + 0.2,
                scale: 2.,
                position: (world_data.layer.underground - world_data.layer.dirt_height) as f32
                    * TILE_SIZE
                    * Vec2::NEG_Y,
                anchor: Anchor::Center,
                ..default()
            },
        ])
        .with_render_layer(BACKGROUND_RENDER_LAYER),
    ));
}

pub(super) fn update_sprites_color(
    game_time: Res<GameTime>,
    mut query_sprite: Query<&mut Sprite, With<LayerTextureComponent>>,
) {
    let ambient_color = game_time.ambient_color();

    for mut sprite in &mut query_sprite {
        sprite.color = ambient_color;
    }
}
