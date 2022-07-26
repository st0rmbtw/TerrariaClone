use std::default::Default;

use bevy::{prelude::{Color, default}, math::Rect, reflect::Reflect};

#[macro_use]
extern crate lazy_static;

pub trait RectExtensions<T: Reflect + PartialEq> {
    fn horizontal(value: T) -> Self;
    fn top(value: T) -> Self;
}

pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);

impl<T: Reflect + PartialEq + Default + Clone> RectExtensions<T> for Rect<T> {
    fn horizontal(value: T) -> Self {
        Self {
            left: value.clone(),
            right: value,
            ..default()
        }
    }

    fn top(value: T) -> Self {
        Self {
            top: value,
            ..default()
        }
    }
}

pub mod plugins;
pub mod item;
pub mod block;