use bevy::{ecs::component::Component, prelude::*};
use super::{Animator, AnimatorState};

use super::tweenable::TweenCompleted;

#[derive(Debug, Clone, Copy)]
pub struct TweeningPlugin;

impl Plugin for TweeningPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TweenCompleted>().add_system(
            component_animator_system::<Transform>.label(AnimationSystem::AnimationUpdate),
        );
    }
}

/// Label enum for the systems relating to animations
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemLabel)]
pub enum AnimationSystem {
    /// Ticks animations
    AnimationUpdate,
}

/// Animator system for components.
///
/// This system extracts all components of type `T` with an `Animator<T>`
/// attached to the same entity, and tick the animator to animate the component.
pub fn component_animator_system<T: Component>(
    time: Res<Time>,
    mut query: Query<(Entity, &mut T, &mut Animator<T>)>,
    mut event_writer: EventWriter<TweenCompleted>,
) {
    for (entity, ref mut target, ref mut animator) in query.iter_mut() {
        if animator.state != AnimatorState::Paused {
            animator
                .tweenable_mut()
                .tick(time.delta(), target, entity, &mut event_writer);
        }
    }
}