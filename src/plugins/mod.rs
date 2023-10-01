use bevy::prelude::{SystemSet, Component};

pub(crate) mod main;
pub(crate) mod assets;
pub(crate) mod background;
pub(crate) mod cursor;
pub(crate) mod fps;
pub(crate) mod inventory;
pub(crate) mod player;
pub(crate) mod config;
pub(crate) mod camera;
pub(crate) mod ui;
pub(crate) mod world;
pub(crate) mod audio;
pub(crate) mod slider;
pub(crate) mod particles;
pub(crate) mod item;

#[cfg(feature = "debug")]
pub(crate) mod debug;

#[derive(Debug, PartialEq, Eq, Hash, Clone, SystemSet)]
pub(crate) enum InGameSystemSet {
    PreUpdate,
    FixedUpdate,
    Update,
    PostUpdate
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, SystemSet)]
pub(crate) enum MenuSystemSet {
    PreUpdate,
    Update,
    PostUpdate
}

#[derive(Component)]
pub(crate) struct DespawnOnGameExit;