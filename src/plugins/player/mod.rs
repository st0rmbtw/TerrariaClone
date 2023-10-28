mod components;
mod resources;
mod systems;
mod utils;
pub(crate) mod body_sprites;

use resources::*;
pub(crate) use components::*;

use crate::{common::{state::{GameState, MovementState}, helpers::tile_to_world_pos, systems::{component_equals, despawn_with}}, plugins::player::utils::simple_animation, world::WorldData};
use std::time::Duration;
use bevy::{prelude::*, time::{Timer, TimerMode, common_conditions::on_timer}, math::vec2, input::InputSystem};

use super::{assets::PlayerAssets, world::constants::TILE_SIZE, inventory::UseItemAnimationData, InGameSystemSet, entity::EntitySet, world_map_view::MapViewStatus};

#[cfg(feature = "debug")]
use crate::plugins::debug::DebugConfiguration;
#[cfg(feature = "debug")]
use bevy::input::common_conditions::input_pressed;

pub(crate) const PLAYER_WIDTH: f32 = 22.;
pub(crate) const PLAYER_HEIGHT: f32 = 42.;

pub(crate) const PLAYER_HALF_WIDTH: f32 = PLAYER_WIDTH / 2.;
pub(crate) const PLAYER_HALF_HEIGHT: f32 = PLAYER_HEIGHT / 2.;

const WALKING_ANIMATION_MAX_INDEX: usize = 13;

const GRAVITY: f32 = 0.4;
const ACCELERATION: f32 = 0.1;
const SLOWDOWN: f32 = 0.2;

const JUMP_HEIGHT: i32 = 15;
const JUMP_SPEED: f32 = 5.01;

pub(crate) const MAX_WALK_SPEED: f32 = 3.;
pub(crate) const MAX_FALL_SPEED: f32 = 10.;

pub(crate) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), (setup, spawn_player));
        app.add_systems(OnExit(GameState::InGame), (cleanup, despawn_with::<Player>));

        let flip_player_systems = (
            systems::update_face_direction,
            systems::flip_player,
            systems::flip_using_item
        )
        .chain();

        #[cfg(feature = "debug")]
        let flip_player_systems = flip_player_systems.run_if(|config: Res<DebugConfiguration>| !config.free_camera);

        app.add_systems(
            Update,
            (
                flip_player_systems,
                (
                    systems::update_movement_animation_timer,
                    systems::update_movement_animation_index,
                    systems::walking_animation.run_if(component_equals::<Player, _>(MovementState::Walking)),
                ).chain(),
                simple_animation::<IdleAnimationData>.run_if(component_equals::<Player, _>(MovementState::Idle)),
                simple_animation::<FlyingAnimationData>.run_if(component_equals::<Player, _>(MovementState::Flying)),
                systems::spawn_particles_on_walk.run_if(on_timer(Duration::from_secs_f32(1. / 20.))),
                systems::spawn_particles_grounded
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            PreUpdate,
            systems::update_input_axis.after(InputSystem)
                .run_if(resource_equals(MapViewStatus::Closed))
                .in_set(InGameSystemSet::PreUpdate)
        );

        let update_jump = systems::update_jump.run_if(resource_equals(MapViewStatus::Closed));
        
        #[cfg(feature = "debug")]
        let update_jump = update_jump.run_if(|config: Res<DebugConfiguration>| !config.free_camera);

        app.add_systems(
            FixedUpdate,
            (
                (
                    systems::horizontal_movement,
                    (
                        update_jump,
                        systems::gravity,
                    ).chain()
                )
                .before(EntitySet::UpdateEntityRect),

                systems::detect_collisions
                    .after(EntitySet::UpdateEntityRect)
                    .before(EntitySet::MoveEntity),
            )
            .in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(
            PostUpdate,
            (
                systems::reset_fallstart,
                systems::update_movement_state
            )
            .in_set(InGameSystemSet::PostUpdate)
        );

        #[cfg(feature = "debug")]
        {
            app.add_systems(
                Update,
                (
                    systems::current_speed,
                    systems::draw_hitbox.run_if(|config: Res<DebugConfiguration>| config.show_hitboxes),
                    systems::teleport_player
                        .before(EntitySet::UpdateEntityRect)
                        .run_if(
                            (|config: Res<DebugConfiguration>| config.free_camera).and_then(
                                input_pressed(MouseButton::Right)
                            )
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
    let spawn_point = tile_to_world_pos(world_data.spawn_point) 
        + TILE_SIZE / 2.
        + vec2(PLAYER_HALF_WIDTH, PLAYER_HALF_HEIGHT);

    commands
        .spawn(PlayerBundle::new(spawn_point.x, spawn_point.y))
        .with_children(|parent| {
            use body_sprites::*;
            spawn_player_head(parent, &player_assets);
            spawn_player_body(parent, &player_assets);

            spawn_player_feet(parent, player_assets.feet.clone_weak(), 0.2);

            spawn_player_item_in_hand(parent, 0.7);
        });
}