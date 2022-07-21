use std::{time::Duration, option::Option, collections::HashSet};

use bevy::{prelude::*, sprite::Anchor, math::XY};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{prelude::{RigidBody, Velocity, Sleeping, Ccd, Collider, ActiveEvents, LockedAxes, Sensor, ExternalForce}, pipeline::CollisionEvent, rapier::prelude::CollisionEventFlags};

const PLAYER_SPRITE_WIDTH: f32 = 37.;
const PLAYER_SPRITE_HEIGHT: f32 = 53.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(spawn_player)
            .add_system(spawn_ground_sensor)
            .add_system(update_axis)
            .add_system(update)
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

#[derive(Default, Component, Inspectable)]
struct Movement {
    direction: FaceDirection,
    state: MovementState
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Inspectable)]
enum MovementState {
    #[default]
    IDLE,
    RUNNING,
    FLYING
}

#[derive(Default, PartialEq, Eq, Inspectable)]
enum FaceDirection {
    #[default]
    LEFT,
    RIGHT
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
struct GroundDetection {
    on_ground: bool
}

#[derive(Component)]
struct GroundSensor {
    ground_detection_entity: Entity,
    intersecting_ground_entities: HashSet<Entity>
}

fn spawn_player(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = assets.load("sprites/npc_22.png");
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle, Vec2::new(PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT), 1, 16, Vec2::new(0., 3.)
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                anchor: Anchor::BottomLeft,
                ..default()
            },
            texture_atlas: texture_atlas_handle,
            ..default()
        })
        .insert(Player)
        .insert(Movement::default())
        .insert(AnimationTimer(Timer::new(Duration::from_millis(50), true)))
        .insert(Jumpable {
            jump_timer: Timer::new(Duration::from_millis(250), true),
            // jump_cooldown_timer: Timer::new(Duration::from_millis(400), true)
        })
        .insert(GroundDetection::default())

        // RigidBody
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ExternalForce::default())
        .with_children(|children| {
            
            // Collider
            children.spawn()
                .insert(Collider::cuboid(PLAYER_SPRITE_WIDTH / 2. - 1., PLAYER_SPRITE_HEIGHT / 2.))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(PLAYER_SPRITE_WIDTH / 2., PLAYER_SPRITE_HEIGHT / 2., 0.)));
        })
        .insert(Name::new("Player"))
        .insert(Axis::default());

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "", 
                TextStyle {
                    font: assets.load("fonts/andyb.ttf"),
                    font_size: 20.,
                    color: Color::WHITE
                },
                TextAlignment::default()
            ),
            ..default()
        })
        .insert(PlayerCoords);

}

fn update(
    time: Res<Time>,
    mut query: Query<(
        &mut Velocity, 
        &mut ExternalForce, 
        &GroundDetection, 
        &mut Jumpable,
        &Axis
    ), With<Player>>,
) {
    let (mut velocity, mut force, ground_detection, mut jumpable, axis) = query.single_mut();

    if axis.is_up() && ground_detection.on_ground {
        force.force = Vec2::Y * 1500.;
        jumpable.jump_timer.reset();
    }
    
    if !ground_detection.on_ground && jumpable.jump_timer.tick(time.delta()).just_finished() {
        force.force = (Vec2::Y * -200.).lerp(Vec2::ZERO, 0.3 * time.delta_seconds());
    }

    velocity.linvel = Vec2::X * axis.x * 200.;
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
                        ground_sensor.intersecting_ground_entities
                            .insert(*a);
                    }
                },
                CollisionEvent::Stopped(a, b, CollisionEventFlags::SENSOR) => {
                    if *b == entity {
                        ground_sensor.intersecting_ground_entities
                            .remove(a);
                    }
                }
                _ => ()
            }
        }

        if let Ok(mut ground_detection) = ground_detectors.get_mut(ground_sensor.ground_detection_entity) {
            ground_detection.on_ground = ground_sensor.intersecting_ground_entities.len() > 0;
        }
    }
}

fn update_coords_text(
    mut text_query: Query<(&mut Text, &mut Transform), (With<PlayerCoords>, Without<Player>)>,
    mut player_query: Query<&Transform, With<Player>>
) {
    let transform = player_query.single_mut();
    let (mut player_coords, mut text_transform) = text_query.single_mut();

    let x = transform.translation.x;
    let y = transform.translation.y;

    let mut new_translation = Vec3::from(transform.translation);

    new_translation.y += PLAYER_SPRITE_HEIGHT + 20.;
    new_translation.x -= PLAYER_SPRITE_WIDTH - 10.;

    player_coords.sections[0].value = format!("({:.1}, {:.1})", x, y);
    text_transform.translation = new_translation;
}

fn update_movement_state(
    mut query: Query<(&mut Movement, &Velocity, &GroundDetection), With<Player>>,
) {
    let (mut movement, velocity, ground_detection) = query.single_mut();

    let on_ground = ground_detection.on_ground;

    let state = match velocity.linvel.into() {
        XY { x, .. } if x != 0. && on_ground => MovementState::RUNNING,
        _ => match on_ground {
            false => MovementState::FLYING,
            _ => MovementState::IDLE
        },
    };

    movement.state = state;
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
        &Movement
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle, movement) in query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();

            sprite.flip_x = movement.direction.is_right();

            sprite.index = match movement.state {
                MovementState::IDLE => 0,
                MovementState::FLYING => 1,
                MovementState::RUNNING => {
                    ((sprite.index + 1) % texture_atlas.textures.len()).clamp(2, 16)
                },
            }
        }
    }
}

fn spawn_ground_sensor(
    mut commands: Commands,
    detect_ground_for: Query<(Entity, &Children, &Transform), Added<GroundDetection>>,
    colliders: Query<&Collider>
) {
    for (entity, children, transform) in detect_ground_for.iter() {
        for child in children.iter() {
            if let Ok(collider) = colliders.get(*child) {
                if let Some(cuboid) = collider.raw.0.as_cuboid() {
                    let sensor_translation = Vec3::new(transform.translation.x + (PLAYER_SPRITE_WIDTH / 2.), transform.translation.y, 0.);

                    commands.entity(entity).with_children(|builder| {
                        builder.spawn()
                            .insert(Collider::cuboid(cuboid.half_extents.x - 2., 1.))
                            .insert(Ccd::enabled())
                            .insert(Sensor)
                            .insert(ActiveEvents::COLLISION_EVENTS)
                            .insert(Transform::from_translation(sensor_translation))
                            .insert(GlobalTransform::default())
                            .insert(GroundSensor {
                                ground_detection_entity: entity,
                                intersecting_ground_entities: HashSet::new(),
                            });
                    });
                }
            }
        }
    }
}


#[derive(Default, Component, Clone, Copy)]
struct Axis {
    x: f32,
    y: f32
}

impl Axis {
    fn is_up(&self) -> bool {
        self.y > 0.01
    }
}

impl Into<Option<FaceDirection>> for &Axis {
    fn into(self) -> Option<FaceDirection> {
        match self.x {
            x if x > 0. => Some(FaceDirection::RIGHT),
            x if x < 0. => Some(FaceDirection::LEFT),
            _ => None
        }
    }
}

fn update_axis(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Axis>
) {
    for mut axis in query.iter_mut() {
        let left = input.any_pressed([KeyCode::A, KeyCode::Left]);
        let right = input.any_pressed([KeyCode::D, KeyCode::Right]);
        let up = input.any_pressed([KeyCode::Space, KeyCode::Up]);

        axis.x = if left { -1. } else if right { 1. } else { 0. };
        axis.y = if up { 1. } else { 0. };
    }
}