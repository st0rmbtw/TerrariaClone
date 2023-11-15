use std::{
    f32::consts::FRAC_PI_2,
    time::Duration,
};

use bevy::{
    ecs::query::Has,
    prelude::{
        in_state, not, on_event, App, Commands, Entity, Handle, Input, IntoSystemConfigs, Local, MouseButton,
        Name, OnEnter, Plugin, PostUpdate, PreUpdate, Quat, Query, Res, ResMut, Transform, Update, Vec2, Vec3, With, Without, Component,
    },
    sprite::{Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    utils::default,
    window::{PrimaryWindow, Window, WindowResized},
};
use interpolation::{EaseFunction, Lerp};
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{
    animation::{lens::TransformScaleLens, Animator, RepeatCount, RepeatStrategy, Tween},
    common::state::GameState,
    common::{components::Dragging, rect::FRect, systems::despawn_with},
    parallax::ParallaxSet,
    plugins::{
        assets::{BackgroundAssets, SunAndMoonAssets},
        background::BACKGROUND_RENDER_LAYER,
        camera::{components::BackgroundCamera, CameraSet},
        config::RESOLUTIONS,
        cursor::position::CursorPosition,
        world::time::GameTime,
        InGameSystemSet, MenuSystemSet,
    },
    BACKGROUND_LAYER,
};

pub(super) struct SunAndMoonPlugin;
impl Plugin for SunAndMoonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), (spawn_sun_and_moon, spawn_stars));

        app.add_systems(
            PreUpdate,
            (despawn_with::<Star>, spawn_stars)
                .run_if(not(in_state(GameState::AssetLoading)))
                .run_if(on_event::<WindowResized>()),
        );

        app.add_systems(
            Update,
            (
                drag_sun_and_moon.in_set(MenuSystemSet::Update),

                (update_sun_and_moon_position).chain(),

                (move_sun_and_moon_menu)
                    .after(ParallaxSet::FollowCamera)
                    .in_set(MenuSystemSet::Update),
                
                update_sun_and_moon_sprite,
                update_sun_and_moon_color,
                update_sun_and_moon_rotation,
                update_sun_and_moon_scale,
                update_star_visibility,
            )
                .run_if(not(in_state(GameState::AssetLoading))),
        );

        app.add_systems(
            PostUpdate,
            (
                move_sun_and_moon_ingame,
                move_star
            )
            .after(CameraSet::MoveCamera)
            .in_set(InGameSystemSet::PostUpdate),
        );
    }
}

const SUN_SIZE: f32 = 42.;
const MOON_SIZE: f32 = 50.;
const SUNRISE_THRESHOLD: f32 = 0.15;
const SUNSET_THRESHOLD: f32 = 0.8;

#[derive(Component, Clone, Copy, Default, PartialEq)]
pub(crate) struct SunAndMoon {
    position: Vec2,
    dragged: bool
}

impl SunAndMoon {
    const fn new(position: Vec2) -> Self {
        Self { position, dragged: false }
    }
}

#[derive(Component)]
struct Star {
    position: Vec2
}

fn spawn_sun_and_moon(
    mut commands: Commands,
    game_time: Res<GameTime>,
    sun_and_moon_assets: Res<SunAndMoonAssets>,
) {
    let progress = game_time.progress();
    let y = get_sun_and_moon_y(progress);

    commands.spawn((
        Name::new("SunAndMoon"),
        SunAndMoon::new(Vec2::new(progress, y)),
        BACKGROUND_RENDER_LAYER,
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 0,
                ..default()
            },
            texture_atlas: sun_and_moon_assets.sun.clone_weak(),
            transform: Transform::from_xyz(0., 0., BACKGROUND_LAYER + 0.1),
            ..default()
        },
    ));
}

fn spawn_stars(mut commands: Commands, background_assets: Res<BackgroundAssets>) {
    let mut rng = thread_rng();

    let star_images = [
        background_assets.star_0.clone_weak(),
        background_assets.star_1.clone_weak(),
        background_assets.star_2.clone_weak(),
        background_assets.star_3.clone_weak(),
        background_assets.star_4.clone_weak(),
    ];

    let max_resolution = RESOLUTIONS[RESOLUTIONS.len() - 1];

    let star_count = (max_resolution.width + max_resolution.height) / 5.;

    let bundles = (0..star_count as i32)
        .map(|i| {
            let x = rng.gen_range(0f32..max_resolution.width);
            let y = rng.gen_range(0f32..max_resolution.height);

            let star_image = star_images.choose(&mut rng).unwrap();

            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                RepeatStrategy::MirroredRepeat,
                Duration::from_secs_f32(rng.gen_range(0.75..1.5)),
                TransformScaleLens {
                    start: Vec3::splat(1.),
                    end: Vec3::splat(0.5),
                },
            )
            .with_repeat_count(RepeatCount::Infinite);

            (
                Name::new(format!("Star {i}")),
                BACKGROUND_RENDER_LAYER,
                Animator::new(tween),
                SpriteBundle {
                    texture: star_image.clone_weak(),
                    transform: Transform {
                        translation: Vec3::new(x - max_resolution.width / 2., y - max_resolution.height / 2., BACKGROUND_LAYER + 0.05),
                        scale: Vec3::splat(rng.gen_range(0.25..=1.0)),
                        ..default()
                    },
                    ..default()
                },
                Star {
                    position: Vec2::new(x, y)
                }
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(bundles)
}

fn update_star_visibility(
    game_time: Res<GameTime>,
    mut query_stars: Query<&mut Sprite, With<Star>>,
) {
    if query_stars.is_empty() {
        return;
    }

    let x = game_time.value as f32 / game_time.duration() as f32;

    let mut alpha = if game_time.is_day { 0. } else { 1. };

    if game_time.is_day {
        if x <= SUNRISE_THRESHOLD {
            alpha = 1. - x / SUNRISE_THRESHOLD;
        } else if x >= SUNSET_THRESHOLD {
            alpha = (x - SUNSET_THRESHOLD) / (1. - SUNSET_THRESHOLD);
        };
    }

    query_stars.for_each_mut(|mut sprite| {
        sprite.color.set_a(alpha);
    });
}

fn move_sun_and_moon_menu(
    game_time: Res<GameTime>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    mut query_sun_and_moon: Query<
        (&mut Transform, &SunAndMoon),
        (Without<Dragging>, Without<BackgroundCamera>),
    >,
) {
    let Ok((mut sun_and_moon_transform, sun_and_moon)) = query_sun_and_moon.get_single_mut() else { return; };
    let Ok(window) = query_window.get_single() else { return; };

    let size = if game_time.is_day { SUN_SIZE } else { MOON_SIZE };

    let area_size = Vec2::new(window.width() + size * 2., window.height());
    let position = Vec2::new(sun_and_moon.position.x, 1. - sun_and_moon.position.y) * area_size;

    let world_position = 
        -Vec2::new(area_size.x / 2., -area_size.y / 2.)
        + Vec2::new(position.x, -position.y)
        - Vec2::new(size, 0.);

    sun_and_moon_transform.translation.x = world_position.x;
    sun_and_moon_transform.translation.y = world_position.y;
}

fn move_sun_and_moon_ingame(
    game_time: Res<GameTime>,
    query_camera: Query<&Transform, With<BackgroundCamera>>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    mut query_sun_and_moon: Query<(&mut Transform, &SunAndMoon), Without<BackgroundCamera>>,
) {
    let Ok(camera_transform) = query_camera.get_single() else { return; };
    let Ok((mut sun_and_moon_transform, sun_and_moon)) = query_sun_and_moon.get_single_mut() else { return; };
    let Ok(window) = query_window.get_single() else { return; };

    let size = if game_time.is_day { SUN_SIZE } else { MOON_SIZE };

    let area_size = Vec2::new(window.width() + size * 2., window.height());
    let position = Vec2::new(sun_and_moon.position.x, 1. - sun_and_moon.position.y) * area_size;

    sun_and_moon_transform.translation.x = camera_transform.translation.x - area_size.x / 2. + position.x - size;
    sun_and_moon_transform.translation.y = (camera_transform.translation.y + area_size.y / 2. - (position.y - camera_transform.translation.y) * 0.05)
        .min(camera_transform.translation.y + area_size.y / 2. - 50.);
}

fn move_star(
    query_camera: Query<&Transform, With<BackgroundCamera>>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    mut query_star: Query<(&mut Transform, &Star), Without<BackgroundCamera>>,
) {
    let Ok(camera_transform) = query_camera.get_single() else { return; };
    let Ok(window) = query_window.get_single() else { return; };

    let area_size = Vec2::new(window.width(), window.height());
    let world_position = camera_transform.translation.truncate() - Vec2::new(area_size.x / 2., -area_size.y / 2.);

    for (mut transform, star) in &mut query_star {
        let new_pos = world_position + Vec2::new(star.position.x, -star.position.y);
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

fn drag_sun_and_moon(
    mut commands: Commands,
    query_window: Query<&Window, With<PrimaryWindow>>,
    input: Res<Input<MouseButton>>,
    mut game_time: ResMut<GameTime>,
    cursor_position: Res<CursorPosition<BackgroundCamera>>,
    mut query_sun_and_moon: Query<(
        Entity,
        &mut Transform,
        &mut SunAndMoon,
        Has<Dragging>,
    )>,
) {
    let Ok((entity, mut transform, mut sun_and_moon, is_dragging)) = query_sun_and_moon.get_single_mut() else { return; };
    let window = query_window.single();

    let size = if game_time.is_day { SUN_SIZE } else { MOON_SIZE };
    let area_size = Vec2::new(window.width() + size * 2., window.height());

    let rect = FRect::new_center(transform.translation.x, transform.translation.y, size, size);

    let cursor_is_on_sun_and_moon = rect.contains(cursor_position.world);

    if input.pressed(MouseButton::Left) && (cursor_is_on_sun_and_moon || is_dragging) {
        transform.translation.x = cursor_position.world.x;
        transform.translation.y = cursor_position.world.y;

        sun_and_moon.position.x = (cursor_position.screen.x + size * 2.) / (area_size.x);
        sun_and_moon.position.y = 1. - (cursor_position.screen.y / area_size.y);

        game_time.value = (sun_and_moon.position.x * game_time.duration() as f32) as u32;

        sun_and_moon.dragged = true;

        commands.entity(entity).insert(Dragging);
    } else {
        commands.entity(entity).remove::<Dragging>();
    }
}   

fn update_sun_and_moon_sprite(
    game_time: Res<GameTime>,
    sun_and_moon_assets: Res<SunAndMoonAssets>,
    mut query: Query<
        (&mut Handle<TextureAtlas>, &mut TextureAtlasSprite),
        With<SunAndMoon>,
    >,
    mut moon_index: Local<usize>,
    mut prev_is_day: Local<bool>,
) {
    if game_time.is_day != *prev_is_day {
        let (mut texture, mut atlas_sprite) = query.single_mut();
        if game_time.is_day {
            *texture = sun_and_moon_assets.sun.clone_weak();
            atlas_sprite.index = 0;
        } else {
            let mut rng = thread_rng();
            let moons = sun_and_moon_assets.moons();

            *moon_index = (*moon_index + 1) % (moons.len() - 1);
            atlas_sprite.index = rng.gen_range(0..8);
            *texture = moons[*moon_index].clone_weak();
        }
    }

    *prev_is_day = game_time.is_day;
}

fn update_sun_and_moon_color(
    game_time: Res<GameTime>,
    mut query: Query<&mut TextureAtlasSprite, With<SunAndMoon>>,
) {
    let Ok(mut atlas_sprite) = query.get_single_mut() else { return; };
    atlas_sprite.color = game_time.sun_and_moon_color();
}

fn update_sun_and_moon_rotation(
    game_time: Res<GameTime>,
    mut query: Query<&mut Transform, With<SunAndMoon>>,
) {
    let Ok(mut transform) = query.get_single_mut() else { return; };

    let progress = game_time.progress();
    let progress = progress * 2. - 1.; // -1..1

    transform.rotation = Quat::from_rotation_z(progress * -FRAC_PI_2);
}

fn update_sun_and_moon_scale(
    game_time: Res<GameTime>,
    mut query: Query<&mut Transform, With<SunAndMoon>>,
) {
    let Ok(mut transform) = query.get_single_mut() else { return; };

    let progress = game_time.progress();
    let progress = 1. - (progress * 2. - 1.).abs(); // 0..1..0

    transform.scale = Vec3::splat(0.5 + progress / 2.);
}

fn get_sun_and_moon_y(progress: f32) -> f32 {
    let new_y = if progress < 0.5 {
        1. - (1. - progress * 2.).powf(2.)
    } else {
        1. - ((progress - 0.5) * 2.).powf(2.)
    };

    0.5.lerp(&0.9, &new_y)
}

fn update_sun_and_moon_position(
    game_time: Res<GameTime>,
    mut query_sun_and_moon: Query<&mut SunAndMoon, Without<Dragging>>,
) {
    let Ok(mut sun_and_moon) = query_sun_and_moon.get_single_mut() else { return; };

    let progress = game_time.progress();

    sun_and_moon.position.x = progress;

    const EPSILON: f32 = 0.0005;

    let new_y = get_sun_and_moon_y(progress);

    if sun_and_moon.dragged && (new_y - sun_and_moon.position.y).abs() > EPSILON {
        sun_and_moon.position.y += (new_y - sun_and_moon.position.y) * (game_time.rate() as f32 / game_time.duration() as f32);
    } else {
        sun_and_moon.dragged = false;
        sun_and_moon.position.y = new_y;
    }
}
