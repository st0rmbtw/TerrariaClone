#![allow(clippy::type_complexity)]
#![allow(clippy::needless_update)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::{Color, Resource};

pub mod animation;
pub mod common;
pub mod items;
pub mod language;
pub mod lighting;
pub mod parallax;
pub mod plugins;

pub const TEXT_COLOR: Color = Color::rgb(156. / 255., 156. / 255., 156. / 255.);

#[derive(Default, Resource)]
pub struct DebugConfiguration {
    pub free_camera: bool,
    pub show_hitboxes: bool,
    pub show_collisions: bool,
    pub instant_break: bool
}