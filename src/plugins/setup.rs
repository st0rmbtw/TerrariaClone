use bevy::{prelude::*, render::{texture::ImageSettings, view::VisibilityPlugin}};

use super::CursorPlugin;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(CursorPlugin)
            .insert_resource(ImageSettings::default_nearest())
            .add_plugin(VisibilityPlugin)
            .add_system(zoom);
    }
}

#[derive(Component)]
pub struct MainCamera;

fn zoom(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>
) {
    let mut projection = camera_query.single_mut();

    if input.any_pressed([KeyCode::Equals]) {
        let scale = projection.scale - (0.2 * time.delta_seconds());

        projection.scale = scale.max(0.55);
    }

    if input.any_pressed([KeyCode::Minus]) {
        let scale = projection.scale + (0.2 * time.delta_seconds());

        projection.scale = scale.min(1.0);
    }
}