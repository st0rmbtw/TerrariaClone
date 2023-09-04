use bevy::{prelude::{Resource, Color, Deref, DerefMut}, window::{PresentMode, WindowMode}, audio::VolumeLevel};
use serde::{Deserialize, Serialize};

use crate::common::BoolValue;

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
        assert!((0.0..=1.0).contains(&value));
        Self(VolumeLevel::new(value))
    }

    pub(crate) fn from_slider_value(value: f32) -> Self {
        assert!((0.0..=100.0).contains(&value));
        Self::new(value / 100.)
    }

    pub(crate) fn slider_value(&self) -> f32 {
        self.get() * 100.
    }
}

#[derive(Resource, Deref, DerefMut, Clone, Copy)]
pub(crate) struct SoundVolume(VolumeLevel);

impl SoundVolume {
    pub(crate) fn new(value: f32) -> Self {
        assert!((0.0..=1.0).contains(&value));
        Self(VolumeLevel::new(value))
    }

    pub(crate) fn from_slider_value(value: f32) -> Self {
        assert!((0.0..=100.0).contains(&value));
        Self::new(value / 100.)
    }

    pub(crate) fn slider_value(&self) -> f32 {
        self.get() * 100.
    }
}