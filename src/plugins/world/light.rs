use ndarray::Array2;
use super::world::WorldData;

type LightMap = Array2::<u8>;

pub fn generate_light_map(world: &WorldData) -> LightMap {
    let cluster_size = 1;

    let mut light_map = LightMap::default((world.size.height * cluster_size, world.size.width * cluster_size));

    let light_map_width = light_map.ncols();
    let light_map_height = light_map.nrows();

    for y in 0..light_map_height {
        for x in 0..light_map_width {
            let block = world.get_solid_block((x / cluster_size, y / cluster_size));
            let wall = world.get_wall((x / cluster_size, y / cluster_size));

            if block.is_some() {
                light_map[(y, x)] = 0;
            } else if wall.is_some() {
                light_map[(y, x)] = 30;
            } else {
                light_map[(y, x)] = 255;
            }
        }
    }

    for y in 0..light_map_height {
        for x in 0..light_map_width {
            propagate_light(x, y, cluster_size, &mut light_map, &world);
        }
    }

    for y in (0..light_map_height).rev() {
        for x in (0..light_map_width).rev() {
            propagate_light(x, y, cluster_size, &mut light_map, &world);
        }
    }

    light_map
}

fn propagate_light(x: usize, y: usize, cluster_size: usize, light_map: &mut Array2<u8>, world: &WorldData) { 
    if x >= light_map.ncols() - 1 { return; }
    if y >= light_map.nrows() - 1 { return; }

    if x.checked_sub(1).is_none() { return; }
    if y.checked_sub(1).is_none() { return; }

    let light_pass = if world.solid_block_exists((x / cluster_size, y / cluster_size)) { 
        36
    } else if world.wall_exists((x / cluster_size, y / cluster_size)) {
        40
    } else {
        return;
    };

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


    // if light_map[(y - 1, x - 1)] > light_map[(y, x)] { 
    //     light_map[(y, x)] = light_map[(y - 1, x - 1)].saturating_sub(light_pass);
    // }

    // if light_map[(y + 1, x + 1)] > light_map[(y, x)] { 
    //     light_map[(y, x)] = light_map[(y + 1, x + 1)].saturating_sub(light_pass);
    // }

    // if light_map[(y + 1, x - 1)] > light_map[(y, x)] { 
    //     light_map[(y, x)] = light_map[(y + 1, x - 1)].saturating_sub(light_pass);
    // }

    // if light_map[(y - 1, x + 1)] > light_map[(y, x)] { 
    //     light_map[(y, x)] = light_map[(y - 1, x + 1)].saturating_sub(light_pass);
    // }
}