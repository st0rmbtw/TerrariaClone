use bevy::{prelude::{IVec2, Image}, math::URect, render::render_resource::{Extent3d, TextureDimension, TextureFormat}, utils::default};

use super::WorldData;

type LightMap = Image;

pub(crate) const SUBDIVISION: usize = 2;

#[derive(Clone, Copy)]
pub(crate) enum PassDirection {
    LeftToRight,
    TopToBottom,
    RightToLeft,
    BottomToTop
}

impl PassDirection {
    pub(crate) const fn to_ivec2(self) -> IVec2 {
        match self {
            PassDirection::LeftToRight => IVec2::new(-1, 0),
            PassDirection::TopToBottom => IVec2::new(0, -1),
            PassDirection::RightToLeft => IVec2::new(1, 0),
            PassDirection::BottomToTop => IVec2::new(0, 1),
        }
    }
}

pub(crate) fn generate_light_map(world: &WorldData) -> LightMap {
    println!("Generating light map...");

    let light_map_width = world.size.width * SUBDIVISION;
    let light_map_height = world.size.height * SUBDIVISION;

    let mut light_map = Image::new_fill(
        Extent3d {
            width: light_map_width as u32,
            height: light_map_height as u32,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::R32Float
    );

    for y in 0..world.layer.underground * SUBDIVISION {
        for x in 0..light_map_width {
            let block_exists = world.solid_block_exists((x / SUBDIVISION, y / SUBDIVISION));
            let wall_exists = world.wall_exists((x / SUBDIVISION, y / SUBDIVISION));
            
            if block_exists || wall_exists {
                set_light(x, y, 0., &mut light_map);
            } else {
                set_light(x, y, 1., &mut light_map);
            }
        }
    }

    blur(URect::new(0, 0, light_map_width as u32, light_map_height as u32), &mut light_map, world);

    light_map
}

fn set_light(x: usize, y: usize, light: f32, light_map: &mut Image) {
    let index = (y * light_map.texture_descriptor.size.width as usize + x) * 4;
    let bytes = light.to_le_bytes();

    light_map.data[index] = bytes[0];
    light_map.data[index + 1] = bytes[1];
    light_map.data[index + 2] = bytes[2];
    light_map.data[index + 3] = bytes[3];
}

fn get_light(x: usize, y: usize, light_map: &Image) -> f32 {
    let index = (y * light_map.texture_descriptor.size.width as usize + x) * 4;
    f32::from_le_bytes([light_map.data[index], light_map.data[index + 1], light_map.data[index + 2], light_map.data[index + 3]])
}

pub(crate) fn blur(area: URect, light_map: &mut LightMap, world: &WorldData) {
    // Top to bottom
    for x in area.min.x..area.max.x {
        for y in area.min.y..area.max.y {
            propagate_light(x as usize, y as usize, light_map, world, PassDirection::TopToBottom);
        }
    }

    // Left to right
    for y in area.min.y..area.max.y {
        for x in area.min.x..area.max.x {
            propagate_light(x as usize, y as usize, light_map, world, PassDirection::LeftToRight);
        }
    }

    // Right to left
    for y in area.min.y..area.max.y {
        for x in (area.min.x..area.max.x).rev() {
            propagate_light(x as usize, y as usize, light_map, world, PassDirection::RightToLeft);
        }
    }

    // Bottom to top
    for x in area.min.x..area.max.x {
        for y in (area.min.y..area.max.y).rev() {
            propagate_light(x as usize, y as usize, light_map, world, PassDirection::BottomToTop);
        }
    }
}

pub(crate) fn propagate_light(x: usize, y: usize, light_map: &mut LightMap, world: &WorldData, direction: PassDirection) { 
    let offset = direction.to_ivec2();

    if (x as i32 + offset.x) as u32 >= light_map.texture_descriptor.size.width { return; }
    if (y as i32 + offset.y) as u32 >= light_map.texture_descriptor.size.height { return; }

    if x as i32 + offset.x < 0 { return; }
    if y as i32 + offset.y < 0 { return; }

    let neighbor_world_pos = (
        ((x as i32 + offset.x) as usize / SUBDIVISION),
        ((y as i32 + offset.y) as usize / SUBDIVISION),
    );

    let neighbor_pos = (
        (x as i32 + offset.x) as usize,
        (y as i32 + offset.y) as usize,
    );

    let decay = if world.solid_block_exists(neighbor_world_pos) { 0.4 } else { 0.91 };

    let this_light = get_light(x, y, light_map);
    let neighbor_light = get_light(neighbor_pos.0, neighbor_pos.1, light_map);

    if this_light < neighbor_light {
        set_light(x, y, neighbor_light * decay, light_map);
    }
}