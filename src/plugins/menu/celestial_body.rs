use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, Component, Resource, Plugin, App, Query, With, EventReader, ResMut, Handle, GlobalTransform, Camera, Vec2, Transform, Local, Input, MouseButton, Color, Vec4, IntoSystemConfig, DetectChanges, IntoSystemConfigs, IntoSystemAppConfig, OnExit, Name}, sprite::{Sprite, SpriteSheetBundle, TextureAtlasSprite, TextureAtlas}, window::{Window, PrimaryWindow}};
use interpolation::Lerp;

use crate::{plugins::{assets::{CelestialBodyAssets}, camera::MainCamera, cursor::CursorPosition, background::Star}, animation::{Tween, EaseMethod, Animator, RepeatStrategy, RepeatCount, TweenCompleted, Lens, component_animator_system, AnimationSystemSet, AnimatorState}, common::state::GameState, common::{math::map_range_f32, rect::FRect}, parallax::LayerTextureComponent};

use super::in_menu_state;

pub(super) struct CelestialBodyPlugin;

impl Plugin for CelestialBodyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimeType>();
        app.add_system(setup.in_schedule(OnExit(GameState::AssetLoading)));
        app.add_system(component_animator_system::<CelestialBody>.in_set(AnimationSystemSet::AnimationUpdate));

        app.add_systems(
            (
                update_celestial_type,
                update_time_type,
                move_celestial_body,
                drag_celestial_body,
                update_sprites_color,
                change_visibility_of_stars,
            )
            .chain()
            .distributive_run_if(in_menu_state)
        );
    }
}

const CELESTIAL_BODY_ANIMATION_COMPLETED: u64 = 1;
const SUN_SIZE: f32 = 42.;
const MOON_SIZE: f32 = 50.;

#[derive(Component, Clone, Copy, Default, PartialEq)]
struct CelestialBody {
    position: Vec2
}

#[derive(Clone, Copy, PartialEq)]
struct CelestialBodyPositionLens {
    start: Vec2,
    end: Vec2
}

#[derive(Default, Resource, Clone, Copy)]
enum TimeType {
    #[default]
    Day,
    Night
}

const SUNRISE_THRESHOLD: f32 = 0.2;
const SUNSET_THRESHOLD: f32 = 0.8;

#[autodefault(except(CelestialBodyPositionLens))]
fn setup(
    mut commands: Commands,
    celestial_body_assets: Res<CelestialBodyAssets>
) {
    let celestial_body_animation = Tween::new(
        EaseMethod::Linear,
        RepeatStrategy::Repeat,
        Duration::from_secs(25),
        CelestialBodyPositionLens {
            start: Vec2::new(-0.1, 1. * 0.7),
            end: Vec2::new(1.1, 1.)
        }
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_completed_event(CELESTIAL_BODY_ANIMATION_COMPLETED);

    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 0
            },
            texture_atlas: celestial_body_assets.sun.clone_weak(),
            transform: Transform::IDENTITY,
        },
        Animator::new(celestial_body_animation),
        CelestialBody::default(),
        Name::new("Celestial Body")
    ));
}

fn change_visibility_of_stars(
    time_type: Res<TimeType>,
    mut query_stars: Query<&mut Sprite, With<Star>>,
    query_celestial_body: Query<&CelestialBody>,
    mut alpha: Local<f32>
) {
    let celestial_body = query_celestial_body.single();
    let celestial_body_position = celestial_body.position.x.clamp(0., 1.);
    
    if celestial_body_position <= SUNRISE_THRESHOLD {
        let s = map_range_f32(0., SUNRISE_THRESHOLD, 0., 1., celestial_body_position);

        *alpha = match *time_type {
            TimeType::Day => alpha.lerp(&0., &s),
            TimeType::Night => alpha.lerp(&1., &s),
        }
    } else if celestial_body_position >= SUNSET_THRESHOLD { 
        let s = map_range_f32(SUNSET_THRESHOLD, 1., 0., 1., celestial_body_position);

        *alpha = match *time_type {
            TimeType::Day => 0f32.lerp(&1., &s),
            TimeType::Night => 1f32.lerp(&0., &s),
        }
    };

    for mut sprite in &mut query_stars {
        sprite.color.set_a(*alpha);
    }
}

fn move_celestial_body(
    query_windows: Query<&Window, With<PrimaryWindow>>,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query_celestial_body: Query<(&mut Transform, &CelestialBody)>
) {
    let window = query_windows.single();
    let window_size = Vec2::new(window.width(), window.height());

    let (camera, camera_transform) = query_camera.single();
    let (mut celestial_body_transform, celestial_body) = query_celestial_body.single_mut();

    if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, celestial_body.position * window_size) {
        celestial_body_transform.translation.x = world_pos.x;
        celestial_body_transform.translation.y = world_pos.y;    
    }
}

fn update_time_type(
    mut events: EventReader<TweenCompleted>,
    mut time_type: ResMut<TimeType>
) {
    for event in events.iter() {
        if event.user_data == CELESTIAL_BODY_ANIMATION_COMPLETED {
            *time_type = match *time_type {
                TimeType::Day => TimeType::Night,
                TimeType::Night => TimeType::Day,
            };
        }
    }
}

fn drag_celestial_body(
    query_windows: Query<&Window>,
    input: Res<Input<MouseButton>>,
    cursor_position: Res<CursorPosition>,
    time_type: Res<TimeType>,
    mut query_celestial_body: Query<(&mut Transform, &mut Animator<CelestialBody>, &mut CelestialBody)>,
    mut dragging: Local<bool>,
) {
    let window = query_windows.single();

    let (mut celestial_body_transform, mut animator, mut celestial_body) = query_celestial_body.single_mut();

    let celestial_body_size = match *time_type {
        TimeType::Day => SUN_SIZE,
        TimeType::Night => MOON_SIZE,
    };

    let cb_rect = FRect {
        left: celestial_body_transform.translation.x - celestial_body_size / 2.,
        right: celestial_body_transform.translation.x + celestial_body_size / 2.,
        top: celestial_body_transform.translation.y + celestial_body_size / 2.,
        bottom: celestial_body_transform.translation.y - celestial_body_size / 2.,
    };

    let cursor_is_on_celestial_body = cb_rect.inside((cursor_position.world_position.x, cursor_position.world_position.y));

    if input.pressed(MouseButton::Left) && (cursor_is_on_celestial_body || *dragging) {
        celestial_body_transform.translation.x = cursor_position.world_position.x;
        celestial_body_transform.translation.y = cursor_position.world_position.y;
        celestial_body.position.x = cursor_position.position.x / window.width();
        celestial_body.position.y = cursor_position.position.y / window.height();

        let tween = animator.tweenable_mut();
        tween.set_progress(cursor_position.position.x / window.width());

        *dragging = true;
        animator.state = AnimatorState::Paused;
    } else {
        *dragging = false;
        animator.state = AnimatorState::Playing;
    }
}

fn update_celestial_type(
    time_type: Res<TimeType>,
    celestial_body_assets: Res<CelestialBodyAssets>,
    mut query: Query<&mut Handle<TextureAtlas>, With<CelestialBody>>,
) {
    if time_type.is_changed() {
        match *time_type {
            TimeType::Day => {
                let mut sprite = query.single_mut();
                *sprite = celestial_body_assets.sun.clone_weak();
            },
            TimeType::Night => {
                let mut sprite = query.single_mut();
                *sprite = celestial_body_assets.moon_0.clone_weak();
            },
        }
    }
}

fn update_sprites_color(
    time_type: Res<TimeType>,
    mut query_sprite: Query<&mut Sprite, With<LayerTextureComponent>>,
    query_celestial_body: Query<&CelestialBody>
) {
    let celestial_body = query_celestial_body.single();
    let celestial_body_position = celestial_body.position.x.clamp(0., 1.);

    let mut color = match *time_type {
        TimeType::Day => Color::WHITE,
        TimeType::Night => Color::rgb(0.2, 0.2, 0.2),
    };

    if celestial_body_position <= SUNRISE_THRESHOLD {
        let start: Vec4 = Color::rgb_u8(0, 54, 107).into();
        let end: Vec4 = color.into();
        let s = map_range_f32(0., SUNRISE_THRESHOLD, 0., 1., celestial_body_position);

        color = start.lerp(end, s).into();
    } else if celestial_body_position >= SUNSET_THRESHOLD {
        let start: Vec4 = color.into();
        let end: Vec4 = Color::rgb(0.3, 0.2, 0.3).into();
        let s = map_range_f32(SUNSET_THRESHOLD, 1., 0., 1., celestial_body_position);

        color = start.lerp(end, s).into();
    }
    
    for mut sprite in &mut query_sprite {
        sprite.color = color;
    }
}

impl Lens<CelestialBody> for CelestialBodyPositionLens {
    fn lerp(&mut self, target: &mut CelestialBody, ratio: f32) {
        target.position.x = self.start.x.lerp(&self.end.x, &ratio);

        let y_ratio = if ratio >= 0.5 {
            1. - ratio
        } else {
            ratio
        };

        target.position.y = self.start.y.lerp(&self.end.y, &y_ratio);
    }
}