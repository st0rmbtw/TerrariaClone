use bevy::{prelude::*, render::texture::ImageSettings};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app;
            // .insert_resource(ImageSettings::default_linear());
    }
}