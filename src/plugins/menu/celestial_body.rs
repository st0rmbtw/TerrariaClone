use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, Component, Resource, Plugin, App, Query, With, EventReader, ResMut, Handle, GlobalTransform, Camera, Vec2, Transform, IntoSystemDescriptor, Local, Input, MouseButton, Color, Vec4}, sprite::{SpriteSheetBundle, TextureAtlasSprite, TextureAtlas, Sprite}, window::Windows};
use interpolation::Lerp;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};

use crate::{plugins::{assets::BackgroundAssets, camera::MainCamera, cursor::CursorPosition}, animation::{Tween, EaseMethod, Animator, RepeatStrategy, RepeatCount, TweenCompleted, Lens, component_animator_system, AnimationSystem, AnimatorState}, state::GameState, util::{screen_to_world, FRect}, parallax::LayerTextureComponent};

pub(super) struct CelestialBodyPlugin;

impl Plugin for CelestialBodyPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TimeType>()
            .add_enter_system(GameState::MainMenu, setup)
            .add_system(update_celestial_type.run_in_state(GameState::MainMenu))
            .add_system(update_time_type.run_in_state(GameState::MainMenu))
            .add_system(move_celestial_body.run_in_state(GameState::MainMenu))
            .add_system(drag_celestial_body.run_in_state(GameState::MainMenu))
            .add_system(component_animator_system::<CelestialBody>.label(AnimationSystem::AnimationUpdate))
            .add_system(update_sprites_color.run_in_state(GameState::MainMenu));
    }
}

const X_ANIMATION_COMPLETED: u64 = 1;
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

#[autodefault(except(CelestialBodyPositionLens))]
fn setup(
    mut commands: Commands,
    background_assets: Res<BackgroundAssets>
) {
    let logo_animation = Tween::new(
        EaseMethod::Linear,
        RepeatStrategy::Repeat,
        Duration::from_secs(25),
        CelestialBodyPositionLens {
            start: Vec2::new(-0.1, 1. * 0.7),
            end: Vec2::new(1.1, 1.)
        }
    )
    .with_repeat_count(RepeatCount::Infinite);

    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 0
            },
            texture_atlas: background_assets.sun.clone_weak(),
            transform: Transform::IDENTITY,
        },
        Animator::new(logo_animation),
        CelestialBody::default()
    ));
}

fn move_celestial_body(
    windows: Res<Windows>,
    query_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut query_celestial_body: Query<(&mut Transform, &CelestialBody)>
) {
    let window = windows.primary();
    let window_size = Vec2::new(window.width(), window.height());

    let (camera, camera_transform) = query_camera.single();
    let (mut celestial_body_transform, celestial_body) = query_celestial_body.single_mut();

    let world_pos = screen_to_world(celestial_body.position * window_size, window_size, camera, camera_transform);

    celestial_body_transform.translation.x = world_pos.x;
    celestial_body_transform.translation.y = world_pos.y;
}

fn update_time_type(
    mut events: EventReader<TweenCompleted>,
    mut time_type: ResMut<TimeType>
) {
    for event in events.iter() {
        if event.user_data == X_ANIMATION_COMPLETED {
            *time_type = match *time_type {
                TimeType::Day => TimeType::Night,
                TimeType::Night => TimeType::Day,
            };
        }
    }
}

fn drag_celestial_body(
    windows: Res<Windows>,
    input: Res<Input<MouseButton>>,
    cursor_position: Res<CursorPosition>,
    time_type: Res<TimeType>,
    mut query_celestial_body: Query<(&mut Transform, &mut Animator<CelestialBody>, &mut CelestialBody)>,
    mut dragging: Local<bool>,
) {
    let window = windows.primary();

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

        animator.state = AnimatorState::Paused;
        *dragging = true;
    } else {
        *dragging = false;
        animator.state = AnimatorState::Playing;
    }
}

fn update_celestial_type(
    time_type: Res<TimeType>,
    background_assets: Res<BackgroundAssets>,
    mut query: Query<&mut Handle<TextureAtlas>, With<CelestialBody>>,
) {
    if time_type.is_changed() {
        match *time_type {
            TimeType::Day => {
                let mut sprite = query.single_mut();
                *sprite = background_assets.sun.clone_weak();
            },
            TimeType::Night => {
                let mut sprite = query.single_mut();
                *sprite = background_assets.moon_0.clone_weak();
            },
        }
    }
}

fn update_sprites_color(
    time_type: Res<TimeType>,
    mut query_sprite: Query<&mut Sprite, With<LayerTextureComponent>>,
    query_celestial_body: Query<&CelestialBody>
) {
    fn map_range(in_min: f32, in_max: f32, out_min: f32, out_max: f32, value: f32) -> f32 {
        return out_min + (((value - in_min) / (in_max - in_min)) * (out_max - out_min));
    }

    let celestial_body = query_celestial_body.single();

    let celestial_body_position = celestial_body.position.x.clamp(0., 1.);

    let mut color = match *time_type {
        TimeType::Day => Color::WHITE,
        TimeType::Night => Color::rgb(0.2, 0.2, 0.2),
    };

    const SUNRISE_THRESHOLD: f32 = 0.2;
    const SUNSET_THRESHOLD: f32 = 0.8;

    if celestial_body_position <= SUNRISE_THRESHOLD {
        let start: Vec4 = Color::rgb_u8(0, 54, 107).into();
        let end: Vec4 = color.into();
        let s = map_range(0., SUNRISE_THRESHOLD, 0., 1., celestial_body_position);

        color = start.lerp(end, s).into();
    } else if celestial_body_position > SUNSET_THRESHOLD {
        let start: Vec4 = color.into();
        let end: Vec4 = Color::rgb(0.3, 0.2, 0.3).into();
        let s = map_range(SUNSET_THRESHOLD, 1., 0., 1., celestial_body_position);

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