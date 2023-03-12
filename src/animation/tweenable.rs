use std::{ops::DerefMut, time::Duration, any::Any};

use bevy::prelude::*;

use crate::animation::{EaseMethod, Lens, RepeatCount, RepeatStrategy, TweeningDirection};

/// The dynamic tweenable type.
///
/// When creating lists of tweenables, you will need to box them to create a
/// homogeneous array like so:
/// ```no_run
/// # use bevy::prelude::Transform;
/// # use bevy_tweening::{BoxedTweenable, Delay, Sequence, Tween};
/// #
/// # let delay: Delay<Transform> = unimplemented!();
/// # let tween: Tween<Transform> = unimplemented!();
///
/// Sequence::new([Box::new(delay) as BoxedTweenable<Transform>, tween.into()]);
/// ```
///
/// When using your own [`Tweenable`] types, APIs will be easier to use if you
/// implement [`From`]:
/// ```no_run
/// # use std::time::Duration;
/// # use bevy::prelude::{Entity, Events, Mut, Transform};
/// # use bevy_tweening::{BoxedTweenable, Sequence, Tweenable, TweenCompleted, TweenState, Targetable, TotalDuration};
/// #
/// # struct MyTweenable;
/// # impl Tweenable<Transform> for MyTweenable {
/// #     fn duration(&self) -> Duration  { unimplemented!() }
/// #     fn total_duration(&self) -> TotalDuration  { unimplemented!() }
/// #     fn set_elapsed(&mut self, elapsed: Duration)  { unimplemented!() }
/// #     fn elapsed(&self) -> Duration  { unimplemented!() }
/// #     fn tick<'a>(&mut self, delta: Duration, target: &'a mut dyn Targetable<Transform>, entity: Entity, events: &mut Mut<Events<TweenCompleted>>) -> TweenState  { unimplemented!() }
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
pub type BoxedTweenable<T> = Box<dyn Tweenable<T> + 'static>;

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
    /// time.
    ///
    /// Note that [`RepeatCount::Infinite`] tweenables never reach this state.
    Completed,
}

/// Event raised when a tween completed.
///
/// This event is raised when a tween completed. When looping, this is raised
/// once per iteration. In case the animation direction changes
/// ([`RepeatStrategy::MirroredRepeat`]), an iteration corresponds to a single
/// progress from one endpoint to the other, whatever the direction. Therefore a
/// complete cycle start -> end -> start counts as 2 iterations and raises 2
/// events (one when reaching the end, one when reaching back the start).
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

/// Calculate the progress fraction in \[0:1\] of the ratio between two
/// [`Duration`]s.
fn fraction_progress(n: Duration, d: Duration) -> f32 {
    // TODO - Replace with div_duration_f32() once it's stable
    (n.as_secs_f64() / d.as_secs_f64()).fract() as f32
}

#[derive(Debug)]
struct AnimClock {
    elapsed: Duration,
    duration: Duration,
    total_duration: TotalDuration,
    strategy: RepeatStrategy,
}

impl AnimClock {
    fn new(duration: Duration, strategy: RepeatStrategy) -> Self {
        Self {
            elapsed: Duration::ZERO,
            duration,
            total_duration: compute_total_duration(duration, RepeatCount::default()),
            strategy
        }
    }

    fn tick(&mut self, tick: Duration) -> (TweenState, i32) {
        self.set_elapsed(self.elapsed.saturating_add(tick))
    }

    fn times_completed(&self) -> u32 {
        (self.elapsed.as_nanos() / self.duration.as_nanos()) as u32
    }

    fn set_elapsed(&mut self, elapsed: Duration) -> (TweenState, i32) {
        let old_times_completed = self.times_completed();

        self.elapsed = elapsed;

        let state = match self.total_duration {
            TotalDuration::Finite(total_duration) => {
                if self.elapsed >= total_duration {
                    self.elapsed = total_duration;
                    TweenState::Completed
                } else {
                    TweenState::Active
                }
            }
            TotalDuration::Infinite => TweenState::Active,
        };

        (
            state,
            self.times_completed() as i32 - old_times_completed as i32,
        )
    }

    fn elapsed(&self) -> Duration {
        self.elapsed
    }

    fn state(&self) -> TweenState {
        match self.total_duration {
            TotalDuration::Finite(total_duration) => {
                if self.elapsed >= total_duration {
                    TweenState::Completed
                } else {
                    TweenState::Active
                }
            }
            TotalDuration::Infinite => TweenState::Active,
        }
    }

    fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
    }
}

/// Possibly infinite duration of an animation.
///
/// Used to measure the total duration of an animation including any looping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TotalDuration {
    /// The duration is finite, of the given value.
    Finite(Duration),
    /// The duration is infinite.
    Infinite,
}

fn compute_total_duration(duration: Duration, count: RepeatCount) -> TotalDuration {
    match count {
        RepeatCount::Finite(times) => TotalDuration::Finite(duration.saturating_mul(times)),
        RepeatCount::For(duration) => TotalDuration::Finite(duration),
        RepeatCount::Infinite => TotalDuration::Infinite,
    }
}

// TODO - Targetable et al. should be replaced with Mut->Mut from Bevy 0.9
// https://github.com/bevyengine/bevy/pull/6199

/// Trait to workaround the discrepancies of the change detection mechanisms of
/// assets and components.
pub trait Targetable<T> {
    /// Dereference the target, triggering any change detection, and return a
    /// mutable reference.
    fn target_mut(&mut self) -> &mut T;
}

pub struct ComponentTarget<'a, T: Component> {
    target: Mut<'a, T>,
}

impl<'a, T: Component> ComponentTarget<'a, T> {
    pub fn new(target: Mut<'a, T>) -> Self {
        Self { target }
    }
}

impl<'a, T: Component> Targetable<T> for ComponentTarget<'a, T> {
    fn target_mut(&mut self) -> &mut T {
        self.target.deref_mut()
    }
}


/// An animatable entity, either a single [`Tween`] or a collection of them.
pub trait Tweenable<T>: Send + Sync {
    /// Get the duration of a single iteration of the animation.
    ///
    /// Note that for [`RepeatStrategy::MirroredRepeat`], this is the duration
    /// of a single way, either from start to end or back from end to start.
    /// The total "loop" duration start -> end -> start to reach back the
    /// same state in this case is the double of the returned value.
    fn duration(&self) -> Duration;

    /// Get the total duration of the entire animation, including looping.
    ///
    /// For [`TotalDuration::Finite`], this is the number of repeats times the
    /// duration of a single iteration ([`duration()`]).
    ///
    /// [`duration()`]: Tweenable::duration
    fn total_duration(&self) -> TotalDuration;

    /// Set the current animation playback elapsed time.
    ///
    /// See [`elapsed()`] for details on the meaning. If `elapsed` is greater
    /// than or equal to [`duration()`], then the animation completes.
    ///
    /// Setting the elapsed time seeks the animation to a new position, but does
    /// not apply that change to the underlying component being animated. To
    /// force the change to apply, call [`tick()`] with a `delta` of
    /// `Duration::ZERO`.
    ///
    /// [`elapsed()`]: Tweenable::elapsed
    /// [`duration()`]: Tweenable::duration
    /// [`tick()`]: Tweenable::tick
    fn set_elapsed(&mut self, elapsed: Duration);

    /// Get the current elapsed duration.
    ///
    /// While looping, the exact value returned by [`duration()`] is never
    /// reached, since the tweenable loops over to zero immediately when it
    /// changes direction at either endpoint. Upon completion, the tweenable
    /// always reports the same value as [`duration()`].
    ///
    /// [`duration()`]: Tweenable::duration
    fn elapsed(&self) -> Duration;

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
        target: &mut dyn Targetable<T>,
        entity: Entity,
        events: &mut Mut<Events<TweenCompleted>>,
    ) -> TweenState;

    /// Rewind the animation to its starting state.
    ///
    /// Note that the starting state depends on the current direction. For
    /// [`TweeningDirection::Forward`] this is the start point of the lens,
    /// whereas for [`TweeningDirection::Backward`] this is the end one.
    fn rewind(&mut self);

    /// Set the current animation playback progress.
    ///
    /// See [`progress()`] for details on the meaning.
    ///
    /// Setting the progress seeks the animation to a new position, but does not
    /// apply that change to the underlying component being animated. To
    /// force the change to apply, call [`tick()`] with a `delta` of
    /// `Duration::ZERO`.
    ///
    /// [`progress()`]: Tweenable::progress
    /// [`tick()`]: Tweenable::tick
    fn set_progress(&mut self, progress: f32) {
        self.set_elapsed(self.duration().mul_f32(progress.max(0.)));
    }

    /// Get the current progress in \[0:1\] of the animation.
    ///
    /// While looping, the exact value `1.0` is never reached, since the
    /// tweenable loops over to `0.0` immediately when it changes direction at
    /// either endpoint. Upon completion, the tweenable always reports exactly
    /// `1.0`.
    fn progress(&self) -> f32 {
        let elapsed = self.elapsed();
        if let TotalDuration::Finite(total_duration) = self.total_duration() {
            if elapsed >= total_duration {
                return 1.;
            }
        }
        fraction_progress(elapsed, self.duration())
    }

    /// Get the number of times this tweenable completed.
    ///
    /// For looping animations, this returns the number of times a single
    /// playback was completed. In the case of
    /// [`RepeatStrategy::MirroredRepeat`] this corresponds to a playback in
    /// a single direction, so tweening from start to end and back to start
    /// counts as two completed times (one forward, one backward).
    fn times_completed(&self) -> u32 {
        (self.elapsed().as_nanos() / self.duration().as_nanos()) as u32
    }

    ///
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

macro_rules! impl_boxed {
    ($tweenable:ty) => {
        impl<T: 'static> From<$tweenable> for BoxedTweenable<T> {
            fn from(t: $tweenable) -> Self {
                Box::new(t)
            }
        }
    };
}

impl_boxed!(Tween<T>);
impl_boxed!(Tracks<T>);

/// Type of a callback invoked when a [`Tween`] or [`Delay`] has completed.
///
/// See [`Tween::set_completed()`] or [`Delay::set_completed()`] for usage.
pub type CompletedCallback<T> = dyn Fn(Entity, &T) + Send + Sync + 'static;

/// Single tweening animation instance.
pub struct Tween<T> {
    ease_function: EaseMethod,
    clock: AnimClock,
    direction: TweeningDirection,
    lens: Box<dyn Lens<T> + Send + Sync + 'static>,
    on_completed: Option<Box<CompletedCallback<Tween<T>>>>,
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
    ///     Duration::from_secs(1),
    ///     TransformPositionLens {
    ///         start: Vec3::ZERO,
    ///         end: Vec3::new(3.5, 0., 0.),
    ///     },
    /// );
    /// ```
    #[must_use]
    pub fn new<L>(ease_function: impl Into<EaseMethod>, strategy: RepeatStrategy, duration: Duration, lens: L) -> Self
    where
        L: Lens<T> + Send + Sync + 'static,
    {
        Self {
            ease_function: ease_function.into(),
            clock: AnimClock::new(duration, strategy),
            direction: TweeningDirection::Forward,
            lens: Box::new(lens),
            on_completed: None,
            event_data: None,
        }
    }

    /// Enable raising a completed event.
    ///
    /// If enabled, the tween will raise a [`TweenCompleted`] event when the
    /// animation completed. This is similar to the [`with_completed()`]
    /// callback, but uses Bevy events instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::{ecs::event::EventReader, math::Vec3};
    /// # use std::time::Duration;
    /// let tween = Tween::new(
    ///     // [...]
    /// #    EaseFunction::QuadraticInOut,
    /// #    Duration::from_secs(1),
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
    /// [`with_completed()`]: Tween::with_completed
    #[must_use]
    pub fn with_completed_event(mut self, user_data: u64) -> Self {
        self.event_data = Some(user_data);
        self
    }

    /// Set a callback invoked when the delay completes.
    ///
    /// The callback when invoked receives as parameters the [`Entity`] on which
    /// the target and the animator are, as well as a reference to the
    /// current [`Tween`]. This is similar to [`with_completed_event()`], but
    /// with a callback instead.
    ///
    /// Only non-looping tweenables can complete.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::{ecs::event::EventReader, math::Vec3};
    /// # use std::time::Duration;
    /// let tween = Tween::new(
    ///     // [...]
    /// #    EaseFunction::QuadraticInOut,
    /// #    Duration::from_secs(1),
    /// #    TransformPositionLens {
    /// #        start: Vec3::ZERO,
    /// #        end: Vec3::new(3.5, 0., 0.),
    /// #    },
    /// )
    /// .with_completed(|entity, _tween| {
    ///   println!("Tween completed on entity {:?}", entity);
    /// });
    /// ```
    ///
    /// [`with_completed_event()`]: Tween::with_completed_event
    pub fn with_completed<C>(mut self, callback: C) -> Self
    where
        C: Fn(Entity, &Self) + Send + Sync + 'static,
    {
        self.on_completed = Some(Box::new(callback));
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

    /// Set the number of times to repeat the animation.
    #[must_use]
    pub fn with_repeat_count(mut self, count: impl Into<RepeatCount>) -> Self {
        self.clock.total_duration = compute_total_duration(self.clock.duration, count.into());
        self
    }

    /// Choose how the animation behaves upon a repetition.
    #[must_use]
    pub fn with_repeat_strategy(mut self, strategy: RepeatStrategy) -> Self {
        self.clock.strategy = strategy;
        self
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
    ///
    /// See also [`set_completed()`].
    ///
    /// [`set_completed()`]: Tween::set_completed
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
    ///
    /// See also [`set_completed_event()`].
    ///
    /// [`set_completed_event()`]: Tween::set_completed_event
    pub fn clear_completed_event(&mut self) {
        self.event_data = None;
    }
}

impl<T: 'static> Tweenable<T> for Tween<T> {
    fn duration(&self) -> Duration {
        self.clock.duration
    }

    fn total_duration(&self) -> TotalDuration {
        self.clock.total_duration
    }

    fn set_elapsed(&mut self, elapsed: Duration) {
        self.clock.set_elapsed(elapsed);
    }

    fn elapsed(&self) -> Duration {
        self.clock.elapsed()
    }

    fn tick(
        &mut self,
        delta: Duration,
        target: &mut dyn Targetable<T>,
        entity: Entity,
        events: &mut Mut<Events<TweenCompleted>>,
    ) -> TweenState {
        if self.clock.state() == TweenState::Completed {
            return TweenState::Completed;
        }

        // Tick the animation clock
        let (state, times_completed) = self.clock.tick(delta);
        let (progress, times_completed_for_direction) = match state {
            TweenState::Active => (self.progress(), times_completed),
            TweenState::Completed => (1., times_completed.max(1) - 1), // ignore last
        };
        if self.clock.strategy == RepeatStrategy::MirroredRepeat
            && times_completed_for_direction & 1 != 0
        {
            self.direction = !self.direction;
        }

        // Apply the lens, even if the animation finished, to ensure the state is
        // consistent
        let mut factor = progress;
        if self.direction.is_backward() {
            factor = 1. - factor;
        }
        let factor = self.ease_function.sample(factor);
        let target = target.target_mut();
        self.lens.lerp(target, factor);

        // If completed at least once this frame, notify the user
        if times_completed > 0 {
            if let Some(user_data) = &self.event_data {
                events.send(TweenCompleted {
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

    fn rewind(&mut self) {
        if self.clock.strategy == RepeatStrategy::MirroredRepeat {
            // In mirrored mode, direction alternates each loop. To reset to the original
            // direction on Tween creation, we count the number of completions, ignoring the
            // last one if the Tween is currently in TweenState::Completed because that one
            // freezes all parameters.
            let mut times_completed = self.clock.times_completed();
            if self.clock.state() == TweenState::Completed {
                debug_assert!(times_completed > 0);
                times_completed -= 1;
            }
            if times_completed & 1 != 0 {
                self.direction = !self.direction;
            }
        }
        self.clock.reset();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A collection of [`Tweenable`] executing in parallel.
pub struct Tracks<T> {
    tracks: Vec<BoxedTweenable<T>>,
    duration: Duration,
    elapsed: Duration
}

impl<T> Tracks<T> {
    /// Create a new [`Tracks`] from an iterator over a collection of
    /// [`Tweenable`].
    #[must_use]
    pub fn new(items: impl IntoIterator<Item = impl Into<BoxedTweenable<T>>>) -> Self {
        let tracks: Vec<_> = items.into_iter().map(Into::into).collect();
        let duration = tracks
            .iter()
            .map(AsRef::as_ref)
            .map(Tweenable::duration)
            .max()
            .unwrap();
        Self {
            tracks,
            duration,
            elapsed: Duration::ZERO
        }
    }

    ///
    pub fn get_track(&self, index: usize) -> Option<&BoxedTweenable<T>> {
        self.tracks.get(index)
    }

    ///
    pub fn get_track_mut(&mut self, index: usize) -> Option<&mut BoxedTweenable<T>> {
        self.tracks.get_mut(index)
    }
}

impl<T: 'static> Tweenable<T> for Tracks<T> {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn total_duration(&self) -> TotalDuration {
        TotalDuration::Finite(self.duration)
    }

    fn set_elapsed(&mut self, elapsed: Duration) {
        self.elapsed = elapsed;

        for tweenable in &mut self.tracks {
            tweenable.set_elapsed(elapsed);
        }
    }

    fn elapsed(&self) -> Duration {
        self.elapsed
    }

    fn tick(
        &mut self,
        delta: Duration,
        target: &mut dyn Targetable<T>,
        entity: Entity,
        events: &mut Mut<Events<TweenCompleted>>,
    ) -> TweenState {
        self.elapsed = self.elapsed.saturating_add(delta).min(self.duration);
        let mut any_active = false;
        for tweenable in &mut self.tracks {
            let state = tweenable.tick(delta, target, entity, events);
            any_active = any_active || (state == TweenState::Active);
        }
        if any_active {
            TweenState::Active
        } else {
            TweenState::Completed
        }
    }

    fn rewind(&mut self) {
        self.elapsed = Duration::ZERO;
        for tween in &mut self.tracks {
            tween.rewind();
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}