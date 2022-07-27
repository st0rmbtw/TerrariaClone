use std::{time::Duration, option::Option, collections::HashSet};

use bevy::{prelude::*, sprite::Anchor, math::XY};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{prelude::{RigidBody, Velocity, Sleeping, Ccd, Collider, ActiveEvents, LockedAxes, Sensor, ExternalForce, Friction, GravityScale}, pipeline::CollisionEvent, rapier::prelude::CollisionEventFlags};

use super::{PlayerAssets, Inventory, FontAssets, PlayerInventoryPlugin};

pub const PLAYER_SPRITE_WIDTH: f32 = 37.;
pub const PLAYER_SPRITE_HEIGHT: f32 = 53.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(PlayerInventoryPlugin)
            .add_startup_system(spawn_player)
            .add_system(update_axis)
            .add_system(update)
            .add_system(update_speed_coefficient)
            .add_system(check_is_on_ground)
            .add_system(update_movement_state)
            .add_system(update_movement_direction)
            .add_system(animate_sprite)
            .add_system(update_coords_text);
    }
}

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

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Jumpable {
    jump_timer: Timer,
    // jump_cooldown_timer: Timer
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
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Player)
        .insert(Movement::default())
        .insert(AnimationTimer(Timer::new(Duration::from_millis(50), true)))
        .insert(Jumpable {
            jump_timer: Timer::new(Duration::from_millis(600), true),
            // jump_cooldown_timer: Timer::new(Duration::from_millis(400), true)
        })
        .insert(GroundDetection::default())
        .insert(Name::new("Player"))
        .insert(Axis::default())
        .insert(Inventory::default())
        .insert(SpeedCoefficient::default())

        // RigidBody
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ExternalForce::default())
        .insert(GravityScale::default())
        .with_children(|children| {

            let entity = children.parent_entity();

            let player_half_width = PLAYER_SPRITE_WIDTH / 2.;
            let player_half_height = PLAYER_SPRITE_HEIGHT / 2.;

            // Collider
            children.spawn()
                .insert(Collider::cuboid(player_half_width - 1., player_half_height - 3.))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Friction::coefficient(0.))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(player_half_width, player_half_height - 3., 0.)));

            // Ground sensor
            children.spawn()
                .insert(Collider::cuboid(player_half_width - 2., 1.))
                .insert(Ccd::enabled())
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(player_half_width, -2., 0.)))
                .insert(GlobalTransform::default())
                .insert(GroundSensor {
                    ground_detection_entity: entity,
                    intersecting_ground_entities: HashSet::new(),
                });
        });

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "", 
                TextStyle {
                    font: font_assets.andy_bold.clone(),
                    font_size: 18.,
                    color: Color::WHITE
                },
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center
                }
            ),
            ..default()
        })
        .insert(PlayerCoords);

}

fn update(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(
        &mut Velocity, 
        &mut ExternalForce, 
        &GroundDetection, 
        &mut Jumpable,
        &Axis,
        &mut SpeedCoefficient,
        &Movement
    ), With<Player>>,
) {
    let (mut velocity, mut force, ground_detection, mut jumpable, axis, mut coefficient, movement) = query.single_mut();

    let direction = movement.direction;

    if input.any_just_pressed([KeyCode::Space, KeyCode::Up]) && ground_detection.on_ground {
        force.force = Vec2::Y * 1000.;
        jumpable.jump_timer.reset();
    }

    if input.any_pressed([KeyCode::Space, KeyCode::Up]) && !ground_detection.on_ground {
        force.force = Vec2::Y * 2000.;
    }
    
    if !ground_detection.on_ground && jumpable.jump_timer.tick(time.delta()).just_finished() {
        force.force = (Vec2::Y * -2000.).lerp(Vec2::ZERO, time.delta_seconds());
    }

    velocity.linvel = Vec2::ZERO.lerp(Vec2::X * f32::from(direction) * 30. * 2.5, coefficient.0);
}

fn check_is_on_ground(
    mut ground_sensors: Query<(Entity, &mut GroundSensor)>,
    mut ground_detectors: Query<&mut GroundDetection>,
    mut collisions: EventReader<CollisionEvent>,
) {
    for (entity, mut ground_sensor) in ground_sensors.iter_mut() {
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
                _ => ()
            }
        }

        if let Ok(mut ground_detection) = ground_detectors.get_mut(ground_sensor.ground_detection_entity) {
            ground_detection.on_ground = !ground_sensor.intersecting_ground_entities.is_empty();
        }
    }
}

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

    movement.state = match velocity.linvel.into() {
        XY { x, .. } if x != 0. && on_ground => MovementState::RUNNING,
        _ => match on_ground {
            false => MovementState::FLYING,
            _ => MovementState::IDLE
        },
    };
}

fn update_movement_direction(mut query: Query<(&mut Movement, &Axis)>) {
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
    for (mut timer, mut sprite, texture_atlas_handle, movement, velocity) in query.iter_mut() {
        if velocity.linvel.x != 0. {
            timer.set_duration(Duration::from_millis((velocity.linvel.x.abs() / 1900.).powi(-1).ceil() as u64));
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
    mut query: Query<(&mut SpeedCoefficient, &Axis)>
) {
    for (mut coeff, axis) in query.iter_mut() {
        let new_coeff = coeff.0 + match coeff.0 {
            c if c < 1. && axis.moving() => (0.9) * time.delta_seconds(),
            c if c > 0. && !axis.moving() => -((1.1) * time.delta_seconds()),
            _ => 0.
        };

        coeff.0 = new_coeff.clamp(0., 1.);
    }
}

#[derive(Default, Component, Clone, Copy)]
struct Axis {
    x: f32
}

impl Axis {
    fn moving(&self) -> bool {
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

#[derive(Component, Default, Deref, DerefMut, Clone, Copy, Inspectable)]
pub struct SpeedCoefficient(f32);

fn update_axis(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Axis>
) {
    for mut axis in query.iter_mut() {
        let left = input.any_pressed([KeyCode::A, KeyCode::Left]);
        let right = input.any_pressed([KeyCode::D, KeyCode::Right]);

        let x = -(left as i8) + right as i8;

        axis.x = x as f32;
    }
}