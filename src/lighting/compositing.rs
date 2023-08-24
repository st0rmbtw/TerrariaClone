use std::ops::Range;

use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, ShaderRef,
        TextureDimension, TextureFormat, AsBindGroup,
    }, reflect::{TypePath, TypeUuid}, sprite::Material2d,
};

use crate::{plugins::{world::resources::LightMap, camera::events::UpdateLightEvent}, world::{WorldData, light::{propagate_light, PassDirection}}};


#[derive(AsBindGroup, TypePath, TypeUuid, Clone, Default)]
#[uuid = "9114bbd2-1bb3-4b5a-a710-1235798db745"]
pub(crate) struct TileMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub(crate) light_map_image: Handle<Image>,

    #[uniform(2)]
    pub(crate) chunk_pos: UVec2
}

impl Material2d for TileMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tile_material.wgsl".into()
    }
}

#[derive(Resource)]
pub(crate) struct LightMapTexture(pub(crate) Handle<Image>);

pub(super) fn update_light_map(
    mut update_light_events: EventReader<UpdateLightEvent>,
    world_data: Res<WorldData>,
    light_map_texture: Res<LightMapTexture>,
    materials: Res<Assets<TileMaterial>>,
    mut light_map: ResMut<LightMap>,
    mut images: ResMut<Assets<Image>>,
    mut asset_events: EventWriter<AssetEvent<TileMaterial>>
) {
    if update_light_events.is_empty() { return; }

    let image = images.get_mut(&light_map_texture.0).unwrap();
    
    for event in update_light_events.iter() {
        let x = event.tile_pos.x as usize;
        let y = event.tile_pos.y as usize;

        if world_data.solid_block_exists((x, y)) || world_data.wall_exists((x, y)) {
            light_map[(y, x)] = 0.;
        } else {
            light_map[(y, x)] = 1.;
        }

        let range_y = y.saturating_sub(20) .. (y + 20).min(world_data.size.height);
        let range_x = x.saturating_sub(20) .. (x + 20).min(world_data.size.width);

        // Top to bottom
        for x in range_x.clone() {
            for y in range_y.clone() {
                propagate_light(x, y, &mut light_map, &world_data, PassDirection::TopToBottom);
            }
        }

        // Left to right
        for y in range_y.clone() {
            for x in range_x.clone() {
                propagate_light(x, y, &mut light_map, &world_data, PassDirection::LeftToRight);
            }
        }

        // Right to left
        for y in range_y.clone() {
            for x in range_x.clone().rev() {
                propagate_light(x, y, &mut light_map, &world_data, PassDirection::RightToLeft);
            }
        }

        // Bottom to top
        for x in range_x.clone() {
            for y in range_y.clone().rev() {
                propagate_light(x, y, &mut light_map, &world_data, PassDirection::BottomToTop);
            }
        }

        copy_light_map_to_texture(range_x.clone(), range_y.clone(), &light_map, &mut image.data);
    }

    for id in materials.ids() {
        asset_events.send(AssetEvent::Modified { handle: Handle::weak(id) });
    }
}

pub(super) fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    light_map: Res<LightMap>,
) {
    let mut bytes = vec![0; light_map.len() * 4];

    let width = light_map.ncols();
    let height = light_map.nrows();

    copy_light_map_to_texture(0..width, 0..height, &light_map, &mut bytes);

    let light_map_image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            ..default()
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::R32Float,
    );

    let light_map_image_handle = images.add(light_map_image);

    commands.insert_resource(LightMapTexture(light_map_image_handle));

}

fn copy_light_map_to_texture(
    range_x: Range<usize>,
    range_y: Range<usize>,
    light_map: &LightMap,
    bytes: &mut Vec<u8>
) {
    for y in range_y {
        for x in range_x.clone() {
            let color = light_map[(y, x)];
            let index = (y * light_map.ncols() + x) * 4;

            let color_bytes = color.to_le_bytes();

            bytes[index]     = color_bytes[0];
            bytes[index + 1] = color_bytes[1];
            bytes[index + 2] = color_bytes[2];
            bytes[index + 3] = color_bytes[3];
        }
    }
}