use bevy::{
    prelude::{*},
    render::render_resource::{
            Extent3d, ShaderRef,
            TextureDimension, TextureFormat, AsBindGroup,
        }, reflect::{TypePath, TypeUuid},
};
use bevy_ecs_tilemap::prelude::MaterialTilemap;

use crate::plugins::world::resources::LightMap;


#[derive(AsBindGroup, TypePath, TypeUuid, Clone, Default)]
#[uuid = "9114bbd2-1bb3-4b5a-a710-8965798db745"]
pub(crate) struct TileMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub(crate) shadow_map_image: Handle<Image>,

    #[uniform(2)]
    pub(crate) chunk_pos: UVec2
}

impl MaterialTilemap for TileMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/post_processing.wgsl".into()
    }
}

#[derive(Component)]
pub(super) struct ShadowMap;

#[derive(Resource)]
pub(crate) struct ShadowMapTexture(pub(crate) Handle<Image>);

// pub(super) fn update_light_map(
//     query_camera: Query<(&Handle<PostProcessingMaterial>, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
//     mut update_light_events: EventReader<UpdateLightEvent>,
//     post_processing_materials: Res<Assets<PostProcessingMaterial>>,
//     mut images: ResMut<Assets<Image>>,
//     mut light_map: ResMut<LightMap>,
//     world_data: Res<WorldData>
// ) {
//     if let Ok((lighting_material_handle, projection, camera_transform)) = query_camera.get_single() {
//         if update_light_events.iter().last().is_some() {
//             let lighting_material = post_processing_materials
//                 .get(lighting_material_handle)
//                 .unwrap();
//             let light_map_texture = images.get_mut(&lighting_material.shadow_map_image).unwrap();
            
//             let x_from = ((camera_transform.translation().x + projection.area.min.x) / TILE_SIZE * light::CLUSTER_SIZE as f32) as usize;
//             let x_to = ((camera_transform.translation().x + projection.area.max.x) / TILE_SIZE * light::CLUSTER_SIZE as f32) as usize;

//             let y_from = ((camera_transform.translation().y + projection.area.max.y) / TILE_SIZE * light::CLUSTER_SIZE as f32).abs() as usize;
//             let y_to = ((camera_transform.translation().y + projection.area.min.y) / TILE_SIZE * light::CLUSTER_SIZE as f32).abs() as usize;

//             for y in y_from..y_to {
//                 for x in x_from..x_to {
//                     light::propagate_light(x, y, &mut light_map.colors, &world_data);

//                     if let Some(color) = light_map.colors.get((y, x)) {
//                         let index = ((y * light_map.colors.ncols()) + x) * 4;
//                         light_map_texture.data[index]     = *color; // R
//                         light_map_texture.data[index + 1] = *color; // G
//                         light_map_texture.data[index + 2] = *color; // B
//                         light_map_texture.data[index + 3] = 0xFF; // A
//                     }
//                 }
//             }

//             for y in (y_from..y_to).rev() {
//                 for x in (x_from..x_to).rev() {
//                     light::propagate_light(x, y, &mut light_map.colors, &world_data);

//                     if let Some(color) = light_map.colors.get((y, x)) {
//                         let index = ((y * light_map.colors.ncols()) + x) * 4;
//                         light_map_texture.data[index]     = *color; // R
//                         light_map_texture.data[index + 1] = *color; // G
//                         light_map_texture.data[index + 2] = *color; // B
//                         light_map_texture.data[index + 3] = 0xFF; // A
//                     }
//                 }
//             }
//         }
//     }
// }       

#[allow(dead_code)]
pub(super) fn setup_post_processing_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    light_map: Res<LightMap>,
) {
    let mut bytes = vec![0; light_map.colors.len() * 4];

    for ((y, x), color) in light_map.colors.indexed_iter() {
        let index = ((y * light_map.colors.ncols()) + x) * 4;

        bytes[index]     = *color;
        bytes[index + 1] = *color;
        bytes[index + 2] = *color;
        bytes[index + 3] = 0xFF;
    }

    let shadow_map_image = Image::new(
        Extent3d {
            width: light_map.colors.ncols() as u32,
            height: light_map.colors.nrows() as u32,
            ..default()
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
    );

    let shadow_map_image_handle = images.add(shadow_map_image);

    commands.insert_resource(ShadowMapTexture(shadow_map_image_handle));
}