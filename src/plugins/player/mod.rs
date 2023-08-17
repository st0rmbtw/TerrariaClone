mod components;
mod resources;
mod systems;
mod utils;
mod body_sprites;

pub(crate) use components::*;
pub(crate) use resources::*;
pub(crate) use body_sprites::*;
use systems::*;

use crate::{common::{state::{GameState, MovementState}, helpers::tile_pos_to_world_coords, systems::component_equals}, plugins::player::utils::simple_animation, world::WorldData, InGameSystemSet};
use std::time::Duration;
use bevy_hanabi::prelude::*;
use bevy::{prelude::*, time::{Timer, TimerMode}, math::vec2};

use super::{assets::PlayerAssets, world::constants::TILE_SIZE, inventory::UseItemAnimationData};

#[cfg(feature = "debug")]
use crate::plugins::debug::DebugConfiguration;

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

pub(crate) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), (setup, spawn_player));

        let flip_player_systems = (
            update_face_direction,
            flip_player,
            flip_using_item
        )
        .chain();

        #[cfg(feature = "debug")]
        let flip_player_systems = flip_player_systems.run_if(|config: Res<DebugConfiguration>| !config.free_camera);

        app.add_systems(
            Update,
            (
                flip_player_systems,
                update_movement_state,
                spawn_particles,
                (
                    update_movement_animation_timer,
                    update_movement_animation_index,
                    walking_animation.run_if(component_equals::<Player, _>(MovementState::Walking)),
                ).chain(),
                simple_animation::<IdleAnimationData>.run_if(component_equals::<Player, _>(MovementState::Idle)),
                simple_animation::<FlyingAnimationData>.run_if(component_equals::<Player, _>(MovementState::Flying))
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(PreUpdate, update_input_axis.in_set(InGameSystemSet::PreUpdate));

        let handle_player_movement_systems = (
            horizontal_movement,
            update_jump,
        );

        #[cfg(feature = "debug")]
        let handle_player_movement_systems = handle_player_movement_systems.run_if(|config: Res<DebugConfiguration>| !config.free_camera);

        app.add_systems(
            FixedUpdate,
            (
                handle_player_movement_systems,
                (
                    gravity,
                    detect_collisions,
                    move_player,
                    update_player_rect,
                )
                .chain()
            )
            .in_set(InGameSystemSet::FixedUpdate)
        );

        #[cfg(feature = "debug")]
        {
            use bevy::input::common_conditions::input_just_pressed;

            app.add_systems(
                Update,
                (
                    current_speed,
                    draw_hitbox.run_if(|config: Res<DebugConfiguration>| config.show_hitboxes),
                    teleport_player.run_if(
                        (|config: Res<DebugConfiguration>| config.free_camera)
                            .and_then(input_just_pressed(MouseButton::Right))
                    )
                )
                .in_set(InGameSystemSet::Update)
            );
        }
    }
}

fn setup(mut commands: Commands) {
    commands.init_resource::<PlayerVelocity>();
    commands.init_resource::<Collisions>();
    commands.init_resource::<PlayerData>();
    commands.insert_resource(InputAxis::default());
    commands.insert_resource(MovementAnimationIndex::default());
    commands.insert_resource(MovementAnimationTimer(Timer::new(Duration::from_millis(80), TimerMode::Repeating)));
}

fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut effects: ResMut<Assets<EffectAsset>>,
    world_data: Res<WorldData>
) {
    let mut module = Module::default();
    let init_position_cone3d = SetPositionCone3dModifier {
        base_radius: module.lit(5.),
        top_radius: module.lit(5.),
        height: module.lit(1.),
        dimension: ShapeDimension::Surface,
    };
    let init_velocity = SetVelocitySphereModifier {
        speed: module.lit(10.),
        center: module.lit(Vec3::splat(0.))
    };
    let init_lifetime = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: module.lit(0.2),
    };

    let spawner = Spawner::rate(40.0.into());
    let effect = effects.add(
        EffectAsset::new(50, spawner, module)
            .with_name("PlayerFeetDust")
            .init(init_position_cone3d)
            .init(init_velocity)
            .init(init_lifetime)
            .render(SetSizeModifier {
                size: CpuValue::Single(Vec2::new(0.8, 2.)),
                screen_space_size: true
            })
            .render(ColorOverLifetimeModifier { 
                gradient: Gradient::constant(Vec4::new(114. / 255., 81. / 255., 56. / 255., 1.))
            }),
    );

    let effect_entity = commands
        .spawn((
            Name::new("Particle Spawner"),
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect),
                transform: Transform::from_xyz(0., -(TILE_SIZE * 3. / 2.), 0.),
                ..default()
            }
        ))
        .id();

    let spawn_point = tile_pos_to_world_coords(world_data.spawn_point) 
        + TILE_SIZE / 2.
        + vec2(PLAYER_HALF_WIDTH, PLAYER_HALF_HEIGHT);

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
            spawn_player_hair(cmd, player_assets.hair.clone_weak(), 0.5);

            spawn_player_head(cmd, player_assets.head.clone_weak(), 0.1);

            spawn_player_eyes(cmd, player_assets.eyes_1.clone_weak(), player_assets.eyes_2.clone_weak(), 0.2);

            spawn_player_left_hand(cmd, player_assets.left_shoulder.clone_weak(), player_assets.left_hand.clone_weak(), 0.9);
            spawn_player_right_hand(cmd, player_assets.right_arm.clone_weak(), 0.);

            spawn_player_chest(cmd, player_assets.chest.clone_weak(), 0.1);

            spawn_player_feet(cmd, player_assets.feet.clone_weak(), 0.2);

            spawn_player_item_in_hand(cmd, 0.7);
        });
}