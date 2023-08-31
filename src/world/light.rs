use bevy::{prelude::IVec2, math::URect};

use ndarray::Array2;

use super::WorldData;

type LightMap = Array2::<f32>;

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

    let mut light_map = LightMap::default((light_map_height, light_map_width));

    for y in 0..light_map_height {
        for x in 0..light_map_width {
            if y / SUBDIVISION >= world.layer.underground {
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

    blur(URect::new(0, 0, light_map_width as u32, light_map_height as u32), &mut light_map, world);

    light_map
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

    if (x as i32 + offset.x) as usize >= light_map.ncols() { return; }
    if (y as i32 + offset.y) as usize >= light_map.nrows() { return; }

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

    let this_light = light_map[(y, x)];
    let neighbor_light = light_map[(neighbor_pos.1, neighbor_pos.0)];

    if this_light < neighbor_light {
        light_map[(y, x)] = neighbor_light * decay;
    }
}