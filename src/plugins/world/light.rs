use ndarray::Array2;
use super::world::WorldData;

type LightMap = Array2::<u8>;

pub(super) fn generate_light_map(world: &WorldData) -> LightMap {
    let cluster_size = 1;

    let mut light_map = LightMap::default((world.size.height * cluster_size, world.size.width * cluster_size));

    let light_map_width = light_map.ncols();
    let light_map_height = light_map.nrows();

    for y in 0..light_map_height {
        for x in 0..light_map_width {
            let block = world.get_solid_block((x / cluster_size, y / cluster_size));
            let wall = world.get_wall((x / cluster_size, y / cluster_size));

            if wall.is_some() {
                light_map[(y, x)] = 30;
            } else if block.is_none() {
                light_map[(y, x)] = 255;
            }
        }
    }

    for y in 0..light_map_height {
        for x in 0..light_map_width {
            propagate_light(x, y, cluster_size, &mut light_map, world);
        }
    }

    for y in (0..light_map_height).rev() {
        for x in (0..light_map_width).rev() {
            propagate_light(x, y, cluster_size, &mut light_map, world);
        }
    }

    light_map
}

pub(crate) fn propagate_light(x: usize, y: usize, cluster_size: usize, light_map: &mut LightMap, world: &WorldData) { 
    if x >= world.size.width - 1 { return; }
    if y >= world.size.height - 1 { return; }

    if x.checked_sub(1).is_none() { return; }
    if y.checked_sub(1).is_none() { return; }

    let light_pass = if world.solid_block_exists((x / cluster_size, y / cluster_size)) { 
        50
    } else if world.wall_exists((x / cluster_size, y / cluster_size)) {
        38
    } else {
        light_map[(y, x)] = 255;
        return;
    };

    let neighbors = [
       light_map[(y, x - 1)],
       light_map[(y, x + 1)],
       light_map[(y + 1, x)],
       light_map[(y - 1, x)],
    ];

    let max_light = neighbors.iter().max().unwrap();

    light_map[(y, x)] = max_light.saturating_sub(light_pass);
}