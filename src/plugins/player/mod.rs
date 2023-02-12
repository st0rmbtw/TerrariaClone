mod components;
mod resources;
mod systems;
mod utils;

pub use components::*;
pub use resources::*;
pub use systems::*;
pub use utils::*;

use crate::{state::{GameState, MovementState}, labels::PlayerLabel, world_generator::WORLD_SIZE_X};
use std::time::Duration;
use iyes_loopless::prelude::*;
use bevy_hanabi::prelude::*;
use bevy::{prelude::*, time::{Timer, TimerMode}, sprite::Anchor};
use autodefault::autodefault;

use super::{world::TILE_SIZE, assets::PlayerAssets};

pub const PLAYER_WIDTH: f32 = 22. /* 2. * TILE_SIZE */;
pub const PLAYER_HEIGHT: f32 = 42.5 /* 3. * TILE_SIZE */;

const WALKING_ANIMATION_MAX_INDEX: usize = 13;

const USE_ITEM_ANIMATION_FRAMES_COUNT: usize = 3;

const MOVEMENT_ANIMATION_LABEL: &str = "movement_animation";
const USE_ITEM_ANIMATION_LABEL: &str = "use_item_animation";

const GRAVITY: f32 = 30. * TILE_SIZE;
const ACCELERATION: f32 = 0.05 * TILE_SIZE;
const SLOWDOWN: f32 = 1.5;
pub const MAX_RUN_SPEED: f32 = 11. * TILE_SIZE;

const JUMP_HEIGHT: i32 = 75;
const JUMP_SPEED: f32 = 5.01 * TILE_SIZE;
pub const MAX_FALL_SPEED: f32 = -37.5 * TILE_SIZE;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InputAxis::default())
            .insert_resource(MovementAnimationIndex::default())
            .insert_resource(UseItemAnimationIndex::default())
            .insert_resource(AnimationTimer(Timer::new(Duration::from_millis(80), TimerMode::Repeating)))
            .insert_resource(UseItemAnimationTimer(Timer::new(
                Duration::from_millis(100),
                TimerMode::Repeating
            )))
            .init_resource::<PlayerVelocity>()
            .init_resource::<PlayerController>()
            .init_resource::<Collisions>()
            .insert_resource(UseItemAnimation(false))
            .add_enter_system(GameState::InGame, spawn_player)
            .add_system(update_axis)
            .add_system_set_to_stage(
                CoreStage::PostUpdate, 
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(update_movement_state)
                    .with_system(update_face_direction)
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

        #[cfg(not(feature = "debug_movement"))] {
            app
            .add_system_to_stage(
                CoreStage::Update,
                horizontal_movement
                    .run_in_state(GameState::InGame)
                    .label(PlayerLabel::HorizontalMovement)
            )
            .add_system_to_stage(
                CoreStage::Update,
                update_jump
                    .run_in_state(GameState::InGame)
                    .label(PlayerLabel::Jump)
                    .after(PlayerLabel::HorizontalMovement)
            )
            .add_system_to_stage(
                CoreStage::Update,
                update
                    .run_in_state(GameState::InGame)
                    .label(PlayerLabel::MovePlayer)
                    .after(PlayerLabel::Jump)
            );
        }

        // app.add_system(current_speed);

        #[cfg(feature = "debug_movement")] {
            app.add_system(
                debug_horizontal_movement
                    .run_in_state(GameState::InGame)
                    .label(PlayerLabel::HorizontalMovement)
            )
            .add_system(
                debug_vertical_movement
                    .run_in_state(GameState::InGame)
                    .label(PlayerLabel::Collide)
                    .after(PlayerLabel::HorizontalMovement)
            );
        }
    }
}

#[autodefault(except(GroundSensor, PlayerParticleEffects))]
pub fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let player = commands
        .spawn(SpatialBundle {
            transform: Transform::from_xyz(WORLD_SIZE_X as f32 * 16. / 2., 0., 3.)
        })
        .insert(Player)
        .insert(Name::new("Player"))
        .insert(MovementState::default())
        .insert(FaceDirection::default())
        .with_children(|cmd| {
            // region: Hair
            cmd.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.55, 0.23, 0.14),
                    },
                    transform: Transform::from_xyz(0., 0., 0.1),
                    texture_atlas: player_assets.hair.clone(),
                },
                MovementAnimationBundle::default()
            ))
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(Name::new("Player hair"));
            // endregion

            // region: Head
            cmd.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.92, 0.45, 0.32),
                    },
                    texture_atlas: player_assets.head.clone(),
                    transform: Transform::from_xyz(0., 0., 0.003),
                },
                MovementAnimationBundle::default()
            ))
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(Name::new("Player head"));
            // endregion

            // region: Eyes
            cmd.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::WHITE,
                    },
                    transform: Transform::from_xyz(0., 0., 0.1),
                    texture_atlas: player_assets.eyes_1.clone(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData {
                        offset: 6,
                        count: 14,
                    }
                }
            ))
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(Name::new("Player left eye"));

            cmd.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(89. / 255., 76. / 255., 64. / 255.),
                    },
                    transform: Transform::from_xyz(0., 0., 0.01),
                    texture_atlas: player_assets.eyes_2.clone(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData {
                        offset: 6,
                        count: 14,
                    }
                }
            ))
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(Name::new("Player right eye"));

            // endregion

            // region: Arms
            // region: Left arm
            cmd.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.58, 0.55, 0.47),
                    },
                    transform: Transform::from_xyz(0., -8., 0.2),
                    texture_atlas: player_assets.left_shoulder.clone(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData {
                        offset: 13,
                        count: 13,
                    },
                    flying: FlyingAnimationData(2)
                }
            ))
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(UseItemAnimationData(2))
            .insert(Name::new("Player left shoulder"));

            cmd.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.92, 0.45, 0.32),
                    },
                    transform: Transform::from_xyz(0., -8., 0.2),
                    texture_atlas: player_assets.left_hand.clone(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData {
                        offset: 13,
                        count: 13,
                    },
                    flying: FlyingAnimationData(2)
                }
            ))
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(UseItemAnimationData(2))
            .insert(Name::new("Player left hand"));
            // endregion

            // region: Right arm
            cmd.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.92, 0.45, 0.32),
                    },
                    transform: Transform::from_xyz(0., -20., 0.001),
                    texture_atlas: player_assets.right_arm.clone(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData { count: 13 },
                    idle: IdleAnimationData(14),
                    flying: FlyingAnimationData(13),
                }
            ))
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(UseItemAnimationData(15))
            .insert(Name::new("Player right hand"));
            // endregion

            // endregion

            // region: Chest
            cmd.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: 0,
                        color: Color::rgb(0.58, 0.55, 0.47),
                    },
                    transform: Transform::from_xyz(0., 0., 0.002),
                    texture_atlas: player_assets.chest.clone(),
                },
                MovementAnimationBundle::default()
            ))
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(Name::new("Player chest"));
            // endregion

            // region: Feet
            cmd.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(190. / 255., 190. / 255., 156. / 255.),
                    },
                    texture_atlas: player_assets.feet.clone(),
                    transform: Transform::from_xyz(0., 0., 0.15),
                    ..default()
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData {
                        offset: 6,
                        count: 13,
                    },
                    flying: FlyingAnimationData(5),
                }
            ))
            .insert(ChangeFlip)
            .insert(PlayerBodySprite)
            .insert(Name::new("Player feet"));
            // endregion

            // region: Used item
            cmd.spawn(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                },
                visibility: Visibility {
                    is_visible: false
                },
                transform: Transform::from_xyz(0., 0., 0.15),
            })
            .insert(ChangeFlip)
            .insert(UsedItem)
            .insert(Name::new("Using item"));

            // endregion

            #[cfg(feature = "debug")] {
                cmd.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::new(PLAYER_WIDTH, 1.))
                    },
                    transform: Transform::from_xyz(0., -PLAYER_HEIGHT / 2., 0.5),
                });

                cmd.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::new(PLAYER_WIDTH, 1.))
                    },
                    transform: Transform::from_xyz(0., PLAYER_HEIGHT / 2., 0.5),
                });

                cmd.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::new(1., PLAYER_HEIGHT))
                    },
                    transform: Transform::from_xyz(-PLAYER_WIDTH / 2., 0., 0.5),
                });

                cmd.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::new(1., PLAYER_HEIGHT))
                    },
                    transform: Transform::from_xyz(PLAYER_WIDTH / 2., 0., 0.5),
                });
            }
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
        .spawn(ParticleEffectBundle::new(effect).with_spawner(spawner))
        .insert(Name::new("Particle Spawner"))
        .id();

    commands.entity(player).add_child(effect_entity);

    commands.entity(player).insert(PlayerParticleEffects {
        walking: effect_entity,
    });
}