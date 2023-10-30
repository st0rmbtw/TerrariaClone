use std::time::Duration;

use bevy::{prelude::{Commands, Res, Plugin, App, Query, With, Handle, GlobalTransform, Camera, Vec2, Transform, Local, Input, MouseButton, Color, IntoSystemConfigs, Name, Update, OnEnter, Without, Entity, Deref, DerefMut, PreUpdate, on_event, Vec3, Component, ResMut, Bezier, CubicGenerator, Resource}, sprite::{Sprite, SpriteSheetBundle, TextureAtlasSprite, TextureAtlas, SpriteBundle}, window::{Window, PrimaryWindow, WindowResized}, utils::default, ecs::query::Has, math::cubic_splines::CubicCurve};
use interpolation::EaseFunction;
use rand::{thread_rng, Rng, seq::SliceRandom};

use crate::{plugins::{assets::{CelestialBodyAssets, BackgroundAssets}, camera::components::BackgroundCamera, background::{BACKGROUND_RENDER_LAYER, BackgroundPlugin}, cursor::position::CursorPosition, MenuSystemSet, world::time::GameTime}, animation::{Tween, Animator, RepeatStrategy, RepeatCount, lens::TransformScaleLens}, common::state::GameState, common::{rect::FRect, systems::despawn_with}, parallax::{LayerTextureComponent, ParallaxSet}, BACKGROUND_LAYER};

use super::{DespawnOnMenuExit, Dragging};

pub(super) struct CelestialBodyPlugin;
impl Plugin for CelestialBodyPlugin {
    fn build(&self, app: &mut App) {
        // This plugin depends on resources and components that BackgroundPlugin creates
        if !app.get_added_plugins::<BackgroundPlugin>().is_empty() {
            app.insert_resource(CelestialBodyCurve(
                Bezier::new(vec![[
                    0.5,
                    0.9,
                    0.9,
                    0.5,
                ]]).to_curve()
            ));

            app.add_systems(
                OnEnter(GameState::Menu),
                (
                    spawn_celestial_body,
                    spawn_stars
                )
            );

            app.add_systems(
                Update,
                (
                    update_celestial_body_position,
                    reset_celestial_body_position
                )
                .chain()
                .in_set(MenuSystemSet::Update)
            );

            app.add_systems(
                Update,
                (
                    move_celestial_body,
                    move_stars
                )
                .in_set(MenuSystemSet::Update)
                .after(ParallaxSet::FollowCamera)
            );

            app.add_systems(
                PreUpdate,
                (
                    despawn_with::<Star>,
                    spawn_stars
                )
                .in_set(MenuSystemSet::PreUpdate)
                .run_if(on_event::<WindowResized>())
            );

            app.add_systems(
                Update,
                (
                    update_celestial_body_sprite,
                    drag_celestial_body,
                    update_sprites_color,
                    change_visibility_of_stars,
                )
                .in_set(MenuSystemSet::Update)
            );
        }
    }
}

const SUN_SIZE: f32 = 42.;
const MOON_SIZE: f32 = 50.;
const SUNRISE_THRESHOLD: f32 = 0.2;
const SUNSET_THRESHOLD: f32 = 0.8;
const OFFSCREEN: f32 = 150.;

#[derive(Component, Clone, Copy, Default, PartialEq, Deref, DerefMut)]
pub(super) struct CelestialBodyPosition(Vec2);

#[derive(Component)]
struct Star {
    screen_position: Vec2
}

#[derive(Resource, Deref)]
struct CelestialBodyCurve(CubicCurve<f32>);

fn spawn_celestial_body(
    mut commands: Commands,
    game_time: Res<GameTime>,
    celestial_body_assets: Res<CelestialBodyAssets>,
    celestial_body_curve: Res<CelestialBodyCurve>
) {
    let progress = game_time.value as f32 / game_time.duration() as f32;

    commands.spawn((
        Name::new("Celestial Body"),
        CelestialBodyPosition(Vec2::new(progress, celestial_body_curve.position(progress))),
        DespawnOnMenuExit,
        BACKGROUND_RENDER_LAYER,
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 0,
                ..default()
            },
            texture_atlas: celestial_body_assets.sun.clone_weak(),
            transform: Transform::from_xyz(0., 0., BACKGROUND_LAYER + 0.1),
            ..default()
        },
    ));
}

fn spawn_stars(
    mut commands: Commands,
    background_assets: Res<BackgroundAssets>,
    query_windows: Query<&Window, With<PrimaryWindow>>
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

    let star_count = (window.width() + window.height() / 2.) / 10.;

    let bundles = (0..star_count as i32)
        .map(|i| {
            let x = rng.gen_range(0f32..window.width());
            let y = rng.gen_range(0f32..window.height() / 2.);

            let star_image = star_images.choose(&mut rng).unwrap();

            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                RepeatStrategy::MirroredRepeat,
                Duration::from_secs_f32(rng.gen_range(0.75..1.5)),
                TransformScaleLens {
                    start: Vec3::splat(1.),
                    end: Vec3::splat(0.5),
                }
            )
            .with_repeat_count(RepeatCount::Infinite);

            (
                Name::new(format!("Star {i}")),
                DespawnOnMenuExit,
                BACKGROUND_RENDER_LAYER,
                Animator::new(tween),
                SpriteBundle {
                    texture: star_image.clone_weak(),
                    transform: Transform {
                        translation: Vec3::new(0., 0., BACKGROUND_LAYER + 0.05),
                        scale: Vec3::splat(rng.gen_range(0.25..=1.0)),
                        ..default()
                    },
                    ..default()
                },
                Star {
                    screen_position: Vec2::new(x, y)
                }
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(bundles)
}

fn move_stars(
    query_camera: Query<(&Camera, &Transform), With<BackgroundCamera>>,
    mut query_stars: Query<(&mut Transform, &Star), Without<BackgroundCamera>>
) {
    let (camera, camera_transform) = query_camera.single();

    let camera_global_transform = GlobalTransform::from_translation(camera_transform.translation);

    for (mut star_transform, star) in &mut query_stars {
        if let Some(world_position) = camera.viewport_to_world_2d(&camera_global_transform, star.screen_position) {
            star_transform.translation.x = world_position.x;
            star_transform.translation.y = world_position.y;
        }
    }
}

fn change_visibility_of_stars(
    game_time: Res<GameTime>,
    mut query_stars: Query<&mut Sprite, With<Star>>,
) {
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

fn move_celestial_body(
    query_windows: Query<&Window, With<PrimaryWindow>>,
    query_camera: Query<(&Camera, &Transform), With<BackgroundCamera>>,
    mut query_celestial_body: Query<(&mut Transform, &CelestialBodyPosition), (Without<Dragging>, Without<BackgroundCamera>)>,
) {
    let (camera, camera_transform) = query_camera.single();
    let Ok((mut celestial_body_transform, celestial_body_pos)) = query_celestial_body.get_single_mut() else { return; };

    let window = query_windows.single();
    let window_size = Vec2::new(window.width() + OFFSCREEN * 2., window.height());
    let camera_global_transform = GlobalTransform::from_translation(camera_transform.translation);
    let position = Vec2::new(celestial_body_pos.x, 1. - celestial_body_pos.y);

    if let Some(world_pos) = camera.viewport_to_world_2d(&camera_global_transform, (position * window_size) - Vec2::new(OFFSCREEN, 0.)) {
        celestial_body_transform.translation.x = world_pos.x;
        celestial_body_transform.translation.y = world_pos.y;
    }
}

fn drag_celestial_body(
    mut commands: Commands,
    query_window: Query<&Window, With<PrimaryWindow>>,
    input: Res<Input<MouseButton>>,
    mut game_time: ResMut<GameTime>,
    cursor_position: Res<CursorPosition<BackgroundCamera>>,
    mut query_celestial_body: Query<(Entity, &mut Transform, &mut CelestialBodyPosition, Has<Dragging>)>,
) {
    let (entity, mut transform, mut position, is_dragging) = query_celestial_body.single_mut();

    let window = query_window.single();
    let window_size = Vec2::new(window.width() + OFFSCREEN * 2., window.height());

    let size = if game_time.is_day { SUN_SIZE } else { MOON_SIZE };

    let rect = FRect::new_center(transform.translation.x, transform.translation.y, size, size);

    let cursor_is_on_celestial_body = rect.contains(cursor_position.world);

    if input.pressed(MouseButton::Left) && (cursor_is_on_celestial_body || is_dragging) {
        transform.translation.x = cursor_position.world.x;
        transform.translation.y = cursor_position.world.y;

        position.x = (cursor_position.screen.x + OFFSCREEN) / (window_size.x);
        position.y = 1. - (cursor_position.screen.y / window_size.y);

        game_time.value = (position.x * game_time.duration() as f32).round() as u32;

        commands.entity(entity).insert(Dragging);
    } else {
        commands.entity(entity).remove::<Dragging>();
    }
}

fn update_celestial_body_sprite(
    game_time: Res<GameTime>,
    celestial_body_assets: Res<CelestialBodyAssets>,
    mut query: Query<(&mut Handle<TextureAtlas>, &mut TextureAtlasSprite), With<CelestialBodyPosition>>,
    mut moon_index: Local<usize>,
    mut prev_is_day: Local<bool>
) {
    if game_time.is_day != *prev_is_day {
        let (mut texture, mut atlas_sprite) = query.single_mut();
        if game_time.is_day {
            *texture = celestial_body_assets.sun.clone_weak();
            atlas_sprite.index = 0;
        } else {
            let mut rng = thread_rng();
            let moons = celestial_body_assets.moons();

            *moon_index = (*moon_index + 1) % (moons.len() - 1);
            atlas_sprite.index = rng.gen_range(0..8);
            *texture = moons[*moon_index].clone_weak();
        }
    }

    *prev_is_day = game_time.is_day;
}

fn update_sprites_color(
    game_time: Res<GameTime>,
    mut query_sprite: Query<&mut Sprite, With<LayerTextureComponent>>,
) { 
    let ambient_color = Color::from(game_time.get_ambient_color().extend(1.));

    for mut sprite in &mut query_sprite {
        sprite.color = ambient_color;
    }
}

fn reset_celestial_body_position(
    game_time: Res<GameTime>,
    mut query_celestial_body: Query<&mut CelestialBodyPosition>,
    mut prev_is_day: Local<Option<bool>>
) {
    if let None = *prev_is_day {
        *prev_is_day = Some(game_time.is_day);
    }

    if game_time.is_day != prev_is_day.unwrap() {
        if let Ok(mut position) = query_celestial_body.get_single_mut() {
            position.x = 0.;
            position.y = 0.5;
        }
    }

    *prev_is_day = Some(game_time.is_day);
}

fn update_celestial_body_position(
    game_time: Res<GameTime>,
    mut query_celestial_body: Query<&mut CelestialBodyPosition, Without<Dragging>>,
    curve: Res<CelestialBodyCurve>
) {
    if let Ok(mut position) = query_celestial_body.get_single_mut() {
        let progress = game_time.value as f32 / game_time.duration() as f32;

        position.x = progress;

        let new_y = curve.position(progress);
        if (new_y - position.y).abs() > 0.005 {
            position.y += (new_y - position.y) * (GameTime::RATE_MENU as f32 / game_time.duration() as f32);
        } else {
            position.y = new_y;
        }
    }
}
