use bevy::prelude::{Component, States};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub(crate) enum GameState {
    #[default]
    AssetLoading,
    Menu,
    WorldLoading,
    InGame,
    Paused
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Default)]
pub(crate) enum MenuState {
    #[default]
    None,
    Main,
    Settings(SettingsMenuState)
}

impl States for MenuState {
    type Iter = std::array::IntoIter<MenuState, 7>;

    fn variants() -> Self::Iter {
        [
            MenuState::None,
            MenuState::Main,
            MenuState::Settings(SettingsMenuState::Main),
            MenuState::Settings(SettingsMenuState::Cursor),
            MenuState::Settings(SettingsMenuState::Video),
            MenuState::Settings(SettingsMenuState::Interface),
            MenuState::Settings(SettingsMenuState::Resolution),
        ].into_iter()
    }
}

impl MenuState {
    pub(crate) fn back(&self) -> MenuState {
        match &self {
            MenuState::None => MenuState::None,
            MenuState::Main => MenuState::Main,
            MenuState::Settings(SettingsMenuState::Main) => MenuState::Main,
            MenuState::Settings(SettingsMenuState::Video) => MenuState::Settings(SettingsMenuState::Main),
            MenuState::Settings(SettingsMenuState::Interface) => MenuState::Settings(SettingsMenuState::Main),
            MenuState::Settings(SettingsMenuState::Cursor) => MenuState::Settings(SettingsMenuState::Main),
            MenuState::Settings(SettingsMenuState::Volume) => MenuState::Settings(SettingsMenuState::Main),
            MenuState::Settings(SettingsMenuState::Language) => MenuState::Settings(SettingsMenuState::Main),
            MenuState::Settings(SettingsMenuState::Resolution) => MenuState::Settings(SettingsMenuState::Video),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd)]
pub(crate) enum SettingsMenuState {
    #[default]
    Main,
    Interface,
    Video,
    Volume,
    Resolution,
    Cursor,
    Language
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Component)]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::InspectorOptions))]
pub(crate) enum MovementState {
    #[default]
    Idle,
    Walking,
    Flying
}