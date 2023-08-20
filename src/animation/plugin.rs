use bevy::prelude::*;

use crate::animation::{tweenable::ComponentTarget, Animator, AnimatorState, TweenCompleted};

/// Plugin to add systems related to tweening of common components and assets.
///
/// This plugin adds systems for a predefined set of components and assets, to
/// allow their respective animators to be updated each frame:
/// - [`Transform`]
/// - [`Text`]
/// - [`Style`]
/// - [`Sprite`]
/// - [`ColorMaterial`]
///
/// This ensures that all predefined lenses work as intended, as well as any
/// custom lens animating the same component or asset type.
///
/// For other components and assets, including custom ones, the relevant system
/// needs to be added manually by the application:
/// - For components, add [`component_animator_system::<T>`] where `T:
///   Component`
/// - For assets, add [`asset_animator_system::<T>`] where `T: Asset`
///
/// This plugin is entirely optional. If you want more control, you can instead
/// add manually the relevant systems for the exact set of components and assets
/// actually animated.
///
/// [`Transform`]: https://docs.rs/bevy/0.9.0/bevy/transform/components/struct.Transform.html
/// [`Text`]: https://docs.rs/bevy/0.9.0/bevy/text/struct.Text.html
/// [`Style`]: https://docs.rs/bevy/0.9.0/bevy/ui/struct.Style.html
/// [`Sprite`]: https://docs.rs/bevy/0.9.0/bevy/sprite/struct.Sprite.html
/// [`ColorMaterial`]: https://docs.rs/bevy/0.9.0/bevy/sprite/struct.ColorMaterial.html
#[derive(Debug, Clone, Copy)]
pub(crate) struct TweeningPlugin;
impl Plugin for TweeningPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TweenCompleted>();
        
        app.add_systems(
            Update,
            (
                component_animator_system::<Transform>,
                component_animator_system::<Style>,
                component_animator_system::<Text>
            )
            .in_set(AnimationSystemSet::AnimationUpdate)
        );
    }
}

/// Label enum for the systems relating to animations
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemSet)]
pub(crate) enum AnimationSystemSet {
    /// Ticks animations
    AnimationUpdate,
}

/// Animator system for components.
///
/// This system extracts all components of type `T` with an `Animator<T>`
/// attached to the same entity, and tick the animator to animate the component.
pub(crate) fn component_animator_system<T: Component>(
    time: Res<Time>,
    mut query: Query<(Entity, &mut T, &mut Animator<T>)>,
    events: ResMut<Events<TweenCompleted>>,
) {
    let mut events: Mut<Events<TweenCompleted>> = events.into();
    for (entity, target, mut animator) in query.iter_mut() {
        if animator.state != AnimatorState::Paused {
            let speed = animator.speed();
            let mut target = ComponentTarget::new(target);
            animator.tweenable_mut().tick(
                time.delta().mul_f32(speed),
                &mut target,
                entity,
                &mut events,
            );
        }
    }
}