use bevy::prelude::Component;

#[derive(Component)]
pub(super) struct SettingsButtonContainer;

#[derive(Component)]
pub(super) struct SettingsButton;

#[derive(Component)]
pub(super) struct MenuContainer;

#[derive(Component)]
pub(super) struct MenuTabs;

#[derive(Component)]
pub(super) struct TabMenu;

#[derive(Component)]
pub(super) struct TabMenuContainer;

#[derive(Component)]
pub(super) struct TabButton;

#[derive(Component)]
pub(super) struct TabMenuButton;

pub(super) mod buttons {
    use bevy::prelude::Component;

    #[derive(Component)]
    pub struct GeneralButton;

    #[derive(Component)]
    pub struct InterfaceButton;

    #[derive(Component)]
    pub struct VideoButton;

    #[derive(Component)]
    pub struct CursorButton;

    #[derive(Component)]
    pub struct CloseMenuButton;

    #[derive(Component)]
    pub struct SaveAndExitButton;
}