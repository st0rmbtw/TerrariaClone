use bevy::prelude::Color;

#[macro_use]
extern crate lazy_static;

pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);

pub mod plugins;
pub mod animation;
pub mod item;
pub mod block;
pub mod util;
pub mod lens;
pub mod world_generator;
pub mod state;
pub mod parallax;
pub mod wall;