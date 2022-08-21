use std::time::Duration;

use bevy::prelude::*;

use super::{EaseMethod, lens::Lens, TweeningDirection, TweeningType};

/// The dynamic tweenable type.
///
/// When creating lists of tweenables, you will need to box them to create a
/// homogeneous array like so:
/// ```no_run
/// # use bevy::prelude::Transform;
/// # use bevy_tweening::{BoxedTweenable, Delay, Sequence, Tween};
/// #
/// # let delay: Delay = unimplemented!();
/// # let tween: Tween<Transform> = unimplemented!();
///
/// Sequence::new([Box::new(delay) as BoxedTweenable<Transform>, tween.into()]);
/// ```
///
/// When using your own [`Tweenable`] types, APIs will be easier to use if you
/// implement [`From`]:
/// ```no_run
/// # use std::time::Duration;
/// # use bevy::prelude::{Entity, EventWriter, Transform};
/// # use bevy_tweening::{BoxedTweenable, Sequence, Tweenable, TweenCompleted, TweenState};
/// #
/// # struct MyTweenable;
/// # impl Tweenable<Transform> for MyTweenable {
/// #     fn duration(&self) -> Duration  { unimplemented!() }
/// #     fn is_looping(&self) -> bool  { unimplemented!() }
/// #     fn set_progress(&mut self, progress: f32)  { unimplemented!() }
/// #     fn progress(&self) -> f32  { unimplemented!() }
/// #     fn tick(&mut self, delta: Duration, target: &mut Transform, entity: Entity, event_writer: &mut EventWriter<TweenCompleted>) -> TweenState  { unimplemented!() }
/// #     fn times_completed(&self) -> u32  { unimplemented!() }
/// #     fn rewind(&mut self) { unimplemented!() }
/// # }
///
/// Sequence::new([Box::new(MyTweenable) as BoxedTweenable<_>]);
///
/// // OR
///
/// Sequence::new([MyTweenable]);
///
/// impl From<MyTweenable> for BoxedTweenable<Transform> {
///     fn from(t: MyTweenable) -> Self {
///         Box::new(t)
///     }
/// }
/// ```
pub type BoxedTweenable<T> = Box<dyn Tweenable<T> + Send + Sync + 'static>;

/// Playback state of a [`Tweenable`].
///
/// This is returned by [`Tweenable::tick()`] to allow the caller to execute
/// some logic based on the updated state of the tweenable, like advanding a
/// sequence to its next child tweenable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweenState {
    /// The tweenable is still active, and did not reach its end state yet.
    Active,
    /// Animation reached its end state. The tweenable is idling at its latest
    /// time. This can only happen for [`TweeningType::Once`], since other
    /// types loop indefinitely.
    Completed,
}

/// Event raised when a tween completed.
///
/// This event is raised when a tween completed. For non-looping tweens, this is
/// raised once at the end of the animation. For looping animations, this is
/// raised once per iteration. In case the animation direction changes
/// ([`TweeningType::PingPong`]), an iteration corresponds to a single progress
/// from one endpoint to the other, whatever the direction. Therefore a complete
/// cycle start -> end -> start counts as 2 iterations and raises 2 events (one
/// when reaching the end, one when reaching back the start).
///
/// # Note
///
/// The semantic is slightly different from [`TweenState::Completed`], which
/// indicates that the tweenable has finished ticking and do not need to be
/// updated anymore, a state which is never reached for looping animation. Here
/// the [`TweenCompleted`] event instead marks the end of a single loop
/// iteration.
#[derive(Copy, Clone)]
pub struct TweenCompleted {
    /// The [`Entity`] the tween which completed and its animator are attached
    /// to.
    pub entity: Entity,
    /// An opaque value set by the user when activating event raising, used to
    /// identify the particular tween which raised this event. The value is
    /// passed unmodified from a call to [`with_completed_event()`]
    /// or [`set_completed_event()`].
    ///
    /// [`with_completed_event()`]: Tween::with_completed_event
    /// [`set_completed_event()`]: Tween::set_completed_event
    pub user_data: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AnimClock {
    pub elapsed: Duration,
    pub duration: Duration,
    pub is_looping: bool,
}

impl AnimClock {
    fn new(duration: Duration, is_looping: bool) -> Self {
        Self {
            elapsed: Duration::ZERO,
            duration,
            is_looping,
        }
    }

    fn tick(&mut self, duration: Duration) -> u32 {
        self.elapsed = self.elapsed.saturating_add(duration);

        if self.elapsed < self.duration {
            0
        } else if self.is_looping {
            let elapsed = self.elapsed.as_nanos();
            let duration = self.duration.as_nanos();

            self.elapsed = Duration::from_nanos((elapsed % duration) as u64);
            (elapsed / duration) as u32
        } else {
            self.elapsed = self.duration;
            1
        }
    }

    fn set_progress(&mut self, progress: f32) {
        let progress = if self.is_looping {
            progress.max(0.).fract()
        } else {
            progress.clamp(0., 1.)
        };

        self.elapsed = self.duration.mul_f32(progress);
    }

    fn progress(&self) -> f32 {
        self.elapsed.as_secs_f32() / self.duration.as_secs_f32()
    }

    fn completed(&self) -> bool {
        self.elapsed >= self.duration
    }

    fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
    }
}

/// An animatable entity, either a single [`Tween`] or a collection of them.
pub trait Tweenable<T>: Send + Sync {
    /// Get the total duration of the animation.
    ///
    /// For non-looping tweenables ([`TweeningType::Once`]), this is the total
    /// animation duration. For looping ones, this is the duration of a
    /// single iteration, since the total animation duration is infinite.
    ///
    /// Note that for [`TweeningType::PingPong`], this is the duration of a
    /// single way, either from start to end or back from end to start. The
    /// total "loop" duration start -> end -> start to reach back the same
    /// state in this case is the double of the returned value.
    fn duration(&self) -> Duration;

    /// Return `true` if the animation is looping.
    ///
    /// Looping tweenables are of type [`TweeningType::Loop`] or
    /// [`TweeningType::PingPong`].
    fn is_looping(&self) -> bool;

    /// Set the current animation playback progress.
    ///
    /// See [`progress()`] for details on the meaning.
    ///
    /// [`progress()`]: Tweenable::progress
    fn set_progress(&mut self, progress: f32);

    fn set_direction(&mut self, direction: TweeningDirection);

    /// Get the current progress in \[0:1\] (non-looping) or \[0:1\[ (looping)
    /// of the animation.
    ///
    /// For looping animations, this reports the progress of the current
    /// iteration, in the current direction:
    /// - [`TweeningType::Loop`] is `0` at start and `1` at end. The exact value
    ///   `1.0` is never reached, since the tweenable loops over to `0.0`
    ///   immediately.
    /// - [`TweeningType::PingPong`] is `0` at the source endpoint and `1` and
    ///   the destination one, which are respectively the start/end for
    ///   [`TweeningDirection::Forward`], or the end/start for
    ///   [`TweeningDirection::Backward`]. The exact value `1.0` is never
    ///   reached, since the tweenable loops over to `0.0` immediately when it
    ///   changes direction at either endpoint.
    fn progress(&self) -> f32;

    /// Tick the animation, advancing it by the given delta time and mutating
    /// the given target component or asset.
    ///
    /// This returns [`TweenState::Active`] if the tweenable didn't reach its
    /// final state yet (progress < `1.0`), or [`TweenState::Completed`] if
    /// the tweenable completed this tick. Only non-looping tweenables return
    /// a completed state, since looping ones continue forever.
    ///
    /// Calling this method with a duration of [`Duration::ZERO`] is valid, and
    /// updates the target to the current state of the tweenable without
    /// actually modifying the tweenable state. This is useful after certain
    /// operations like [`rewind()`] or [`set_progress()`] whose effect is
    /// otherwise only visible on target on next frame.
    ///
    /// [`rewind()`]: Tweenable::rewind
    /// [`set_progress()`]: Tweenable::set_progress
    fn tick(
        &mut self,
        delta: Duration,
        target: &mut T,
        entity: Entity,
        event_writer: &mut EventWriter<TweenCompleted>,
    ) -> TweenState;

    /// Get the number of times this tweenable completed.
    ///
    /// For looping animations, this returns the number of times a single
    /// playback was completed. In the case of [`TweeningType::PingPong`]
    /// this corresponds to a playback in a single direction, so tweening
    /// from start to end and back to start counts as two completed times (one
    /// forward, one backward).
    fn times_completed(&self) -> u32;

    /// Rewind the animation to its starting state.
    ///
    /// Note that the starting state depends on the current direction. For
    /// [`TweeningDirection::Forward`] this is the start point of the lens,
    /// whereas for [`TweeningDirection::Backward`] this is the end one.
    fn rewind(&mut self);

    fn completed(&self) -> bool;

    fn clock_mut(&mut self) -> &mut AnimClock;
}

/// Type of a callback invoked when a [`Tween`] has completed.
///
/// See [`Tween::set_completed()`] for usage.
pub type CompletedCallback<T> = dyn Fn(Entity, &Tween<T>) + Send + Sync + 'static;

/// Single tweening animation instance.
pub struct Tween<T> {
    ease_function: EaseMethod,
    pub clock: AnimClock,
    times_completed: u32,
    tweening_type: TweeningType,
    direction: TweeningDirection,
    lens: Box<dyn Lens<T> + Send + Sync + 'static>,
    on_completed: Option<Box<CompletedCallback<T>>>,
    event_data: Option<u64>,
}

impl<T> Tween<T> {
    /// Create a new tween animation.
    ///
    /// # Example
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::math::Vec3;
    /// # use std::time::Duration;
    /// let tween = Tween::new(
    ///     EaseFunction::QuadraticInOut,
    ///     TweeningType::Once,
    ///     Duration::from_secs_f32(1.0),
    ///     TransformPositionLens {
    ///         start: Vec3::ZERO,
    ///         end: Vec3::new(3.5, 0., 0.),
    ///     },
    /// );
    /// ```
    #[must_use]
    pub fn new<L>(
        ease_function: impl Into<EaseMethod>,
        tweening_type: TweeningType,
        duration: Duration,
        lens: L,
    ) -> Self
    where
        L: Lens<T> + Send + Sync + 'static,
    {
        Self {
            ease_function: ease_function.into(),
            clock: AnimClock::new(duration, tweening_type != TweeningType::Once),
            times_completed: 0,
            tweening_type,
            direction: TweeningDirection::Forward,
            lens: Box::new(lens),
            on_completed: None,
            event_data: None,
        }
    }

    /// Enable or disable raising a completed event.
    ///
    /// If enabled, the tween will raise a [`TweenCompleted`] event when the
    /// animation completed. This is similar to the [`set_completed()`]
    /// callback, but uses Bevy events instead.
    ///
    /// # Example
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::{ecs::event::EventReader, math::Vec3};
    /// # use std::time::Duration;
    /// let tween = Tween::new(
    ///     // [...]
    /// #    EaseFunction::QuadraticInOut,
    /// #    TweeningType::Once,
    /// #    Duration::from_secs_f32(1.0),
    /// #    TransformPositionLens {
    /// #        start: Vec3::ZERO,
    /// #        end: Vec3::new(3.5, 0., 0.),
    /// #    },
    /// )
    /// .with_completed_event(42);
    ///
    /// fn my_system(mut reader: EventReader<TweenCompleted>) {
    ///   for ev in reader.iter() {
    ///     assert_eq!(ev.user_data, 42);
    ///     println!("Entity {:?} raised TweenCompleted!", ev.entity);
    ///   }
    /// }
    /// ```
    ///
    /// [`set_completed()`]: Tween::set_completed
    #[must_use]
    pub fn with_completed_event(mut self, user_data: u64) -> Self {
        self.event_data = Some(user_data);
        self
    }

    /// Set the playback direction of the tween.
    ///
    /// The playback direction influences the mapping of the progress ratio (in
    /// \[0:1\]) to the actual ratio passed to the lens.
    /// [`TweeningDirection::Forward`] maps the `0` value of progress to the
    /// `0` value of the lens ratio. Conversely, [`TweeningDirection::Backward`]
    /// reverses the mapping, which effectively makes the tween play reversed,
    /// going from end to start.
    ///
    /// Changing the direction doesn't change any target state, nor any progress
    /// of the tween. Only the direction of animation from this moment
    /// potentially changes. To force a target state change, call
    /// [`Tweenable::tick()`] with a zero delta (`Duration::ZERO`).
    pub fn set_direction(&mut self, direction: TweeningDirection) {
        self.direction = direction;
    }

    /// Set the playback direction of the tween.
    ///
    /// See [`Tween::set_direction()`].
    #[must_use]
    pub fn with_direction(mut self, direction: TweeningDirection) -> Self {
        self.direction = direction;
        self
    }

    /// The current animation direction.
    ///
    /// See [`TweeningDirection`] for details.
    #[must_use]
    pub fn direction(&self) -> TweeningDirection {
        self.direction
    }

    /// Set a callback invoked when the animation completes.
    ///
    /// The callback when invoked receives as parameters the [`Entity`] on which
    /// the target and the animator are, as well as a reference to the
    /// current [`Tween`].
    ///
    /// Only non-looping tweenables can complete.
    pub fn set_completed<C>(&mut self, callback: C)
    where
        C: Fn(Entity, &Self) + Send + Sync + 'static,
    {
        self.on_completed = Some(Box::new(callback));
    }

    /// Clear the callback invoked when the animation completes.
    pub fn clear_completed(&mut self) {
        self.on_completed = None;
    }

    /// Enable or disable raising a completed event.
    ///
    /// If enabled, the tween will raise a [`TweenCompleted`] event when the
    /// animation completed. This is similar to the [`set_completed()`]
    /// callback, but uses Bevy events instead.
    ///
    /// See [`with_completed_event()`] for details.
    ///
    /// [`set_completed()`]: Tween::set_completed
    /// [`with_completed_event()`]: Tween::with_completed_event
    pub fn set_completed_event(&mut self, user_data: u64) {
        self.event_data = Some(user_data);
    }

    /// Clear the event sent when the animation completes.
    pub fn clear_completed_event(&mut self) {
        self.event_data = None;
    }
}

impl<T> Tweenable<T> for Tween<T> {
    fn duration(&self) -> Duration {
        self.clock.duration
    }

    fn is_looping(&self) -> bool {
        self.tweening_type != TweeningType::Once
    }

    fn set_progress(&mut self, progress: f32) {
        self.clock.set_progress(progress);
    }

    fn set_direction(&mut self, direction: TweeningDirection) {
        self.direction = direction;
    }

    fn progress(&self) -> f32 {
        self.clock.progress()
    }

    fn tick(
        &mut self,
        delta: Duration,
        target: &mut T,
        entity: Entity,
        event_writer: &mut EventWriter<TweenCompleted>,
    ) -> TweenState {
        if !self.is_looping() && self.clock.completed() {
            return TweenState::Completed;
        }

        // Tick the animation clock
        let times_completed = self.clock.tick(delta);
        self.times_completed += times_completed;
        if times_completed & 1 != 0 && self.tweening_type == TweeningType::PingPong {
            self.direction = !self.direction;
        }
        let state = if self.is_looping() || times_completed == 0 {
            TweenState::Active
        } else {
            TweenState::Completed
        };
        let progress = self.clock.progress();

        // Apply the lens, even if the animation finished, to ensure the state is
        // consistent
        let mut factor = progress;
        if self.direction.is_backward() {
            factor = 1. - factor;
        }
        let factor = self.ease_function.sample(factor);
        self.lens.lerp(target, factor);

        // If completed at least once this frame, notify the user
        if times_completed > 0 {
            if let Some(user_data) = &self.event_data {
                event_writer.send(TweenCompleted {
                    entity,
                    user_data: *user_data,
                });
            }
            if let Some(cb) = &self.on_completed {
                cb(entity, self);
            }
        }

        state
    }

    fn times_completed(&self) -> u32 {
        self.times_completed
    }

    fn rewind(&mut self) {
        self.clock.reset();
        self.times_completed = 0;
    }

    fn completed(&self) -> bool {
        self.clock.completed()
    }

    fn clock_mut(&mut self) -> &mut AnimClock {
        &mut self.clock
    }
}
