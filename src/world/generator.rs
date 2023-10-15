use std::collections::VecDeque;

use bevy::math::URect;
use bevy::prelude::UVec2;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::TilemapSize;
use bevy_ecs_tilemap::tiles::TilePos;
use ndarray::prelude::*;
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use noise::{Perlin, MultiFractal, Fbm, OpenSimplex};
use rand::{rngs::StdRng, Rng, SeedableRng};
use simdnoise::NoiseBuilder;

use crate::common::math::map_range_f32;
use crate::world::block::BlockType;

use super::block::Block;
use super::wall::WallType;
use super::tree::{TreeType, TreeFrameType};
use super::{WorldSize, WorldData, Layer, BlockArray, WallArray, AsWorldPos};

pub(crate) const DIRT_HILL_HEIGHT: usize = 75;

macro_rules! tree {
    ($tree_type: expr, $frame_type: expr) => {
        super::block::BlockType::Tree(super::tree::Tree::new($tree_type, $frame_type))
    };
}

pub fn generate_world(seed: u32, world_size: WorldSize) -> WorldData {
    println!("Generating world...");

    let world_size = world_size.size();

    let area = URect::from_corners(UVec2::ZERO, UVec2::new(world_size.width as u32, world_size.height as u32) + UVec2::new(16, 1));
    let playable_area = URect::from_corners(area.min + UVec2::new(8, 0), area.max - UVec2::new(8, 1));

    let blocks = BlockArray::default((area.height() as usize, area.width() as usize));
    let walls = WallArray::default((area.height() as usize, area.width() as usize));

    let surface = (playable_area.min.y + playable_area.height() / 10) as usize;
    let underground = playable_area.min.y as usize + (playable_area.height() as f32 / 3.) as usize;
    let cavern = playable_area.min.y as usize + (playable_area.height() as f32 / 2.) as usize;

    let layer = Layer {
        surface,
        underground,
        cavern,
        dirt_height: (underground - surface) / 4
    };

    let mut world = WorldData {
        blocks, 
        walls,
        area,
        playable_area,
        layer,
        spawn_point: TilePos::new(0, 0),
    };

    spawn_terrain(&mut world);

    make_hills(&mut world, seed);   

    generate_walls(&mut world);

    extend_terrain(&mut world);

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

    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;

    let playable_area_min_y = world.playable_area.min.y as usize;
    let playable_area_max_y = world.playable_area.max.y as usize;

    for ((y, _), block) in world.blocks
        .slice_mut(s![playable_area_min_y..playable_area_max_y, playable_area_min_x..playable_area_max_x])
        .indexed_iter_mut() 
    {
        if y >= world.layer.surface {
            *block = None;
        }

        if y >= world.layer.underground - world.layer.dirt_height {
            *block = Some(BlockType::Dirt.into());
        }

        if y >= world.layer.underground {
            *block = Some(BlockType::Stone.into());
        }
    }
}

fn make_hills(world: &mut WorldData, seed: u32) {
    println!("Making hills...");

    let mut rng = StdRng::seed_from_u64(seed as u64);

    let level = world.layer.underground - world.layer.dirt_height;
    
    let fbm = NoiseBuilder::fbm_1d(world.playable_width())
        .with_seed(rng.gen())
        .with_freq(0.005)
        .with_octaves(3)
        .generate_scaled(0., 1.);

    let gradient = NoiseBuilder::gradient_1d(world.playable_width())
        .with_seed(rng.gen())
        .with_freq(0.01)
        .generate_scaled(0., 1.);

    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;

    for x in playable_area_min_x..playable_area_max_x {
        let noise_value = fbm[x - playable_area_min_x] * gradient[x - playable_area_min_x];

        let hill_height = level - (noise_value * DIRT_HILL_HEIGHT as f32) as usize;
        world.blocks.slice_mut(s![hill_height..level, x]).fill(Some(BlockType::Dirt.into()));
    }
}

fn rough_cavern_layer_border(world: &mut WorldData, seed: u32) {
    let mut rng = StdRng::seed_from_u64(seed as u64);

    let level = world.layer.underground;
    
    let noise = NoiseBuilder::gradient_1d(world.playable_width())
        .with_seed(rng.gen())
        .with_freq(0.1)
        .generate_scaled(-1., 0.);

    const ROUGHNESS: f32 = 20.;

    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;

    for x in playable_area_min_x..playable_area_max_x {
        let noise_value = noise[x - playable_area_min_x];

        let x_offset = {
            let offset = rng.gen_range(-5f32..5f32);

            ((x as f32 + offset) as usize).clamp(0, world.width() - 1)
        };

        let x_range = if x < x_offset {
            x..=x_offset
        } else {
            x_offset..=x
        };

        let hill_height = level - (noise_value.abs() * ROUGHNESS) as usize;
        
        if x != x_offset {
            world.blocks.slice_mut(s![level..hill_height, x_range]).fill(Some(BlockType::Dirt.into()));
        } else {
            world.blocks.slice_mut(s![level..hill_height, x]).fill(Some(BlockType::Dirt.into()));
        }
    }
}

fn generate_walls(world: &mut WorldData) {
    println!("Generating walls...");

    let dirt_level = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;
    let underground_level = world.layer.underground;

    let world_width = world.width();

    for ((y, x), wall) in world.walls.slice_mut(s![dirt_level..underground_level, ..]).indexed_iter_mut() {
        let block_not_exists = |y: usize, x: usize| -> bool {
            world.blocks.get((y, x)).and_then(|b| b.as_ref()).is_none()
        };

        let prev_x = x.saturating_sub(1);
        let next_x = (x + 1).clamp(0, world_width);

        if block_not_exists(dirt_level + y - 1, x) { continue; }
        if block_not_exists(dirt_level + y + 1, x) { continue; }
        if block_not_exists(dirt_level + y, prev_x) { continue; }
        if block_not_exists(dirt_level + y, next_x) { continue; }
        if block_not_exists(dirt_level + y - 1, prev_x) { continue; }
        if block_not_exists(dirt_level + y + 1, prev_x) { continue; }
        if block_not_exists(dirt_level + y + 1, next_x) { continue; }
        if block_not_exists(dirt_level + y - 1, next_x) { continue; }

        *wall = Some(WallType::Dirt.into());
    }
}

fn generate_dirt_in_rocks(world: &mut WorldData, seed: u32) {
    println!("Generating rocks in dirt...");
    let underground_level = world.layer.underground;
    let cavern_level = world.layer.cavern;

    generate_dirt(world, seed, underground_level, cavern_level, 0.2, 0.4, 0.8);
    generate_dirt(world, seed, cavern_level, world.height(), 0.3, 0.72, 0.72);
}

fn generate_dirt(world: &mut WorldData, seed: u32, from: usize, to: usize, freq: f32, min_prevalence: f32, max_prevalence: f32) {
    let world_width = world.playable_width();

    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;

    let mut slice = world.blocks
        .slice_mut(s![from..to, playable_area_min_x..playable_area_max_x]);

    let height = slice.nrows();

    let noise = NoiseBuilder::fbm_2d(slice.ncols(), height)
        .with_seed(seed as i32)
        .with_freq(freq)
        .generate_scaled(0., 1.);

    for ((y, x), block) in slice.indexed_iter_mut() {
        let block_type = block.map(|b| b.block_type);
        let index = (y * world_width) + x;

        let a = map_range_f32(0., height as f32, min_prevalence, max_prevalence, y as f32);

        let noise_value = noise[index];

        if noise_value >= a {
            if let Some(BlockType::Stone) = block_type {
                *block = Some(BlockType::Dirt.into());
            }
        }
    }
}

fn generate_rocks_in_dirt(world: &mut WorldData, seed: u32) {
    println!("Generating dirt in rocks...");

    let dirt_level = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;
    let underground_level = world.layer.underground;

    let noise = NoiseBuilder::fbm_2d(world.playable_width(), underground_level - dirt_level)
        .with_seed(seed as i32)
        .with_freq(0.15)
        .generate_scaled(-1., 1.);

    let world_width = world.playable_width();

    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;

    let mut slice = world.blocks
        .slice_mut(s![dirt_level..underground_level, playable_area_min_x..playable_area_max_x]);

    for ((y, x), block) in slice.indexed_iter_mut() {
        let index = (y * world_width) + x;

        let noise_value = noise[index];

        if noise_value >= 0.5 {
            let block_type = block.map(|b| b.block_type);
            if matches!(block_type, Some(BlockType::Dirt | BlockType::Grass)) {
                *block = Some(BlockType::Stone.into());
            }
        }
    }
}

fn generate_big_caves(world: &mut WorldData, seed: u32) {
    println!("Generating big caves...");

    let dirt_level = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;

    let height = world.playable_height() - dirt_level;

    let noise = Fbm::<Perlin>::new(seed);

    let noise_map = PlaneMapBuilder::<_, 2>::new(noise)
        .set_size(world.playable_width(), height)
        .set_x_bounds(-30., 30.)
        .set_y_bounds(-15., 15.)
        .build();

    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;
    
    let playable_area_max_y = world.playable_area.max.y as usize;

    let mut slice = world.blocks.slice_mut(s![dirt_level..playable_area_max_y, playable_area_min_x..playable_area_max_x]);

    for ((y, x), block) in slice.indexed_iter_mut() {
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
        .set_size(world.playable_width(), world.height() - underground_level + 10)
        .set_x_bounds(-60., 60.)
        .set_y_bounds(-30., 30.)
        .build();

    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;

    let mut slice = world.blocks.slice_mut(s![underground_level..world.playable_height() - 10, playable_area_min_x..playable_area_max_x]);

    for ((y, x), block) in slice.indexed_iter_mut() {
        let noise_value = noise_map.get_value(x, y);

        if noise_value < -0.3 {
            *block = None;
        }
    }
}

fn grassify(world: &mut WorldData) {
    println!("Growing grass...");

    fn is_valid(world: &mut WorldData, x: usize, y: usize) -> bool {
        if x >= world.width() { return false; }
        if y >= world.height() { return false; }
        if !world.block_exists_with_type((x, y), BlockType::Dirt) { return false; }

        any_neighbor_not_exist(world, x, y)
    }
    
    fn flood_fill(world: &mut WorldData, x: usize, y: usize) {
        let mut queue = VecDeque::new();
        queue.push_back((x, y));

        world.set_block((x, y), BlockType::Grass);
    
        while !queue.is_empty() {
            let (x, y) = queue.pop_back().unwrap();

            let prev_x = x.saturating_sub(1);

            if is_valid(world, x + 1, y) {
                let pos = (x + 1, y);
                world.set_block(pos, BlockType::Grass);
                queue.push_back(pos);
            }
    
            if is_valid(world, prev_x, y) {
                let pos = (prev_x, y);
                world.set_block(pos, BlockType::Grass);
                queue.push_back(pos);
            }
    
            if is_valid(world, x, y + 1) {
                let pos = (x, y + 1);
                world.set_block(pos, BlockType::Grass);
                queue.push_back(pos);
            }
    
            if is_valid(world, x, y - 1) {
                let pos = (x, y - 1);
                world.set_block(pos, BlockType::Grass);
                queue.push_back(pos);
            }

            if is_valid(world, prev_x, y - 1) {
                let pos = (prev_x, y - 1);
                world.set_block(pos, BlockType::Grass);
                queue.push_back(pos);
            }

            if is_valid(world, x + 1, y - 1) {
                let pos = (x + 1, y - 1);
                world.set_block(pos, BlockType::Grass);
                queue.push_back(pos);
            }

            if is_valid(world, prev_x, y + 1) {
                let pos = (prev_x, y + 1);
                world.set_block(pos, BlockType::Grass);
                queue.push_back(pos);
            }

            if is_valid(world, x + 1, y + 1) {
                let pos = (x + 1, y + 1);
                world.set_block(pos, BlockType::Grass);
                queue.push_back(pos);
            }
        }
    }

    for x in 0..world.width() {
        let y = get_surface_block_y(world, x);
        if world.block_exists_with_type((x, y), BlockType::Dirt) {
            flood_fill(world, x, y);
        }
    }

}

fn remove_walls_from_surface(world: &mut WorldData) {
    fn is_valid(world: &mut WorldData, pos: (usize, usize)) -> bool {
        let x = pos.0;
        let y = pos.1;

        if x >= world.width() { return false; }
        if y >= world.height() { return false; }

        if !world.wall_exists((x, y)) { return false; }

        if any_neighbor_not_exist(world, x, y) { return true; }

        if world.solid_block_exists((x, y)) { return false; }

        true
    }
    
    fn flood_fill(world: &mut WorldData, x: usize, y: usize) {
        let mut queue = VecDeque::new();
        queue.push_back(((x, y), (0i32, 0i32)));

        world.remove_wall((x, y));
    
        while !queue.is_empty() {
            let ((x, y), (depth_x, depth_y)) = queue.pop_back().unwrap();

            let prev_x = x.saturating_sub(1);

            if depth_x.abs() >= depth_y / 2 + 5 { continue; }

            {
                let next_pos = (x + 1, y);
                let next_depth = (depth_x + 1, depth_y);
                if is_valid(world, next_pos) {
                    world.remove_wall(next_pos);
                    queue.push_back((next_pos, next_depth));
                }
            }
            {
                let next_pos = (prev_x, y);
                let next_depth = (depth_x - 1, depth_y);
                if is_valid(world, next_pos) {
                    world.remove_wall(next_pos);
                    queue.push_back((next_pos, next_depth));
                }
            }
            {
                let next_pos = (x, y + 1);
                let next_depth = (depth_x, depth_y + 1);
                if is_valid(world, next_pos) {
                    world.remove_wall(next_pos);
                    queue.push_back((next_pos, next_depth));
                }
            }
            {    
                let next_pos = (x, y - 1);
                let next_depth = (depth_x, depth_y - 1);
                if is_valid(world, next_pos) {
                    world.remove_wall(next_pos);
                    queue.push_back((next_pos, next_depth));
                }
            }
            {    
                let next_pos = (x - 1, y - 1);
                let next_depth = (depth_x - 1, depth_y - 1);
                if is_valid(world, next_pos) {
                    world.remove_wall(next_pos);
                    queue.push_back((next_pos, next_depth));
                }
            }
            {    
                let next_pos = (x + 1, y + 1);
                let next_depth = (depth_x + 1, depth_y + 1);
                if is_valid(world, next_pos) {
                    world.remove_wall(next_pos);
                    queue.push_back((next_pos, next_depth));
                }
            }
            {    
                let next_pos = (x + 1, y - 1);
                let next_depth = (depth_x + 1, depth_y - 1);
                if is_valid(world, next_pos) {
                    world.remove_wall(next_pos);
                    queue.push_back((next_pos, next_depth));
                }
            }
            {    
                let next_pos = (x - 1, y + 1);
                let next_depth = (depth_x - 1, depth_y + 1);
                if is_valid(world, next_pos) {
                    world.remove_wall(next_pos);
                    queue.push_back((next_pos, next_depth));
                }
            }
        }
    }

    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;

    for x in playable_area_min_x..playable_area_max_x {
        let y = get_surface_wall_y(world, x);

        if world.solid_block_exists((x, y)) { continue; }

        flood_fill(world, x, y);
    }
}

fn grow_tree(world: &mut WorldData, rng: &mut StdRng, root_pos: impl AsWorldPos) {
    let height: usize = rng.gen_range(5..=16);

    let root_pos_x = root_pos.x();
    let root_pos_y = root_pos.y();

    if root_pos_x >= world.width() - 2 || root_pos_x <= 2 {
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

    let left_base = rng.gen_bool(0.5) && left_block;
    let right_base = rng.gen_bool(0.5) && right_block;

    let mut variant: u32;

    // Base
    if left_base {
        variant = rng.gen_range(0..3);
        world.set_block(
            (root_pos_x - 1, root_pos_y),
            Block::new(tree!(TreeType::Forest, TreeFrameType::BasePlainLeft), variant)
        );
    }

    if right_base {
        variant = rng.gen_range(0..3);
        world.set_block(
            (root_pos_x + 1, root_pos_y),
            Block::new(tree!(TreeType::Forest, TreeFrameType::BasePlainRight), variant)
        );
    }

    let trunk = if left_base && right_base {
        TreeFrameType::BasePlainAD
    } else if left_base {
        TreeFrameType::BasePlainA
    } else if right_base {
        TreeFrameType::BasePlainD
    } else {
        TreeFrameType::TrunkPlain
    };

    variant = rng.gen_range(0..3);
    world.set_block(
        (root_pos_x, root_pos_y),
        Block::new(tree!(TreeType::Forest, trunk), variant)
    );

    // Trunk
    variant = rng.gen_range(0..3);
    world.blocks
        .slice_mut(s![root_pos_y - height..root_pos_y, root_pos_x])
        .fill(Block::new(tree!(TreeType::Forest, TreeFrameType::TrunkPlain), variant).into());

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

            variant = rng.gen_range(0..3);
            world.set_block(
                (root_pos_x - 1, y),
                Block::new(tree!(TreeType::Forest, frame_type), variant)
            );
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

            variant = rng.gen_range(0..3);
            world.set_block(
                (root_pos_x + 1, y),
                Block::new(tree!(TreeType::Forest, frame_type), variant)
            );
        }
    }

    // Top
    let bare = rng.gen_bool(1. / 5.);
    let jagged = rng.gen_bool(1. / 3.);

    let frame_type = if jagged {
        TreeFrameType::TopBareJagged
    } else if bare {
        TreeFrameType::TopBare
    } else {
        TreeFrameType::TopLeaves
    };

    variant = rng.gen_range(0..3);
    world.set_block(
        (root_pos_x, root_pos_y - height - 1),
        Block::new(tree!(TreeType::Forest, frame_type), variant)
    );
}

fn grow_trees(world: &mut WorldData, seed: u32) {
    println!("Growing trees...");

    let mut rng = StdRng::seed_from_u64(seed as u64);

    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;

    for x in playable_area_min_x..playable_area_max_x {
        let y = get_surface_block_y(world, x);

        let grow = rng.gen_bool(1. / 5.);

        if grow {
            // Trees can only grow on dirt or grass
            let is_valid_block = world.get_block((x, y))
                .filter(|b| matches!(b.block_type, BlockType::Dirt | BlockType::Grass))
                .is_some();

            if is_valid_block {
                grow_tree(world, &mut rng, (x, y - 1))
            }
        }
    }

}

fn extend_terrain(world: &mut WorldData) {
    let playable_area_min_x = world.playable_area.min.x as usize;
    let playable_area_max_x = world.playable_area.max.x as usize;

    let playable_area_min_y = world.playable_area.min.y as usize;
    let playable_area_max_y = world.playable_area.max.y as usize;

    for y in playable_area_min_y..playable_area_max_y {
        let block_start = world.get_block((playable_area_min_x, y)).copied();
        let block_end = world.get_block((playable_area_max_x - 1, y)).copied();

        let wall_start = world.get_wall((playable_area_min_x + 1 + 1, y)).copied();
        let wall_end = world.get_wall((playable_area_max_x - 1 - 1, y)).copied();

        world.blocks.slice_mut(s![y, 0..playable_area_min_x]).fill(block_start);
        world.blocks.slice_mut(s![y, playable_area_max_x..world.width()]).fill(block_end);

        world.walls.slice_mut(s![y, 0..playable_area_min_x + 1]).fill(wall_start);
        world.walls.slice_mut(s![y, (playable_area_max_x - 1)..world.width()]).fill(wall_end);
    }

    for x in playable_area_min_x..playable_area_max_x {
        let block_bottom = world.get_block((x, playable_area_max_y - 1)).copied();

        world.blocks.slice_mut(s![playable_area_max_y..world.height(), x]).fill(block_bottom);
    }
}

fn get_surface_block_y(world: &WorldData, x: usize) -> usize {
    let mut y = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;

    while y < world.height() {
        if world.solid_block_exists((x, y)) {
            break;
        }

        y += 1;
    }

    y
}

fn get_surface_wall_y(world: &WorldData, x: usize) -> usize {
    let mut y = world.layer.underground - world.layer.dirt_height - DIRT_HILL_HEIGHT;

    while y < world.height() {
        if world.wall_exists((x, y)) {
            break;
        }

        y += 1;
    }

    y
}

fn set_spawn_point(world: &mut WorldData) {
    let x = world.area.width() as usize / 2;
    let y = get_surface_block_y(world, x - 1).min(get_surface_block_y(world, x));

    world.spawn_point = TilePos::new(x as u32, y as u32);
}

#[inline]
fn any_neighbor_not_exist(world: &WorldData, x: usize, y: usize) -> bool {
    Neighbors::get_square_neighboring_positions(
        &TilePos::new(x as u32, y as u32),
        &TilemapSize::from(world.area.size()),
        true
    )
    .iter()
    .any(|pos| !world.solid_block_exists(pos))
}