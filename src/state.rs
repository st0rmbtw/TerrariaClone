use bevy::prelude::{Component, States};
use bevy_inspector_egui::InspectorOptions;

use crate::plugins::settings_menu::SettingsMenuState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    MainMenu,
    WorldLoading,
    InGame,
    Paused,
    Settings(SettingsMenuState)
}

impl States for GameState {
    type Iter = std::array::IntoIter<GameState, 10>;

    fn variants() -> Self::Iter {
        [
            GameState::AssetLoading, GameState::MainMenu, GameState::WorldLoading, 
            GameState::InGame, GameState::Paused, GameState::Settings(SettingsMenuState::None),
            GameState::Settings(SettingsMenuState::Cursor), GameState::Settings(SettingsMenuState::Video),
            GameState::Settings(SettingsMenuState::Interface), GameState::Settings(SettingsMenuState::Resolution)
        ].into_iter()
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Component, InspectorOptions)]
pub enum MovementState {
    #[default]
    Idle,
    Walking,
    Flying
}