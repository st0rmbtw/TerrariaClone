use std::{time::Duration, option::Option, collections::HashSet};

use bevy::{prelude::*, sprite::Anchor};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{prelude::{RigidBody, Velocity, Sleeping, Ccd, Collider, ActiveEvents, LockedAxes, Sensor, ExternalForce, Friction, GravityScale, ColliderMassProperties}, pipeline::CollisionEvent, rapier::prelude::CollisionEventFlags};

use crate::util::Lerp;

use super::{PlayerAssets, FontAssets, PlayerInventoryPlugin, MainCamera};

pub const PLAYER_SPRITE_WIDTH: f32 = 37.;
pub const PLAYER_SPRITE_HEIGHT: f32 = 53.;

const PLAYER_SPEED: f32 = 30. * 5.;

// region: Plugin

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(PlayerInventoryPlugin)
            .add_startup_system(spawn_player)
            .add_system(update_axis)
            .add_system(update_movement_state)
            .add_system(update_movement_direction)
            .add_system(update_speed_coefficient)
            .add_system(update)
            .add_system(check_is_on_ground)
            .add_system(animate_sprite)
            .add_system(gravity);

        if cfg!(debug_assertions) {
            app.add_system(update_coords_text);
        }
    }
}

// endregion

// region: Structs

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerCoords;

#[derive(Default, Component, Inspectable, Clone, Copy)]
pub struct Movement {
    direction: FaceDirection,
    state: MovementState
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Inspectable)]
pub enum MovementState {
    #[default]
    IDLE,
    RUNNING,
    FLYING
}

#[derive(Default, PartialEq, Eq, Inspectable, Clone, Copy)]
pub enum FaceDirection {
    #[default]
    LEFT,
    RIGHT
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Default)]
struct Jumpable {
    time_after_jump: f32
}

#[derive(Debug, Component, Default, Inspectable)]
pub struct GroundDetection {
    on_ground: bool
}

#[derive(Component)]
struct GroundSensor {
    ground_detection_entity: Entity,
    intersecting_ground_entities: HashSet<Entity>
}

#[derive(Component, Default, Deref, DerefMut, Clone, Copy, Inspectable)]
pub struct SpeedCoefficient(f32);

#[derive(Default, Component, Clone, Copy)]
struct Axis {
    x: f32
}

// endregion

// region: Impls

impl Axis {
    fn is_moving(&self) -> bool {
        self.x != 0.
    }
}

impl From<&Axis> for Option<FaceDirection> {
    fn from(axis: &Axis) -> Self {
        match axis.x {
            x if x > 0. => Some(FaceDirection::RIGHT),
            x if x < 0. => Some(FaceDirection::LEFT),
            _ => None
        }
    }
}

impl From<FaceDirection> for f32 {
    fn from(direction: FaceDirection) -> Self {
        match direction {
            FaceDirection::LEFT => -1.,
            FaceDirection::RIGHT => 1.,
        }
    }
}

impl FaceDirection {
    #[inline]
    fn is_right(&self) -> bool {
        *self == FaceDirection::RIGHT
    }
}

// endregion

fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    font_assets: Res<FontAssets>
) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                anchor: Anchor::BottomLeft,
                ..default()
            },
            texture_atlas: player_assets.main.clone(),
            ..default()
        })
        .insert(Player)
        .insert(Movement::default())
        .insert(AnimationTimer(Timer::new(Duration::from_millis(50), true)))
        .insert(Jumpable::default())
        .insert(GroundDetection::default())
        .insert(Name::new("Player"))
        .insert(Axis::default())
        .insert(SpeedCoefficient::default())

        // RigidBody
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ExternalForce::default())
        .insert(GravityScale::default())
        .insert(ColliderMassProperties::Mass(1.))
        .with_children(|children| {

            // region: Camera
            let mut camera = Camera2dBundle::default();
            camera.projection.scale = 0.9;

            children.spawn()
                .insert_bundle(camera)
                .insert(MainCamera);
            // endregion

            let entity = children.parent_entity();

            let player_half_width = PLAYER_SPRITE_WIDTH / 2.;
            let player_half_height = PLAYER_SPRITE_HEIGHT / 2.;

            // region: Collider
            children.spawn()
                .insert(Collider::cuboid(player_half_width - 5., player_half_height - 3.))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Friction::coefficient(0.))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(player_half_width - 1., player_half_height - 3., 0.)));
            // endregion

            // region: Ground sensor
            children.spawn()
                .insert(Collider::cuboid(player_half_width - 8., 1.))
                .insert(Ccd::enabled())
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(player_half_width, -2., 0.)))
                .insert(GlobalTransform::default())
                .insert(GroundSensor {
                    ground_detection_entity: entity,
                    intersecting_ground_entities: HashSet::new(),
                });
            // endregion
        });

    if cfg!(debug_assertions) {
        commands
            .spawn_bundle(Text2dBundle {
                text: Text::from_section(
                    "", 
                    TextStyle {
                        font: font_assets.andy_bold.clone(),
                        font_size: 18.,
                        color: Color::WHITE
                    },
                ).with_alignment(TextAlignment::CENTER),
                ..default()
            })
            .insert(PlayerCoords);
    }
}

fn update(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(
        &mut Velocity,
        &GroundDetection,
        &mut Jumpable,
        &SpeedCoefficient,
        &Movement,
        &Axis
    ), With<Player>>,
) {
    let (mut velocity, ground_detection, mut jumpable, coefficient, movement, axis) = query.single_mut();

    let on_ground = ground_detection.on_ground;
    let direction = movement.direction;

    if input.any_just_pressed([KeyCode::Space, KeyCode::Up]) && on_ground {
        jumpable.time_after_jump = 0.01;
        velocity.linvel.y = 400.;
    }

    if jumpable.time_after_jump > 0. && !on_ground {
        jumpable.time_after_jump += time.delta_seconds();
    }
    
    if on_ground && jumpable.time_after_jump > 0. {
        jumpable.time_after_jump = 0.;
    }

    let vel_sign = velocity.linvel.x.signum();
    let dir_sign = f32::from(direction);

    if vel_sign != dir_sign && axis.is_moving() {
        velocity.linvel.x -= (PLAYER_SPEED * 4. * vel_sign) * time.delta_seconds();
    } else {
        velocity.linvel.x = 0_f32.lerp(f32::from(direction) * PLAYER_SPEED, coefficient.0);
    }
}

fn gravity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &GroundDetection, &Jumpable), With<Player>>
) {
    for (mut velocity, ground_detection, jumpable) in &mut query {
        if !ground_detection.on_ground && velocity.linvel.y > -500. {
            let new_velocity = 7_f32.lerp(10., jumpable.time_after_jump.clamp(0., 1.)) * 100.;

            velocity.linvel.y -= new_velocity * time.delta_seconds();
        }
    }
}

fn check_is_on_ground(
    mut ground_sensors: Query<(Entity, &mut GroundSensor)>,
    mut ground_detectors: Query<&mut GroundDetection>,
    mut collisions: EventReader<CollisionEvent>,
) {
    for (entity, mut ground_sensor) in &mut ground_sensors {
        for collision in collisions.iter() {
            match collision {
                CollisionEvent::Started(a, b, CollisionEventFlags::SENSOR) => {
                    if *b == entity {
                        ground_sensor.intersecting_ground_entities.insert(*a);
                    }
                },
                CollisionEvent::Stopped(a, b, CollisionEventFlags::SENSOR) => {
                    if *b == entity {
                        ground_sensor.intersecting_ground_entities.remove(a);
                    }
                }
                _ => {}
            }
        }

        if let Ok(mut ground_detection) = ground_detectors.get_mut(ground_sensor.ground_detection_entity) {
            ground_detection.on_ground = !ground_sensor.intersecting_ground_entities.is_empty();
        }
    }
}

#[cfg(debug_assertions)]
fn update_coords_text(
    mut text_query: Query<(&mut Text, &mut Transform), (With<PlayerCoords>, Without<Player>)>,
    mut player_query: Query<(&Transform, &Velocity), With<Player>>
) {
    let (transform, player_velocity) = player_query.single_mut();
    let (mut player_coords, mut text_transform) = text_query.single_mut();

    let x = transform.translation.x;
    let y = transform.translation.y;

    let mut new_translation = transform.translation;

    new_translation.y += PLAYER_SPRITE_HEIGHT + 10.;
    new_translation.x += PLAYER_SPRITE_WIDTH / 2.;

    let velocity = player_velocity.linvel.x * (42240. / 216000.);

    player_coords.sections[0].value = format!("({:.1}, {:.1}) {:.0}", x, y, velocity.abs());
    text_transform.translation = new_translation;
}

fn update_movement_state(
    mut query: Query<(&mut Movement, &Velocity, &GroundDetection), With<Player>>,
) {
    let (mut movement, velocity, ground_detection) = query.single_mut();

    let on_ground = ground_detection.on_ground;

    movement.state = match velocity.linvel {
        Vec2 { x, .. } if x != 0. && on_ground => MovementState::RUNNING,
        _ => match on_ground {
            false => MovementState::FLYING,
            _ => MovementState::IDLE
        },
    };
}

fn update_movement_direction(
    mut query: Query<(&mut Movement, &Axis)>
) {
    let (mut movement, axis) = query.single_mut();

    if let Some(direction) = axis.into() {
        movement.direction = direction;
    }
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        &Movement,
        &Velocity
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle, movement, velocity) in &mut query {
        if velocity.linvel.x != 0. {
            timer.set_duration(Duration::from_millis((4500. / velocity.linvel.x.abs()).max(1.) as u64));
        }

        match movement.state {
            MovementState::IDLE => {
                sprite.index = 0;
            },
            MovementState::FLYING => {
                sprite.index = 1;
            },
            _ => {}
        }

        if timer.tick(time.delta()).just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();

            sprite.flip_x = movement.direction.is_right();

            if movement.state == MovementState::RUNNING {
                sprite.index = ((sprite.index + 1) % texture_atlas.textures.len()).max(2);
            }
        }
    }
}

fn update_speed_coefficient(
    time: Res<Time>,
    mut query: Query<(&mut SpeedCoefficient, &Axis, &Movement, &Velocity)>
) {
    for (mut coeff, axis, movement, velocity) in &mut query {
        let direction = movement.direction;

        coeff.0 = if velocity.linvel.x.signum() != f32::from(direction) {
            0.
        } else {
            let new_coeff = coeff.0 + match coeff.0 {
                c if c < 1. && axis.is_moving() => 1.5,
                c if c > 0. && !axis.is_moving() => -1.7,
                _ => 0.
            } * time.delta_seconds();

            new_coeff.clamp(0., 1.)
        }
    }
}

fn update_axis(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Axis>
) {
    for mut axis in &mut query {
        let left = input.any_pressed([KeyCode::A, KeyCode::Left]);
        let right = input.any_pressed([KeyCode::D, KeyCode::Right]);

        let x = -(left as i8) + right as i8;

        axis.x = x as f32;
    }
}