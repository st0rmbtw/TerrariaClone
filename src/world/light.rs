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
    pub(crate) const fn to_ivec2(self) -> IVec2 {
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
            if y >= world.layer.underground {
                light_map[(y, x)] = 0.;
                continue;
            }

            let block_exists = world.solid_block_exists((x / SUBDIVISION, y / SUBDIVISION));
            let wall_exists = world.wall_exists((x / SUBDIVISION, y / SUBDIVISION));
            
            if block_exists || wall_exists {
                light_map[(y, x)] = 0.;
            } else {
                light_map[(y, x)] = 1.;
            }
        }
    }

    // Top to bottom
    for x in 0..light_map_width {
        for y in 0..light_map_height {
            propagate_light(x, y, &mut light_map, world, PassDirection::TopToBottom);
        }
    }

    // Left to right
    for y in 0..light_map_height {
        for x in 0..light_map_width {
            propagate_light(x, y, &mut light_map, world, PassDirection::LeftToRight);
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
    let offset = direction.to_ivec2();

    if ((x / SUBDIVISION) as i32 + offset.x) as usize >= light_map.ncols() { return; }
    if ((y / SUBDIVISION) as i32 + offset.y) as usize >= light_map.nrows() { return; }

    if (x / SUBDIVISION) as i32 + offset.x < 0 { return; }
    if (y / SUBDIVISION) as i32 + offset.y < 0 { return; }

    let neighbor_world_pos = (
        ((x as i32 + offset.x) as usize / SUBDIVISION),
        ((y as i32 + offset.y) as usize / SUBDIVISION),
    );

    let neighbor_pos = (
        (x as i32 + offset.x) as usize,
        (y as i32 + offset.y) as usize,
    );

    let decay = if world.solid_block_exists(neighbor_world_pos) { 0.4 } else { 0.91 };

    let this_light = light_map[(y, x)];
    let neighbor_light = light_map[(neighbor_pos.1, neighbor_pos.0)];

    if this_light < neighbor_light {
        light_map[(y, x)] = neighbor_light * decay;
    }
}