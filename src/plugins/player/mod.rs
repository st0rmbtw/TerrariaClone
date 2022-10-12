mod components;
mod resources;
mod systems;
mod utils;

pub use components::*;
pub use resources::*;
pub use systems::*;
pub use utils::*;

use crate::state::GameState;
use std::time::Duration;
use iyes_loopless::prelude::*;
use bevy::{prelude::{Plugin, App, CoreStage}, time::Timer};
use super::{inventory::PlayerInventoryPlugin};

pub const PLAYER_SPRITE_WIDTH: f32 = 20. /* 2. * TILE_SIZE */;
pub const PLAYER_SPRITE_HEIGHT: f32 = 42. /* 3. * TILE_SIZE */;

const WALKING_ANIMATION_MAX_INDEX: usize = 13;

const USE_ITEM_ANIMATION_FRAMES_COUNT: usize = 3;

const MOVEMENT_ANIMATION_LABEL: &str = "movement_animation";
const USE_ITEM_ANIMATION_LABEL: &str = "use_item_animation";

const GRAVITY: f32 = 0.4;
const ACCELERATION: f32 = 0.08;
const SLOWDOWN: f32 = 0.2;
const MAX_RUN_SPEED: f32 = 3.;

const JUMP_HEIGHT: i32 = 15;
const JUMP_SPEED: f32 = 5.01;
const MAX_FALL_SPEED: f32 = 10.;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayerInventoryPlugin)
            .insert_resource(InputAxis::default())
            .insert_resource(MovementAnimationIndex::default())
            .insert_resource(UseItemAnimationIndex::default())
            .insert_resource(AnimationTimer(Timer::new(Duration::from_millis(80), true)))
            .insert_resource(UseItemAnimationTimer(Timer::new(
                Duration::from_millis(100),
                true,
            )))
            .init_resource::<PlayerVelocity>()
            .init_resource::<PlayerController>()
            .init_resource::<Collisions>()
            .insert_resource(UseItemAnimation(false))
            .add_enter_system(GameState::InGame, spawn_player)
            .add_system(update_axis)
            // Update
            .add_system(
                collide
                    .run_in_state(GameState::InGame)
                    .label("collide")
                    .after("gravity")
            )
            .add_system(
                move_character
                    .run_in_state(GameState::InGame)
                    .after("collide")
            )
            //
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
            .add_system(use_item)
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

        #[cfg(feature = "debug_movement")] {
            app.add_system(
                debug_horizontal_movement
                    .run_in_state(GameState::InGame)
                    .label("horizontal_movement")
            )
            .add_system(
                debug_vertical_movement
                    .run_in_state(GameState::InGame)
                    .label("gravity")
                    .after("horizontal_movement")
            );
        }

        #[cfg(not(feature = "debug_movement"))] {
            app.add_system(
                horizontal_movement
                    .run_in_state(GameState::InGame)
                    .label("horizontal_movement")
            )
            .add_system(
                update_jump
                    .run_in_state(GameState::InGame)
                    .label("update_jump")
                    .after("horizontal_movement")
            )
            .add_system(
                gravity
                    .run_in_state(GameState::InGame)
                    .label("gravity")
                    .after("update_jump")
            );
        }
    }
}
