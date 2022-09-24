use bevy::prelude::{Color, Vec2};

#[macro_use]
extern crate lazy_static;

pub mod animation;
pub mod block;
pub mod items;
pub mod lens;
pub mod parallax;
pub mod plugins;
pub mod state;
pub mod util;
pub mod wall;
pub mod world_generator;

pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);
pub const TEXT_COLOR: Color = Color::rgb(156. / 255., 156. / 255., 156. / 255.);

pub type Velocity = Vec2;

#[derive(Clone, Copy)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2
}