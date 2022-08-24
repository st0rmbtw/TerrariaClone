use bevy::prelude::Component;
use bevy_inspector_egui::Inspectable;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    AssetLoading,
    MainMenu,
    WorldLoading,
    InGame,
    Paused
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Inspectable, Component)]
pub enum MovementState {
    #[default]
    IDLE,
    WALKING,
    FLYING,
    FALLING
}