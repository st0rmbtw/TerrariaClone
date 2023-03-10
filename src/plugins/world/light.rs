use ndarray::Array2;

use super::world::WorldData;


pub fn generate_light_map(world: &WorldData) -> Array2<u8> {
    let mut light_map = Array2::<u8>::default((world.size.height, world.size.width));

    for ((y, x), block) in world.blocks.indexed_iter() {
        if block.is_some() {
            light_map[[y, x]] = 0;
        } else {
            light_map[[y, x]] = 255;
        }
    }

    for y in 0..light_map.nrows() {
        for x in 0..light_map.ncols() {
            propagate_light(x, y, &mut light_map);
        }
    }

    light_map
}

fn propagate_light(x: usize, y: usize, light_map: &mut Array2<u8>) { 
    if x >= light_map.ncols() - 1 { return; }
    if y >= light_map.nrows() - 1 { return; }

    if x.checked_sub(1).is_none() { return; }
    if y.checked_sub(1).is_none() { return; }

    let light_pass = 50;

    if light_map[(y, x - 1)] > light_map[(y, x)] { 
        light_map[(y, x)] = light_map[(y, x - 1)].saturating_sub(light_pass);
    }

    if light_map[(y - 1, x)] > light_map[(y, x)] { 
        light_map[(y, x)] = light_map[(y - 1, x)].saturating_sub(light_pass);
    }

    if light_map[(y, x + 1)] > light_map[(y, x)] { 
        light_map[(y, x)] = light_map[(y, x + 1)].saturating_sub(light_pass);
    }

    if light_map[(y + 1, x)] > light_map[(y, x)] { 
        light_map[(y, x)] = light_map[(y + 1, x)].saturating_sub(light_pass);
    }
}