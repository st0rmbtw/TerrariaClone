use bevy::{prelude::{Resource, Color, Deref, DerefMut}, window::{PresentMode, WindowMode}, audio::VolumeLevel};
use serde::{Deserialize, Serialize};

use crate::{common::{BoolValue, Toggle}, language::keys::UIStringKey};

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

impl Toggle for ShowTileGrid {
    fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

impl BoolValue for ShowTileGrid {
    fn value(&self) -> bool {
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

#[derive(Resource, Deref, DerefMut, Clone, Copy)]
pub(crate) struct MusicVolume(VolumeLevel);

impl MusicVolume {
    pub(crate) fn new(value: f32) -> Self {
        Self(VolumeLevel::new(value.clamp(0., 1.)))
    }
}

#[derive(Resource, Deref, DerefMut, Clone, Copy)]
pub(crate) struct SoundVolume(VolumeLevel);

impl SoundVolume {
    pub(crate) fn new(value: f32) -> Self {
        Self(VolumeLevel::new(value.clamp(0., 1.)))
    }
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LightSmoothness {
    Classic,
    Medium,
    High,
    Ultra
}

impl LightSmoothness {
    pub(crate) const fn length() -> u8 {
        4
    }

    pub(crate) const fn new(value: u8) -> Self {
        match value {
            0 => Self::Classic,
            1 => Self::Medium,
            2 => Self::High,
            3 => Self::Ultra,
            _ => panic!("Expected either 0, 1, 2 or 3 value")
        }
    }

    pub(crate) const fn to_u8(self) -> u8 {
        match self {
            Self::Classic => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Ultra => 3,
        }
    }

    pub(crate) fn subdivision(&self) -> u32 {
        match self {
            Self::Classic => 2,
            Self::Medium => 4,
            Self::High => 8,
            Self::Ultra => 16,
        }
    }

    pub(crate) fn name(&self) -> UIStringKey {
        match &self {
            Self::Classic => UIStringKey::Classic,
            Self::Medium => UIStringKey::Medium,
            Self::High => UIStringKey::High,
            Self::Ultra => UIStringKey::Ultra,
        }
    }
}