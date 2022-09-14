use autodefault::autodefault;
use bevy::{render::render_resource::{AsBindGroup, ShaderRef}, reflect::TypeUuid, prelude::{Plugin, App, Commands, Color, OrthographicProjection, With, Query, Component, Res, BuildChildren, Transform, GlobalTransform, ResMut, Assets, Mesh, shape}, sprite::{Material2dPlugin, Material2d, SpriteBundle, Sprite, MaterialMesh2dBundle}, math::vec2};
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};

use crate::{plugins::{MainCamera, TILE_SIZE}, state::GameState};

// region: Plugin
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::InGame,setup)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(set_sprite_size)
                    .with_system(set_sprite_position)
                    .into()
            )
            .add_plugin(Material2dPlugin::<LightingMaterial>::default());
    }
}
// endregion

#[derive(Component)]
struct LightingRectangle;

#[autodefault(except(LightingMaterial))]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LightingMaterial>>,
) {
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::from_xyz(0., 0., 10.),
        material: materials.add(LightingMaterial {})
    })
    .insert(LightingRectangle);
}

fn set_sprite_size(
    camera_query: Query<&OrthographicProjection, With<MainCamera>>,
    mut lighting_rect_query: Query<&mut Sprite, With<LightingRectangle>>
) {
    let projection = camera_query.single();

    for mut sprite in &mut lighting_rect_query {

        let width = (projection.left.abs() + projection.right) * projection.scale;
        let height = (projection.bottom.abs() + projection.top) * projection.scale;

        let width = (width / TILE_SIZE + 2.).ceil() * TILE_SIZE;
        let height = (height / TILE_SIZE + 2.).ceil() * TILE_SIZE;

        sprite.custom_size = Some(vec2(width, height));
    }
}

fn set_sprite_position(
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut lighting_rect_query: Query<&mut Transform, With<LightingRectangle>>
) {
    let camera_transform = camera_query.single();

    for mut transform in &mut lighting_rect_query {
        transform.translation.x = camera_transform.translation().x;
        transform.translation.y = camera_transform.translation().y;
    }
}


#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "4c486a35-04ca-46e9-a7ea-c718fdb95ebd"]
pub struct LightingMaterial {

}

impl Material2d for LightingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tile_shader.wgsl".into()
    }
}