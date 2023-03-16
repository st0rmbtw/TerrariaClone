use bevy::{prelude::{Resource, Color}, window::{PresentMode, WindowMode}};
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Resolution {
    pub width: f32,
    pub height: f32
}

impl Resolution {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Self { width: 1920., height: 1080. }
    }
}

#[derive(Resource)]
pub struct VSync(pub bool);

#[derive(Resource)]
pub struct FullScreen(pub bool);

#[derive(Resource, PartialEq, Clone)]
pub struct ShowTileGrid(pub bool);

#[derive(Resource, Clone, Copy, Serialize, Deserialize)]
pub struct CursorColor {
    pub background_color: Color,
    pub foreground_color: Color
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
    pub fn as_present_mode(&self) -> PresentMode {
        match self.0 {
            true => PresentMode::Fifo,
            false => PresentMode::Immediate
        }
    }
}

impl FullScreen {
    pub fn as_window_mode(&self) -> WindowMode {
        match self.0 {
            true => WindowMode::Fullscreen,
            false => WindowMode::Windowed
        }
    }
}