use bevy::prelude::Color;

#[macro_use]
extern crate lazy_static;

pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);

pub mod animation;
pub mod block;
pub mod item;
pub mod lens;
pub mod parallax;
pub mod plugins;
pub mod state;
pub mod util;
pub mod wall;
pub mod world_generator;
