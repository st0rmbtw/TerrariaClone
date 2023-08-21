use bevy::prelude::IVec2;

use ndarray::Array2;

use super::WorldData;

type LightMap = Array2::<u8>;

pub(crate) const CLUSTER_SIZE: usize = 1;

#[allow(dead_code)]
pub(crate) fn generate_light_map(world: &WorldData) -> LightMap {
    println!("Generating light map...");

    let light_map_width = world.size.width;
    let light_map_height = world.size.height;

    let mut light_map = LightMap::default((light_map_height, light_map_width));

    for y in 0..light_map_height {
        for x in 0..light_map_width {
            let block = world.get_block((x, y));
            let wall = world.get_wall((x, y));
            
            if block.is_some() || wall.is_some() {
                light_map[(y, x)] = 0;
            } else {
                light_map[(y, x)] = 255;
            }
        }
    }

    // Left to right
    for y in 0..light_map_height {
        for x in 0..light_map_width {
            propagate_light(x, y, &mut light_map, world, IVec2::new(-1, 0));
        }
    }

    // Top to bottom
    for x in (0..light_map_width).rev() {
        for y in 0..light_map_height {
            propagate_light(x, y, &mut light_map, world, IVec2::new(0, -1));
        }
    }

    // Right to left
    for y in 0..light_map_height {
        for x in (0..light_map_width).rev() {
            propagate_light(x, y, &mut light_map, world, IVec2::new(1, 0));
        }
    }

    // Bottom to top
    for x in 0..light_map_width {
        for y in (0..light_map_height).rev() {
            propagate_light(x, y, &mut light_map, world, IVec2::new(0, 1));
        }
    }

    light_map
}

pub(crate) fn propagate_light(x: usize, y: usize, light_map: &mut LightMap, world: &WorldData, offset: IVec2) { 
    if x >= light_map.ncols() - 1 { return; }
    if y >= light_map.nrows() - 1 { return; }

    if x.checked_sub(1).is_none() { return; }
    if y.checked_sub(1).is_none() { return; }

    let neighbor_pos = (
        (x as i32 + offset.x) as usize,
        (y as i32 + offset.y) as usize,
    );

    let decay = if world.solid_block_exists(neighbor_pos) { 
        50
    } else if world.wall_exists(neighbor_pos) {
        25
    } else {
        0
    };

    let this_light = light_map[(y, x)];
    let neighbor_light = light_map[(neighbor_pos.1, neighbor_pos.0)];

    if neighbor_light > 0 && this_light == 0 {
        light_map[(y, x)] = neighbor_light.saturating_sub(decay);
    }
}