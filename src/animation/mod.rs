pub mod lens;
pub mod plugin;
pub mod tweenable;

use std::time::Duration;

use bevy::prelude::{default, Component};
pub use interpolation::{EaseFunction, Ease};

use self::tweenable::{BoxedTweenable, Tweenable};


/// How many times to repeat a tween animation. See also: [`RepeatStrategy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatCount {
    /// Run the animation N times.
    Finite(u32),
    /// Run the animation for some amount of time.
    For(Duration),
    /// Loop the animation indefinitely.
    Infinite,
}

/// What to do when a tween animation needs to be repeated.
///
/// Only applicable when [`RepeatCount`] is greater than the animation duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatStrategy {
    /// Reset the animation back to its starting position.
    Repeat,
    /// Follow a ping-pong pattern, changing the direction each time an endpoint
    /// is reached.
    ///
    /// A complete cycle start -> end -> start always counts as 2 loop
    /// iterations for the various operations where looping matters. That
    /// is, a 1 second animation will take 2 seconds to end up back where it
    /// started.
    MirroredRepeat,
}

impl Default for RepeatCount {
    fn default() -> Self {
        Self::Finite(1)
    }
}

impl Default for RepeatStrategy {
    fn default() -> Self {
        Self::Repeat
    }
}


/// Type of looping for a tween animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweeningType {
    /// Run the animation once from start to end only.
    Once,
    /// Loop the animation indefinitely, restarting from the start each time the
    /// end is reached.
    Loop,
    /// Loop the animation back and forth, changing direction each time an
    /// endpoint is reached. A complete cycle start -> end -> start always
    /// counts as 2 loop iterations for the various operations where looping
    /// matters.
    PingPong,
}

impl Default for TweeningType {
    fn default() -> Self {
        Self::Once
    }
}

/// Playback state of an animator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatorState {
    /// The animation is playing. This is the default state.
    Playing,
    /// The animation is paused in its current state.
    Paused,
}

impl Default for AnimatorState {
    fn default() -> Self {
        Self::Playing
    }
}

impl std::ops::Not for AnimatorState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Paused => Self::Playing,
            Self::Playing => Self::Paused,
        }
    }
}

/// Describe how eased value should be computed.
#[derive(Clone, Copy)]
pub enum EaseMethod {
    /// Follow `EaseFunction`.
    EaseFunction(EaseFunction),
    /// Linear interpolation, with no function.
    Linear,
    /// Discrete interpolation, eased value will jump from start to end when
    /// stepping over the discrete limit.
    Discrete(f32),
    /// Use a custom function to interpolate the value.
    CustomFunction(fn(f32) -> f32),
}

impl EaseMethod {
    #[must_use]
    fn sample(self, x: f32) -> f32 {
        match self {
            Self::EaseFunction(function) => x.calc(function),
            Self::Linear => x,
            Self::Discrete(limit) => {
                if x > limit {
                    1.
                } else {
                    0.
                }
            }
            Self::CustomFunction(function) => function(x),
        }
    }
}

impl Default for EaseMethod {
    fn default() -> Self {
        Self::Linear
    }
}

impl From<EaseFunction> for EaseMethod {
    fn from(ease_function: EaseFunction) -> Self {
        Self::EaseFunction(ease_function)
    }
}

/// Direction a tweening animation is playing.
///
/// When playing a tweenable forward, the progress values `0` and `1` are
/// respectively mapped to the start and end bounds of the lens(es) being used.
/// Conversely, when playing backward, this mapping is reversed, such that a
/// progress value of `0` corresponds to the state of the target at the end
/// bound of the lens, while a progress value of `1` corresponds to the state of
/// that target at the start bound of the lens, effectively making the animation
/// play backward.
///
/// For all but [`RepeatStrategy::MirroredRepeat`] this is always
/// [`TweeningDirection::Forward`], unless manually configured with
/// [`Tween::set_direction()`] in which case the value is constant equal to the
/// value set. When using [`RepeatStrategy::MirroredRepeat`], this is either
/// forward (from start to end; ping) or backward (from end to start; pong),
/// depending on the current iteration of the loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweeningDirection {
    /// Animation playing from start to end.
    Forward,
    /// Animation playing from end to start, in reverse.
    Backward,
}

impl TweeningDirection {
    /// Is the direction equal to [`TweeningDirection::Forward`]?
    #[must_use]
    pub fn is_forward(&self) -> bool {
        *self == Self::Forward
    }

    /// Is the direction equal to [`TweeningDirection::Backward`]?
    #[must_use]
    pub fn is_backward(&self) -> bool {
        *self == Self::Backward
    }
}

impl Default for TweeningDirection {
    fn default() -> Self {
        Self::Forward
    }
}

impl std::ops::Not for TweeningDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

macro_rules! animator_impl {
    () => {
        /// Set the initial playback state of the animator.
        #[must_use]
        pub fn with_state(mut self, state: AnimatorState) -> Self {
            self.state = state;
            self
        }

        /// Set the initial speed of the animator. See [`Animator::set_speed`] for
        /// details.
        #[must_use]
        pub fn with_speed(mut self, speed: f32) -> Self {
            self.speed = speed;
            self
        }

        /// Set the animation speed. Defaults to 1.
        ///
        /// A speed of 2 means the animation will run twice as fast while a speed of 0.1
        /// will result in a 10x slowed animation.
        pub fn set_speed(&mut self, speed: f32) {
            self.speed = speed;
        }

        /// Set the top-level tweenable item this animator controls.
        pub fn set_tweenable(&mut self, tween: impl Tweenable<T> + Send + Sync + 'static) {
            self.tweenable = Box::new(tween);
        }

        /// Get the top-level tweenable this animator is currently controlling.
        #[must_use]
        pub fn tweenable(&self) -> &(dyn Tweenable<T> + Send + Sync + 'static) {
            self.tweenable.as_ref()
        }

        /// Get the top-level mutable tweenable this animator is currently controlling.
        #[must_use]
        pub fn tweenable_mut(&mut self) -> &mut (dyn Tweenable<T> + Send + Sync + 'static) {
            self.tweenable.as_mut()
        }

        /// Stop animation playback and rewind the animation.
        ///
        /// This changes the animator state to [`AnimatorState::Paused`] and rewind its
        /// tweenable.
        pub fn stop(&mut self) {
            self.state = AnimatorState::Paused;
            self.tweenable_mut().rewind();
        }

        pub fn start(&mut self) {
            self.state = AnimatorState::Playing;
        }

        pub fn completed(&self) -> bool {
            self.tweenable().completed()
        }
    };
}

/// Component to control the animation of another component.
#[derive(Component)]
pub struct Animator<T: Component> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
    tweenable: BoxedTweenable<T>,
    speed: f32,
}

impl<T: Component + std::fmt::Debug> std::fmt::Debug for Animator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Animator")
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Component> Animator<T> {
    /// Create a new animator component from a single tweenable.
    #[must_use]
    pub fn new(tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        Self {
            state: default(),
            tweenable: Box::new(tween),
            speed: 1.,
        }
    }

    animator_impl!();
}