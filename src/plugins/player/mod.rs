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
use super::{world::TILE_SIZE, inventory::PlayerInventoryPlugin};

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
            .init_resource::<PlayerRect>()
            .init_resource::<PlayerController>()
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
    }
}
