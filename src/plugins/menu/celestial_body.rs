use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, Component, Resource, Plugin, App, Query, With, EventReader, ResMut, Handle, GlobalTransform, Camera, Vec2, Transform, IntoSystemDescriptor}, sprite::{SpriteSheetBundle, TextureAtlasSprite, TextureAtlas}, window::Windows};
use interpolation::Lerp;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};

use crate::{plugins::{assets::BackgroundAssets, camera::MainCamera}, animation::{Tween, EaseMethod, Animator, RepeatStrategy, Tracks, RepeatCount, TweenCompleted, Lens, component_animator_system, AnimationSystem}, state::GameState, util::screen_to_world};

pub(super) struct CelestialBodyPlugin;

impl Plugin for CelestialBodyPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TimeType>()
            .add_enter_system(GameState::MainMenu, setup)
            .add_system(update_celestial_type.run_in_state(GameState::MainMenu))
            .add_system(update_time_type.run_in_state(GameState::MainMenu))
            .add_system(move_celestial_body.run_in_state(GameState::MainMenu))
            .add_system(component_animator_system::<CelestialBody>.label(AnimationSystem::AnimationUpdate));
    }
}

const X_ANIMATION_COMPLETED: u64 = 1;

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
    background_assets: Res<BackgroundAssets>,
    windows: Res<Windows>
) {
    let window = windows.primary();

    let x_animation = Tween::new(
        EaseMethod::Linear,
        RepeatStrategy::Repeat,
        Duration::from_secs(25),
        CelestialBodyPositionXLens {
            start: 0.,
            end: window.width()
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
            start: window.height() * 0.7,
            end: window.height()
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

    let world_pos = screen_to_world(celestial_body.position, window_size, camera, camera_transform);

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