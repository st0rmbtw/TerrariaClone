use bevy::prelude::Component;
use leafwing_input_manager::Actionlike;

#[derive(Component)]
pub struct MainCamera;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum MouseAction {
    ZoomIn,
    ZoomOut
}