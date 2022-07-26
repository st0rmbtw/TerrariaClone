use bevy::{prelude::{Plugin, Commands, Res, AssetServer, Transform, default, ImageBundle, Handle, Image, Color, NodeBundle, ParallelSystemDescriptorCoercion}, math::{Size, Rect}, ui::{Style, Val, AlignItems, JustifyContent}, hierarchy::BuildChildren};


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // app.add_startup_system(spawn_hotbar.label(SPAWN_PLAYER_UI_LABEL));
    }
}
