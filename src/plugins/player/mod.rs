mod components;
mod resources;
mod systems;
mod utils;
mod body_sprites;

pub(crate) use components::*;
pub(crate) use resources::*;
pub(crate) use body_sprites::*;
use systems::*;

use crate::{common::{state::GameState, helpers::tile_pos_to_world_coords}, DebugConfiguration, plugins::player::utils::{simple_animation, is_walking, is_idle, is_flying}, world::WorldData};
use std::time::Duration;
use bevy_hanabi::prelude::*;
use bevy::{prelude::*, time::{Timer, TimerMode}, sprite::Anchor, math::vec2};
use autodefault::autodefault;

use super::{assets::PlayerAssets, world::TILE_SIZE, inventory::UseItemAnimationData};

const PLAYER_WIDTH: f32 = 22.;
const PLAYER_HEIGHT: f32 = 42.;

const PLAYER_HALF_WIDTH: f32 = PLAYER_WIDTH / 2.;
const PLAYER_HALF_HEIGHT: f32 = PLAYER_HEIGHT / 2.;

const WALKING_ANIMATION_MAX_INDEX: usize = 13;

const GRAVITY: f32 = 0.4;
const ACCELERATION: f32 = 0.1;
const SLOWDOWN: f32 = 0.2;

const JUMP_HEIGHT: i32 = 15;
const JUMP_SPEED: f32 = 5.01;

pub(crate) const MAX_RUN_SPEED: f32 = 3.;
pub(crate) const MAX_FALL_SPEED: f32 = -10.;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhysicsSet {
    SetVelocity,
    Update
}

pub(crate) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputAxis::default());
        app.insert_resource(MovementAnimationIndex::default());
        app.insert_resource(MovementAnimationTimer(Timer::new(Duration::from_millis(80), TimerMode::Repeating)));
        app.init_resource::<PlayerVelocity>();
        app.init_resource::<Collisions>();
        app.init_resource::<PlayerData>();

        app.add_system(spawn_player.in_schedule(OnEnter(GameState::InGame)));

        app.add_systems(
            (
                update_face_direction,
                flip_player,
                flip_using_item
            )
            .chain()
            .in_set(OnUpdate(GameState::InGame))
            .distributive_run_if(|config: Res<DebugConfiguration>| !config.free_camera)
        );

        app.add_systems(
            (
                update_movement_state,
                spawn_particles
            )
            .after(PhysicsSet::Update)
            .in_set(OnUpdate(GameState::InGame))
        );

        app.add_systems(
            (
                update_movement_animation_timer,
                update_movement_animation_index,
                walking_animation.run_if(is_walking),
                simple_animation::<IdleAnimationData>.run_if(is_idle),
                simple_animation::<FlyingAnimationData>.run_if(is_flying)
            )
            .in_set(OnUpdate(GameState::InGame))
        );

        app.add_system(update_input_axis.in_set(OnUpdate(GameState::InGame)));

        app.add_systems(
            (
                horizontal_movement,
                update_jump,
            )
            .distributive_run_if(in_state(GameState::InGame))
            .distributive_run_if(|config: Res<DebugConfiguration>| !config.free_camera)
            .in_set(PhysicsSet::SetVelocity)
            .before(PhysicsSet::Update)
            .in_schedule(CoreSchedule::FixedUpdate)
        );

        app.add_systems(
            (
                gravity,
                detect_collisions,
                move_player
            )
            .chain()
            .distributive_run_if(in_state(GameState::InGame))
            .in_set(PhysicsSet::Update)
            .in_schedule(CoreSchedule::FixedUpdate)
        );

        #[cfg(feature = "debug")]
        {
            use bevy::input::common_conditions::input_just_pressed;

            app.add_systems(
                (
                    draw_hitbox.run_if(|config: Res<DebugConfiguration>| config.show_hitboxes),
                    teleport_player.run_if(
                        (|config: Res<DebugConfiguration>| config.free_camera)
                            .and_then(input_just_pressed(MouseButton::Right))
                    )
                )
                .in_set(OnUpdate(GameState::InGame))
            );

            app.add_system(current_speed);
        }
    }
}

#[autodefault(except(GroundSensor, PlayerParticleEffects))]
fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut effects: ResMut<Assets<EffectAsset>>,
    world_data: Res<WorldData>
) {
    let spawner = Spawner::rate(40.0.into());
    let effect = effects.add(
        EffectAsset {
            name: "PlayerFeetDust".to_string(),
            capacity: 50,
            spawner,
            z_layer_2d: 10.,
        }
        .init(InitPositionCone3dModifier {
            base_radius: 5.,
            top_radius: 5.,
            height: 1.,
            dimension: ShapeDimension::Surface,
        })
        .init(InitVelocitySphereModifier {
            speed: 10.0.into()
        })
        .init(InitSizeModifier {
            size: DimValue::D1(Value::Uniform((0.8, 2.)))
        })
        .update(AccelModifier::constant(Vec3::new(0., 0., 0.)))
        .init(InitLifetimeModifier { lifetime: 0.2.into() })
        .render(ColorOverLifetimeModifier { 
            gradient: Gradient::constant(Vec4::new(114. / 255., 81. / 255., 56. / 255., 1.))
        }),
    );

    let effect_entity = commands
        .spawn(
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect),
                transform: Transform::from_xyz(0., -(TILE_SIZE * 3. / 2.), 0.),
                ..default()
            }
            .with_spawner(spawner)
        )
        .insert(Name::new("Particle Spawner"))
        .id();

    let spawn_point = tile_pos_to_world_coords(world_data.spawn_point) + vec2(PLAYER_HALF_WIDTH, PLAYER_HALF_HEIGHT) + TILE_SIZE / 2.;

    commands
        .spawn((
            PlayerBundle::new(spawn_point.x, spawn_point.y),
            PlayerParticleEffects {
                walking: effect_entity,
            }
        ))
        .add_child(effect_entity)
        .with_children(|cmd| {
            use body_sprites::*;
            spawn_player_hair(cmd, player_assets.hair.clone_weak());

            spawn_player_head(cmd, player_assets.head.clone_weak());

            spawn_player_eyes(cmd, player_assets.eyes_1.clone_weak(), player_assets.eyes_2.clone_weak());

            spawn_player_left_hand(cmd, player_assets.left_shoulder.clone_weak(), player_assets.left_hand.clone_weak());
            spawn_player_right_hand(cmd, player_assets.right_arm.clone_weak());

            spawn_player_chest(cmd, player_assets.chest.clone_weak());

            spawn_player_feet(cmd, player_assets.feet.clone_weak());

            spawn_player_item_in_hand(cmd);
        });
}