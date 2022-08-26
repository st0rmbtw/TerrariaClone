use std::{time::Duration, option::Option, collections::HashSet};

use bevy::{prelude::*, sprite::Anchor};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{prelude::{RigidBody, Velocity, Ccd, Collider, ActiveEvents, LockedAxes, Sensor, ExternalForce, Friction, GravityScale, ColliderMassProperties}, pipeline::CollisionEvent, rapier::prelude::CollisionEventFlags};
use iyes_loopless::prelude::*;

use crate::{util::{Lerp, map_range}, TRANSPARENT, state::{GameState, MovementState}, item::{ITEM_ANIMATION_DATA}, parallax::{ParallaxCameraComponent}};

use super::{PlayerAssets, PlayerInventoryPlugin, MainCamera, WorldSettings, ItemAssets, SelectedItem};

pub const PLAYER_SPRITE_WIDTH: f32 = 37.;
pub const PLAYER_SPRITE_HEIGHT: f32 = 53.;

const PLAYER_SPEED: f32 = 30. * 5.;

const WALKING_ANIMATION_MAX_INDEX: usize = 13;

const USE_ITEM_ANIMATION_FRAMES_COUNT: usize = 3;

const MOVEMENT_ANIMATION_LABEL: &str = "movement_animation";
const USE_ITEM_ANIMATION_LABEL: &str = "use_item_animation";

// region: Plugin

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(PlayerInventoryPlugin)

            .insert_resource(Axis::default())
            .insert_resource(MovementAnimationIndex::default())
            .insert_resource(UseItemAnimationIndex::default())
            .insert_resource(AnimationTimer(Timer::new(Duration::from_millis(80), true)))
            .insert_resource(UseItemAnimationTimer(Timer::new(Duration::from_millis(100), true)))
            .insert_resource(UseItemAnimation(false))

            .add_enter_system(GameState::InGame, spawn_player)
            
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    // .with_system(asdasd.run_if_resource_exists::<ParallaxResource>().label("follow_camera"))
                    .with_system(update_axis)
                    .with_system(update_movement_state)
                    .with_system(update_face_direction)
                    .with_system(update_speed_coefficient)
                    .with_system(update)
                    .with_system(check_is_on_ground)
                    .with_system(gravity)
                    .into()
                )

            .add_system_to_stage(CoreStage::PostUpdate, flip_player)
            // .add_system_to_stage(CoreStage::PostUpdate, set_sprite_index)

            .add_system_set_to_stage(
                CoreStage::PreUpdate, 
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .label(MOVEMENT_ANIMATION_LABEL)
                    .before(USE_ITEM_ANIMATION_LABEL)
                    .with_system(update_movement_animation_timer_duration)
                    .with_system(update_movement_animation_index)
                    .with_system(movement_animation)
                    .into()
            )
            
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .label(USE_ITEM_ANIMATION_LABEL)
                    .after(MOVEMENT_ANIMATION_LABEL)
                    .with_system(set_using_item_image.run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)))
                    .with_system(set_using_item_position.run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)))
                    .with_system(set_using_item_rotation.run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)))
                    .with_system(set_using_item_visible)
                    .with_system(update_use_item_animation_index.run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)))
                    .with_system(set_using_item_rotation_on_player_direction_change)
                    .with_system(use_item_animation.run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)))
                    .with_system(player_using_item)
                    .into()
            );
    }
}

// endregion

// region: Structs

#[derive(Component)]
struct Player;

#[derive(Default, PartialEq, Eq, Inspectable, Clone, Copy, Component)]
pub enum FaceDirection {
    LEFT,
    #[default]
    RIGHT
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct UseItemAnimationTimer(Timer);

#[derive(Component, PartialEq)]
struct UseItemAnimation(bool);

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

#[derive(Default, Clone, Copy)]
struct Axis {
    x: f32
}

#[derive(Component)]
struct ChangeFlip;

#[derive(Default, Clone, Copy)]
struct MovementAnimationIndex(usize);

#[derive(Default, Clone, Copy)]
struct UseItemAnimationIndex(usize);

#[derive(Component)]
struct PlayerBodySprite;

#[derive(Component)]
struct UsingItemMarker;

// region: Animation data

#[derive(Component, Clone, Copy)]
struct WalkingAnimationData {
    offset: usize,
    count: usize
}

#[derive(Component, Clone, Copy)]
struct IdleAnimationData {
    idle: usize
}

#[derive(Component, Clone, Copy)]
struct FlyingAnimationData {
    flying: usize
}

#[derive(Component, Clone, Copy)]
struct FallingAnimationData {
    falling: usize
}

#[derive(Component, Clone, Copy)]
struct UseItemAnimationData {
    offset: usize
}

// endregion

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
    fn is_left(&self) -> bool {
        *self == FaceDirection::LEFT
    }
}

impl Default for WalkingAnimationData {
    fn default() -> Self {
        WalkingAnimationData { 
            offset: 0, 
            count: 0,
        }
    }
}

// endregion

fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    world: Res<WorldSettings>
) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: TRANSPARENT,
                custom_size: Some(Vec2::splat(1.)),
                ..default()
            },
            computed_visibility: ComputedVisibility::not_visible(),
            ..default()
        })
        .with_children(|cmd| {

            // region: Hair
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    color: Color::rgb(0.55, 0.23, 0.14),
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 0.1),
                texture_atlas: player_assets.hair.clone(),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(Name::new("Player hair"));
            // endregion

            // region: Head
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    color: Color::rgb(0.92, 0.45, 0.32),
                    ..default()
                },
                texture_atlas: player_assets.head.clone(),
                transform: Transform::from_xyz(0., 0., 0.003),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(Name::new("Player head"));
            // endregion

            // region: Eyes
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    color: Color::WHITE,
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 0.1),
                texture_atlas: player_assets.eyes_1.clone(),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(WalkingAnimationData {
                offset: 6,
                count: 14,
                ..default()
            })
            .insert(Name::new("Player left eye"));

            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    color: Color::rgb(89. / 255., 76. / 255., 64. / 255.),
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 0.01),
                texture_atlas: player_assets.eyes_2.clone(),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(WalkingAnimationData {
                offset: 6,
                count: 14,
                ..default()
            })
            .insert(Name::new("Player right eye"));

            // endregion

            // region: Arms
            // region: Left arm
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    color: Color::rgb(0.58, 0.55, 0.47),
                    ..default()
                },
                transform: Transform::from_xyz(0., -8., 0.2),
                texture_atlas: player_assets.left_shoulder.clone(),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(WalkingAnimationData {
                offset: 14,
                count: 13
            })
            .insert(IdleAnimationData {
                idle: 0
            })
            .insert(FlyingAnimationData {
                flying: 2
            })
            .insert(FallingAnimationData {
                falling: 13
            })
            .insert(UseItemAnimationData {
                offset: 2
            })
            .insert(Name::new("Player left shoulder"));

            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    color: Color::rgb(0.92, 0.45, 0.32),
                    ..default()
                },
                transform: Transform::from_xyz(0., -8., 0.2),
                texture_atlas: player_assets.left_hand.clone(),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(WalkingAnimationData {
                offset: 14,
                count: 13
            })
            .insert(IdleAnimationData {
                idle: 0
            })
            .insert(FlyingAnimationData {
                flying: 2
            })
            .insert(FallingAnimationData {
                falling: 13
            })
            .insert(UseItemAnimationData {
                offset: 2,
            })
            .insert(Name::new("Player left hand"));
            // endregion

            // region: Right arm
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    color: Color::rgb(0.92, 0.45, 0.32),
                    ..default()
                },
                transform: Transform::from_xyz(0., -20., 0.001),
                texture_atlas: player_assets.right_arm.clone(),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(WalkingAnimationData {
                count: 13,
                ..default()
            })
            .insert(IdleAnimationData {
                idle: 14
            })
            .insert(FlyingAnimationData {
                flying: 13
            })
            .insert(FallingAnimationData {
                falling: 0
            })
            .insert(UseItemAnimationData {
                offset: 15,
            })
            .insert(Name::new("Player right hand"));
            // endregion

            // endregion

            // region: Chest
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    index: 0,
                    color: Color::rgb(0.58, 0.55, 0.47),
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 0.002),
                texture_atlas: player_assets.chest.clone(),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(AnimationTimer(Timer::new(Duration::from_millis(50), true)))
            .insert(Name::new("Player chest"));
            // endregion

            // region: Feet
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite { 
                    color: Color::rgb(190. / 255., 190. / 255., 156. / 255.),
                    ..default()
                },
                texture_atlas: player_assets.feet.clone(),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(WalkingAnimationData {
                offset: 6,
                count: 13,
                ..default()
            })
            .insert(FlyingAnimationData {
                flying: 5
            })
            .insert(Name::new("Player feet"));
            // endregion

            // region: Using item
            cmd.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 0.15),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(UsingItemMarker)
            .insert(Name::new("Using item"));

            // endregion

        })
        .insert(Player)
        .insert(Jumpable::default())
        .insert(GroundDetection::default())
        .insert(Name::new("Player"))
        .insert(SpeedCoefficient::default())
        .insert(MovementState::default())
        .insert(FaceDirection::default())

        // RigidBody
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Ccd::enabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ExternalForce::default())
        .insert(GravityScale::default())
        .insert(ColliderMassProperties::Mass(1.))
        .insert(Transform::from_xyz(world.width as f32 / 2., 6., 0.))
        .with_children(|children| {

            // region: Camera
            children.spawn()
                .insert_bundle(Camera2dBundle {
                    projection: OrthographicProjection {
                        scale: 0.9,
                        ..default()
                    },
                    ..default()
                })
                .insert(ParallaxCameraComponent)
                .insert(MainCamera);
            // endregion

            let entity = children.parent_entity();

            let player_half_width = PLAYER_SPRITE_WIDTH / 2.;
            let player_half_height = PLAYER_SPRITE_HEIGHT / 2.;

            // region: Collider
            children.spawn()
                .insert(Collider::cuboid(player_half_width - 5., player_half_height - 2.))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Ccd::enabled())
                .insert(Transform::from_xyz(0., -3., 0.))
                .insert(Friction::coefficient(0.));
            // endregion

            // region: Ground sensor
            children.spawn()
                .insert(Collider::cuboid(player_half_width - 8., 1.))
                .insert(Ccd::enabled())
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Transform::from_xyz(0., -player_half_height - 2., 0.))
                .insert(GlobalTransform::default())
                .insert(GroundSensor {
                    ground_detection_entity: entity,
                    intersecting_ground_entities: HashSet::new(),
                });
            // endregion
        });
}

fn update(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    axis: Res<Axis>,
    mut query: Query<(
        &mut Velocity,
        &GroundDetection,
        &mut Jumpable,
        &SpeedCoefficient,
        &FaceDirection
    ), With<Player>>,
) {
    let (mut velocity, ground_detection, mut jumpable, coefficient, direction) = query.single_mut();

    let on_ground = ground_detection.on_ground;

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
    let dir_sign = f32::from(*direction);

    if vel_sign != dir_sign && axis.is_moving() {
        velocity.linvel.x -= (PLAYER_SPEED * 4. * vel_sign) * time.delta_seconds();
    } else {
        velocity.linvel.x = 0_f32.lerp(dir_sign * PLAYER_SPEED, coefficient.0);
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

fn update_movement_state(
    mut query: Query<(&GroundDetection, &Velocity, &mut MovementState), With<Player>>,
) {
    let (ground_detection, velocity, mut movement_state) = query.single_mut();

    let on_ground = ground_detection.on_ground;

    *movement_state = match velocity.linvel {
        Vec2 { x, .. } if x != 0. && on_ground => MovementState::WALKING,
        _ => match on_ground {
            false => match velocity.linvel {
                Vec2 { y, .. } if y < 0. => MovementState::FLYING,
                _ => MovementState::FALLING
            },
            _ => MovementState::IDLE
        },
    };
}

fn update_face_direction(
    axis: Res<Axis>,
    mut query: Query<&mut FaceDirection>
) {
    let mut direction = query.single_mut();
    let axis: &Axis = &axis;

    if let Some(new_direction) = axis.into() {
        if *direction != new_direction {
            *direction = new_direction;
        }
    }
}

fn update_speed_coefficient(
    time: Res<Time>,
    axis: Res<Axis>,
    mut query: Query<(&mut SpeedCoefficient, &Velocity, &FaceDirection)>
) {
    for (mut coeff, velocity, direction) in &mut query {

        coeff.0 = if velocity.linvel.x.signum() != f32::from(*direction) {
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
    mut axis: ResMut<Axis>
) {
    let left = input.any_pressed([KeyCode::A, KeyCode::Left]);
    let right = input.any_pressed([KeyCode::D, KeyCode::Right]);

    let x = -(left as i8) + right as i8;

    axis.x = x as f32;
}

fn update_movement_animation_timer_duration(
    mut timer: ResMut<AnimationTimer>,
    query: Query<&Velocity, With<Player>>,
) {
    let velocity = query.single();

    if velocity.linvel.x != 0. {
        timer.set_duration(Duration::from_millis((5000. / velocity.linvel.x.abs()).max(1.) as u64));
    }
}

fn update_movement_animation_index(
    time: Res<Time>,
    mut timer: ResMut<AnimationTimer>,
    mut index: ResMut<MovementAnimationIndex>,
) {
    if timer.tick(time.delta()).just_finished() {
        index.0 = (index.0 + 1) % WALKING_ANIMATION_MAX_INDEX;
    }
}

fn flip_player(
    player_query: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut sprite_query: Query<&mut TextureAtlasSprite, With<ChangeFlip>>
) {
    let direction = player_query.get_single();

    if let Ok(direction) = direction {
        sprite_query.for_each_mut(|mut sprite| {
            sprite.flip_x = direction.is_left();
        });
    }    
}

fn movement_animation(
    texture_atlases: Res<Assets<TextureAtlas>>,
    index: Res<MovementAnimationIndex>,
    player_query: Query<&MovementState, With<Player>>,
    mut query: Query<(
        &mut TextureAtlasSprite, 
        &Handle<TextureAtlas>,
        Option<&WalkingAnimationData>, 
        Option<&IdleAnimationData>,
        Option<&FlyingAnimationData>,
        Option<&FallingAnimationData>,
    ), With<PlayerBodySprite>>,
) {
    let movement_state = player_query.single();

    query.for_each_mut(|(
        mut sprite, 
        texture_atlas_handle,
        walking_anim_data, 
        idle_anim_data, 
        flying_anim_data, 
        falling_anim_data,
    )| {
        let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();

        let walking_anim_offset = walking_anim_data.map(|data| data.offset).unwrap_or(0);
        let walking_anim_count = walking_anim_data.map(|data| data.count).unwrap_or(texture_atlas.textures.len());
        let idle_anim_index = idle_anim_data.map(|data| data.idle).unwrap_or(0);
        let flying_anim_index = flying_anim_data.map(|data| data.flying).unwrap_or(0);
        let falling_anim_index = falling_anim_data.map(|data| data.falling).unwrap_or(flying_anim_index);
        
        sprite.index = match movement_state {
            MovementState::IDLE => idle_anim_index,
            MovementState::FLYING => flying_anim_index,
            MovementState::FALLING => falling_anim_index,
            MovementState::WALKING => walking_anim_offset + map_range((0, WALKING_ANIMATION_MAX_INDEX), (0, walking_anim_count), index.0)
        }
    });
}

fn player_using_item(
    input: Res<Input<MouseButton>>,
    selected_item: Res<SelectedItem>,
    mut anim: ResMut<UseItemAnimation>
) {
    let using_item = input.pressed(MouseButton::Left) && selected_item.is_some();

    if using_item {
        anim.0 = true;
    }
}

fn set_using_item_visible(
    anim: Res<UseItemAnimation>,
    mut using_item_query: Query<&mut Visibility, With<UsingItemMarker>>
) {
    let mut visibility = using_item_query.single_mut();

    visibility.is_visible = anim.0;
}

fn set_using_item_image(
    item_assets: Res<ItemAssets>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Handle<Image>, With<UsingItemMarker>>,
) {
    let mut image = using_item_query.single_mut();

    if selected_item.is_some() {
        let item_id = selected_item.unwrap().id;

        *image = item_assets.get_by_id(item_id);
    }
}

fn set_using_item_position(
    index: Res<UseItemAnimationIndex>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Transform, With<UsingItemMarker>>,
    player_query: Query<&FaceDirection, With<Player>>
) {
    let mut transform = using_item_query.single_mut();
    let direction = player_query.single();

    if selected_item.is_some() {
        let item_type = selected_item.unwrap().item_type;

        let position = ITEM_ANIMATION_DATA.get(&item_type).unwrap()[index.0];

        transform.translation.x = position.x * f32::from(*direction);
        transform.translation.y = position.y;
    }
}


fn get_rotation_by_direction(direction: FaceDirection) -> Quat {
    let start_rotation = match direction {
        FaceDirection::LEFT => -0.5,
        FaceDirection::RIGHT => 2.,
    };

    Quat::from_rotation_z(start_rotation)
}

fn set_using_item_rotation_on_player_direction_change(
    player_query: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut using_item_query: Query<&mut Transform, With<UsingItemMarker>>,
) {
    let player_query_result = player_query.get_single();
    let using_item_query_result = using_item_query.get_single_mut();

    if let Ok(mut transform) = using_item_query_result {
        if let Ok(direction) = player_query_result {
            transform.rotation = get_rotation_by_direction(*direction);
        }
    }
}

fn set_using_item_rotation(
    time: Res<Time>,
    index: Res<UseItemAnimationIndex>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Transform, With<UsingItemMarker>>,
    player_query: Query<&FaceDirection, With<Player>>
) {
    const ROTATION_STEP: f32 = -11.;

    let direction = player_query.single();
    let mut transform = using_item_query.single_mut();

    if selected_item.is_some() {
        let item_type = selected_item.unwrap().item_type;
        let direction_f = f32::from(*direction);

        let position = ITEM_ANIMATION_DATA.get(&item_type).unwrap()[index.0];

        if index.0 == 0 && index.is_changed() {
        transform.rotation = get_rotation_by_direction(*direction);
        }

        transform.rotate_around(position.extend(0.15), Quat::from_rotation_z(ROTATION_STEP * direction_f * time.delta_seconds()));
    }
}

fn update_use_item_animation_index(
    time: Res<Time>,
    mut index: ResMut<UseItemAnimationIndex>,
    mut timer: ResMut<UseItemAnimationTimer>,
    mut anim: ResMut<UseItemAnimation>
) {
    if timer.tick(time.delta()).just_finished() {
        index.0 = (index.0 + 1) % USE_ITEM_ANIMATION_FRAMES_COUNT;
    }

    if index.is_changed() && index.0 == 0 {
        anim.0 = false;
    }
}

fn use_item_animation(
    index: Res<UseItemAnimationIndex>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &UseItemAnimationData
    ), With<PlayerBodySprite>>
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        let use_item_anim_offset = anim_data.offset;
        
        sprite.index = use_item_anim_offset + index.0;
    });
}

// TODO: Debug function, remove in feature
#[cfg(debug_assertions)]
fn set_sprite_index(
    input: Res<Input<KeyCode>>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &UseItemAnimationData
    ), With<PlayerBodySprite>>,
) {
    query.for_each_mut(|(mut sprite, animation_data)| {
        let anim_offset = animation_data.offset;

        let mut new_sprite_index = sprite.index;

        if input.just_pressed(KeyCode::J) {
            new_sprite_index = sprite.index.checked_sub(1).unwrap_or(0);
        }
    
        if input.just_pressed(KeyCode::L) {
            new_sprite_index = sprite.index + 1;
        }

        new_sprite_index = new_sprite_index.checked_sub(anim_offset).unwrap_or(0);

        sprite.index = anim_offset + (new_sprite_index % USE_ITEM_ANIMATION_FRAMES_COUNT);
    });
}