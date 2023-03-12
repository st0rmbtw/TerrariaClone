#![allow(clippy::type_complexity)]
#![allow(clippy::needless_update)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::{Color, Vec2};

#[macro_use]
extern crate lazy_static;

pub mod animation;
pub mod items;
pub mod lens;
pub mod parallax;
pub mod plugins;
pub mod state;
pub mod util;
pub mod language;
pub mod lighting;
pub mod rect;

pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);
pub const TEXT_COLOR: Color = Color::rgb(156. / 255., 156. / 255., 156. / 255.);

pub type Velocity = Vec2;