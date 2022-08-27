use autodefault::autodefault;
use bevy::prelude::default;
use ndarray::prelude::*;
use noise::{NoiseFn, Seedable, SuperSimplex, OpenSimplex};
use rand::{SeedableRng, rngs::StdRng, Rng};

use crate::block::{BLOCK_STONE, BLOCK_DIRT, BLOCK_AIR, BLOCK_GRASS, get_block_by_id, BlockId};

const WORLD_SIZE_X: usize = 1750;
const WORLD_SIZE_Y: usize = 900;

#[derive(Clone, Copy)]
pub struct Level {
    pub sky: (usize, usize),
    pub dirt: (usize, usize),
    pub stone: (usize, usize)
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Slope {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Slope {
    pub fn is_all(&self) -> bool {
        self.top && self.bottom && self.left && self.right
    }

    pub fn is_none(&self) -> bool {
        !self.top && !self.bottom && !self.left && !self.right
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Tile {
    pub id: BlockId,
    pub slope: Slope
}   

#[autodefault(except(Level))]
pub fn generate(seed: u32) -> Array2<Tile> {

    // region: Init world

    let mut rng = StdRng::seed_from_u64(seed as u64);

    println!("Initializing world...");

    let mut world = Array2::<Tile>::default((WORLD_SIZE_Y, WORLD_SIZE_X));
    world.fill(Tile { 
        id: BLOCK_STONE.id, 
        ..default()
    });

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
    
    replace(&mut world.slice_mut(s![..level.dirt.1, ..]), Tile { id: BLOCK_STONE.id, ..default() }, Tile { id: BLOCK_DIRT.id, ..default() });

    insert_stone_specks_into_dirt(&mut world, level, noise.set_seed(seed % rng.gen::<u32>()));
    
    insert_dirt_specks_into_stone(&mut world, level, noise.set_seed(seed % rng.gen::<u32>()));
    
    make_caves(&mut world, noise.set_seed(seed % rng.gen::<u32>()), level.dirt.0 + 20, 0.009, 0.6);

    world.slice_collapse(s![level.sky.1.., ..]);

    make_epic_cave(&mut world, epic_cave_noise, 0.0009, 0.011);
    
    make_surface_rough(&mut world, terrain_noise.set_seed(seed % rng.gen::<u32>()), level.stone.0, 5., 25., Tile { id: BLOCK_DIRT.id }, Tile { id: BLOCK_STONE.id });
    
    make_surface_rough(&mut world, terrain_noise.set_seed(seed % rng.gen::<u32>()), level.sky.1, 4., (WORLD_SIZE_X / 110) as f64, Tile { id: BLOCK_AIR }, Tile { id: BLOCK_DIRT.id });
    
    add_grass(&mut world, level);

    set_tile_slope(&mut world);

    world
}

fn insert_specks<F: NoiseFn<[f64; 2]>>(world: &mut ArrayViewMut2<Tile>, noise: F, frequency: f64, size: f64, speck_block: Tile) {
    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let a = noise.get([x as f64 / (WORLD_SIZE_X as f64 / 30. * size), y as f64 / (WORLD_SIZE_Y as f64 / 30. * size)]);

            if a > (frequency * 10.).powi(-1) {
                world[[y, x]] = speck_block;
            }
        }
    }
}

#[autodefault]
fn add_grass(world: &mut Array2<Tile>, level: Level) {
    println!("Adding grass...");

    for x in 0..world.ncols() {
        let mut prev_block: Option<BlockId> = None;
        let mut y: usize = 0;

        loop {
            if y >= level.stone.1 {
                break;
            }

            let block = world
                .get((y, x))
                .and_then(|tile| get_block_by_id(tile.id))
                .map(|b| b.id);


            if let Some(b) = block {
                if b == BLOCK_DIRT.id && matches!(prev_block, None) {
                    world[[y, x]] = Tile { id: BLOCK_GRASS.id };
                }
            }

            prev_block = block;
            y += 1;
        }
    }
}

fn make_surface_rough<F: NoiseFn<[f64; 2]>>(world: &mut Array2<Tile>, terrain_noise: F, start_y: usize, q: f64, height: f64, down: Tile, up: Tile) {
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

#[autodefault]
fn make_epic_cave<F: NoiseFn<[f64; 2]>>(world: &mut Array2<Tile>, epic_cave_noise: F, frequency: f64, threshold: f64) {
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
                world[[y, x]] = Tile { id: BLOCK_AIR };
            }
        }
    }
}

#[autodefault]
fn make_caves<F: NoiseFn<[f64; 2]>>(world: &mut Array2<Tile>, noise: F, max_level: usize, frequency: f64, threshold: f64) {
    println!("Making caves...");

    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let mut k = noise.get([x as f64 / (WORLD_SIZE_X as f64 * frequency), y as f64 / (WORLD_SIZE_Y as f64 * frequency)])
                + noise.get([x as f64 / (WORLD_SIZE_X as f64 * frequency * 4.), y as f64 / (WORLD_SIZE_Y as f64 * frequency)]) * 1./4.;
                // + noise.get([x as f64 / (WORLD_SIZE_X as f64 / 50.), y as f64 / (WORLD_SIZE_Y as f64 / 50.)]);

            if y.abs_diff(max_level) < 30 {
                k *= 0.000000001;
            }

            if k > threshold {
                world[[y, x]] = Tile { id: BLOCK_AIR };
            }
        }
    }

    make_small_caves(world, noise, max_level);
}

#[autodefault]
fn make_small_caves<F: NoiseFn<[f64; 2]>>(world: &mut Array2<Tile>, noise: F, max_level: usize) {
    let q = 120.;

    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let mut k = noise.get([x as f64 / (WORLD_SIZE_X as f64 / q), y as f64 / (WORLD_SIZE_Y as f64 / q)]) 
                + noise.get([x as f64 / (WORLD_SIZE_X as f64 / q + 10.), y as f64 / (WORLD_SIZE_Y as f64 / q + 10.)])
                + noise.get([x as f64 / (WORLD_SIZE_X as f64 / q + 20.), y as f64 / (WORLD_SIZE_Y as f64 / q + 20.)]);

            if y.abs_diff(max_level) < 20 {
                k *= 0.5;
            }

            if k > 0.8 {
                world[[y, x]] = Tile { id: BLOCK_AIR };
            }
        }
    }
}

fn replace<D: Dimension>(world: &mut ArrayViewMut<Tile, D>, replace: Tile, replacement: Tile) {
    for tile in world.iter_mut().filter(|tile| **tile == replace) {
        *tile = replacement;
    }
}

#[inline]
#[autodefault]
fn insert_stone_specks_into_dirt<F: NoiseFn<[f64; 2]>>(world: &mut Array2<Tile>, level: Level, noise: F) {
    println!("Inserting stone specks into dirt...");

    insert_specks(&mut world.slice_mut(s![level.dirt.0+40..level.dirt.1, ..]), noise, 0.5, 0.7, Tile { id: BLOCK_STONE.id });
}

#[inline]
#[autodefault]
fn insert_dirt_specks_into_stone<F: NoiseFn<[f64; 2]>>(world: &mut Array2<Tile>, level: Level, noise: F) {
    println!("Inserting dirt specks into stone...");

    insert_specks(&mut world.slice_mut(s![level.stone.0..level.stone.1, ..]), noise, 0.2, 0.1, Tile { id: BLOCK_DIRT.id });
}

fn set_tile_slope(world: &mut Array2<Tile>) {
    for y in 0..WORLD_SIZE_Y {
        for x in 0..WORLD_SIZE_X {
            if let Some(mut tile) = world.get_mut((y, x)).cloned() {
                if tile.id == BLOCK_AIR {
                    continue;
                }

                let prev_y_option = y.checked_sub(1);
                let prev_x_option = x.checked_sub(1);
                let next_y_option = y.checked_add(1);
                let next_x_option = x.checked_add(1);
                
                let prev_y = prev_y_option.unwrap_or(y);
                let prev_x = prev_x_option.unwrap_or(x);
                let next_y = next_y_option.unwrap_or(y);
                let next_x = next_x_option.unwrap_or(x);
                
                tile.slope.left = matches!(prev_x_option.and(world.get((y, prev_x))), Some(t) if t.id != BLOCK_AIR);
                tile.slope.right = matches!(next_x_option.and(world.get((y, next_x))), Some(t) if t.id != BLOCK_AIR);
                tile.slope.top = matches!(prev_y_option.and(world.get((prev_y, x))), Some(t) if t.id != BLOCK_AIR);
                tile.slope.bottom = matches!(next_y_option.and(world.get((next_y, x))), Some(t) if t.id != BLOCK_AIR);

                world[[y, x]] = tile
            }
        }
    }
}