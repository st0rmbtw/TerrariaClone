// Taken from https://raw.githubusercontent.com/djeedai/bevy_hanabi/main/src/gradient.rs

use bevy::{
    math::{Quat, Vec2, Vec3, Vec3A, Vec4},
    utils::FloatOrd,
};
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    vec::Vec,
};

/// Describes a type that can be linearly interpolated between two keys.
///
/// This trait is used for values in a gradient, which are primitive types and
/// are therefore copyable.
pub trait Lerp: Copy {
    fn lerp(self, other: Self, ratio: f32) -> Self;
}

impl Lerp for f32 {
    #[inline]
    fn lerp(self, other: Self, ratio: f32) -> Self {
        self.mul_add(1. - ratio, other * ratio)
    }
}

impl Lerp for f64 {
    #[inline]
    fn lerp(self, other: Self, ratio: f32) -> Self {
        self.mul_add((1. - ratio) as f64, other * ratio as f64)
    }
}

macro_rules! impl_lerp_vecn {
    ($t:ty) => {
        impl Lerp for $t {
            #[inline]
            fn lerp(self, other: Self, ratio: f32) -> Self {
                // Force use of type's own lerp() to disambiguate and prevent infinite recursion
                <$t>::lerp(self, other, ratio)
            }
        }
    };
}

impl_lerp_vecn!(Vec2);
impl_lerp_vecn!(Vec3);
impl_lerp_vecn!(Vec3A);
impl_lerp_vecn!(Vec4);

impl Lerp for Quat {
    fn lerp(self, other: Self, ratio: f32) -> Self {
        // We use slerp() instead of lerp() as conceptually we want a smooth
        // interpolation and we expect Quat to be used to represent a rotation.
        // lerp() would produce an interpolation with varying speed, which feels
        // non-natural.
        self.slerp(other, ratio)
    }
}

/// A single key point for a [`Gradient`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GradientKey<T: Lerp> {
    /// Ratio in \[0:1\] where the key is located.
    ratio: f32,

    /// Value associated with the key.
    ///
    /// The value is uploaded as is to the render shader. For colors, this means
    /// the value does not imply any particular color space by itself.
    pub value: T,
}

impl<T: Lerp> GradientKey<T> {
    /// Get the ratio where the key point is located, in \[0:1\].
    pub fn ratio(&self) -> f32 {
        self.ratio
    }
}

impl Hash for GradientKey<f32> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.ratio).hash(state);
        FloatOrd(self.value).hash(state);
    }
}

impl Hash for GradientKey<Vec2> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.ratio).hash(state);
        FloatOrd(self.value.x).hash(state);
        FloatOrd(self.value.y).hash(state);
    }
}

impl Hash for GradientKey<Vec3> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.ratio).hash(state);
        FloatOrd(self.value.x).hash(state);
        FloatOrd(self.value.y).hash(state);
        FloatOrd(self.value.z).hash(state);
    }
}

impl Hash for GradientKey<Vec4> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.ratio).hash(state);
        FloatOrd(self.value.x).hash(state);
        FloatOrd(self.value.y).hash(state);
        FloatOrd(self.value.z).hash(state);
        FloatOrd(self.value.w).hash(state);
    }
}

/// A gradient curve made of keypoints and associated values.
///
/// The gradient can be sampled anywhere, and will return a linear interpolation
/// of the values of its closest keys. Sampling before 0 or after 1 returns a
/// constant value equal to the one of the closest bound.
///
/// # Construction
///
/// The most efficient constructors take the entirety of the key points upfront.
/// This prevents costly linear searches to insert key points one by one:
/// - [`constant()`] creates a gradient with a single key point;
/// - [`linear()`] creates a linear gradient between two key points;
/// - [`from_keys()`] creates a more general gradient with any number of key
///   points.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gradient<T: Lerp> {
    keys: Vec<GradientKey<T>>,
}

// SAFETY: This is consistent with the derive, but we can't derive due to trait
// bounds.
#[allow(clippy::derived_hash_with_manual_eq)]
impl<T> Hash for Gradient<T>
where
    T: Default + Lerp,
    GradientKey<T>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.keys.hash(state);
    }
}

impl<T: Lerp> Gradient<T> {
    /// Create a new empty gradient.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_hanabi::Gradient;
    /// let g: Gradient<f32> = Gradient::new();
    /// assert!(g.is_empty());
    /// ```
    pub const fn new() -> Self {
        Self { keys: vec![] }
    }

    /// Create a constant gradient.
    ///
    /// The gradient contains `value` at key 0.0 and nothing else. Any sampling
    /// evaluates to that single value.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_hanabi::Gradient;
    /// # use bevy::math::Vec2;
    /// let g = Gradient::constant(Vec2::X);
    /// assert_eq!(g.sample(0.3), Vec2::X);
    /// ```
    pub fn constant(value: T) -> Self {
        Self {
            keys: vec![GradientKey::<T> { ratio: 0., value }],
        }
    }

    /// Create a linear gradient between two values.
    ///
    /// The gradient contains the `start` value at key 0.0 and the `end` value
    /// at key 1.0.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_hanabi::Gradient;
    /// # use bevy::math::Vec3;
    /// let g = Gradient::linear(Vec3::ZERO, Vec3::Y);
    /// assert_eq!(g.sample(0.3), Vec3::new(0., 0.3, 0.));
    /// ```
    pub fn linear(start: T, end: T) -> Self {
        Self {
            keys: vec![
                GradientKey::<T> {
                    ratio: 0.,
                    value: start,
                },
                GradientKey::<T> {
                    ratio: 1.,
                    value: end,
                },
            ],
        }
    }

    /// Create a new gradient from a series of key points.
    ///
    /// If one or more duplicate ratios already exist, append each new key after
    /// all the existing keys with same ratio. The keys are inserted in order,
    /// but do not need to be sorted by ratio.
    ///
    /// The ratio must be a finite floating point value.
    ///
    /// This variant is slightly more performant than [`with_keys()`] because it
    /// can sort all keys before inserting them in batch.
    ///
    /// If you have only one or two keys, consider using [`constant()`] or
    /// [`linear()`], respectively, instead of this.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_hanabi::Gradient;
    /// let g = Gradient::from_keys([(0., 3.2), (1., 13.89), (0.3, 9.33)]);
    /// assert_eq!(g.len(), 3);
    /// assert_eq!(g.sample(0.3), 9.33);
    /// ```
    ///
    /// # Panics
    ///
    /// This method panics if any `ratio` is not in the \[0:1\] range.
    pub fn from_keys(keys: impl IntoIterator<Item = (f32, T)>) -> Self {
        // Note that all operations below are stable, including the sort. This ensures
        // the keys are kept in the correct order.
        let mut keys = keys
            .into_iter()
            .map(|(ratio, value)| GradientKey { ratio, value })
            .collect::<Vec<_>>();
        keys.sort_by(|a, b| FloatOrd(a.ratio).cmp(&FloatOrd(b.ratio)));
        Self { keys }
    }

    /// Returns `true` if the gradient contains no key points.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_hanabi::Gradient;
    /// let mut g = Gradient::new();
    /// assert!(g.is_empty());
    ///
    /// g.add_key(0.3, 3.42);
    /// assert!(!g.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Returns the number of key points in the gradient, also referred to as
    /// its 'length'.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_hanabi::Gradient;
    /// let g = Gradient::linear(3.5, 7.8);
    /// assert_eq!(g.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Add a key point to the gradient.
    ///
    /// If one or more duplicate ratios already exist, append the new key after
    /// all the existing keys with same ratio.
    ///
    /// The ratio must be a finite floating point value.
    ///
    /// Note that this function needs to perform a linear search into the
    /// gradient's key points to find an insertion point. If you already know
    /// all key points in advance, it's more efficient to use [`constant()`],
    /// [`linear()`], or [`with_keys()`].
    ///
    /// # Panics
    ///
    /// This method panics if `ratio` is not in the \[0:1\] range.
    pub fn with_key(mut self, ratio: f32, value: T) -> Self {
        self.add_key(ratio, value);
        self
    }

    /// Add a series of key points to the gradient.
    ///
    /// If one or more duplicate ratios already exist, append each new key after
    /// all the existing keys with same ratio. The keys are inserted in order.
    ///
    /// The ratio must be a finite floating point value.
    ///
    /// This variant is slightly more performant than [`add_key()`] because it
    /// can reserve storage for all key points upfront, which requires an exact
    /// size iterator.
    ///
    /// Note that if all key points are known upfront, [`from_keys()`] is a lot
    /// more performant.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_hanabi::Gradient;
    /// fn add_some_keys(mut g: Gradient<f32>) -> Gradient<f32> {
    ///     g.with_keys([(0.7, 12.9), (0.32, 9.31)].into_iter())
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This method panics if any `ratio` is not in the \[0:1\] range.
    pub fn with_keys(mut self, keys: impl ExactSizeIterator<Item = (f32, T)>) -> Self {
        self.keys.reserve(keys.len());
        for (ratio, value) in keys {
            self.add_key(ratio, value);
        }
        self
    }

    /// Add a key point to the gradient.
    ///
    /// If one or more duplicate ratios already exist, append the new key after
    /// all the existing keys with same ratio.
    ///
    /// The ratio must be a finite floating point value.
    ///
    /// # Panics
    ///
    /// This method panics if `ratio` is not in the \[0:1\] range.
    pub fn add_key(&mut self, ratio: f32, value: T) {
        assert!(ratio >= 0.0);
        assert!(ratio <= 1.0);
        let index = match self
            .keys
            .binary_search_by(|key| FloatOrd(key.ratio).cmp(&FloatOrd(ratio)))
        {
            Ok(mut index) => {
                // When there are duplicate keys, binary_search_by() returns the index of an
                // unspecified one. Make sure we insert always as the last
                // duplicate one, for determinism.
                let len = self.keys.len();
                while index + 1 < len && self.keys[index].ratio == self.keys[index + 1].ratio {
                    index += 1;
                }
                index + 1 // insert after last duplicate
            }
            Err(upper_index) => upper_index,
        };
        self.keys.insert(index, GradientKey { ratio, value });
    }

    /// Get the gradient keys.
    pub fn keys(&self) -> &[GradientKey<T>] {
        &self.keys[..]
    }

    /// Get mutable access to the gradient keys.
    pub fn keys_mut(&mut self) -> &mut [GradientKey<T>] {
        &mut self.keys[..]
    }

    /// Sample the gradient at the given ratio.
    ///
    /// If the ratio is exactly equal to those of one or more keys, sample the
    /// first key in the collection. If the ratio falls between two keys,
    /// return a linear interpolation of their values. If the ratio is
    /// before the first key or after the last one, return the first and
    /// last value, respectively.
    ///
    /// # Panics
    ///
    /// This method panics if the gradient is empty (has no key point).
    pub fn sample(&self, ratio: f32) -> T {
        assert!(!self.keys.is_empty());
        match self
            .keys
            .binary_search_by(|key| FloatOrd(key.ratio).cmp(&FloatOrd(ratio)))
        {
            Ok(mut index) => {
                // When there are duplicate keys, binary_search_by() returns the index of an
                // unspecified one. Make sure we sample the first duplicate, for determinism.
                while index > 0 && self.keys[index - 1].ratio == self.keys[index].ratio {
                    index -= 1;
                }
                self.keys[index].value
            }
            Err(upper_index) => {
                if upper_index > 0 {
                    if upper_index < self.keys.len() {
                        let key0 = &self.keys[upper_index - 1];
                        let key1 = &self.keys[upper_index];
                        let t = (ratio - key0.ratio) / (key1.ratio - key0.ratio);
                        key0.value.lerp(key1.value, t)
                    } else {
                        // post: sampling point located after the last key
                        self.keys[upper_index - 1].value
                    }
                } else {
                    // pre: sampling point located before the first key
                    self.keys[upper_index].value
                }
            }
        }
    }

    /// Sample the gradient at regular intervals.
    ///
    /// Create a list of sample points starting at ratio `start` and spaced with
    /// `inc` delta ratio. The number of samples is equal to the length of
    /// the `dst` slice. Sample the gradient at all those points, and fill
    /// the `dst` slice with the resulting values.
    ///
    /// This is equivalent to calling [`sample()`] in a loop, but is more
    /// efficient.
    ///
    /// [`sample()`]: Gradient::sample
    pub fn sample_by(&self, start: f32, inc: f32, dst: &mut [T]) {
        let count = dst.len();
        assert!(!self.keys.is_empty());
        let mut ratio = start;
        // pre: sampling points located before the first key
        let first_ratio = self.keys[0].ratio;
        let first_col = self.keys[0].value;
        let mut idst = 0;
        while idst < count && ratio <= first_ratio {
            dst[idst] = first_col;
            idst += 1;
            ratio += inc;
        }
        // main: sampling points located on or after the first key
        let mut ikey = 1;
        let len = self.keys.len();
        for i in idst..count {
            // Find the first key after the ratio
            while ikey < len && ratio > self.keys[ikey].ratio {
                ikey += 1;
            }
            if ikey >= len {
                // post: sampling points located after the last key
                let last_col = self.keys[len - 1].value;
                for d in &mut dst[i..] {
                    *d = last_col;
                }
                return;
            }
            if self.keys[ikey].ratio == ratio {
                dst[i] = self.keys[ikey].value;
            } else {
                let k0 = &self.keys[ikey - 1];
                let k1 = &self.keys[ikey];
                let t = (ratio - k0.ratio) / (k1.ratio - k0.ratio);
                dst[i] = k0.value.lerp(k1.value, t);
            }
            ratio += inc;
        }
    }
}