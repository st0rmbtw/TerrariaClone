use bevy::prelude::IVec2;

use ndarray::Array2;

use super::WorldData;

type LightMap = Array2::<f32>;

pub(crate) const SUBDIVISION: usize = 1;

#[derive(Clone, Copy)]
pub(crate) enum PassDirection {
    LeftToRight,
    TopToBottom,
    RightToLeft,
    BottomToTop
}

impl PassDirection {
    pub(crate) const fn to_ivec2(&self) -> IVec2 {
        match self {
            PassDirection::LeftToRight => IVec2::new(-1, 0),
            PassDirection::TopToBottom => IVec2::new(0, -1),
            PassDirection::RightToLeft => IVec2::new(1, 0),
            PassDirection::BottomToTop => IVec2::new(0, 1),
        }
    }
}

#[allow(dead_code)]
pub(crate) fn generate_light_map(world: &WorldData) -> LightMap {
    println!("Generating light map...");

    let light_map_width = world.size.width * SUBDIVISION;
    let light_map_height = world.size.height * SUBDIVISION;

    let mut light_map = LightMap::default((light_map_height, light_map_width));

    for y in 0..light_map_height {
        for x in 0..light_map_width {
            let block = world.get_block((x / SUBDIVISION, y / SUBDIVISION));
            let wall = world.get_wall((x / SUBDIVISION, y / SUBDIVISION));
            
            if block.is_some() || wall.is_some() {
                light_map[(y, x)] = 0.;
            } else {
                light_map[(y, x)] = 1.;
            }
        }
    }

    // Left to right
    for y in 0..light_map_height {
        for x in 0..light_map_width {
            propagate_light(x, y, &mut light_map, world, PassDirection::LeftToRight);
        }
    }

    // Top to bottom
    for x in 0..light_map_width {
        for y in 0..light_map_height {
            propagate_light(x, y, &mut light_map, world, PassDirection::TopToBottom);
        }
    }

    // Right to left
    for y in 0..light_map_height {
        for x in (0..light_map_width).rev() {
            propagate_light(x, y, &mut light_map, world, PassDirection::RightToLeft);
        }
    }

    // Bottom to top
    for x in 0..light_map_width {
        for y in (0..light_map_height).rev() {
            propagate_light(x, y, &mut light_map, world, PassDirection::BottomToTop);
        }
    }

    light_map
}

pub(crate) fn propagate_light(x: usize, y: usize, light_map: &mut LightMap, world: &WorldData, direction: PassDirection) { 
    if x >= light_map.ncols() - 1 { return; }
    if y >= light_map.nrows() - 1 { return; }

    if (x / SUBDIVISION).checked_sub(1).is_none() { return; }
    if (y / SUBDIVISION).checked_sub(1).is_none() { return; }

    let offset = direction.to_ivec2();

    let neighbor_world_pos = (
        ((x / SUBDIVISION) as i32 + offset.x) as usize,
        ((y / SUBDIVISION) as i32 + offset.y) as usize,
    );

    let neighbor_pos = (
        (x as i32 + offset.x) as usize,
        (y as i32 + offset.y) as usize,
    );

    let decay = if world.solid_block_exists(neighbor_world_pos) { 0.56 } else { 0.91 };

    let this_light = light_map[(y, x)];
    let neighbor_light = light_map[(neighbor_pos.1, neighbor_pos.0)];

    if this_light < neighbor_light {
        light_map[(y, x)] = neighbor_light * decay;
    }
}