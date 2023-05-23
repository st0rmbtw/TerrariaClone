use bevy::prelude::Component;

#[derive(Component)]
pub(super) struct HoverableInfoMarker;

#[derive(Component)]
pub(super) struct TileGrid;

#[derive(Component)]
pub(super) struct CursorContainer;

#[derive(Component)]
pub(super) struct CursorBackground;

#[derive(Component)]
pub(super) struct CursorForeground;

#[derive(Component)]
pub(crate) enum Hoverable {
    None,
    SimpleText(String)
}