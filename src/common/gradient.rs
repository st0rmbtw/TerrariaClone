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
#[derive(Debug, Default, Clone, PartialEq)]
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
}