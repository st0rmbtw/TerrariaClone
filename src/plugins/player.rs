use std::{collections::HashSet, option::Option, time::Duration};

use autodefault::autodefault;
use bevy::{prelude::*, sprite::Anchor, math::vec2};
use bevy_hanabi::{
    AccelModifier, ColorOverLifetimeModifier, EffectAsset, Gradient, ParticleEffect,
    ParticleEffectBundle, ParticleLifetimeModifier, PositionCone3dModifier, ShapeDimension,
    SizeOverLifetimeModifier, Spawner,
};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{
    pipeline::CollisionEvent,
    prelude::{
        ActiveEvents, Ccd, Collider, Friction, LockedAxes, RigidBody, Sensor,
        Velocity, RapierContext, QueryFilter,
    },
    rapier::prelude::CollisionEventFlags,
};
use iyes_loopless::prelude::*;

use crate::{
    items::{get_animation_points, Item},
    state::{GameState, MovementState},
    util::{map_range, Lerp, get_tile_coords, get_rotation_by_direction},
    TRANSPARENT, world_generator::WORLD_SIZE_X,
};

use super::{
    CursorPosition, ItemAssets, PlayerAssets, PlayerInventoryPlugin, SelectedItem, TILE_SIZE, BlockPlaceEvent, Inventory,
};

pub const PLAYER_SPRITE_WIDTH: f32 = 2. * TILE_SIZE * 0.75;
pub const PLAYER_SPRITE_HEIGHT: f32 = 3. * TILE_SIZE * 0.95;

const PLAYER_SPEED: f32 = 30. * 5.;

const WALKING_ANIMATION_MAX_INDEX: usize = 13;

const USE_ITEM_ANIMATION_FRAMES_COUNT: usize = 3;

const MOVEMENT_ANIMATION_LABEL: &str = "movement_animation";
const USE_ITEM_ANIMATION_LABEL: &str = "use_item_animation";

// region: Plugin

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayerInventoryPlugin)
            .insert_resource(Axis::default())
            .insert_resource(MovementAnimationIndex::default())
            .insert_resource(UseItemAnimationIndex::default())
            .insert_resource(AnimationTimer(Timer::new(Duration::from_millis(80), true)))
            .insert_resource(UseItemAnimationTimer(Timer::new(
                Duration::from_millis(100),
                true,
            )))
            .insert_resource(UseItemAnimation(false))
            .add_enter_system(GameState::InGame, spawn_player)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(update_axis)
                    .with_system(update_movement_state)
                    .with_system(update_face_direction)
                    .with_system(update_speed_coefficient)
                    .with_system(update)
                    .with_system(check_is_on_ground)
                    .with_system(use_item)
                    .with_system(auto_jump)
                    .into(),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate, 
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(flip_player)
                    .with_system(spawn_particles)
                    .into()
            )
            .add_system_set_to_stage(
                CoreStage::Last, 
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(limit_player_move)
                    .into()
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .label(MOVEMENT_ANIMATION_LABEL)
                    .before(USE_ITEM_ANIMATION_LABEL)
                    .with_system(update_movement_animation_timer_duration)
                    .with_system(update_movement_animation_index)
                    
                    .with_system(walking_animation.run_if(is_walking))
                    .with_system(simple_animation::<IdleAnimationData>.run_if(is_idle))
                    .with_system(simple_animation::<FlyingAnimationData>.run_if(is_flying))
                    .with_system(simple_animation::<FallingAnimationData>.run_if(is_falling))

                    .into(),
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .label(USE_ITEM_ANIMATION_LABEL)
                    .after(MOVEMENT_ANIMATION_LABEL)
                    .with_system(set_using_item_visibility)
                    .with_system(
                        set_using_item_image
                            .run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)),
                    )
                    .with_system(
                        set_using_item_position
                            .run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)),
                    )
                    .with_system(
                        set_using_item_rotation
                            .run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)),
                    )
                    .with_system(
                        update_use_item_animation_index
                            .run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)),
                    )
                    .with_system(set_using_item_rotation_on_player_direction_change)
                    .with_system(
                        use_item_animation
                            .run_if_resource_equals::<UseItemAnimation>(UseItemAnimation(true)),
                    )
                    .with_system(player_using_item)
                    .into(),
            );
    }
}

// endregion

// region: Structs

#[derive(Component)]
pub struct Player;

#[derive(Default, PartialEq, Eq, Inspectable, Clone, Copy, Component)]
pub enum FaceDirection {
    LEFT,
    #[default]
    RIGHT,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct UseItemAnimationTimer(Timer);

#[derive(Component, PartialEq)]
struct UseItemAnimation(bool);

#[derive(Component, Default, Inspectable)]
pub struct GroundDetection {
    pub on_ground: bool,
}

#[derive(Component)]
struct GroundSensor {
    ground_detection_entity: Entity,
    intersecting_ground_entities: HashSet<Entity>,
}

#[derive(Component, Default, Deref, DerefMut, Clone, Copy, Inspectable)]
pub struct SpeedCoefficient(pub f32);

#[derive(Default, Clone, Copy)]
struct Axis {
    x: f32,
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

trait AnimationData {
    fn index(&self) -> usize;
}

#[derive(Bundle, Default)]
struct MovementAnimationBundle {
    walking: WalkingAnimationData,
    idle: IdleAnimationData,
    flying: FlyingAnimationData,
    falling: FallingAnimationData
}

#[derive(Component, Clone, Copy)]
struct WalkingAnimationData {
    offset: usize,
    count: usize,
}

#[derive(Component, Clone, Copy, Default)]
struct IdleAnimationData(usize);

#[derive(Component, Clone, Copy, Default)]
struct FlyingAnimationData(usize);

#[derive(Component, Clone, Copy, Default)]
struct FallingAnimationData(usize);

#[derive(Component, Clone, Copy, Default)]
struct UseItemAnimationData(usize);

impl AnimationData for IdleAnimationData {
    fn index(&self) -> usize { self.0 }
}

impl AnimationData for FlyingAnimationData {
    fn index(&self) -> usize { self.0 }
}

impl AnimationData for FallingAnimationData {
    fn index(&self) -> usize { self.0 }
}

// endregion

#[derive(Component)]
struct PlayerParticleEffects {
    walking: Entity,
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
    fn is_left(&self) -> bool {
        *self == FaceDirection::LEFT
    }
}

impl Default for WalkingAnimationData {
    fn default() -> Self {
        WalkingAnimationData {
            offset: 0,
            count: WALKING_ANIMATION_MAX_INDEX,
        }
    }
}

// endregion

#[autodefault(except(GroundSensor, PlayerParticleEffects))]
fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let player = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: TRANSPARENT,
                custom_size: Some(Vec2::splat(1.)),
            },
            computed_visibility: ComputedVisibility::not_visible(),
        })
        .with_children(|cmd| {
            // region: Hair
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.55, 0.23, 0.14),
                },
                transform: Transform::from_xyz(0., 0., 0.1),
                texture_atlas: player_assets.hair.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle::default())
            .insert(Name::new("Player hair"));
            // endregion

            // region: Head
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.92, 0.45, 0.32),
                },
                texture_atlas: player_assets.head.clone(),
                transform: Transform::from_xyz(0., 0., 0.003),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle::default())
            .insert(Name::new("Player head"));
            // endregion

            // region: Eyes
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::WHITE,
                },
                transform: Transform::from_xyz(0., 0., 0.1),
                texture_atlas: player_assets.eyes_1.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 6,
                    count: 14,
                }
            })
            .insert(Name::new("Player left eye"));

            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(89. / 255., 76. / 255., 64. / 255.),
                },
                transform: Transform::from_xyz(0., 0., 0.01),
                texture_atlas: player_assets.eyes_2.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 6,
                    count: 14,
                }
            })
            .insert(Name::new("Player right eye"));

            // endregion

            // region: Arms
            // region: Left arm
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.58, 0.55, 0.47),
                },
                transform: Transform::from_xyz(0., -8., 0.2),
                texture_atlas: player_assets.left_shoulder.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 14,
                    count: 13,
                },
                flying: FlyingAnimationData(2),
                falling: FallingAnimationData(13)
            })
            .insert(UseItemAnimationData(2))
            .insert(Name::new("Player left shoulder"));

            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.92, 0.45, 0.32),
                },
                transform: Transform::from_xyz(0., -8., 0.2),
                texture_atlas: player_assets.left_hand.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 14,
                    count: 13,
                },
                flying: FlyingAnimationData(2),
                falling: FallingAnimationData(13)
            })
            .insert(UseItemAnimationData(2))
            .insert(Name::new("Player left hand"));
            // endregion

            // region: Right arm
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(0.92, 0.45, 0.32),
                },
                transform: Transform::from_xyz(0., -20., 0.001),
                texture_atlas: player_assets.right_arm.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData { count: 13 },
                idle: IdleAnimationData(14),
                flying: FlyingAnimationData(13),
            })
            .insert(UseItemAnimationData(15))
            .insert(Name::new("Player right hand"));
            // endregion

            // endregion

            // region: Chest
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: 0,
                    color: Color::rgb(0.58, 0.55, 0.47),
                },
                transform: Transform::from_xyz(0., 0., 0.002),
                texture_atlas: player_assets.chest.clone(),
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(AnimationTimer(Timer::new(Duration::from_millis(50), true)))
            .insert_bundle(MovementAnimationBundle::default())
            .insert(Name::new("Player chest"));
            // endregion

            // region: Feet
            cmd.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::rgb(190. / 255., 190. / 255., 156. / 255.),
                },
                texture_atlas: player_assets.feet.clone(),
                transform: Transform::from_xyz(0., 0., 0.15),
                ..default()
            })
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert_bundle(MovementAnimationBundle {
                walking: WalkingAnimationData {
                    offset: 6,
                    count: 13,
                },
                flying: FlyingAnimationData(5),
                falling: FallingAnimationData(5),
            })
            .insert(Name::new("Player feet"));
            // endregion

            // region: Using item
            cmd.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                },
                transform: Transform::from_xyz(0., 0., 0.15),
            })
            .insert(ChangeFlip)
            .insert(UsingItemMarker)
            .insert(Name::new("Using item"));

            // endregion
        })
        .insert(Player)
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
        .insert(Transform::from_xyz(WORLD_SIZE_X as f32 * 16. / 2., 10., 0.1))
        .with_children(|children| {
            let entity = children.parent_entity();

            let player_half_width = PLAYER_SPRITE_WIDTH / 2.;
            let player_half_height = PLAYER_SPRITE_HEIGHT / 2.;

            // region: Collider
            children
                .spawn()
                .insert(Collider::cuboid(player_half_width, player_half_height))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Transform::from_xyz(0., -4.5, 0.))
                .insert(Friction::coefficient(0.));
            // endregion

            // region: Ground sensor
            children
                .spawn()
                .insert(Collider::cuboid(player_half_width - 1., 1.))
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Transform::from_xyz(0., -player_half_height - 6., 0.))
                .insert(GlobalTransform::default())
                .insert(GroundSensor {
                    ground_detection_entity: entity,
                    intersecting_ground_entities: HashSet::new(),
                });
            // endregion
        })
        .id();

    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(114. / 255., 81. / 255., 56. / 255., 1.));

    let spawner = Spawner::rate(20.0.into());

    // Create the effect asset
    let effect = effects.add(
        EffectAsset {
            name: "MyEffect".to_string(),
            // Maximum number of particles alive at a time
            capacity: 30,
            spawner,
        }
        .init(PositionCone3dModifier {
            base_radius: 0.5,
            top_radius: 0.,
            height: 1.,
            dimension: ShapeDimension::Volume,
            speed: 10.0.into(),
        })
        .update(AccelModifier {
            accel: Vec3::new(0., 0., 0.),
        })
        // Render the particles with a color gradient over their
        // lifetime.
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant(Vec2::splat(3.)),
        })
        .init(ParticleLifetimeModifier { lifetime: 0.1 })
        .render(ColorOverLifetimeModifier { gradient }),
    );

    let effect_entity = commands
        .spawn_bundle(ParticleEffectBundle::new(effect).with_spawner(spawner))
        .insert(Name::new("Particle Spawner"))
        .id();

    commands.entity(player).add_child(effect_entity);

    commands.entity(player).insert(PlayerParticleEffects {
        walking: effect_entity,
    });
}

fn update(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    axis: Res<Axis>,
    mut query: Query<
        (
            &mut Velocity,
            &GroundDetection,
            &SpeedCoefficient,
            &FaceDirection,
        ),
        With<Player>,
    >,
    mut jump: Local<i32>
) {
    let (
        mut velocity, 
        GroundDetection { on_ground },
        coefficient, 
        direction
    ) = query.single_mut();

    if input.just_pressed(KeyCode::Space) && *on_ground {
        *jump = 15;
    }

    if *jump > 0 {
        velocity.linvel.y = 300.;
    }

    if input.pressed(KeyCode::Space) && *jump > 0 {
        *jump -= 1;
    }

    if input.just_released(KeyCode::Space) {
        *jump = 0;
    }

    let vel_sign = velocity.linvel.x.signum();
    let dir_sign = f32::from(*direction);

    if vel_sign != dir_sign && axis.is_moving() {
        velocity.linvel.x -= (PLAYER_SPEED * 4. * vel_sign) * time.delta_seconds();
    } else {
        velocity.linvel.x = 0_f32.lerp(dir_sign * PLAYER_SPEED, coefficient.0);
    }
}

fn spawn_particles(
    player: Query<(&MovementState, &FaceDirection, &PlayerParticleEffects), With<Player>>,
    mut effects: Query<(&mut ParticleEffect, &mut Transform)>,
) {
    for (movement_state, face_direction, particle_effects) in &player {
        let (mut effect, mut effect_transform) = effects.get_mut(particle_effects.walking).unwrap();

        effect_transform.translation = match face_direction {
            FaceDirection::LEFT => Vec3::new(0., -PLAYER_SPRITE_HEIGHT / 2., 0.),
            FaceDirection::RIGHT => Vec3::new(0., -PLAYER_SPRITE_HEIGHT / 2., 0.),
        };

        effect
            .maybe_spawner()
            .unwrap()
            .set_active(*movement_state == MovementState::WALKING);
    }
}

fn check_is_on_ground(
    mut ground_sensors: Query<&mut GroundSensor>,
    mut ground_detectors: Query<&mut GroundDetection>,
    mut collisions: EventReader<CollisionEvent>,
) {
    for mut ground_sensor in &mut ground_sensors {
        for collision in collisions.iter() {
            match collision {
                CollisionEvent::Started(a, _, CollisionEventFlags::SENSOR) => {
                    ground_sensor.intersecting_ground_entities.insert(*a);
                }
                CollisionEvent::Stopped(a, _, CollisionEventFlags::SENSOR) => {
                    ground_sensor.intersecting_ground_entities.remove(a);
                }
                _ => {} 
            }
        }

        if let Ok(mut ground_detection) =
            ground_detectors.get_mut(ground_sensor.ground_detection_entity)
        {
            ground_detection.on_ground = !ground_sensor.intersecting_ground_entities.is_empty();
        }
    }
}

fn update_movement_state(
    mut query: Query<(&GroundDetection, &Velocity, &mut MovementState), With<Player>>,
) {
    let (GroundDetection { on_ground }, velocity, mut movement_state) = query.single_mut();

    *movement_state = match velocity.linvel {
        Vec2 { x, .. } if x != 0. && *on_ground => MovementState::WALKING,
        _ => match on_ground {
            false => match velocity.linvel {
                Vec2 { y, .. } if y < 0. => MovementState::FLYING,
                _ => MovementState::FALLING,
            },
            _ => MovementState::IDLE,
        },
    };
}

fn update_face_direction(axis: Res<Axis>, mut query: Query<&mut FaceDirection>) {
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
    mut query: Query<(&mut SpeedCoefficient, &Velocity, &FaceDirection)>,
) {
    for (mut coeff, velocity, direction) in &mut query {
        coeff.0 = if velocity.linvel.x.signum() != f32::from(*direction) {
            0.
        } else {
            let new_coeff = coeff.0 + match coeff.0 {
                c if c < 1. && axis.is_moving() => 1.5,
                c if c > 0. && !axis.is_moving() => -1.7,
                _ => 0.,
            } * time.delta_seconds();

            new_coeff.clamp(0., 1.)
        }
    }
}

fn update_axis(input: Res<Input<KeyCode>>, mut axis: ResMut<Axis>) {
    let left = input.pressed(KeyCode::A);
    let right = input.pressed(KeyCode::D);

    let x = -(left as i8) + right as i8;

    axis.x = x as f32;
}

fn update_movement_animation_timer_duration(
    mut timer: ResMut<AnimationTimer>,
    query: Query<&Velocity, With<Player>>,
) {
    let velocity = query.single();

    if velocity.linvel.x != 0. {
        let mut time = 5000. / velocity.linvel.x.abs();
        if time < 1. {
            time = 1.;
        }

        timer.set_duration(Duration::from_millis(time as u64));
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
    mut sprite_query: Query<&mut TextureAtlasSprite, With<ChangeFlip>>,
) {
    let direction = player_query.get_single();

    if let Ok(direction) = direction {
        sprite_query.for_each_mut(|mut sprite| {
            sprite.flip_x = direction.is_left();
        });
    }
}

fn walking_animation(
    index: Res<MovementAnimationIndex>,
    mut query: Query<
        (&mut TextureAtlasSprite, &WalkingAnimationData),
        With<PlayerBodySprite>,
    >,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        let walking_anim_offset = anim_data.offset;
        let walking_anim_count = anim_data.count;

        sprite.index = walking_anim_offset + map_range(
            (0, WALKING_ANIMATION_MAX_INDEX),
            (0, walking_anim_count),
            index.0,
        );
    });
}

fn simple_animation<C: AnimationData + Component>(
    mut query: Query<
        (&mut TextureAtlasSprite, &C),
        With<PlayerBodySprite>,
    >,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        sprite.index = anim_data.index();
    });
}

fn is_walking(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::WALKING {
            return true;
        }
    }

    false
}

fn is_idle(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::IDLE {
            return true;
        }
    }

    false
}

fn is_flying(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::FLYING {
            return true;
        }
    }

    false
}

fn is_falling(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::FALLING {
            return true;
        }
    }

    false
}

fn player_using_item(
    input: Res<Input<MouseButton>>,
    selected_item: Res<SelectedItem>,
    mut anim: ResMut<UseItemAnimation>,
) {
    let using_item = input.pressed(MouseButton::Left) && selected_item.is_some();

    if using_item {
        anim.0 = true;
    }
}

fn set_using_item_visibility(
    anim: Res<UseItemAnimation>,
    mut using_item_query: Query<&mut Visibility, With<UsingItemMarker>>,
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

    if let Some(item_stack) = selected_item.0 {
        *image = item_assets.get_by_item(item_stack.item);
    }
}

fn set_using_item_position(
    index: Res<UseItemAnimationIndex>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Transform, With<UsingItemMarker>>,
    player_query: Query<&FaceDirection, With<Player>>,
) {
    let mut transform = using_item_query.single_mut();
    let direction = player_query.single();

    if let Some(item_stack) = selected_item.0 {
        let position = get_animation_points(item_stack.item)[index.0];

        transform.translation.x = position.x * f32::from(*direction);
        transform.translation.y = position.y;
    }
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
    player_query: Query<&FaceDirection, With<Player>>,
) {
    const ROTATION_STEP: f32 = -11.;

    let direction = player_query.single();
    let mut transform = using_item_query.single_mut();

    if selected_item.is_some() {
        let item_type = selected_item.unwrap().item;
        let direction_f = f32::from(*direction);

        let position = get_animation_points(item_type)[index.0];

        if index.0 == 0 && index.is_changed() {
            transform.rotation = get_rotation_by_direction(*direction);
        }

        transform.rotate_around(
            position.extend(0.15),
            Quat::from_rotation_z(ROTATION_STEP * direction_f * time.delta_seconds()),
        );
    }
}

fn update_use_item_animation_index(
    time: Res<Time>,
    mut index: ResMut<UseItemAnimationIndex>,
    mut timer: ResMut<UseItemAnimationTimer>,
    mut anim: ResMut<UseItemAnimation>,
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
    mut query: Query<(&mut TextureAtlasSprite, &UseItemAnimationData), With<PlayerBodySprite>>,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        sprite.index = anim_data.0 + index.0;
    });
}

fn auto_jump(
    axis: Res<Axis>,
    mut player_query: Query<(&mut Transform, &MovementState), With<Player>>,
    rapier_context: Res<RapierContext>
) {
    let (mut player_transform, movement_state) = player_query.single_mut();
    let player_x = player_transform.translation.x;
    let player_y = player_transform.translation.y;

    let player_direction = axis.x;

    if player_direction != 0. && *movement_state == MovementState::WALKING {
        
        let ray_dir = vec2(player_direction, 0.);
        let toi = 12.;
        let solid = true;
        let filter = QueryFilter::only_fixed();

        if rapier_context.cast_ray(
                vec2(player_x, player_y - PLAYER_SPRITE_HEIGHT / 2.), ray_dir, toi, solid, filter
        ).is_some() && rapier_context.cast_ray(
            vec2(player_x, (player_y - PLAYER_SPRITE_HEIGHT / 2.) + TILE_SIZE), ray_dir, toi, solid, filter
        ).is_none() {

            player_transform.translation.y += 1. * TILE_SIZE;
            player_transform.translation.x += player_direction * TILE_SIZE / 3.;
        }
    }
}

fn use_item(
    input: Res<Input<MouseButton>>,
    cursor: Res<CursorPosition>,
    mut inventory: ResMut<Inventory>,
    mut block_place_event_writer: EventWriter<BlockPlaceEvent>
) {
    if input.just_pressed(MouseButton::Left) {
        let selected_item_index = inventory.selected_slot;

        if let Some(item_stack) = inventory.selected_item() {
            match item_stack.item {
                Item::Pickaxe(_) => (),
                Item::Block(block) => {
                    let tile_coords = get_tile_coords(cursor.world_position);
                    block_place_event_writer.send(BlockPlaceEvent { coords: tile_coords, block });
                },
            }

            if item_stack.item.consumable() {
                inventory.consume_item(selected_item_index);
            }
        }
    }
}

fn limit_player_move(
    mut player_query: Query<&mut Transform, With<Player>>
) {
    let mut transform = player_query.single_mut();

    let min = PLAYER_SPRITE_WIDTH / 2.;
    let max = WORLD_SIZE_X as f32 * 16. - PLAYER_SPRITE_WIDTH / 2.;

    transform.translation.x = transform.translation.x.clamp(min, max);
}

// TODO: Debug function, remove in feature
#[cfg(debug_assertions)]
fn set_sprite_index(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut TextureAtlasSprite, &UseItemAnimationData), With<PlayerBodySprite>>,
) {
    query.for_each_mut(|(mut sprite, animation_data)| {
        let anim_offset = animation_data.0;

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
