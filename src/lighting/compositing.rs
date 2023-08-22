use bevy::{
    prelude::{*},
    render::render_resource::{
            Extent3d, ShaderRef,
            TextureDimension, TextureFormat, AsBindGroup,
        }, reflect::{TypePath, TypeUuid},
};
use bevy_ecs_tilemap::prelude::MaterialTilemap;

use crate::{plugins::{world::resources::LightMap, camera::events::UpdateLightEvent}, world::{WorldData, light::propagate_light}};


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

#[derive(Resource)]
pub(crate) struct LightMapTexture(pub(crate) Handle<Image>);

pub(super) fn update_light_map(
    mut update_light_events: EventReader<UpdateLightEvent>,
    world_data: Res<WorldData>,
    light_map_texture: Res<LightMapTexture>,
    mut light_map: ResMut<LightMap>,
    mut images: ResMut<Assets<Image>>
) {
    if update_light_events.is_empty() { return; }

    let image = images.get_mut(&light_map_texture.0).unwrap();
    
    for event in update_light_events.iter() {
        let range_y = event.tile_pos.y - 5 .. event.tile_pos.y + 5;
        let range_x = event.tile_pos.x - 5 .. event.tile_pos.x + 5;

        // Left to right
        for y in range_y.clone() {
            for x in range_x.clone() {
                propagate_light(x as usize, y as usize, &mut light_map, &world_data, IVec2::new(-1, 0));
            }
        }

        // Top to bottom
        for x in range_x.clone() {
            for y in range_y.clone() {
                propagate_light(x as usize, y as usize, &mut light_map, &world_data, IVec2::new(0, -1));
            }
        }

        // Right to left
        for y in range_y.clone() {
            for x in range_x.clone().rev() {
                propagate_light(x as usize, y as usize, &mut light_map, &world_data, IVec2::new(1, 0));
            }
        }

        // Bottom to top
        for x in range_x.clone() {
            for y in range_y.clone().rev() {
                propagate_light(x as usize, y as usize, &mut light_map, &world_data, IVec2::new(0, 1));
            }
        }

        for y in range_y.clone() {
            for x in range_x.clone() {
                let y = y as usize;
                let x = x as usize;

                let color = light_map[(y, x)];
                let color_bytes = color.to_le_bytes();
                let index = ((y * light_map.ncols()) + x) * 4;    

                image.data[index]     = color_bytes[0];
                image.data[index + 1] = color_bytes[1];
                image.data[index + 2] = color_bytes[2];
                image.data[index + 3] = color_bytes[3];    
            }
        }
    }
}

pub(super) fn setup_post_processing_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    light_map: Res<LightMap>,
) {
    let mut bytes = vec![0; light_map.len() * 4];

    for ((y, x), color) in light_map.indexed_iter() {
        let index = ((y * light_map.ncols()) + x) * 4;

        let color_bytes = color.to_le_bytes();

        bytes[index]     = color_bytes[0];
        bytes[index + 1] = color_bytes[1];
        bytes[index + 2] = color_bytes[2];
        bytes[index + 3] = color_bytes[3];
    }

    let light_map_image = Image::new(
        Extent3d {
            width: light_map.ncols() as u32,
            height: light_map.nrows() as u32,
            ..default()
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::R32Float,
    );

    let light_map_image_handle = images.add(light_map_image);

    commands.insert_resource(LightMapTexture(light_map_image_handle));
}