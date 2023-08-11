use bevy::prelude::{Component, States};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum GameState {
    #[default]
    AssetLoading,
    Menu(MenuState),
    WorldLoading,
    InGame,
    Paused
}

impl States for GameState {
    type Iter = std::array::IntoIter<GameState, 10>;

    fn variants() -> Self::Iter {
        [
            GameState::AssetLoading, GameState::WorldLoading, GameState::InGame, GameState::Paused,
            GameState::Menu(MenuState::Main),
            GameState::Menu(MenuState::Settings(SettingsMenuState::Main)),
            GameState::Menu(MenuState::Settings(SettingsMenuState::Cursor)),
            GameState::Menu(MenuState::Settings(SettingsMenuState::Video)),
            GameState::Menu(MenuState::Settings(SettingsMenuState::Interface)),
            GameState::Menu(MenuState::Settings(SettingsMenuState::Resolution)),
        ].into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub(crate) enum MenuState {
    Main,
    Settings(SettingsMenuState)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd)]
pub(crate) enum SettingsMenuState {
    #[default]
    Main,
    Interface,
    Video,
    Resolution,
    Cursor,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Component)]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::InspectorOptions))]
pub(crate) enum MovementState {
    #[default]
    Idle,
    Walking,
    Flying
}