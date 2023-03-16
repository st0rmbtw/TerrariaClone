mod components;
mod resources;
mod systems;
mod utils;

pub use components::*;
pub use resources::*;
pub use systems::*;
pub use utils::*;

use crate::{state::GameState, util::tile_to_world_coords};
use std::time::Duration;
use bevy_hanabi::prelude::*;
use bevy::{prelude::*, time::{Timer, TimerMode}, sprite::Anchor};
use autodefault::autodefault;
use leafwing_input_manager::prelude::InputManagerPlugin;

use super::{assets::PlayerAssets, world::{WorldData, TILE_SIZE}, debug::DebugConfiguration};

pub const PLAYER_WIDTH: f32 = 22. /* 2. * TILE_SIZE */;
pub const PLAYER_HEIGHT: f32 = 42. /* 3. * TILE_SIZE */;

pub const PLAYER_HALF_WIDTH: f32 = PLAYER_WIDTH / 2.;
pub const PLAYER_HALF_HEIGHT: f32 = PLAYER_HEIGHT / 2.;

const WALKING_ANIMATION_MAX_INDEX: usize = 13;

const USE_ITEM_ANIMATION_FRAMES_COUNT: usize = 3;

const GRAVITY: f32 = 0.4;
const ACCELERATION: f32 = 0.1;
const SLOWDOWN: f32 = 0.2;
pub const MAX_RUN_SPEED: f32 = 3.;

const JUMP_HEIGHT: i32 = 15;
const JUMP_SPEED: f32 = 5.01;
pub const MAX_FALL_SPEED: f32 = -10.;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhysicsSet {
    SetVelocity,
    Movement,
    CollisionDetection
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default());
        app.insert_resource(InputAxis::default());
        app.insert_resource(MovementAnimationIndex::default());
        app.insert_resource(UseItemAnimationIndex::default());
        app.insert_resource(AnimationTimer(Timer::new(Duration::from_millis(80), TimerMode::Repeating)));
        app.insert_resource(UseItemAnimation(false));
        app.insert_resource(UseItemAnimationTimer(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating
        )));

        app.init_resource::<PlayerVelocity>();
        app.init_resource::<Collisions>();

        app.add_system(spawn_player.in_schedule(OnEnter(GameState::InGame)));

        app.add_system(update_face_direction.in_set(OnUpdate(GameState::InGame)));
        app.add_system(flip_player.in_set(OnUpdate(GameState::InGame)));
        app.add_systems(
            (
                update_movement_state,
                spawn_particles
            )
            .chain()
            .after(PhysicsSet::Movement)
            .in_set(OnUpdate(GameState::InGame))
        );

        app.add_systems(
            (
                update_movement_animation_timer_duration,
                update_movement_animation_index,
                walking_animation.run_if(is_walking),
                simple_animation::<IdleAnimationData>.run_if(is_idle),
                simple_animation::<FlyingAnimationData>.run_if(is_flying)
            )
            .chain()
            .in_set(OnUpdate(GameState::InGame))
        );

        app.add_systems(
            (
                set_using_item_image,
                set_using_item_position,
                set_using_item_rotation,
                update_use_item_animation_index,
                set_using_item_rotation_on_player_direction_change,
                use_item_animation
            )
            .chain()
            .distributive_run_if(|res: Res<UseItemAnimation>| *res == UseItemAnimation(true))
            .in_set(OnUpdate(GameState::InGame))
        );

        app.add_systems(
            (
                update_axis,
                player_using_item,
                set_using_item_visibility,
            )
            .chain()
            .in_set(OnUpdate(GameState::InGame))
        );

        app.add_systems(
            (
                horizontal_movement,
                update_jump,
                gravity
            )
            .chain()
            .distributive_run_if(in_state(GameState::InGame))
            .distributive_run_if(|config: Res<DebugConfiguration>| !config.free_camera)
            .in_set(PhysicsSet::SetVelocity)
            .before(PhysicsSet::CollisionDetection)
            .in_schedule(CoreSchedule::FixedUpdate)
        );

        app.add_system(
            detect_collisions
                .run_if(in_state(GameState::InGame))
                .run_if(|config: Res<DebugConfiguration>| !config.free_camera)
                .in_schedule(CoreSchedule::FixedUpdate)
                .in_set(PhysicsSet::CollisionDetection)
                .after(PhysicsSet::SetVelocity)
                .before(PhysicsSet::Movement)
        );

        app.add_system(
            move_player
                .run_if(in_state(GameState::InGame))
                .run_if(|config: Res<DebugConfiguration>| !config.free_camera)
                .in_schedule(CoreSchedule::FixedUpdate)
                .in_set(PhysicsSet::Movement)
                .after(PhysicsSet::CollisionDetection)
        );

        // app.add_system(
        //     interpolate_player_transform
        //         .run_if(in_state(GameState::InGame))
        //         .run_if(|config: Res<DebugConfiguration>| !config.free_camera)
        //         .in_set(PhysicsSet::Movement)
        // );

        #[cfg(feature = "debug")]
        app.add_system(
            draw_hitbox
                .run_if(|config: Res<DebugConfiguration>| config.show_hitboxes)
                .in_set(OnUpdate(GameState::InGame))
        );

        // app.add_system(current_speed);
    }
}

#[autodefault(except(GroundSensor, PlayerParticleEffects))]
pub fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut effects: ResMut<Assets<EffectAsset>>,
    world_data: Res<WorldData>
) {
    let player_spawn_point = {
        let spawn_point = tile_to_world_coords(world_data.spawn_point) + TILE_SIZE / 2.;

        Vec2::new(spawn_point.x + PLAYER_HALF_WIDTH, spawn_point.y + PLAYER_HALF_HEIGHT)
    };

    commands.insert_resource(PlayerData {
        prev_position: player_spawn_point
    });

    let player = commands
        .spawn(PlayerBundle::new(
            Transform::from_xyz(player_spawn_point.x, player_spawn_point.y, 3.)
        ))
        .with_children(|cmd| {
            // region: Hair
            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Player hair"),
                MovementAnimationBundle::default(),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.55, 0.23, 0.14),
                    },
                    transform: Transform::from_xyz(0., 0., 0.1),
                    texture_atlas: player_assets.hair.clone_weak(),
                }
            ));
            // endregion

            // region: Head
            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Player head"),
                MovementAnimationBundle::default(),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.92, 0.45, 0.32),
                    },
                    texture_atlas: player_assets.head.clone_weak(),
                    transform: Transform::from_xyz(0., 0., 0.003),
                }
            ));
            // endregion

            // region: Eyes
            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Player left eye"),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::WHITE,
                    },
                    transform: Transform::from_xyz(0., 0., 0.1),
                    texture_atlas: player_assets.eyes_1.clone_weak(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData {
                        offset: 6,
                        count: 14,
                    }
                }
            ));

            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Player right eye"),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(89. / 255., 76. / 255., 64. / 255.),
                    },
                    transform: Transform::from_xyz(0., 0., 0.01),
                    texture_atlas: player_assets.eyes_2.clone_weak(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData {
                        offset: 6,
                        count: 14,
                    }
                }
            ));

            // endregion

            // region: Arms
            // region: Left arm
            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Player left shoulder"),
                UseItemAnimationData(2),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.58, 0.55, 0.47),
                    },
                    transform: Transform::from_xyz(0., -8., 0.2),
                    texture_atlas: player_assets.left_shoulder.clone_weak(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData {
                        offset: 13,
                        count: 13,
                    },
                    flying: FlyingAnimationData(2)
                }
            ));

            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Player left hand"),
                UseItemAnimationData(2),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.92, 0.45, 0.32),
                    },
                    transform: Transform::from_xyz(0., -8., 0.2),
                    texture_atlas: player_assets.left_hand.clone_weak(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData {
                        offset: 13,
                        count: 13,
                    },
                    flying: FlyingAnimationData(2)
                }
            ));
            // endregion

            // region: Right arm
            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Player right hand"),
                UseItemAnimationData(15),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(0.92, 0.45, 0.32),
                    },
                    transform: Transform::from_xyz(0., -20., 0.001),
                    texture_atlas: player_assets.right_arm.clone_weak(),
                },
                MovementAnimationBundle {
                    walking: WalkingAnimationData { count: 13 },
                    idle: IdleAnimationData(14),
                    flying: FlyingAnimationData(13),
                }
            ));
            // endregion

            // endregion

            // region: Chest
            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Player chest"),
                MovementAnimationBundle::default(),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: 0,
                        color: Color::rgb(0.58, 0.55, 0.47),
                    },
                    transform: Transform::from_xyz(0., 0., 0.002),
                    texture_atlas: player_assets.chest.clone_weak(),
                },
            ));
            // endregion

            // region: Feet
            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Player feet"),
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgb(190. / 255., 190. / 255., 156. / 255.),
                    },
                    texture_atlas: player_assets.feet.clone_weak(),
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
            ));
            // endregion

            // region: Used item
            cmd.spawn((
                ChangeFlip,
                PlayerBodySprite,
                Name::new("Using item"),
                UsedItem,
                SpriteBundle {
                    sprite: Sprite {
                        anchor: Anchor::BottomLeft,
                    },
                    visibility: Visibility::Hidden,
                    transform: Transform::from_xyz(0., 0., 0.15),
                }
            ));

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
        .update(AccelModifier::constant(Vec3::new(0., 0., 0.)))
        // Render the particles with a color gradient over their
        // lifetime.
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant(Vec2::splat(3.)),
        })
        .init(InitLifetimeModifier { lifetime: 1_f32.into() })
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