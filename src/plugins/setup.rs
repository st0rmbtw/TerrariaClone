use bevy::prelude::*;

use super::CursorPlugin;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(CursorPlugin)
            .add_system(zoom);
            // .add_system(camera_view_check);
    }
}

#[derive(Component)]
pub struct MainCamera;


const MAX_CAMERA_ZOOM: f32 = 1.;
const MIN_CAMERA_ZOOM: f32 = 0.5;
const CAMERA_ZOOM_STEP: f32 = 0.3;

fn zoom(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>
) {
    if let Ok(mut projection) = camera_query.get_single_mut() {
        if input.any_pressed([KeyCode::Equals]) {
            let scale = projection.scale - (CAMERA_ZOOM_STEP * time.delta_seconds());

            projection.scale = scale.max(MIN_CAMERA_ZOOM);
        }

        if input.any_pressed([KeyCode::Minus]) {
            let scale = projection.scale + (CAMERA_ZOOM_STEP * time.delta_seconds());

            projection.scale = scale.min(MAX_CAMERA_ZOOM);
        }
    }
}