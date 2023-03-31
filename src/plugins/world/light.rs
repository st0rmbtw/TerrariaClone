use bevy_ecs_tilemap::tiles::TilePos;
use ndarray::Array2;
use super::world::WorldData;

type LightMap = Array2::<u8>;

pub(crate) const CLUSTER_SIZE: usize = 2;

pub(super) fn generate_light_map(world: &WorldData) -> LightMap {
    println!("Generating light map...");

    let light_map_width = world.size.width * CLUSTER_SIZE;
    let light_map_height = world.size.height * CLUSTER_SIZE;

    let mut light_map = LightMap::default((light_map_height, light_map_width));

    for y in 0..light_map_height {
        for x in 0..light_map_width {
            let block = world.get_solid_block((x / CLUSTER_SIZE, y / CLUSTER_SIZE));
            let wall = world.get_wall((x / CLUSTER_SIZE, y / CLUSTER_SIZE));

            if wall.is_some() {
                light_map[(y, x)] = 30;
            } else if block.is_none() {
                light_map[(y, x)] = 255;
            }
        }
    }

    for y in 0..light_map_height {
        for x in 0..light_map_width {
            propagate_light(x, y, &mut light_map, world);
        }
    }

    for y in (0..light_map_height).rev() {
        for x in (0..light_map_width).rev() {
            propagate_light(x, y, &mut light_map, world);
        }
    }

    light_map
}

pub(crate) fn propagate_light(x: usize, y: usize, light_map: &mut LightMap, world: &WorldData) { 
    if x >= light_map.ncols() - 1 { return; }
    if y >= light_map.nrows() - 1 { return; }

    if x.checked_sub(1).is_none() { return; }
    if y.checked_sub(1).is_none() { return; }

    let world_pos = TilePos::new((x / CLUSTER_SIZE) as u32, (y / CLUSTER_SIZE) as u32);

    let light_pass = if world.solid_block_exists(world_pos) { 
        50
    } else if world.wall_exists(world_pos) {
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

    light_map[(y, x)] = max_light.saturating_sub(light_pass / CLUSTER_SIZE as u8);
}