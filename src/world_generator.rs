use ndarray::prelude::*;
use noise::{Perlin, NoiseFn, Seedable, Fbm, MultiFractal};

use crate::block::{BLOCK_STONE, BLOCK_DIRT, BLOCK_AIR, Block, BLOCK_GRASS, get_block_by_id};

const WORLD_SIZE_X: usize = 1700;
const WORLD_SIZE_Y: usize = 900;

pub fn generate(seed: u32) -> Array2<u32> {
    // region: Init world

    println!("Initializing world...");

    let mut tiles = Array2::<u32>::zeros((WORLD_SIZE_Y, WORLD_SIZE_X));
    tiles.fill(BLOCK_STONE.id);
    tiles.slice_mut(s![..(WORLD_SIZE_Y / 3) + 100, ..]).fill(BLOCK_DIRT.id);

    // endregion

    // region: Init noises and get current time

    let noise = Fbm::new()
        .set_octaves(24)
        .set_frequency(18.)
        .set_persistence(0.)
        .set_seed(seed);

    let terrain_noise = Perlin::new().set_seed(seed % rand::random::<u32>());

    let epic_cave_noise = Fbm::new()
        .set_octaves(24)
        .set_frequency(8.)
        .set_persistence(1.)
        .set_lacunarity(0.2)
        .set_seed(seed % rand::random::<u32>());

    // endregion

    // region: Make caves
    println!("Making caves...");
    make_caves(&mut tiles, &noise);
    // endregion

    // region: Make epic cave
    println!("Making epic cave...");
    make_epic_cave(&mut tiles, &epic_cave_noise);
    // endregion

    // tiles.slice_mut(s![..200, ..]).fill(BLOCK_AIR);

    tiles.slice_collapse(s![200.., ..]);

    // region: Make surface rough
    println!("Making surface rough...");
    make_surface_rough(&mut tiles, &terrain_noise);
    // endregion

    // region: Add grass
    println!("Adding grass...");
    add_grass(&mut tiles);
    // endregion

    tiles
}

fn make_caves<N>(
    tiles: &mut Array2<u32>,
    noise: N
) where N: NoiseFn<[f64; 2]> {
    for y in 0..tiles.nrows() {
        for x in 0..tiles.ncols() {
            let k = noise.get([x as f64 / WORLD_SIZE_X as f64, y as f64 / WORLD_SIZE_Y as f64]);

            if k > 0.2 {
                tiles[[y, x]] = BLOCK_AIR;
            }
        }
    }
}

fn make_epic_cave<N>(
    tiles: &mut Array2<u32>,
    epic_cave_noise: N
) where N: NoiseFn<[f64; 2]> {
    let world_width = tiles.ncols();
    let world_half_width = world_width / 2;

    let k: usize = rand::random::<bool>() as usize;

    let part = (world_half_width + world_half_width * k, world_half_width + world_half_width * k);

    let mut part_slice = tiles.slice_mut(s![.., (part.0)..(part.1)]);

    for y in 0..part_slice.nrows() {
        for x in (part.0)..part_slice.ncols() {
            let a = epic_cave_noise.get([(x as f64 / part_slice.ncols() as f64), (y as f64 / part_slice.nrows() as f64)]);

            if a > 0.2 {
                part_slice[[y, x]] = BLOCK_AIR;
            }
        }
    }
}

fn make_surface_rough<N>(
    tiles: &mut Array2<u32>,
    terrain_noise: &N
) where N: NoiseFn<[f64; 2]> {
    for x in 0..tiles.ncols() {
        let xf = (x as f32) * 0.1;
        let height = 10.;

        let a = xf.sin() * 0.1 * terrain_noise.get([x as f64 * 0.1, 0.1]) as f32;

        let y = (height * a * 10.).ceil() as usize;

        tiles.slice_mut(s![..(y+height as usize) as usize, x]).fill(BLOCK_AIR);

        // world[[y + 200, ..height as usize]] = BLOCK_AIR.index;
    }
}

fn add_grass(
    tiles: &mut Array2<u32>,
) {
    for x in 0..tiles.ncols() {
        let mut block: Option<&'static Block> = None;
        let mut y: usize = 0;

        loop {
            if let Some(b) = block {
                if b.id == BLOCK_DIRT.id {
                    break;
                }
            }

            y += 1;
            block = get_block_by_id(tiles[[y, x]]);
        }

        tiles[[y, x]] = BLOCK_GRASS.id;
    }
}