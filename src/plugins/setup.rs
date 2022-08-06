use bevy::{prelude::*, render::texture::ImageSettings};

use super::CursorPlugin;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(CursorPlugin)
            .insert_resource(ImageSettings::default_nearest());
    }
}

#[derive(Component)]
pub struct MainCamera;