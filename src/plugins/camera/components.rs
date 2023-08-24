use bevy::prelude::Component;

#[derive(Component)]
pub(crate) struct MainCamera;

#[derive(Component)]
pub(crate) struct PostProcessingCamera;

#[derive(Component)]
pub(crate) struct BackgroundCamera;

#[derive(Component)]
pub(crate) struct WorldCamera;

#[derive(Component)]
pub(crate) struct ZoomableCamera;

#[derive(Component)]
pub(crate) struct MoveCamera;