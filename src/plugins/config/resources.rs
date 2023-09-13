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
        debug_assert!((0.0..=1.0).contains(&value));
        Self(VolumeLevel::new(value))
    }
}

#[derive(Resource, Deref, DerefMut, Clone, Copy)]
pub(crate) struct SoundVolume(VolumeLevel);

impl SoundVolume {
    pub(crate) fn new(value: f32) -> Self {
        debug_assert!((0.0..=1.0).contains(&value));
        Self(VolumeLevel::new(value))
    }
}

#[derive(Resource, Clone)]
pub(crate) struct LightSettings {
    pub(crate) subdivision: u32,
    pub(crate) decay_solid: f32,
    pub(crate) decay_air: f32
}

#[derive(Resource, Clone, Copy)]
pub(crate) enum LightSmoothness {
    Classic,
    Medium,
    High
}

impl LightSmoothness {
    pub(crate) const fn new(value: u8) -> Self {
        match value {
            0 => Self::Classic,
            1 => Self::Medium,
            2 => Self::High,
            _ => panic!("Expected either 0, 1 or 2 value")
        }
    }

    pub(crate) const fn to_u8(self) -> u8 {
        match self {
            Self::Classic => 0,
            Self::Medium => 1,
            Self::High => 2,
        }
    }

    pub(crate) const fn settings(&self) -> LightSettings {
        match self {
            LightSmoothness::Classic => LightSettings {
                subdivision: 2,
                decay_solid: 0.56,
                decay_air: 0.56
            },
            LightSmoothness::Medium => LightSettings {
                subdivision: 4,
                decay_solid: 0.78,
                decay_air: 0.78
            },
            LightSmoothness::High => LightSettings {
                subdivision: 8,
                decay_solid: 0.86,
                decay_air: 0.86
            },
        }
    }

    pub(crate) fn name(&self) -> UIStringKey {
        match &self {
            LightSmoothness::Classic => UIStringKey::Classic,
            LightSmoothness::Medium => UIStringKey::Medium,
            LightSmoothness::High => UIStringKey::High,
        }
    }
}