use std::collections::VecDeque;

use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::tiles::TilePos;
use ndarray::prelude::*;
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use noise::{Perlin, MultiFractal, Fbm, OpenSimplex};
use rand::{rngs::StdRng, Rng, SeedableRng};
use simdnoise::NoiseBuilder;

use crate::common::math::map_range_f32;
use crate::world::block::BlockType;

use super::block::Block;
use super::wall::Wall;
use super::tree::{TreeType, TreeFrameType};
use super::{WorldSize, WorldData, Layer, BlockArray, WallArray, AsWorldPos};

pub(crate) const DIRT_HILL_HEIGHT: usize = 75;

macro_rules! tree {
    ($tree_type: path, $frame_type: ident, $variant: ident) => {
        super::block::Block {
            block_type: super::block::BlockType::Tree(super::tree::Tree::new($tree_type, $frame_type, $variant)),
            hp: super::block::BlockType::Tree(super::tree::Tree::new($tree_type, $frame_type, $variant)).max_health(),
        }
        
    };
}

pub(crate) fn generate_world(seed: u32, world_size: WorldSize) -> WorldData {
    println!("Generating world...");

    let world_size = world_size.size();

    let blocks = BlockArray::default((world_size.height, world_size.width));
    let walls = WallArray::default((world_size.height, world_size.width));

    let surface = world_size.height / 10;
    let underground = (world_size.height as f32 / 3.) as usize;
    let cavern = (world_size.height as f32 / 2.) as usize;

    let layer = Layer {
        surface,
        underground,
        cavern,
        dirt_height: (underground - surface) / 4
    };

    let mut world = WorldData {
        blocks, 
        walls,
        size: world_size,
        layer,
        spawn_point: TilePos::new(0, 0)
    };

    spawn_terrain(&mut world);

    make_hills(&mut world, seed);

    generate_walls(&mut world);

    generate_big_caves(&mut world, seed);

    generate_small_caves(&mut world, seed);   

    generate_dirt_in_rocks(&mut world, seed);

    grassify(&mut world);

    generate_rocks_in_dirt(&mut world, seed);

    rough_cavern_layer_border(&mut world, seed);

    remove_walls_from_surface(&mut world);

    grow_trees(&mut world, seed);

    set_spawn_point(&mut world);

    world
}

fn spawn_terrain(world: &mut WorldData) {
    println!("Generating terrain...");

    for ((y, _), block) in world.blocks.indexed_iter_mut() {
        if y >= world.layer.surface {
            *block = None;
        }

        if y >= world.layer.underground - world.layer.dirt_height {
            *block = Some(Block::Dirt);
        }

        if y >= world.layer.underground {
            *block = Some(Block::Stone);
        }
    }
}

fn make_hills(world: &mut WorldData, seed: u32) {
    println!("Making hills...");

    let mut rng = StdRng::seed_from_u64(seed as u64);

    let level = world.layer.underground - world.layer.dirt_height;
    
    let fbm = NoiseBuilder::fbm_1d(world.size.width)
        .with_seed(rng.gen())
        .with_freq(0.005)
        .with_octaves(3)
        .generate_scaled(0., 1.);

    let gradient = NoiseBuilder::gradient_1d(world.size.width)
        .with_seed(rng.gen())
        .with_freq(0.01)
        .generate_scaled(0., 1.);

    for i in 0..world.size.width {
        let block_x = i;
        let noise_value = fbm[i] * gradient[i];

        let hill_height = level - (noise_value * DIRT_HILL_HEIGHT as f32) as usize;
        world.blocks.slice_mut(s![hill_height..level, block_x]).fill(Some(Block::Dirt));
    }
}

fn rough_cavern_layer_border(world: &mut WorldData, seed: u32) {
    let mut rng = StdRng::seed_from_u64(seed as u64);

    let level = world.layer.underground;
    
    let noise = NoiseBuilder::gradient_1d(world.size.width)
        .with_seed(rng.gen())
        .with_freq(0.1)
        .generate_scaled(-1., 0.);

    const ROUGHNESS: f32 = 20.;

    for (block_x, noise_value) in noise.iter().enumerate() {
        let x_offset = {
            let offset = rng.gen_range(-5f32..5f32);

            ((block_x as f32 + offset) as usize).clamp(0, world.size.width - 1)
        };

        let x_range = if block_x < x_offset {
            block_x..=x_offset
        } else {
            x_offset..=block_x
        };

        let hill_height = level - (noise_value.abs() * ROUGHNESS) as usize;
        
        if block_x != x_offset {
            world.blocks.slice_mut(s![level..hill_height, x_range]).fill(Some(Block::Dirt));
        } else {
            world.blocks.slice_mut(s![level..hill_height, block_x]).fill(Some(Block::Dirt));
        }
    }
}

fn generate_walls(world: &mut WorldData) {
    println!("Generating walls...");

    let dirt_level = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;
    let underground_level = world.layer.underground;

    for ((y, x), wall) in world.walls.slice_mut(s![dirt_level..underground_level, ..]).indexed_iter_mut() {
        let block_not_exists = |y: usize, x: usize| -> bool {
            world.blocks.get((y, x)).and_then(|b| b.as_ref()).is_none()
        };

        let prev_x = x.saturating_sub(1);
        let next_x = (x + 1).clamp(0, world.size.width);

        if block_not_exists(dirt_level + y - 1, x) { continue; }
        if block_not_exists(dirt_level + y + 1, x) { continue; }
        if block_not_exists(dirt_level + y, prev_x) { continue; }
        if block_not_exists(dirt_level + y, next_x) { continue; }
        if block_not_exists(dirt_level + y - 1, prev_x) { continue; }
        if block_not_exists(dirt_level + y + 1, prev_x) { continue; }
        if block_not_exists(dirt_level + y + 1, next_x) { continue; }
        if block_not_exists(dirt_level + y - 1, next_x) { continue; }

        *wall = Some(Wall::Dirt);
    }
}

fn generate_dirt_in_rocks(world: &mut WorldData, seed: u32) {
    println!("Generating rocks in dirt...");
    let underground_level = world.layer.underground;
    let cavern_level = world.layer.cavern;

    generate_dirt(world, seed, underground_level, cavern_level, 0.2, 0.4, 0.8);
    generate_dirt(world, seed, cavern_level, world.size.height, 0.3, 0.72, 0.72);
}

fn generate_dirt(world: &mut WorldData, seed: u32, from: usize, to: usize, freq: f32, min_prevalence: f32, max_prevalence: f32) {
    let mut slice = world.blocks.slice_mut(s![from..to, ..]);

    let height = slice.nrows();

    let noise = NoiseBuilder::fbm_2d(slice.ncols(), height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .generate_scaled(0., 1.);

    for ((y, x), block) in slice.indexed_iter_mut() {
        let index = (y * world.size.width) + x;

        let a = map_range_f32(0., height as f32, min_prevalence, max_prevalence, y as f32);

        let noise_value = noise[index];

        if noise_value >= a {
            if let Some(Block::Stone) = block {
                *block = Some(Block::Dirt);
            }
        }
    }
}

fn generate_rocks_in_dirt(world: &mut WorldData, seed: u32) {
    println!("Generating dirt in rocks...");

    let dirt_level = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;
    let underground_level = world.layer.underground;

    let noise = NoiseBuilder::fbm_2d(world.size.width, underground_level - dirt_level)
        .with_seed(seed as i32)
        .with_freq(0.15)
        .generate_scaled(-1., 1.);

    let mut slice = world.blocks.slice_mut(s![dirt_level..underground_level, ..]);

    for ((y, x), block) in slice.indexed_iter_mut() {
        let index = (y * world.size.width) + x;

        let noise_value = noise[index];

        if noise_value >= 0.5 {
            let block_type = block.map(|b| b.block_type);
            if matches!(block_type, Some(BlockType::Dirt | BlockType::Grass)) {
                *block = Some(Block::Stone);
            }
        }
    }
}

fn generate_big_caves(world: &mut WorldData, seed: u32) {
    println!("Generating big caves...");

    let dirt_level = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;

    let height = world.size.height - dirt_level;

    let noise = Fbm::<Perlin>::new(seed);

    let noise_map = PlaneMapBuilder::<_, 2>::new(noise)
        .set_size(world.size.width, height)
        .set_x_bounds(-30., 30.)
        .set_y_bounds(-15., 15.)
        .build();

    for ((y, x), block) in world.blocks.slice_mut(s![dirt_level.., ..]).indexed_iter_mut() {
        let noise_value = noise_map.get_value(x, y) as f32;

        if noise_value < -0.5 {
            *block = None;
        }
    }
}

fn generate_small_caves(world: &mut WorldData, seed: u32) {
    println!("Generating small caves...");

    let underground_level = world.layer.underground;

    let noise = Fbm::<OpenSimplex>::new(seed)
        .set_lacunarity(2.42)
        .set_frequency(1.5);

    let noise_map = PlaneMapBuilder::<_, 2>::new(noise)
        .set_size(world.size.width, world.size.height - underground_level + 10)
        .set_x_bounds(-60., 60.)
        .set_y_bounds(-30., 30.)
        .build();

    for ((y, x), block) in world.blocks.slice_mut(s![underground_level..world.size.height - 10, ..]).indexed_iter_mut() {
        let noise_value = noise_map.get_value(x, y);

        if noise_value < -0.3 {
            *block = None; 
        }
    }
}

fn grassify(world: &mut WorldData) {
    println!("Growing grass...");

    fn is_valid(world: &mut WorldData, x: usize, y: usize) -> bool {
        if x >= world.size.width { return false; }
        if y >= world.size.height { return false; }
        if !world.block_exists_with_type((x, y), BlockType::Dirt) { return false; }

        any_neighbor_not_exist(&world, x, y)
    }
    
    fn flood_fill(world: &mut WorldData, x: usize, y: usize) {
        let mut queue = VecDeque::new();
        queue.push_back((x, y));

        world.set_block((x, y), &Block::Grass);
    
        while !queue.is_empty() {
            let (x, y) = queue[queue.len() - 1];
            queue.pop_back();

            let prev_x = x.saturating_sub(1);

            if is_valid(world, x + 1, y) {
                let pos = (x + 1, y);
                world.set_block(pos, &Block::Grass);
                queue.push_back(pos);
            }
    
            if is_valid(world, prev_x, y) {
                let pos = (prev_x, y);
                world.set_block(pos, &Block::Grass);
                queue.push_back(pos);
            }
    
            if is_valid(world, x, y + 1) {
                let pos = (x, y + 1);
                world.set_block(pos, &Block::Grass);
                queue.push_back(pos);
            }
    
            if is_valid(world, x, y - 1) {
                let pos = (x, y - 1);
                world.set_block(pos, &Block::Grass);
                queue.push_back(pos);
            }

            if is_valid(world, prev_x, y - 1) {
                let pos = (prev_x, y - 1);
                world.set_block(pos, &Block::Grass);
                queue.push_back(pos);
            }

            if is_valid(world, x + 1, y - 1) {
                let pos = (x + 1, y - 1);
                world.set_block(pos, &Block::Grass);
                queue.push_back(pos);
            }

            if is_valid(world, prev_x, y + 1) {
                let pos = (prev_x, y + 1);
                world.set_block(pos, &Block::Grass);
                queue.push_back(pos);
            }

            if is_valid(world, x + 1, y + 1) {
                let pos = (x + 1, y + 1);
                world.set_block(pos, &Block::Grass);
                queue.push_back(pos);
            }
        }
    }

    for x in 0..world.size.width {
        let y = get_surface_block_y(world, x);
        if world.block_exists_with_type((x, y), BlockType::Dirt) {
            flood_fill(world, x, y);
        }
    }

}

fn remove_walls_from_surface(world: &mut WorldData) {
    fn is_valid(world: &mut WorldData, x: usize, y: usize) -> bool {
        if x >= world.size.width { return false; }
        if y >= world.size.height { return false; }

        if !world.wall_exists((x, y)) { return false; }

        if any_neighbor_not_exist(&world, x, y) { return true; }

        if world.solid_block_exists((x, y)) { return false; }

        return true;
    }
    
    fn flood_fill(world: &mut WorldData, x: usize, y: usize) {
        let mut queue = VecDeque::new();
        queue.push_back(((x, y), (0i32, 0i32)));

        world.remove_wall((x, y));
    
        while !queue.is_empty() {
            let ((x, y), (depth_x, depth_y)) = queue[queue.len() - 1];
            queue.pop_back();

            if depth_x.abs() >= depth_y / 2 + 5 { continue; }

            let prev_x = x.saturating_sub(1);

            if is_valid(world, x + 1, y) {
                let pos = (x + 1, y);
                world.remove_wall(pos);
                queue.push_back((pos, (depth_x + 1, depth_y)));
            }
    
            if is_valid(world, prev_x, y) {
                let pos = (prev_x, y);
                world.remove_wall(pos);
                queue.push_back((pos, (depth_x - 1, depth_y)));
            }
    
            if is_valid(world, x, y + 1) {
                let pos = (x, y + 1);
                world.remove_wall(pos);
                queue.push_back((pos, (depth_x, depth_y + 1)));
            }
    
            if is_valid(world, x, y - 1) {
                let pos = (x, y - 1);
                world.remove_wall(pos);
                queue.push_back((pos, (depth_x, depth_y - 1)));
            }
        }
    }

    for x in 0..world.size.width {
        let y = get_surface_wall_y(world, x);

        if world.solid_block_exists((x, y)) { continue; }

        flood_fill(world, x, y);
    }
}

fn grow_tree(world: &mut WorldData, rng: &mut StdRng, root_pos: impl AsWorldPos) {
    let height: usize = rng.gen_range(5..=16);

    let left_base = rng.gen_bool(0.5);
    let right_base = rng.gen_bool(0.5);

    let root_pos_x = root_pos.x();
    let root_pos_y = root_pos.y();

    if root_pos_x >= world.size.width - 2 || root_pos_x <= 2 {
        return;
    }

    let left_block = world.block_exists((root_pos_x - 1, root_pos_y + 1));
    let right_block = world.block_exists((root_pos_x + 1, root_pos_y + 1));

    // Check enough space
    {
        let left = if left_block { 2 } else { 1 };
        let right = if right_block { 2 } else { 1 };

        // Check enough space for base
        for block in world.blocks.slice(s![root_pos_y, (root_pos_x - left)..=(root_pos_x + right)]).iter() {
            if block.is_some() {
                return;
            }
        }
        
        // Check enough space for branches 
        for block in world.blocks.slice(s![(root_pos_y - 16)..root_pos_y, (root_pos_x - 2)..=(root_pos_x + 2)]).iter() {
            if block.is_some() {
                return;
            }
        }
    }

    // Base
    if left_base && right_base && left_block && right_block {
        {
            let frame_type = TreeFrameType::BasePlainLeft;
            let variant = rng.gen_range(0..3);
            world.blocks[(root_pos_y, root_pos_x - 1)] = Some(tree!(TreeType::Forest, frame_type, variant));
        }
        {
            let frame_type = TreeFrameType::BasePlainAD;
            let variant = rng.gen_range(0..3);
            world.blocks[(root_pos_y, root_pos_x)] = Some(tree!(TreeType::Forest, frame_type, variant));
        }
        {
            let frame_type = TreeFrameType::BasePlainRight;
            let variant = rng.gen_range(0..3);
            world.blocks[(root_pos_y, root_pos_x + 1)] = Some(tree!(TreeType::Forest, frame_type, variant));
        }
    } else if left_base && left_block {
        {
            let frame_type = TreeFrameType::BasePlainLeft;
            let variant = rng.gen_range(0..3);
            world.blocks[(root_pos_y, root_pos_x - 1)] = Some(tree!(TreeType::Forest, frame_type, variant));
        }
        {
            let frame_type = TreeFrameType::BasePlainA;
            let variant = rng.gen_range(0..3);
            world.blocks[(root_pos_y, root_pos_x)] = Some(tree!(TreeType::Forest, frame_type, variant));
        }
    } else if right_base && right_block {
        {
            let frame_type = TreeFrameType::BasePlainD;
            let variant = rng.gen_range(0..3);
            world.blocks[(root_pos_y, root_pos_x)] = Some(tree!(TreeType::Forest, frame_type, variant));
        }
        {
            let frame_type = TreeFrameType::BasePlainRight;
            let variant = rng.gen_range(0..3);
            world.blocks[(root_pos_y, root_pos_x + 1)] = Some(tree!(TreeType::Forest, frame_type, variant));
        }
    } else {
        let frame_type = TreeFrameType::TrunkPlain;
        let variant = rng.gen_range(0..3);
        world.blocks[(root_pos_y, root_pos_x)] = Some(tree!(TreeType::Forest, frame_type, variant));
    }

    // Trunk
    {
        let frame_type = TreeFrameType::TrunkPlain;
        let variant = rng.gen_range(0..3);
        world.blocks
            .slice_mut(s![root_pos_y - height..root_pos_y, root_pos_x])
            .fill(Some(tree!(TreeType::Forest, frame_type, variant)));
    }

    // Branches
    
    // Left side
    for y in root_pos_y - height..root_pos_y {
        let place = rng.gen_bool(1. / 10.);
        let bare = rng.gen_bool(1. / 5.);

        if place && world.blocks[(y - 1, root_pos_x - 1)].is_none() {
            let frame_type = if bare {
                TreeFrameType::BranchLeftBare
            } else {
                TreeFrameType::BranchLeftLeaves
            };

            let variant = rng.gen_range(0..3);

            world.blocks[(y, root_pos_x - 1)] = Some(tree!(TreeType::Forest, frame_type, variant));
        }
    }

    // Right side
    for y in root_pos_y - height..root_pos_y {
        let place = rng.gen_bool(1. / 10.);
        let bare = rng.gen_bool(1. / 5.);

        if place && world.blocks[(y - 1, root_pos_x + 1)].is_none() {
            let frame_type = if bare {
                TreeFrameType::BranchRightBare
            } else {
                TreeFrameType::BranchRightLeaves
            };

            let variant = rng.gen_range(0..3);

            world.blocks[(y, root_pos_x + 1)] = Some(tree!(TreeType::Forest, frame_type, variant));
        }
    }

    // Top
    let bare = rng.gen_bool(1. / 5.);
    let frame_type = if bare {
        let jagged = rng.gen_bool(1. - 1./3.);
        if jagged {
            TreeFrameType::TopBareJagged
        } else {
            TreeFrameType::TopBare
        }
    } else {
        TreeFrameType::TopLeaves
    };

    let variant = rng.gen_range(0..3);

    world.blocks[(root_pos_y - height - 1, root_pos_x)] = Some(tree!(TreeType::Forest, frame_type, variant));
}

fn grow_trees(world: &mut WorldData, seed: u32) {
    println!("Growing trees...");

    let mut rng = StdRng::seed_from_u64(seed as u64);

    for x in 0..world.size.width {
        let y = get_surface_block_y(world, x);

        let grow = rng.gen_bool(1. / 5.);

        if grow {
            // Trees can only grow on dirt or grass
            let appropriate_block = world.get_block((x, y))
                .filter(|b| match b.block_type {
                    BlockType::Dirt | BlockType::Grass => true,
                    _ => false
                })
                .is_some();

            if appropriate_block {
                grow_tree(world, &mut rng, (x, y - 1))
            }
        }
    }

}

fn get_surface_block_y(world: &mut WorldData, x: usize) -> usize {
    let mut y = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;

    while y < world.size.height {
        if world.solid_block_exists((x, y)) {
            break;
        }

        y += 1;
    }

    y
}

fn get_surface_wall_y(world: &mut WorldData, x: usize) -> usize {
    let mut y = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;

    while y < world.size.height {
        if world.wall_exists((x, y)) {
            break;
        }

        y += 1;
    }

    y
}

fn set_spawn_point(world: &mut WorldData) {
    let x = world.size.width / 2;
    let y = get_surface_block_y(world, x);

    world.spawn_point = TilePos::new(x as u32, y as u32);
}

fn any_neighbor_not_exist(world: &WorldData, x: usize, y: usize) -> bool {
    Neighbors::get_square_neighboring_positions(
        &TilePos::new(x as u32, y as u32),
        &world.size.as_tilemap_size(),
        true
    )
    .iter()
    .any(|pos| !world.solid_block_exists(pos))
}