use std::time::Duration;

use bevy::{prelude::{Commands, Res, Component, Resource, Plugin, App, Query, With, EventReader, ResMut, Handle, GlobalTransform, Camera, Vec2, Transform, Local, Input, MouseButton, Color, Vec4, DetectChanges, IntoSystemConfigs, OnExit, Name, Update, OnEnter, Without, Entity, Deref, DerefMut, PreUpdate, on_event, Condition, Vec3}, sprite::{Sprite, SpriteSheetBundle, TextureAtlasSprite, TextureAtlas, SpriteBundle}, window::{Window, PrimaryWindow, WindowResized}, utils::default, ecs::query::Has};
use bevy_hanabi::Gradient;
use interpolation::{Lerp, EaseFunction};
use rand::{thread_rng, Rng, seq::SliceRandom};

use crate::{plugins::{assets::{CelestialBodyAssets, BackgroundAssets}, camera::components::BackgroundCamera, background::{BACKGROUND_RENDER_LAYER, BackgroundPlugin}, cursor::resources::CursorPosition}, animation::{Tween, EaseMethod, Animator, RepeatStrategy, RepeatCount, TweenCompleted, Lens, component_animator_system, AnimationSystemSet, AnimatorState, lens::TransformScaleLens}, common::state::GameState, common::{math::map_range_f32, rect::FRect}, parallax::{LayerTextureComponent, ParallaxSet}};

use super::{in_menu_state, DespawnOnMenuExit, systems::despawn_with};

pub(super) struct CelestialBodyPlugin;
impl Plugin for CelestialBodyPlugin {
    fn build(&self, app: &mut App) {
        // This plugin depends on resources and components that BackgroundPlugin creates
        if !app.get_added_plugins::<BackgroundPlugin>().is_empty() {
            app.add_systems(
                OnExit(GameState::AssetLoading),
                (
                    setup,
                    spawn_celestial_body,
                    spawn_stars
                )
            );

            app.add_systems(OnEnter(GameState::InGame), cleanup);

            app.add_systems(
                Update,
                move_stars
                    .run_if(in_menu_state)
                    .after(ParallaxSet::FollowCamera)
            );

            app.add_systems(
                PreUpdate,
                (
                    despawn_with::<Star>,
                    spawn_stars
                )
                .run_if(in_menu_state.and_then(on_event::<WindowResized>()))
            );

            app.add_systems(
                Update,
                (
                    (
                        update_time_type,
                        update_celestial_type,
                    ).chain(),
                    move_celestial_body,
                    drag_celestial_body,
                    update_sprites_color,
                    change_visibility_of_stars,
                )
                .run_if(in_menu_state)
            );

            app.add_systems(
                Update,
                component_animator_system::<CelestialBodyPosition>
                    .in_set(AnimationSystemSet::AnimationUpdate)
                    .run_if(in_menu_state)
            );
        }
    }
}

const CELESTIAL_BODY_ANIMATION_COMPLETED: u64 = 1;
const SUN_SIZE: f32 = 42.;
const MOON_SIZE: f32 = 50.;

#[derive(Component, Clone, Copy, Default, PartialEq, Deref, DerefMut)]
struct CelestialBodyPosition(Vec2);

#[derive(Component)]
struct Star {
    screen_position: Vec2
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

#[derive(Resource)]
struct Gradients {
    night: Gradient<Vec4>,
    day: Gradient<Vec4>
}

#[derive(Component)]
struct Dragging;

const SUNRISE_THRESHOLD: f32 = 0.2;
const SUNSET_THRESHOLD: f32 = 0.8;

fn setup(mut commands: Commands) {
    commands.init_resource::<TimeType>();
    commands.insert_resource(Gradients {
        night: {
            let mut gradient = Gradient::<Vec4>::new();
            gradient.add_key(0.0, Color::rgb(0.07, 0.07, 0.07).into());
            gradient.add_key(0.8, Color::rgb(0.07, 0.07, 0.07).into());
            gradient.add_key(1., Color::rgb_u8(0, 54, 107).into());
            gradient
        },
        day: {
            let mut gradient = Gradient::<Vec4>::new();
            gradient.add_key(0., Color::rgb_u8(0, 54, 107).into());
            gradient.add_key(0.2, Color::WHITE.into());
            gradient.add_key(0.8, Color::WHITE.into());
            gradient.add_key(1.0, Color::rgb(0.07, 0.07, 0.07).into());
            gradient
        }
    });
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<TimeType>();
    commands.remove_resource::<Gradients>();
}

fn spawn_celestial_body(
    mut commands: Commands,
    celestial_body_assets: Res<CelestialBodyAssets>
) {
    let celestial_body_animation = Tween::new(
        EaseMethod::Linear,
        RepeatStrategy::Repeat,
        Duration::from_secs(25),
        CelestialBodyPositionLens {
            start: Vec2::new(-0.1, 0.3),
            end: Vec2::new(1.1, 0.)
        }
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_completed_event(CELESTIAL_BODY_ANIMATION_COMPLETED);

    commands.spawn((
        Name::new("Celestial Body"),
        CelestialBodyPosition::default(),
        DespawnOnMenuExit,
        Animator::new(celestial_body_animation),
        BACKGROUND_RENDER_LAYER,
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 0,
                ..default()
            },
            texture_atlas: celestial_body_assets.sun.clone_weak(),
            transform: Transform::from_xyz(0., 0., 0.5),
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

    let star_count = (window.width() + window.height() / 2.) / 25.;

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
                        translation: Vec3::new(0., 0., 0.1),
                        scale: Vec3::splat(rng.gen_range(0.5..1.0)),
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
    query_camera: Query<(&Camera, &GlobalTransform), With<BackgroundCamera>>,
    mut query_stars: Query<(&mut Transform, &Star)>
) {
    let (camera, camera_transform) = query_camera.single();

    for (mut star_transform, star) in &mut query_stars {
        if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, star.screen_position) {
            star_transform.translation.x = world_position.x;
            star_transform.translation.y = world_position.y;
        }    
    }
}

fn change_visibility_of_stars(
    time_type: Res<TimeType>,
    mut query_stars: Query<&mut Sprite, With<Star>>,
    query_celestial_body: Query<&CelestialBodyPosition>,
    mut alpha: Local<f32>
) {
    let celestial_body_pos = query_celestial_body.single();
    let x = celestial_body_pos.x.clamp(0., 1.);

    if x <= SUNRISE_THRESHOLD {
        let s = map_range_f32(0., SUNRISE_THRESHOLD, 0., 1., x);

        *alpha = match *time_type {
            TimeType::Day => alpha.lerp(&0., &s),
            TimeType::Night => alpha.lerp(&1., &s),
        }
    } else if x >= SUNSET_THRESHOLD { 
        let s = map_range_f32(SUNSET_THRESHOLD, 1., 0., 1., x);

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
    query_camera: Query<(&Camera, &GlobalTransform), With<BackgroundCamera>>,
    mut query_celestial_body: Query<(&mut Transform, &CelestialBodyPosition), Without<Dragging>>
) {
    let (camera, camera_transform) = query_camera.single();
    let Ok((mut celestial_body_transform, celestial_body_pos)) = query_celestial_body.get_single_mut() else { return; };
    let window = query_windows.single();
    let window_size = Vec2::new(window.width(), window.height());


    if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, celestial_body_pos.0 * window_size) {
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
    mut commands: Commands,
    query_window: Query<&Window, With<PrimaryWindow>>,
    input: Res<Input<MouseButton>>,
    time_type: Res<TimeType>,
    cursor_position: Res<CursorPosition>,
    mut query_celestial_body: Query<(Entity, &mut Transform, &mut Animator<CelestialBodyPosition>, &mut CelestialBodyPosition, Has<Dragging>)>,
    query_background_camera: Query<(&Camera, &GlobalTransform), With<BackgroundCamera>>,
) {
    let window = query_window.single();

    let Ok((camera, camera_transform)) = query_background_camera.get_single() else { return; };
    let Some(cursor_world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position.screen) else { return; };

    let (
        celestial_body_entity, mut celestial_body_transform, 
        mut animator, mut celestial_body_pos, is_dragging
    ) = query_celestial_body.single_mut();

    let celestial_body_size = match *time_type {
        TimeType::Day => SUN_SIZE,
        TimeType::Night => MOON_SIZE,
    };

    let cb_rect = FRect::new_center(
        celestial_body_transform.translation.x,
        celestial_body_transform.translation.y,
        celestial_body_size,
        celestial_body_size
    );

    let cursor_is_on_celestial_body = cb_rect.inside((cursor_world_position.x, cursor_world_position.y));

    if input.pressed(MouseButton::Left) && (cursor_is_on_celestial_body || is_dragging) {
        celestial_body_transform.translation.x = cursor_world_position.x;
        celestial_body_transform.translation.y = cursor_world_position.y;
        celestial_body_pos.x = cursor_position.screen.x / window.width();
        celestial_body_pos.y = cursor_position.screen.y / window.height();

        let tween = animator.tweenable_mut();
        tween.set_progress(cursor_position.screen.x / window.width());

        commands.entity(celestial_body_entity).insert(Dragging);
        animator.state = AnimatorState::Paused;
    } else {
        commands.entity(celestial_body_entity).remove::<Dragging>();
        animator.state = AnimatorState::Playing;
    }
}

fn update_celestial_type(
    time_type: Res<TimeType>,
    celestial_body_assets: Res<CelestialBodyAssets>,
    mut query: Query<(&mut Handle<TextureAtlas>, &mut TextureAtlasSprite), With<CelestialBodyPosition>>,
    mut moon_index: Local<usize>,
) {
    if time_type.is_changed() {
        let (mut texture, mut atlas_sprite) = query.single_mut();
        match *time_type {
            TimeType::Day => {
                *texture = celestial_body_assets.sun.clone_weak();
                atlas_sprite.index = 0;
            },
            TimeType::Night => {
                let mut rng = thread_rng();
                let moons = celestial_body_assets.moons();

                *moon_index = (*moon_index + 1) % (moons.len() - 1);
                atlas_sprite.index = rng.gen_range(0..8);
                *texture = moons[*moon_index].clone_weak();
            },
        }
    }
}

fn update_sprites_color(
    time_type: Res<TimeType>,
    mut query_sprite: Query<&mut Sprite, With<LayerTextureComponent>>,
    query_celestial_body: Query<&CelestialBodyPosition>,
    gradients: Res<Gradients>
) {
    let celestial_body_pos = query_celestial_body.single();
    let x = celestial_body_pos.x.clamp(0., 1.);

    let gradient = match *time_type {
        TimeType::Day => &gradients.day,
        TimeType::Night => &gradients.night,
    };
    
    for mut sprite in &mut query_sprite {
        sprite.color = gradient.sample(x).into();
    }
}

impl Lens<CelestialBodyPosition> for CelestialBodyPositionLens {
    fn lerp(&mut self, target: &mut CelestialBodyPosition, ratio: f32) {
        target.x = self.start.x.lerp(&self.end.x, &ratio);

        let y_ratio = if ratio >= 0.5 {
            1. - ratio
        } else {
            ratio
        };

        target.y = self.start.y.lerp(&self.end.y, &y_ratio);
    }
}