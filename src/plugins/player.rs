use std::{option::Option, time::Duration};

use autodefault::autodefault;
use bevy::{prelude::*, sprite::Anchor, math::{Vec3Swizzles, vec2}};
use bevy_hanabi::{
    AccelModifier, ColorOverLifetimeModifier, EffectAsset, Gradient, ParticleEffect,
    ParticleEffectBundle, ParticleLifetimeModifier, PositionCone3dModifier, ShapeDimension,
    SizeOverLifetimeModifier, Spawner,
};
use bevy_inspector_egui::Inspectable;
use iyes_loopless::prelude::*;

use crate::{
    items::{get_animation_points, Item},
    state::{GameState, MovementState},
    util::{map_range, get_tile_coords, get_rotation_by_direction, move_towards, FRect, Lerp, inverse_lerp},
    world_generator::WORLD_SIZE_X, Velocity, DefaultBundle,
};

use super::{world::{TILE_SIZE, WorldData, BlockPlaceEvent}, inventory::{PlayerInventoryPlugin, SelectedItem, Inventory}, assets::{PlayerAssets, ItemAssets}, cursor::CursorPosition};

pub const PLAYER_SPRITE_WIDTH: f32 = 2. * TILE_SIZE * 0.75;
pub const PLAYER_SPRITE_HEIGHT: f32 = 3. * TILE_SIZE;

const WALKING_ANIMATION_MAX_INDEX: usize = 13;

const USE_ITEM_ANIMATION_FRAMES_COUNT: usize = 3;

const MOVEMENT_ANIMATION_LABEL: &str = "movement_animation";
const USE_ITEM_ANIMATION_LABEL: &str = "use_item_animation";

const ACCELERATION: f32 = 6.;
const SLOWDOWN: f32 = 8.;
const MOVE_CLAMP: f32 = 3.;

const JUMP_HEIGHT: i32 = 15;
const JUMP_SPEED: f32 = 5.01;
const MAX_FALL_SPEED: f32 = 25.;

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
            .init_resource::<PlayerVelocity>()
            .init_resource::<PlayerRect>()
            .insert_resource(PlayerController {
                ..default()
            })
            .init_resource::<Collisions>()
            .insert_resource(UseItemAnimation(false))
            .add_enter_system(GameState::InGame, spawn_player)
            .add_system_set(update())
            .add_system_set_to_stage(
                CoreStage::PostUpdate, 
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(flip_player)
                    .with_system(spawn_particles)
                    .into()
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
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
                CoreStage::PostUpdate,
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

#[derive(Clone, Copy, Default, Deref, DerefMut)]
pub struct PlayerVelocity(pub Velocity);

#[derive(Clone, Copy, Default, Deref, DerefMut)]
pub struct PlayerRect(pub FRect);

#[derive(Clone, Copy, Default)]
struct PlayerController {
    fall_speed: f32,
    apex_point: f32,
    jump: i32
}

#[derive(Clone, Copy, Default)]
struct Collisions {
    up: bool,
    down: bool,
    left: bool,
    right: bool
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
        .spawn()
        .insert(Player)
        .insert_bundle(DefaultBundle {
            transform: Transform::from_xyz(WORLD_SIZE_X as f32 * 16. / 2., 5. * TILE_SIZE, 0.1)
        })
        .insert(Name::new("Player"))
        .insert(GroundDetection::default())
        .insert(MovementState::default())
        .insert(FaceDirection::default())
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
                    offset: 13,
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
                    offset: 13,
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
                visibility: Visibility {
                    is_visible: false
                },
                transform: Transform::from_xyz(0., 0., 0.15),
            })
            .insert(ChangeFlip)
            .insert(UsingItemMarker)
            .insert(Name::new("Using item"));

            // endregion

            // cmd.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::RED,
            //         custom_size: Some(Vec2::new(PLAYER_SPRITE_WIDTH, 1.))
            //     },
            //     transform: Transform::from_xyz(0., -PLAYER_SPRITE_HEIGHT / 2., 0.5),
            // });

            // cmd.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::RED,
            //         custom_size: Some(Vec2::new(PLAYER_SPRITE_WIDTH, 1.))
            //     },
            //     transform: Transform::from_xyz(0., PLAYER_SPRITE_HEIGHT / 2., 0.5),
            // });

            // cmd.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::RED,
            //         custom_size: Some(Vec2::new(1., PLAYER_SPRITE_HEIGHT))
            //     },
            //     transform: Transform::from_xyz(-PLAYER_SPRITE_WIDTH / 2., 0., 0.5),
            // });

            // cmd.spawn_bundle(SpriteBundle {
            //     sprite: Sprite {
            //         color: Color::RED,
            //         custom_size: Some(Vec2::new(1., PLAYER_SPRITE_HEIGHT))
            //     },
            //     transform: Transform::from_xyz(PLAYER_SPRITE_WIDTH / 2., 0., 0.5),
            // });
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

fn update() -> SystemSet {
    ConditionSet::new()
        .run_in_state(GameState::InGame)
        .with_system(update_rect)
        .with_system(update_axis)
        .with_system(update_face_direction)
        .with_system(collision_check)
        .with_system(horizontal_movement)
        .with_system(gravity)
        .with_system(jump)
        .with_system(jump_apex)
        .with_system(update_movement_state)
        .with_system(move_character)
        .with_system(use_item)
        .into()
}

fn update_rect(
    mut player_rect: ResMut<PlayerRect>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.single();
    let player_pos = player_transform.translation.xy();

    player_rect.0 = get_player_rect(player_pos);
}

fn collision_check(
    world_data: Res<WorldData>,
    player_rect: Res<PlayerRect>,
    mut collisions: ResMut<Collisions>,
    mut player_query: Query<&mut GroundDetection, With<Player>>,
) {
    let mut ground_detection = player_query.single_mut();

    let left = (player_rect.left / TILE_SIZE).round() as usize;
    let right = (player_rect.right / TILE_SIZE).round() as usize;
    let bottom = (player_rect.bottom / TILE_SIZE).round().abs() as usize;
    let top = (player_rect.top / TILE_SIZE).floor().abs() as usize;

    let mut col_left = false;
    let mut col_right = false;
    let mut col_down = false;
    let mut col_top = false;

    for x in left..(right + 1) {
        if col_down {
            break;
        }

        if world_data.tiles.get((bottom, x)).and_then(|cell| cell.tile).is_some() {
            col_down = true;
        }
    }
    
    for x in left..(right + 1) {
        if col_top {
            break;
        }

        if world_data.tiles.get((top, x)).and_then(|cell| cell.tile).is_some() {
            col_top = true;
        }
    }

    for y in top..bottom {
        if col_left {
            break;
        }

        if world_data.tiles.get((y, left)).and_then(|cell| cell.tile).is_some() {
            col_left = true;
        }
    }

    for y in top..bottom {
        if col_right {
            break;
        }

        if world_data.tiles.get((y, right)).and_then(|cell| cell.tile).is_some() {
            col_right = true;
        }
    }

    ground_detection.on_ground = col_down;
    collisions.up = col_top;
    collisions.down = col_down;
    collisions.left = col_left;
    collisions.right = col_right;
}

fn horizontal_movement(
    axis: Res<Axis>,
    time: Res<Time>,
    collsions: Res<Collisions>,
    mut velocity: ResMut<PlayerVelocity>
) {
    if axis.is_moving() {
        velocity.x += axis.x * ACCELERATION * time.delta_seconds();
        velocity.x = velocity.x.clamp(-MOVE_CLAMP, MOVE_CLAMP);
    } else {
        velocity.x = move_towards(velocity.x, 0., SLOWDOWN * time.delta_seconds());
    }

    if (velocity.x < 0. && collsions.left) || (velocity.x > 0. && collsions.right) {
        velocity.x = 0.;
    }
}

fn gravity(
    time: Res<Time>,
    collisions: Res<Collisions>,
    player_controller: Res<PlayerController>,
    mut velocity: ResMut<PlayerVelocity>,
) {
    if collisions.down {
        if velocity.y < 0. {
            velocity.y = 0.;
        }
    } else {
        velocity.y -= player_controller.fall_speed * time.delta_seconds();

        if velocity.y < -MAX_FALL_SPEED {
            velocity.y = -MAX_FALL_SPEED;
        }
    }
}

fn jump_apex(
    collisions: Res<Collisions>,
    mut player_controller: ResMut<PlayerController>
) {
    if !collisions.down {
        player_controller.apex_point = inverse_lerp(JUMP_HEIGHT as f32, 0., player_controller.jump as f32);
        player_controller.fall_speed = 0f32.lerp(MAX_FALL_SPEED, player_controller.apex_point);
    } else {
        player_controller.apex_point = 0.;
    }
}

fn jump(
    input: Res<Input<KeyCode>>,
    collisions: Res<Collisions>,
    mut velocity: ResMut<PlayerVelocity>,
    mut player_controller: ResMut<PlayerController>,
) {
    if input.just_pressed(KeyCode::Space) && collisions.down {
        player_controller.jump = JUMP_HEIGHT;
        velocity.y = JUMP_SPEED;
    }

    if input.pressed(KeyCode::Space) {
        if player_controller.jump > 0 {
            if velocity.y == 0. {
                player_controller.jump = 0;
            } else {
                velocity.y = JUMP_SPEED;

                player_controller.jump -= 1;
            }
        }
    } else {
        player_controller.jump = 0;
    }

    if collisions.up {
        if velocity.y > 0. {
            velocity.y = 0.;
        }
    }
}

fn move_character(
    velocity: Res<PlayerVelocity>,
    collisions: Res<Collisions>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = player_query.single_mut();

    const MIN: f32 = TILE_SIZE / 2.;
    const MAX: f32 = WORLD_SIZE_X as f32 * TILE_SIZE - TILE_SIZE / 2.;

    let raw = (transform.translation.xy() + velocity.0).clamp(vec2(MIN, -MAX), vec2(MAX, -MIN));
    
    transform.translation.x = raw.x;
    transform.translation.y = raw.y;

    let player_rect = get_player_rect(raw);

    if collisions.down && velocity.y == 0. {
        let threshold = (player_rect.bottom / TILE_SIZE).round() * TILE_SIZE + TILE_SIZE / 2. - 1.;
        
        if player_rect.bottom < threshold {
            transform.translation.y = threshold + PLAYER_SPRITE_HEIGHT / 2.;
        }
    }

    if collisions.left && velocity.x == 0. {
        let threshold = (player_rect.left / TILE_SIZE).round() * TILE_SIZE + TILE_SIZE / 2. - 2.;

        if player_rect.left < threshold {
            transform.translation.x = threshold + PLAYER_SPRITE_WIDTH / 2. + 0.5;
        }
    }

    if collisions.right && velocity.x == 0. {
        let threshold = (player_rect.right / TILE_SIZE).round() * TILE_SIZE - TILE_SIZE / 2. + 2.;

        if player_rect.right > threshold {
            transform.translation.x = threshold - PLAYER_SPRITE_WIDTH / 2. - 0.5;
        }
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

fn update_movement_state(
    velocity: Res<PlayerVelocity>,
    mut query: Query<(&GroundDetection, &mut MovementState), With<Player>>,
) {
    let (GroundDetection { on_ground }, mut movement_state) = query.single_mut();

    *movement_state = match velocity.x {
        x if x != 0. && *on_ground => MovementState::WALKING,
        _ => match on_ground {
            false => match velocity.y {
                y if y < 0. => MovementState::FLYING,
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

fn update_axis(input: Res<Input<KeyCode>>, mut axis: ResMut<Axis>) {
    let left = input.pressed(KeyCode::A);
    let right = input.pressed(KeyCode::D);

    let x = -(left as i8) + right as i8;

    axis.x = x as f32;
}

fn update_movement_animation_timer_duration(
    velocity: Res<PlayerVelocity>,
    mut timer: ResMut<AnimationTimer>,
) {
    if velocity.x != 0. {
        let mut time = 100. / velocity.x.abs();

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

fn use_item(
    input: Res<Input<MouseButton>>,
    cursor: Res<CursorPosition>,
    inventory: Res<Inventory>,
    mut block_place_event_writer: EventWriter<BlockPlaceEvent>
) {
    if input.pressed(MouseButton::Left) {
        let selected_item_index = inventory.selected_slot;

        if let Some(item_stack) = inventory.selected_item() {
            match item_stack.item {
                Item::Pickaxe(_) => (),
                Item::Block(block) => {
                    let tile_pos = get_tile_coords(cursor.world_position);
                    block_place_event_writer.send(BlockPlaceEvent { tile_pos, block, inventory_item_index: selected_item_index });
                },
            }
        }
    }
}

// TODO: Debug function, remove in feature
#[cfg(debug_assertions)]
fn set_sprite_index(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut TextureAtlasSprite, &WalkingAnimationData), With<PlayerBodySprite>>,
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

        sprite.index = anim_offset + (new_sprite_index % WALKING_ANIMATION_MAX_INDEX);
    });
}

fn get_player_rect(position: Vec2) -> FRect {
    FRect {
        left: position.x - PLAYER_SPRITE_WIDTH / 2.,
        right: position.x + PLAYER_SPRITE_WIDTH / 2.,
        bottom: position.y - PLAYER_SPRITE_HEIGHT / 2.,
        top: position.y + PLAYER_SPRITE_HEIGHT / 2.
    }
}