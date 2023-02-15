use bevy::{prelude::{Resource, Color}, window::{PresentMode, WindowMode}};
use serde::{Deserialize, Serialize};

#[derive(Resource, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub struct Resolution {
    pub width: f32,
    pub height: f32,
    pub index: i32
}

#[derive(Resource)]
pub struct VSync(pub bool);

#[derive(Resource)]
pub struct FullScreen(pub bool);

#[derive(Resource, PartialEq, Clone)]
pub struct ShowTileGrid(pub bool);

#[derive(Resource)]
pub struct CursorColor(pub Color);

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