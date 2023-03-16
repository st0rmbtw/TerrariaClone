use bevy::prelude::{Component, States};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    Menu(MenuState),
    WorldLoading,
    InGame,
    Paused
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuState {
    Main,
    Settings(SettingsMenuState)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SettingsMenuState {
    #[default]
    None,
    Interface,
    Video,
    Cursor,
    Resolution,
    Main
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


#[derive(Default, Clone, Copy, PartialEq, Eq, Component)]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::InspectorOptions))]
pub enum MovementState {
    #[default]
    Idle,
    Walking,
    Flying
}