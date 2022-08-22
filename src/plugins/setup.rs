use bevy::{prelude::*, render::camera::WindowOrigin};
use bevy_rapier2d::prelude::Collider;

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
const MIN_CAMERA_ZOOM: f32 = 0.4;
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

fn camera_view_check(
    mut camera_query: Query<(&Camera, &GlobalTransform, &OrthographicProjection), Changed<GlobalTransform>>,
    mut draw_query: Query<(&mut Visibility, &Transform), (Without<Node>, With<Collider>)>,
) {
    const THRESHOLD: f32 = 1.0;

    camera_query.for_each_mut(|(camera, camera_transform, ortho_proj)| {
        draw_query.for_each_mut(|(mut visibility, transform)| {
            
            let visual_check = camera.projection_matrix().transform_point3(Vec3::from(
                camera_transform.translation() - transform.translation
            ));

            if matches!(ortho_proj.window_origin, WindowOrigin::Center) {
                visibility.is_visible = visual_check[0].abs() <= THRESHOLD || visual_check[1].abs() <= THRESHOLD;
            }
        });
    });
}