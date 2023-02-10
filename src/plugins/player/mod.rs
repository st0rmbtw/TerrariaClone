mod components;
mod resources;
mod systems;
mod utils;

pub use components::*;
pub use resources::*;
pub use systems::*;
pub use utils::*;

use crate::{state::GameState, labels::PlayerLabel};
use std::time::Duration;
use iyes_loopless::prelude::*;
use bevy::{prelude::{Plugin, App, CoreStage}, time::{Timer, TimerMode}};

use super::world::TILE_SIZE;

pub const PLAYER_WIDTH: f32 = 22. /* 2. * TILE_SIZE */;
pub const PLAYER_HEIGHT: f32 = 42. /* 3. * TILE_SIZE */;

const WALKING_ANIMATION_MAX_INDEX: usize = 13;

const USE_ITEM_ANIMATION_FRAMES_COUNT: usize = 3;

const MOVEMENT_ANIMATION_LABEL: &str = "movement_animation";
const USE_ITEM_ANIMATION_LABEL: &str = "use_item_animation";

const GRAVITY: f32 = 30. * TILE_SIZE;
const ACCELERATION: f32 = 0.05 * TILE_SIZE;
const SLOWDOWN: f32 = 1.5;
pub const MAX_RUN_SPEED: f32 = 11. * TILE_SIZE;

const JUMP_HEIGHT: i32 = 15;
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
