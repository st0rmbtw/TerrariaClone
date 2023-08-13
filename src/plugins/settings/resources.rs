use bevy::{prelude::{Resource, Color}, window::{PresentMode, WindowMode}};
use serde::{Deserialize, Serialize};

use crate::common::IsVisible;

#[derive(Resource, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(crate) struct Resolution {
    pub(crate) width: f32,
    pub(crate) height: f32
}

impl Resolution {
    pub(crate) const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Self { width: 1920., height: 1080. }
    }
}

#[derive(Resource, Clone, Copy)]
pub(crate) struct VSync(pub(crate) bool);

#[derive(Resource, Clone, Copy)]
pub(crate) struct FullScreen(pub(crate) bool);

#[derive(Resource, PartialEq, Clone, Copy)]
pub(crate) struct ShowTileGrid(pub(crate) bool);

impl IsVisible for ShowTileGrid {
    fn is_visible(&self) -> bool {
        self.0
    }
}

#[derive(Resource, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct CursorColor {
    pub(crate) background_color: Color,
    pub(crate) foreground_color: Color
}

impl Default for CursorColor {
    fn default() -> Self {
        Self { 
            background_color: Color::rgb(0.7, 0.7, 0.7),
            foreground_color: Color::PINK
        }
    }
}

impl VSync {
    pub(crate) fn as_present_mode(&self) -> PresentMode {
        match self.0 {
            true => PresentMode::Fifo,
            false => PresentMode::Immediate
        }
    }
}

impl FullScreen {
    pub(crate) fn as_window_mode(&self) -> WindowMode {
        match self.0 {
            true => WindowMode::BorderlessFullscreen,
            false => WindowMode::Windowed
        }
    }
}