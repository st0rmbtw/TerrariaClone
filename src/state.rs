use bevy::prelude::Component;
use bevy_inspector_egui::InspectorOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    AssetLoading,
    MainMenu,
    WorldLoading,
    InGame,
    Paused,
    Settings
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Component, InspectorOptions)]
pub enum MovementState {
    #[default]
    Idle,
    Walking,
    Flying
}