use bevy::prelude::Component;

#[derive(Component)]
pub(super) enum ButtonRole {
    MenuButton,
    ControlButton
}