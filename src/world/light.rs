use bevy::{prelude::Image, math::URect, render::render_resource::{Extent3d, TextureDimension, TextureFormat}, utils::{default, syncunsafecell::SyncUnsafeCell}};
use rayon::prelude::{ParallelIterator, IntoParallelIterator};

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

    let area = URect::new(0, 0, light_map_width as u32, light_map_height as u32);
    scan(&mut light_map, world, area);

    light_map
}

pub(crate) fn blur(light_map: &mut Image, world: &WorldData, area: URect) {
    let width = light_map.texture_descriptor.size.width as usize;
    let height = light_map.texture_descriptor.size.height as usize;

    let cell = SyncUnsafeCell::new(&mut light_map.data);

    let min_y = area.min.y as usize;
    let max_y = (area.max.y as usize).min(height - 1);
    let min_x = area.min.x as usize;
    let max_x = area.max.x as usize;

    // Top to bottom
    (min_x..max_x)
        .into_par_iter()
        .for_each(|x| {
            let start = min_y * width + x;
            let end = max_y * width + x;

            let r = unsafe { &mut *cell.get() };

            blur_line(r, world, start, end, width as i32, width);
        });

    // Left to right
    (min_y..max_y)
        .into_par_iter()
        .for_each(|y| {
            let start = y * width + min_x;
            let end = y * width + max_x;

            let r = unsafe { &mut *cell.get() };

            blur_line(r, world, start, end, 1, width);
        });

    // Bottom to top
    (min_x..max_x)
        .into_par_iter()
        .for_each(|x| {
            let start = max_y * width + x;
            let end = min_y * width + x;

            let r = unsafe { &mut *cell.get() };

            blur_line(r, world, start, end, -(width as i32), width);
        });
        
    // Right to left
    (min_y..max_y)
        .into_par_iter()
        .for_each(|y| {
            let start = y * width + max_x;
            let end = y * width + min_x;

            let r = unsafe { &mut *cell.get() };

            blur_line(r, world, start, end, -1, width);
        });
}

fn blur_line(light_map: &mut [u8], world: &WorldData, start: usize, end: usize, stride: i32, width: usize) {
    let mut prev_light = light_map[start] as f32 / u8::MAX as f32;

    let mut index = (start as i32 + stride) as usize;
    let mut decay = {
        let x = (start % width) / SUBDIVISION;
        let y = (start / width) / SUBDIVISION;
        if world.solid_block_exists((x, y)) { DECAY_THROUGH_SOLID } else { DECAY_THROUGH_AIR }
    };

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

fn set_light(x: usize, y: usize, light: f32, light_map: &mut Image) {
    let index = y * light_map.texture_descriptor.size.width as usize + x;
    light_map.data[index] = (light * u8::MAX as f32) as u8;
}

pub(crate) fn scan(light_map: &mut LightMap, world: &WorldData, area: URect) {
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