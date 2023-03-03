use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, Component, Resource, Plugin, App, Query, With, EventReader, ResMut, Handle, GlobalTransform, Camera, Vec2, Transform, IntoSystemDescriptor, Local, Input, MouseButton}, sprite::{SpriteSheetBundle, TextureAtlasSprite, TextureAtlas}, window::Windows};
use interpolation::Lerp;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};

use crate::{plugins::{assets::BackgroundAssets, camera::MainCamera, cursor::CursorPosition}, animation::{Tween, EaseMethod, Animator, RepeatStrategy, Tracks, RepeatCount, TweenCompleted, Lens, component_animator_system, AnimationSystem, AnimatorState}, state::GameState, util::{screen_to_world, FRect}};

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
            .add_system(component_animator_system::<CelestialBody>.label(AnimationSystem::AnimationUpdate));
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
struct CelestialBodyPositionXLens {
    start: f32,
    end: f32
}

#[derive(Clone, Copy, PartialEq)]
struct CelestialBodyPositionYLens {
    start: f32,
    end: f32
}

#[derive(Default, Resource, Clone, Copy)]
enum TimeType {
    #[default]
    Day,
    Night
}

#[autodefault(except(CelestialBodyPositionXLens, CelestialBodyPositionYLens))]
fn setup(
    mut commands: Commands,
    background_assets: Res<BackgroundAssets>
) {
    let x_animation = Tween::new(
        EaseMethod::Linear,
        RepeatStrategy::Repeat,
        Duration::from_secs(25),
        CelestialBodyPositionXLens {
            start: 0.,
            end: 1.
        }
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_completed_event(X_ANIMATION_COMPLETED);

    let y_animation = Tween::new(
        EaseMethod::CustomFunction(|x| {
            if x >= 0.5 {
                1. - x
            } else {
                x
            }
        }),
        RepeatStrategy::Repeat,
        Duration::from_secs(25),
        CelestialBodyPositionYLens {
            start: 1. * 0.7,
            end: 1.
        }
    )
    .with_repeat_count(RepeatCount::Infinite);

    let logo_animation = Tracks::new([
        x_animation,
        y_animation
    ]);

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
    mut query_celestial_body: Query<(&mut Transform, &mut Animator<CelestialBody>), With<CelestialBody>>,
    mut dragging: Local<bool>,
) {
    let window = windows.primary();

    let (mut celestial_body_transform, mut animator) = query_celestial_body.single_mut();

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
        *dragging = true;

        let tracks = animator.tweenable_mut().as_any_mut().downcast_mut::<Tracks<CelestialBody>>().unwrap();
        tracks.get_track_mut(0).unwrap().set_progress(cursor_position.position.x / window.width());
        animator.state = AnimatorState::Paused;
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

impl Lens<CelestialBody> for CelestialBodyPositionXLens {
    fn lerp(&mut self, target: &mut CelestialBody, ratio: f32) {
        target.position.x = self.start.lerp(&self.end, &ratio);
    }
}

impl Lens<CelestialBody> for CelestialBodyPositionYLens {
    fn lerp(&mut self, target: &mut CelestialBody, ratio: f32) {
        target.position.y = self.start.lerp(&self.end, &ratio);
    }
}