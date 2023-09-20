mod components;
mod resources;
mod systems;
mod utils;
mod body_sprites;

use resources::*;
use systems::*;
pub(crate) use components::*;
pub(crate) use body_sprites::*;

use crate::{common::{state::{GameState, MovementState}, helpers::tile_pos_to_world_coords, systems::{component_equals, despawn_with, set_resource}}, plugins::player::utils::simple_animation, world::WorldData};
use std::time::Duration;
use bevy::{prelude::*, time::{Timer, TimerMode}, math::vec2};

use super::{assets::PlayerAssets, world::constants::TILE_SIZE, inventory::UseItemAnimationData, InGameSystemSet};

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
pub(crate) const MAX_FALL_SPEED: f32 = 10.;

pub(crate) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), (setup, spawn_player));
        app.add_systems(OnExit(GameState::InGame), (cleanup, despawn_with::<Player>));

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
                    update_player_position,
                    update_player_rect,
                )
                .chain()
            )
            .in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(Update, move_player.in_set(InGameSystemSet::Update));

        app.add_systems(PostUpdate, set_resource(InputAxis::default()));

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
    commands.init_resource::<Collisions>();
    commands.init_resource::<PlayerData>();
    commands.insert_resource(InputAxis::default());
    commands.insert_resource(MovementAnimationIndex::default());
    commands.insert_resource(MovementAnimationTimer(Timer::new(Duration::from_millis(80), TimerMode::Repeating)));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Collisions>();
    commands.remove_resource::<PlayerData>();
    commands.remove_resource::<InputAxis>();
    commands.remove_resource::<MovementAnimationIndex>();
    commands.remove_resource::<MovementAnimationTimer>();
}

fn spawn_player(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    world_data: Res<WorldData>
) {
    let spawn_point = tile_pos_to_world_coords(world_data.spawn_point) 
        + TILE_SIZE / 2.
        + vec2(PLAYER_HALF_WIDTH, PLAYER_HALF_HEIGHT);

    commands
        .spawn(PlayerBundle::new(spawn_point.x, spawn_point.y))
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