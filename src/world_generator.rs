use ndarray::{prelude::*};
use noise::{NoiseFn, Seedable, SuperSimplex, OpenSimplex};

use crate::block::{BLOCK_STONE, BLOCK_DIRT, BLOCK_AIR, BLOCK_GRASS, get_block_by_id, BlockId};

const WORLD_SIZE_X: usize = 1750;
const WORLD_SIZE_Y: usize = 900;

#[derive(Clone, Copy)]
struct Level {
    sky: (usize, usize),
    dirt: (usize, usize),
    stone: (usize, usize)
}

pub fn generate(seed: u32) -> Array2<BlockId> {

    // region: Init world

    println!("Initializing world...");

    let mut world = Array2::<BlockId>::zeros((WORLD_SIZE_Y, WORLD_SIZE_X));
    world.fill(BLOCK_STONE.id);

    let level = Level {
        sky: (0, WORLD_SIZE_Y / 5),
        dirt: (WORLD_SIZE_Y / 5, WORLD_SIZE_Y / 3),
        stone: (WORLD_SIZE_Y / 3, world.nrows())
    };

    // endregion

    // region: Init noises

    let noise = SuperSimplex::new()
        // .set_octaves(24)
        // .set_frequency(10.)
        // .set_persistence(0.)
        // .set_lacunarity(1.)
        .set_seed(seed);

    let terrain_noise = SuperSimplex::new();

    let epic_cave_noise = OpenSimplex::new()
        // .set_octaves(3)
        .set_seed(seed);

    // endregion
    
    replace(&mut world.slice_mut(s![..level.dirt.1, ..]), BLOCK_STONE.id, BLOCK_DIRT.id);

    insert_stone_specks_into_dirt(&mut world, level, noise.set_seed(seed % rand::random::<u32>()));
    
    insert_dirt_specks_into_stone(&mut world, level, noise.set_seed(seed % rand::random::<u32>()));
    
    make_caves(&mut world, noise.set_seed(seed % rand::random::<u32>()), level.dirt.0 + 20);

    world.slice_collapse(s![level.sky.1.., ..]);

    make_epic_cave(&mut world, epic_cave_noise, 0.0019, 0.015);
    
    make_surface_rough(&mut world, terrain_noise.set_seed(seed % rand::random::<u32>()), level.stone.0, 5., 25., BLOCK_DIRT.id, BLOCK_STONE.id);
    
    make_surface_rough(&mut world, terrain_noise.set_seed(seed % rand::random::<u32>()), level.sky.1, 4., (WORLD_SIZE_X / 110) as f64, BLOCK_AIR, BLOCK_DIRT.id);
    
    add_grass(&mut world, level);

    world
}

fn insert_specks<F: NoiseFn<[f64; 2]>>(world: &mut ArrayViewMut2<BlockId>, noise: F, frequency: f64, size: f64, speck_block: BlockId) {
    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let a = noise.get([x as f64 / (WORLD_SIZE_X as f64 / 30. * size), y as f64 / (WORLD_SIZE_Y as f64 / 30. * size)]);

            if a > (frequency * 10.).powi(-1) {
                world[[y, x]] = speck_block;
            }
        }
    }
}

fn add_grass(world: &mut Array2<BlockId>, level: Level) {
    println!("Adding grass...");

    for x in 0..world.ncols() {
        let mut prev_block: Option<BlockId> = None;
        let mut y: usize = level.stone.0;

        loop {
            if y == 0 { break; }

            let block = world.get((y, x)).and_then(|tile| get_block_by_id(*tile));

            if let Some(b) = block {
                if let Some(prev_b) = prev_block {

                    if prev_b == BLOCK_DIRT.id && (b.id == BLOCK_AIR) {

                        world[[y+1, x]] = BLOCK_GRASS.id;
                    }
                }
            }

            prev_block = block.map(|b| b.id);
            y -= 1;
        }
    }
}

fn make_surface_rough<F: NoiseFn<[f64; 2]>>(world: &mut Array2<BlockId>, terrain_noise: F, start_y: usize, q: f64, height: f64, down: BlockId, up: BlockId) {
    for x in 0..world.ncols() {

        let a = terrain_noise.get([x as f64 / (world.ncols() as f64 / q), x as f64 / (world.nrows() as f64 / q)]);

        let y = a * height;

        let yu = y.abs() as usize;

        if y > 0. {
            replace(&mut world.slice_mut(s![start_y-yu..start_y, x]), down, up);
        } else {
            replace(&mut world.slice_mut(s![start_y..start_y+yu, x]), up, down);
        }
    }
}

fn make_epic_cave<F: NoiseFn<[f64; 2]>>(world: &mut Array2<BlockId>, epic_cave_noise: F, frequency: f64, threshold: f64) {
    println!("Making epic cave...");

    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let xf = x as f64;
            let yf = y as f64;

            let k = (
                epic_cave_noise.get([frequency * xf, frequency * yf])
                    // + epic_cave_noise.get([2. * frequency * xf, 2. * frequency * yf]) * 0.5
                    // + epic_cave_noise.get([4. * frequency * xf, 4. * frequency * yf]) * 1./4.
                    + epic_cave_noise.get([8. * frequency * xf, 8. * frequency * yf]) * 1./8.
            ).abs();

            if k < threshold {
                world[[y, x]] = BLOCK_AIR;
            }
        }
    }
}

fn make_caves<F: NoiseFn<[f64; 2]>>(world: &mut Array2<BlockId>, noise: F, max_level: usize) {
    println!("Making caves...");

    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let mut k = noise.get([x as f64 / (WORLD_SIZE_X as f64 / 30.), y as f64 / (WORLD_SIZE_Y as f64 / 30.)]); 
                // + noise.get([x as f64 / (WORLD_SIZE_X as f64 / 40.), y as f64 / (WORLD_SIZE_Y as f64 / 40.)])
                // + noise.get([x as f64 / (WORLD_SIZE_X as f64 / 50.), y as f64 / (WORLD_SIZE_Y as f64 / 50.)]);

            if y.abs_diff(max_level) < 30 {
                k *= 0.000000001;
            }

            if k > 0.4 {
                world[[y, x]] = BLOCK_AIR;
            }
        }
    }

    make_small_caves(world, noise, max_level);
}

fn make_small_caves<F: NoiseFn<[f64; 2]>>(world: &mut Array2<BlockId>, noise: F, max_level: usize) {
    let q = 110.;

    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let mut k = noise.get([x as f64 / (WORLD_SIZE_X as f64 / q), y as f64 / (WORLD_SIZE_Y as f64 / q)]) 
                + noise.get([x as f64 / (WORLD_SIZE_X as f64 / q + 10.), y as f64 / (WORLD_SIZE_Y as f64 / q + 10.)])
                + noise.get([x as f64 / (WORLD_SIZE_X as f64 / q + 20.), y as f64 / (WORLD_SIZE_Y as f64 / q + 20.)]);

            if y.abs_diff(max_level) < 20 {
                k *= 0.5;
            }

            if k > 0.8 {
                world[[y, x]] = BLOCK_AIR;
            }
        }
    }
}

fn replace<D: Dimension>(world: &mut ArrayViewMut<BlockId, D>, replace: BlockId, replacement: BlockId) {
    for tile in world.iter_mut() {
        if *tile == replace {
            *tile = replacement;
        }
    }
}

#[inline]
fn insert_stone_specks_into_dirt<F: NoiseFn<[f64; 2]>>(world: &mut Array2<BlockId>, level: Level, noise: F) {
    println!("Inserting stone specks into dirt...");

    insert_specks(&mut world.slice_mut(s![level.dirt.0+40..level.dirt.1, ..]), noise, 0.5, 0.7, BLOCK_STONE.id);
}

#[inline]
fn insert_dirt_specks_into_stone<F: NoiseFn<[f64; 2]>>(world: &mut Array2<BlockId>, level: Level, noise: F) {
    println!("Inserting dirt specks into stone...");

    insert_specks(&mut world.slice_mut(s![level.stone.0..level.stone.1, ..]), noise, 0.2, 0.1, BLOCK_DIRT.id);
}