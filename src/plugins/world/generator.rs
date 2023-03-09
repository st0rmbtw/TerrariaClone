use bevy_ecs_tilemap::tiles::TilePos;
use ndarray::prelude::*;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use simdnoise::NoiseBuilder;

use super::{DIRT_HILL_HEIGHT, STONE_HILL_HEIGHT};
use super::block::{Block, BlockType};
use super::world::{Layer, Level};

use super::Wall;
use super::tree::{TreeType, TreeFrameType, tree};
use super::world::{WorldSize, WorldData, Point};

pub type BlockId = i8;
pub type WallId = i8;

pub type BlockArray = Array2<Option<Block>>;
pub type WallArray = Array2<Option<Wall>>;

pub fn generate(seed: u32, world_size: WorldSize) -> WorldData {
    println!("Generating world...");

    let world_size = world_size.size();

    let blocks = BlockArray::default((world_size.height, world_size.width));
    let walls = WallArray::default((world_size.height, world_size.width));

    let layer = Layer {
        sky: Level::new(0, world_size.height / 5),
        dirt: Level::new(world_size.height / 5, world_size.height / 3),
        stone: Level::new(world_size.height / 3, world_size.height),
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

    grassify(&mut world);

    generate_rocks_in_dirt(&mut world, seed);

    generate_dirt_in_rocks(&mut world, seed);

    generate_small_caves(&mut world, seed);

    generate_big_caves(&mut world, seed);

    // grow_trees(&mut world, seed);

    set_spawn_point(&mut world);

    world
}

pub fn set_spawn_point(world: &mut WorldData) {
    let x = world.size.width / 2;
    
    let mut y: usize = 0;

    loop {
        if world.blocks.get((y, x)).map(|b| b).is_some() {
            break;
        }

        y += 1;
    }

    world.spawn_point = TilePos::new(x as u32, y as u32);
}

fn spawn_terrain(world: &mut WorldData) {
    println!("Generating terrain...");

    for ((y, _), block) in world.blocks.indexed_iter_mut() {
        if y >= world.layer.sky.from {
            *block = None;
        }

        if y >= world.layer.dirt.from {
            *block = Some(Block::Dirt);
        }

        if y >= world.layer.stone.from {
            *block = Some(Block::Stone);
        }
    }
}

fn make_hills(world: &mut WorldData, seed: u32) {
    println!("Making hills...");

    let mut rng = StdRng::seed_from_u64(seed as u64);

    fn make_surface_rough(blocks: &mut BlockArray, rng: &mut StdRng, level: usize, block_in: Option<Block>, block_out: Option<Block>, hill_height: f32) {
        let noise = NoiseBuilder::gradient_1d(blocks.ncols())
            .with_seed(rng.gen())
            .with_freq(1. / 100.)
            .generate_scaled(-1., 1.);

        for block_x in 0..blocks.ncols() {
            let noise_y = noise[block_x];

            let x_offset = {
                let offset = rng.gen_range(-5f32..5f32);

                ((block_x as f32 + offset) as usize).clamp(0, blocks.ncols() - 1)
            };

            let x_range = if block_x < x_offset {
                block_x..=x_offset
            } else {
                x_offset..=block_x
            };

            if noise_y > 0. {
                let hill_height = level - (noise_y * hill_height) as usize;

                if block_x != x_offset {
                    blocks.slice_mut(s![hill_height..level, x_range]).fill(block_out);
                } else {
                    blocks.slice_mut(s![hill_height..level, block_x]).fill(block_out);
                }
            } else if noise_y < 0. {
                let hill_height = level + (-noise_y * hill_height) as usize;

                if block_x != x_offset {
                    blocks.slice_mut(s![level..hill_height, x_range]).fill(block_in);
                } else {
                    blocks.slice_mut(s![level..hill_height, block_x]).fill(block_in);
                }
            }
        }
    }

    // Dirt level
    make_surface_rough(&mut world.blocks, &mut rng, world.layer.dirt.from, None, Some(Block::Dirt), DIRT_HILL_HEIGHT);

    // Stone level
    make_surface_rough(&mut world.blocks, &mut rng, world.layer.stone.from, Some(Block::Dirt), Some(Block::Stone), STONE_HILL_HEIGHT);
}

fn generate_walls(world: &mut WorldData) {
    println!("Generating walls...");

    let dirt_level = world.layer.dirt.from - DIRT_HILL_HEIGHT as usize;

    for ((y, x), wall) in world.walls.slice_mut(s![dirt_level.., ..]).indexed_iter_mut() {
        if y < dirt_level && world.blocks[(dirt_level + y, x)].is_none() {
            continue;
        }

        if x == 0 {
            continue;
        }

        if world.blocks.get((dirt_level + y - 1, x)).and_then(|b| b.as_ref()).is_none() {
            continue;
        }

        if world.blocks.get((dirt_level + y - 1, x)).and_then(|b| b.as_ref()).is_none() {
            continue;
        }

        if world.blocks.get((dirt_level + y, x - 1)).and_then(|b| b.as_ref()).is_none() {
            continue;
        }

        if world.blocks.get((dirt_level + y, x + 1)).and_then(|b| b.as_ref()).is_none() {
            continue;
        }

        if world.blocks.get((dirt_level + y - 1, x - 1)).and_then(|b| b.as_ref()).is_none() {
            continue;
        }

        if world.blocks.get((dirt_level + y + 1, x - 1)).and_then(|b| b.as_ref()).is_none() {
            continue;
        }

        if world.blocks.get((dirt_level + y + 1, x + 1)).and_then(|b| b.as_ref()).is_none() {
            continue;
        }

        if world.blocks.get((dirt_level + y - 1, x + 1)).and_then(|b| b.as_ref()).is_none() {
            continue;
        }

        *wall = Some(Wall::Dirt);
    }
}

fn generate_rocks_in_dirt(world: &mut WorldData, seed: u32) {
    println!("Generating rocks in dirt...");

    let stone_level = world.layer.stone.from + STONE_HILL_HEIGHT as usize;

    let noise = NoiseBuilder::fbm_2d(world.size.width, world.size.height - stone_level)
        .with_seed(seed as i32)
        .with_freq(1. / 2.)
        .generate_scaled(-1., 1.);

    let mut slice = world.blocks.slice_mut(s![..stone_level, ..]);

    let offset_from_surface = 5;

    for y in 0..slice.nrows() {
        for x in 0..slice.ncols() {
            let block_above = if y < offset_from_surface {
                None
            } else {
                slice[(y - offset_from_surface, x)]
            };

            let block = slice.get_mut((y, x)).unwrap();

            if let Some(Block { block_type: BlockType::Dirt, .. }) = block {
                if block_above.is_none() {
                    continue;
                }
            }

            let index = (y * world.size.width) + x;

            let noise_value = noise[index];

            if noise_value > 0. {
                if let Some(Block { block_type: BlockType::Dirt, .. }) = block {
                    *block = Some(Block::Stone);
                }
            }
        }
    }
}

fn generate_dirt_in_rocks(world: &mut WorldData, seed: u32) {
    println!("Generating dirt in rocks...");

    let stone_level = world.layer.stone.from;

    let noise = NoiseBuilder::fbm_2d(world.size.width, world.size.height - stone_level)
        .with_seed(seed as i32)
        .with_freq(1. / 10.)
        .generate_scaled(-1., 1.);

    let mut slice = world.blocks.slice_mut(s![stone_level.., ..]);

    for ((y, x), block) in slice.indexed_iter_mut() {
        let index = (y * world.size.width) + x;

        let noise_value = noise[index];

        if noise_value > 0.3 {
            if let Some(Block { block_type: BlockType::Stone, .. }) = block {
                *block = Some(Block::Dirt);
            }
        }
    }
}

fn generate_small_caves(world: &mut WorldData, seed: u32) {
    println!("Generating small caves...");

    let dirt_level = world.layer.dirt.from;

    let mut rng = StdRng::seed_from_u64(seed as u64);

    let noise = NoiseBuilder::fbm_2d(world.size.width, world.size.height - dirt_level)
        .with_seed(seed as i32)
        .with_octaves(1)
        .with_lacunarity(2.43)
        .with_freq(2. / 50.)
        .generate_scaled(-1., 1.);

    for ((y, x), block) in world.blocks.slice_mut(s![dirt_level.., ..]).indexed_iter_mut() {
        let index = (y * world.size.width) + x;

        let noise_value = noise[index];

        if noise_value < -rng.gen_range(0.5..=0.8) {
            *block = None;
        }
    }
}

fn generate_big_caves(world: &mut WorldData, seed: u32) {
    println!("Generating big caves...");

    let dirt_level = world.layer.dirt.from + DIRT_HILL_HEIGHT as usize;

    let mut rng = StdRng::seed_from_u64(seed as u64);

    let noise = NoiseBuilder::fbm_2d(world.size.width, world.size.height - dirt_level)
        .with_seed(seed as i32)
        .with_octaves(1)
        .with_lacunarity(2.43)
        .with_freq(2. / 90.)
        .generate_scaled(-1., 1.);

    for ((y, x), block) in world.blocks.slice_mut(s![dirt_level.., ..]).indexed_iter_mut() {
        let index = (y * world.size.width) + x;

        let noise_value = noise[index];

        if noise_value < -rng.gen_range(0.65..=0.7) {
            *block = None;
        }
    }
}

fn grassify(world: &mut WorldData) {
    println!("Growing grass...");

    for x in 0..world.size.width {
        let y = get_surface_y(world, x);

        world.blocks[(y, x)] = Some(Block::Grass);
    }
}

fn grow_tree(world: &mut WorldData, rng: &mut StdRng, root_pos: Point) {
    let height: usize = rng.gen_range(5..=16);

    let left_base = rng.gen_bool(0.5);
    let right_base = rng.gen_bool(0.5);

    if root_pos.x >= world.size.width - 2 || root_pos.x <= 2 {
        return;
    }

    let left_block = world.blocks.get((root_pos.y + 1, root_pos.x - 1)).and_then(|b| b.as_ref()).is_some();
    let right_block = world.blocks.get((root_pos.y + 1, root_pos.x + 1)).and_then(|b| b.as_ref()).is_some();

    // Check enough space
    let mut enough_space = true;
    {
        let left = if left_block { 2 } else { 1 };
        let right = if right_block { 2 } else { 1 };

        // Check enough space for base
        for block in world.blocks.slice(s![root_pos.y, (root_pos.x - left)..=(root_pos.x + right)]).iter() {
            if block.is_some() {
                enough_space = false;
                break;
            }
        }

        // Don't check enough space further if it's already not enough
        if enough_space {
            // Check enough space for branches 
            for block in world.blocks.slice(s![(root_pos.y - 16)..root_pos.y, (root_pos.x - 2)..=(root_pos.x + 2)]).iter() {
                if block.is_some() {
                    enough_space = false;
                    break;
                }
            }
        }
    }

    if !enough_space {
        return;
    }

    // Base
    if left_base && right_base && left_block && right_block {
        {
            let frame = *TreeFrameType::BasePlainLeft.frame(TreeType::Forest).choose(rng).unwrap();
            world.blocks[(root_pos.y, root_pos.x - 1)] = Some(tree!(TreeType::Forest, frame));
        }
        {
            let frame = *TreeFrameType::BasePlainAD.frame(TreeType::Forest).choose(rng).unwrap();
            world.blocks[(root_pos.y, root_pos.x)] = Some(tree!(TreeType::Forest, frame));
        }
        {
            let frame = *TreeFrameType::BasePlainRight.frame(TreeType::Forest).choose(rng).unwrap();
            world.blocks[(root_pos.y, root_pos.x + 1)] = Some(tree!(TreeType::Forest, frame));
        }
    } else if left_base && left_block {
        {
            let frame = *TreeFrameType::BasePlainLeft.frame(TreeType::Forest).choose(rng).unwrap();
            world.blocks[(root_pos.y, root_pos.x - 1)] = Some(tree!(TreeType::Forest, frame));
        }
        {
            let frame = *TreeFrameType::BasePlainA.frame(TreeType::Forest).choose(rng).unwrap();
            world.blocks[(root_pos.y, root_pos.x)] = Some(tree!(TreeType::Forest, frame));
        }
    } else if right_base && right_block {
        {
            let frame = *TreeFrameType::BasePlainD.frame(TreeType::Forest).choose(rng).unwrap();
            world.blocks[(root_pos.y, root_pos.x)] = Some(tree!(TreeType::Forest, frame));
        }
        {
            let frame = *TreeFrameType::BasePlainRight.frame(TreeType::Forest).choose(rng).unwrap();
            world.blocks[(root_pos.y, root_pos.x + 1)] = Some(tree!(TreeType::Forest, frame));
        }
    } else {
        let frame = *TreeFrameType::TrunkPlain.frame(TreeType::Forest).choose(rng).unwrap();
        world.blocks[(root_pos.y, root_pos.x)] = Some(tree!(TreeType::Forest, frame));
    }

    // Trunk
    {
        let frame = *TreeFrameType::TrunkPlain.frame(TreeType::Forest).choose(rng).unwrap();
        world.blocks
            .slice_mut(s![root_pos.y - height..root_pos.y, root_pos.x])
            .fill(Some(tree!(TreeType::Forest, frame)));
    }

    // Branches
    
    // Left side
    for y in root_pos.y - height..root_pos.y {
        let place = rng.gen_bool(1. / 10.);
        let bare = rng.gen_bool(1. / 5.);

        if place && world.blocks[(y - 1, root_pos.x - 1)].is_none() {
            let frame = if bare {
                *TreeFrameType::BranchLeftBare.frame(TreeType::Forest).choose(rng).unwrap()
            } else {
                *TreeFrameType::BranchLeftLeaves.frame(TreeType::Forest).choose(rng).unwrap()
            };
            world.blocks[(y, root_pos.x - 1)] = Some(tree!(TreeType::Forest, frame));
        }
    }

    // Right side
    for y in root_pos.y - height..root_pos.y {
        let place = rng.gen_bool(1. / 10.);
        let bare = rng.gen_bool(1. / 5.);

        if place && world.blocks[(y - 1, root_pos.x + 1)].is_none() {
            let frame = if bare {
                *TreeFrameType::BranchRightBare.frame(TreeType::Forest).choose(rng).unwrap()
            } else {
                *TreeFrameType::BranchRightLeaves.frame(TreeType::Forest).choose(rng).unwrap()
            };
            world.blocks[(y, root_pos.x + 1)] = Some(tree!(TreeType::Forest, frame));
        }
    }

    // Top
    let bare = rng.gen_bool(1. / 5.);
    let frame = if bare {
        let jagged = rng.gen_bool(1. / 3.);
        if jagged {
            *TreeFrameType::TopBareJagged.frame(TreeType::Forest).choose(rng).unwrap()
        } else {
            *TreeFrameType::TopBare.frame(TreeType::Forest).choose(rng).unwrap()
        }
    } else {
        *TreeFrameType::TopLeaves.frame(TreeType::Forest).choose(rng).unwrap()
    };
    world.blocks[(root_pos.y - height - 1, root_pos.x)] = Some(tree!(TreeType::Forest, frame));
}

fn grow_trees(world: &mut WorldData, seed: u32) {
    println!("Growing trees...");

    let mut rng = StdRng::seed_from_u64(seed as u64);

    for x in 0..world.size.width {
        let y = get_surface_y(world, x);
        let grow = rng.gen_bool(1. / 5.);

        if grow {
            grow_tree(world, &mut rng, Point::new(x, y - 1))
        }
    }

}

fn get_surface_y(world: &mut WorldData, x: usize) -> usize {
    let mut y = world.layer.dirt.from - DIRT_HILL_HEIGHT as usize;

    loop {
        if world.blocks[(y, x)].is_some() {
            break;
        }

        y += 1;
    }

    y
}