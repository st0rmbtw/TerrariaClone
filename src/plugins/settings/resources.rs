use bevy::{prelude::{Resource, Color}, window::{PresentMode, WindowMode}};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, FromRepr};

#[derive(Resource, Default, Serialize, Deserialize, Clone, Copy, EnumCount, FromRepr)]
pub enum Resolution {
    R800x600,
    R1024x768,
    R1152x864,
    R1176x664,
    R1280x720,
    R1280x768,
    R1280x800,
    R1280x960,
    R1280x1024,
    R1360x768,
    R1366x768,
    R1440x900,
    R1600x900,
    R1600x1024,
    R1680x1050,
    #[default]
    R1920x1080,
}

impl Resolution {
    pub const fn data(&self) -> ResolutionData {
        use Resolution::*;

        match self {
            R800x600 => ResolutionData::new(800., 600.),
            R1024x768 => ResolutionData::new(1024., 768.),
            R1152x864 => ResolutionData::new(1152., 864.),
            R1176x664 => ResolutionData::new(1176., 664.),
            R1280x720 => ResolutionData::new(1280., 720.),
            R1280x768 => ResolutionData::new(1280., 768.),
            R1280x800 => ResolutionData::new(1280., 800.),
            R1280x960 => ResolutionData::new(1280., 960.),
            R1280x1024 => ResolutionData::new(1280., 1024.),
            R1360x768 => ResolutionData::new(1360., 768.),
            R1366x768 => ResolutionData::new(1366., 768.),
            R1440x900 => ResolutionData::new(1440., 900.),
            R1600x900 => ResolutionData::new(1600., 900.),
            R1600x1024 => ResolutionData::new(1600., 1024.),
            R1680x1050 => ResolutionData::new(1680., 1050.),
            R1920x1080 => ResolutionData::new(1920., 1080.)
        }
    }
}

#[derive(Clone, Copy)]
pub struct ResolutionData {
    pub width: f32,
    pub height: f32
}

impl ResolutionData {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
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