use bevy::{prelude::Image, math::URect, render::render_resource::{Extent3d, TextureDimension, TextureFormat}, utils::default};

use super::WorldData;

type LightMap = Image;

pub(crate) const SUBDIVISION: usize = 2;
const DECAY_THROUGH_SOLID: f32 = 0.56;
const DECAY_THROUGH_AIR: f32 = 0.91;

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
        &[0],
        TextureFormat::R8Unorm
    );

    scan(URect::new(0, 0, light_map_width as u32, light_map_height as u32), &mut light_map, world);

    light_map
}

fn set_light(x: usize, y: usize, light: f32, light_map: &mut Image) {
    let index = y * light_map.texture_descriptor.size.width as usize + x;
    light_map.data[index] = (light * u8::MAX as f32) as u8;
}

pub(crate) fn scan(area: URect, light_map: &mut LightMap, world: &WorldData) {
    let min_y = area.min.y as usize;
    let max_y = (area.max.y as usize).min(world.layer.underground * SUBDIVISION);

    let min_x = area.min.x as usize;
    let max_x = area.max.x as usize;

    for y in min_y..max_y {
        for x in min_x..max_x {
            let block_exists = world.solid_block_exists((x / SUBDIVISION, y / SUBDIVISION));
            let wall_exists = world.wall_exists((x / SUBDIVISION, y / SUBDIVISION));
            
            if block_exists || wall_exists {
                set_light(x, y, 0., light_map);
            } else {
                set_light(x, y, 1., light_map);
            }
        }
    }
}

pub(crate) fn blur(area: URect, light_map: &mut LightMap, world: &WorldData) {
    let width = light_map.texture_descriptor.size.width as usize;

    let min_x = area.min.x as usize;
    let max_x = area.max.x as usize;
    let min_y = area.min.y as usize;
    let max_y = area.max.y as usize;

    // Top to bottom
    for x in min_x..max_x {
        let start = min_y * width + x;
        let end = max_y * width + x;
        blur_line(&mut light_map.data, world, start, end, width as i32, width);
    }

    // Left to right
    for y in min_y..max_y {
        let start = y * width + min_x;
        let end = y * width + max_x;
        blur_line(&mut light_map.data, world, start, end, 1, width);
    }

    // Bottom to top
    for x in min_x..max_x {
        let start = max_y * width + x;
        let end = min_y * width + x;
        blur_line(&mut light_map.data, world, start, end, -(width as i32), width);
    }

    // Right to left
    for y in min_y..max_y {
        let start = y * width + max_x;
        let end = y * width + min_x;

        blur_line(&mut light_map.data, world, start, end, -1, width);
    }

}

pub(crate) fn blur_line(light_map: &mut [u8], world: &WorldData, start: usize, end: usize, stride: i32, width: usize) { 
    let mut prev_light = light_map[start] as f32 / u8::MAX as f32;
    let mut decay = {
        let x = (start % width) / SUBDIVISION;
        let y = (start / width) / SUBDIVISION;
        if world.solid_block_exists((x, y)) { DECAY_THROUGH_SOLID } else { DECAY_THROUGH_AIR }
    };

    let mut index = (start as i32 + stride) as usize;
    loop {
        if stride > 0 && index >= end {
            break;
        } else if stride < 0 && index <= end {
            break;
        }

        let cur_light = light_map[index] as f32 / u8::MAX as f32;

        if cur_light < prev_light {
            let new_light = prev_light * decay;
            light_map[index] = (new_light * u8::MAX as f32) as u8;
            prev_light = new_light;
        } else {
            prev_light = cur_light;
        }

        decay = {
            let x = (index % width) / SUBDIVISION;
            let y = (index / width) / SUBDIVISION;
            if world.solid_block_exists((x, y)) { DECAY_THROUGH_SOLID } else { DECAY_THROUGH_AIR }
        };
        index = (index as i32 + stride) as usize;
    }
}