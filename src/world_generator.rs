use autodefault::autodefault;
use bevy_ecs_tilemap::{helpers::square_grid::neighbors::Neighbors, tiles::TilePos};
use ndarray::prelude::*;
use noise::{NoiseFn, OpenSimplex, Seedable, SuperSimplex};
use rand::{rngs::StdRng, Rng, SeedableRng, thread_rng};

use crate::{wall::Wall as WallType, util::{get_tile_start_index, get_wall_start_index}, items::Block, CellArrayExtensions};

pub const WORLD_SIZE_X: usize = 1750;
pub const WORLD_SIZE_Y: usize = 900;

pub type CellArray = Array2<Cell>;

#[derive(Clone, Copy)]
pub struct Level {
    pub sky: (usize, usize),
    pub dirt: (usize, usize),
    pub stone: (usize, usize),
}

#[derive(Clone, Copy)]
pub struct Wall {
    pub wall_type: WallType
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub block: Block,
}

#[derive(Clone, Copy, Default)]
pub struct Cell {
    pub tile: Option<Tile>,
    pub wall: Option<Wall>,
}

#[autodefault(except(Level, Tile, Wall, Neighbors))]
pub fn generate(seed: u32) -> CellArray {
    // region: Init world

    let mut rng = StdRng::seed_from_u64(seed as u64);

    println!("Initializing world...");

    let mut world = Array2::<Cell>::default((WORLD_SIZE_Y, WORLD_SIZE_X));
    world.fill(Cell {
        tile: Some(Tile {
            block: Block::Stone
        }),
        wall: None,
    });

    for cell in world.slice_mut(s![.., 0..WORLD_SIZE_X]).iter_mut() {
        cell.wall = Some(Wall { 
            wall_type: WallType::DirtWall
        });
    }

    let level = Level {
        sky: (0, WORLD_SIZE_Y / 5),
        dirt: (WORLD_SIZE_Y / 5, WORLD_SIZE_Y / 3),
        stone: (WORLD_SIZE_Y / 3, world.nrows()),
    };

    // endregion

    // region: Init noises

    let noise = SuperSimplex::new(seed);
        // .set_octaves(24)
        // .set_frequency(10.)
        // .set_persistence(0.)
        // .set_lacunarity(1.)

    let terrain_noise = SuperSimplex::new(seed);

    let epic_cave_noise = OpenSimplex::new(seed);
        // .set_octaves(3)

    // endregion

    replace(
        &mut world.slice_mut(s![..level.dirt.1, ..]),
        Some(Block::Stone),
        Some(Block::Dirt),
    );

    insert_stone_specks_into_dirt(&mut world, level, noise.set_seed(seed % rng.gen::<u32>()));

    insert_dirt_specks_into_stone(&mut world, level, noise.set_seed(seed % rng.gen::<u32>()));

    make_caves(
        &mut world,
        noise.set_seed(seed % rng.gen::<u32>()),
        level.dirt.0 + 20,
        0.009,
        0.6,
    );

    world.slice_mut(s![..level.sky.1, ..]).fill(Cell::default());

    make_epic_cave(&mut world, epic_cave_noise, 0.0009, 0.011);

    make_surface_rough(
        &mut world,
        terrain_noise.set_seed(seed % rng.gen::<u32>()),
        level.stone.0,
        5.,
        25.,
        Some(Block::Dirt),
        Some(Block::Stone),
    );

    make_surface_rough(
        &mut world,
        terrain_noise.set_seed(seed % rng.gen::<u32>()),
        level.sky.1,
        4.,
        (WORLD_SIZE_X / 110) as f64,
        None,
        Some(Block::Dirt),
    );

    add_grass(&mut world, level);

    remove_extra_walls(&mut world);

    world
}

pub fn get_spawn_point(tiles: &CellArray) -> TilePos {
    let x = tiles.ncols() / 2;
    
    let mut y: usize = 0;

    loop {
        if tiles.get_tile(TilePos::new(x as u32, y as u32)).is_some() {
            break;
        }

        y += 1;
    }

    return TilePos::new(x as u32, y as u32);
}

pub fn generate_light_map(tiles: &CellArray) -> Array2<f32> {
    let mut light_map = Array2::<f32>::default((tiles.nrows(), tiles.ncols()));

    for ((y, x), cell) in tiles.indexed_iter() {
        if cell.tile.is_some() {
            light_map[[y, x]] = 0.;
        } else {
            light_map[[y, x]] = 255.;
        }
    }

    for y in 0..light_map.nrows() {
        for x in 0..light_map.ncols() {
            propagate_light(x, y, &mut light_map);
        }
    }

    light_map
}

fn propagate_light(x: usize, y: usize, light_map: &mut Array2<f32>) { 
    if x >= light_map.ncols() - 1 { return; }
    if y >= light_map.nrows() - 1 { return; }

    if x.checked_sub(1).is_none() { return; }
    if y.checked_sub(1).is_none() { return; }

    let light_pass = 50.;

    if light_map[(y, x - 1)] > light_map[(y, x)] { 
        light_map[(y, x)] = (light_map[(y, x - 1)] - light_pass).max(0.);
    }

    if light_map[(y - 1, x)] > light_map[(y, x)] { 
        light_map[(y, x)] = (light_map[(y - 1, x)] - light_pass).max(0.);
    }

    if light_map[(y, x + 1)] > light_map[(y, x)] { 
        light_map[(y, x)] = (light_map[(y, x + 1)] - light_pass).max(0.);
    }

    if light_map[(y + 1, x)] > light_map[(y, x)] { 
        light_map[(y, x)] = (light_map[(y + 1, x)] - light_pass).max(0.);
    }
}

fn remove_extra_walls(world: &mut CellArray) {
    for x in 0..WORLD_SIZE_X {
        let mut y: usize = 0;

        loop {
            let cell = world.get((y, x));

            if let Some(cell) = cell {
                if let Some(tile) = cell.tile {
                    if tile.block == Block::Grass {
                        for cell in world.slice_mut(s![..y + 2, x]).iter_mut() {
                            cell.wall = None;
                        }

                        break;
                    }
                }
            }

            y += 1;
        }
    }
}

fn insert_specks<F: NoiseFn<f64, 2>>(
    world: &mut ArrayViewMut2<Cell>,
    noise: F,
    frequency: f64,
    size: f64,
    speck_block: Block,
) {
    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let a = noise.get([
                x as f64 / (WORLD_SIZE_X as f64 / 30. * size),
                y as f64 / (WORLD_SIZE_Y as f64 / 30. * size),
            ]);

            if a > (frequency * 10.).powi(-1) {
                world[[y, x]].tile = Some(Tile { block: speck_block });
            }
        }
    }
}

fn add_grass(world: &mut CellArray, level: Level) {
    println!("Adding grass...");

    for x in 0..world.ncols() {
        let mut prev_block: Option<Block> = None;
        let mut y: usize = 0;

        loop {
            if y >= level.stone.1 {
                break;
            }

            let block = world
                .get((y, x))
                .and_then(|cell| cell.tile)
                .map(|tile| tile.block);

            if let Some(Block::Dirt) = block {
                if prev_block.is_none() {
                    world[[y, x]].tile = Some(Tile { block: Block::Grass });
                }
            }

            prev_block = block;
            y += 1;
        }
    }
}

fn make_surface_rough<F: NoiseFn<f64, 2>>(
    world: &mut CellArray,
    terrain_noise: F,
    start_y: usize,
    q: f64,
    height: f64,
    down: Option<Block>,
    up: Option<Block>,
) {
    for x in 0..world.ncols() {
        let a = terrain_noise.get([
            x as f64 / (world.ncols() as f64 / q),
            x as f64 / (world.nrows() as f64 / q),
        ]);

        let y = a * height;

        let yu = y.abs() as usize;

        if y > 0. {
            replace(&mut world.slice_mut(s![start_y - yu..start_y, x]), down, up);
        } else {
            replace(&mut world.slice_mut(s![start_y..start_y + yu, x]), up, down);
        }
    }
}

fn make_epic_cave<F: NoiseFn<f64, 2>>(
    world: &mut CellArray,
    epic_cave_noise: F,
    frequency: f64,
    threshold: f64,
) {
    println!("Making epic cave...");

    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let xf = x as f64;
            let yf = y as f64;

            let k = (epic_cave_noise.get([frequency * xf, frequency * yf])
                    // + epic_cave_noise.get([2. * frequency * xf, 2. * frequency * yf]) * 0.5
                    // + epic_cave_noise.get([4. * frequency * xf, 4. * frequency * yf]) * 1./4.
                    + epic_cave_noise.get([8. * frequency * xf, 8. * frequency * yf]) * 1./8.)
                .abs();

            if k < threshold {
                world[[y, x]].tile = None;
            }
        }
    }
}

fn make_caves<F: NoiseFn<f64, 2>>(
    world: &mut CellArray,
    noise: F,
    max_level: usize,
    frequency: f64,
    threshold: f64,
) {
    println!("Making caves...");

    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let mut k = noise.get([
                x as f64 / (WORLD_SIZE_X as f64 * frequency),
                y as f64 / (WORLD_SIZE_Y as f64 * frequency),
            ]) + noise.get([
                x as f64 / (WORLD_SIZE_X as f64 * frequency * 4.),
                y as f64 / (WORLD_SIZE_Y as f64 * frequency),
            ]) * 1.
                / 4.;
            // + noise.get([x as f64 / (WORLD_SIZE_X as f64 / 50.), y as f64 / (WORLD_SIZE_Y as f64 / 50.)]);

            if y.abs_diff(max_level) < 30 {
                k *= 0.000000001;
            }

            if k > threshold {
                world[[y, x]].tile = None;
            }
        }
    }

    make_small_caves(world, noise, max_level);
}

fn make_small_caves<F: NoiseFn<f64, 2>>(world: &mut CellArray, noise: F, max_level: usize) {
    let q = 120.;

    for y in 0..world.nrows() {
        for x in 0..world.ncols() {
            let mut k = noise.get([
                x as f64 / (WORLD_SIZE_X as f64 / q),
                y as f64 / (WORLD_SIZE_Y as f64 / q),
            ]) + noise.get([
                x as f64 / (WORLD_SIZE_X as f64 / q + 10.),
                y as f64 / (WORLD_SIZE_Y as f64 / q + 10.),
            ]) + noise.get([
                x as f64 / (WORLD_SIZE_X as f64 / q + 20.),
                y as f64 / (WORLD_SIZE_Y as f64 / q + 20.),
            ]);

            if y.abs_diff(max_level) < 20 {
                k *= 0.5;
            }

            if k > 0.8 {
                world[[y, x]].tile = None;
            }
        }
    }
}

fn replace<D: Dimension>(
    world: &mut ArrayViewMut<Cell, D>,
    replace: Option<Block>,
    replacement: Option<Block>,
) {
    for cell in world.iter_mut() {
        if cell.tile.map(|tile| tile.block) == replace {
            cell.tile = replacement.map(|block| Tile { block })
        }
    }
}

#[inline]
fn insert_stone_specks_into_dirt<F: NoiseFn<f64, 2>>(
    world: &mut CellArray,
    level: Level,
    noise: F,
) {
    println!("Inserting stone specks into dirt...");

    insert_specks(
        &mut world.slice_mut(s![level.dirt.0 + 40..level.dirt.1, ..]),
        noise,
        0.5,
        0.7,
        Block::Stone,
    );
}

#[inline]
fn insert_dirt_specks_into_stone<F: NoiseFn<f64, 2>>(
    world: &mut CellArray,
    level: Level,
    noise: F,
) {
    println!("Inserting dirt specks into stone...");

    insert_specks(
        &mut world.slice_mut(s![level.stone.0..level.stone.1, ..]),
        noise,
        0.2,
        0.1,
        Block::Dirt,
    );
}


impl Tile {
    pub fn get_sprite_index(neighbors: &Neighbors<Block>, block: Block) -> u32 {
        /*
         * "$" - Any block
         * "#" - Dirt
         * "X" - This block
        */

        let rand: u32 = thread_rng().gen_range(0..3);

        let mut index = Self::get_sprite_index_by_neighbors(neighbors, rand);

        if block.dirt_mergable() {
            if let Some(idx) = Self::get_sprite_index_by_dirt_connections(neighbors, rand) {
                index = idx;
            }
        }

        get_tile_start_index(block) + index
    }

    fn get_sprite_index_by_dirt_connections(neighbors: &Neighbors<Block>, rand: u32) -> Option<u32> {
        match neighbors {
            //  #
            // #X#
            //  #
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass), 
                south: Some(Block::Dirt), 
                west: Some(Block::Dirt), 
                east: Some(Block::Dirt),
                .. 
            }
            => Some(16 * 11 + 6 + rand),

            //  #
            // $X$
            //  $
            Neighbors {
                north: Some(Block::Dirt | Block::Grass), 
                south: Some(bb), 
                west: Some(bl), 
                east: Some(br),
                ..
            } if *bb != Block::Dirt && *bl != Block::Dirt && *br != Block::Dirt
            => Some(16 * 6 + 8 + rand),

            //
            // #X
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: Some(Block::Dirt), 
                east: None,
                .. 
            }
            => Some(13 * 16 + rand),

            //
            // X#
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: None, 
                east: Some(Block::Dirt),
                .. 
            }
            => Some(13 * 16 + 3 + rand),

            //  $
            // $X#
            //  $
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(Block::Dirt),
                .. 
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bl != Block::Dirt && *bb != Block::Dirt
            => Some((7 + rand) * 16 + 8),

            //
            // X
            // #
            Neighbors { 
                north: None, 
                south: Some(Block::Dirt), 
                west: None,  
                east: None,
                .. 
            }
            => Some((5 + rand) * 16 + 6),

            //  $
            // $X
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(Block::Dirt),
                west: Some(bl),
                east: None,
                .. 
            } if *bl != Block::Dirt && *bt != Block::Dirt
            => Some((5 + rand) * 16 + 5),

            // #
            // X$
            // $
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass),
                south: Some(bb),
                west: None,
                east: Some(br),
                ..
            } if *br != Block::Dirt && *bb != Block::Dirt
            => Some((8 + rand) * 16 + 4),

            // $
            // X$
            // #
            Neighbors { 
                north: Some(bt),
                south: Some(Block::Dirt),
                west: None,
                east: Some(br),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *br != Block::Dirt
            => Some((5 + rand) * 16 + 4),

            //  #
            // $X
            //  $
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass),
                south: Some(bb),
                west: Some(bl),
                east: None,
                ..
            } if *bb != Block::Dirt && *bl != Block::Dirt
            => Some((8 + rand) * 16 + 5),

            //  #
            //  X
            //
            Neighbors {
                north: Some(Block::Dirt | Block::Grass),
                south: None, 
                west: None, 
                east: None,
                ..
            }
            => Some((8 + rand) * 16 + 6),

            //  #
            // #X#
            //  $
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass), 
                south: Some(bb),
                west: Some(Block::Dirt),
                east: Some(Block::Dirt),
                ..
            } if *bb != Block::Dirt
            => Some((5 + rand) * 16 + 11),

            //  $
            // #X#
            //  #
            Neighbors { 
                north: Some(bt), 
                south: Some(Block::Dirt),
                west: Some(Block::Dirt),
                east: Some(Block::Dirt),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass)
            => Some((8 + rand) * 16 + 11),

            // 
            // #X#
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(Block::Dirt),
                east: Some(Block::Dirt),
                ..
            }
            => Some(11 * 16 + 9 + rand),

            //  $
            // #X$
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(Block::Dirt),
                west: Some(Block::Dirt),
                east: Some(br),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *br != Block::Dirt
            => Some((6 + rand * 2) * 16 + 2),

            //  $
            // $X# 
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(Block::Dirt),
                west: Some(bl),
                east: Some(Block::Dirt),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bl != Block::Dirt
            => Some((6 + rand * 2) * 16 + 3),

            //  #
            // $X#
            //  $
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass),
                south: Some(bb),
                west: Some(bl),
                east: Some(Block::Dirt),
                ..
            } if *bb != Block::Dirt && *bl != Block::Dirt
            => Some((5 + rand * 2) * 16 + 3),

            //  #
            // #X$
            //  $
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass),
                south: Some(bb),
                west: Some(Block::Dirt),
                east: Some(br),
                ..
            } if *bb != Block::Dirt && *br != Block::Dirt
            => Some((5 + rand * 2) * 16 + 2),

            //  $
            // $X$
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(Block::Dirt),
                west: Some(bl),
                east: Some(br),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bl != Block::Dirt && *br != Block::Dirt
            => Some(5 * 16 + 8 + rand),

            //  #
            // $X$
            //
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass),
                south: None,
                west: Some(bl),
                east: Some(br),
                ..
            } if *bl != Block::Dirt && *br != Block::Dirt
            => Some(16 + 13 + rand),

            //  
            // $X$
            //  #
            Neighbors { 
                north: None,
                south: Some(Block::Dirt),
                west: Some(bl),
                east: Some(br),
                ..
            } if *bl != Block::Dirt && *br != Block::Dirt
            => Some(13 + rand),

            //  #
            //  X
            //  $
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass),
                south: Some(bb),
                west: None,
                east: None,
                ..
            } if *bb != Block::Dirt
            => Some((8 + rand) * 16 + 7),

            //  $
            //  X
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(Block::Dirt),
                west: None,
                east: None,
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass)
            => Some((5 + rand) * 16 + 7),

            // 
            // #X$
            // 
            Neighbors { 
                north: None,
                south: None,
                west: Some(Block::Dirt),
                east: Some(br),
                ..
            } if *br != Block::Dirt
            => Some(14 * 16 + rand),

            // 
            // $X#
            // 
            Neighbors { 
                north: None,
                south: None,
                west: Some(bl),
                east: Some(Block::Dirt),
                ..
            } if *bl != Block::Dirt
            => Some(14 * 16 + 3 + rand),

            //  #
            // $X$
            //  #
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass),
                south: Some(Block::Dirt),
                west: Some(bl),
                east: Some(br),
                ..
            } if *bl != Block::Dirt && *br != Block::Dirt
            => Some(10 * 16 + 8 + rand),

            //  #
            // #X$
            //  #
            Neighbors { 
                north: Some(Block::Dirt | Block::Grass),
                south: Some(Block::Dirt),
                west: Some(Block::Dirt),
                east: Some(br),
                ..
            } if *br != Block::Dirt
            => Some((5 + rand) * 16 + 12),

            //  $
            // #X$
            //  $
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(Block::Dirt),
                east: Some(br),
                ..
            } if *bt != Block::Dirt && *bb != Block::Dirt && *br != Block::Dirt
            => Some((7 + rand) * 16 + 9),

            //  $
            // $X#
            //  $
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(Block::Dirt),
                ..
            } if *bt != Block::Dirt && *bb != Block::Dirt && *bl != Block::Dirt
            => Some((7 + rand) * 16 + 8),

            //  
            // #X$
            //  $
            Neighbors { 
                north: None,
                south: Some(bb),
                west: Some(Block::Dirt),
                east: Some(br),
                ..
            } if *bb != Block::Dirt && *br != Block::Dirt
            => Some(11 * 16 + rand),

            //  $
            // #X
            //  $
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(Block::Dirt),
                east: None,
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bb != Block::Dirt
            => Some(3 * 16 + 13 + rand),

            //  #
            // $X#
            //  #
            Neighbors { 
                north: Some(Block::Dirt),
                south: Some(Block::Dirt),
                west: Some(bl),
                east: Some(Block::Dirt),
                ..
            } if *bl != Block::Dirt
            => Some((8 + rand) * 16 + 12),

            //  $
            // $X$
            //  $#
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                south_east: Some(Block::Dirt),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bb != Block::Dirt && *bl != Block::Dirt && *br != Block::Dirt
            => Some((5 + rand * 2) * 16),

            //  $#
            // $X$
            //  $
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                north_east: Some(Block::Dirt),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bb != Block::Dirt && *bl != Block::Dirt && *br != Block::Dirt
            => Some((6 + rand * 2) * 16),

            //  $
            // $X$
            // #$
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                south_west: Some(Block::Dirt),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bb != Block::Dirt && *bl != Block::Dirt && *br != Block::Dirt
            => Some((5 + rand * 2) * 16 + 1),

            // #$
            // $X$
            //  $
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                north_west: Some(Block::Dirt),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bb != Block::Dirt && *bl != Block::Dirt && *br != Block::Dirt
            => Some((6 + rand * 2) * 16 + 1),

            //  $
            // $X#
            //  
            Neighbors {
                north: Some(bt),
                south: None,
                west: Some(bl),
                east: Some(Block::Dirt),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bl != Block::Dirt
            => Some(12 * 16 + 3 + rand),

            //  
            // $X#
            //  $
            Neighbors {
                north: None,
                south: Some(bb),
                west: Some(bl),
                east: Some(Block::Dirt),
                ..
            } if *bb != Block::Dirt && *bl != Block::Dirt
            => Some(11 * 16 + 3 + rand),

            //  
            // #X$
            //  $
            Neighbors {
                north: None,
                south: Some(bb),
                west: Some(Block::Dirt),
                east: Some(br),
                ..
            } if *bb != Block::Dirt && *br != Block::Dirt
            => Some(11 * 16 + rand),

            //  $
            // #X$
            //  
            Neighbors {
                north: Some(bt),
                south: None,
                west: Some(Block::Dirt),
                east: Some(br),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *br != Block::Dirt
            => Some(12 * 16 + rand),

            //  $
            //  X#
            //  $
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: None,
                east: Some(Block::Dirt),
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bb != Block::Dirt
            => Some(2 * 16 + 13 + rand),

            //  $
            //  X#
            //  $
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(Block::Dirt),
                east: None,
                ..
            } if (*bt != Block::Dirt && *bt != Block::Grass) && *bb != Block::Dirt
            => Some(3 * 16 + 13 + rand),

            _ => None
        }
    }

    fn get_sprite_index_by_neighbors(neighbors: &Neighbors<Block>, rand: u32) -> u32 {
        match neighbors {
            //  $
            // $X$
            //  $
            Neighbors { 
                north: Some(_), 
                south: Some(_), 
                west: Some(_), 
                east: Some(_),
                ..
            } => 16 + 1 + rand,
            
            //
            // X
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: None, 
                east: None,
                ..
            } => 16 * 3 + rand + 9,

            // $
            // X
            //
            Neighbors { 
                north: Some(_), 
                south: None, 
                west: None, 
                east: None,
                ..
            } => 16 * 3 + rand + 6,

            //
            // X
            // $
            Neighbors { 
                north: None, 
                south: Some(_), 
                west: None, 
                east: None,
                ..
            } => rand + 6,

            //
            // $X
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: Some(_), 
                east: None,
                ..
            } => rand * 16 + 12,

            //
            //  X$
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: None, 
                east: Some(_),
                ..
            } => rand * 16 + 9,

            //  $
            //  X
            //  $
            Neighbors { 
                north: Some(_), 
                south: Some(_), 
                west: None, 
                east: None,
                ..
            } => rand * 16 + 5,

            //  $
            // $X$
            //
            Neighbors { 
                north: Some(_), 
                south: None,
                west: Some(_),
                east: Some(_),
                ..
            } => 16 * 2 + 1 + rand,

            //  
            // $X$
            //  $
            Neighbors { 
                north: None, 
                south: Some(_),
                west: Some(_),
                east: Some(_),
                ..
            } => rand + 1,

            //  
            // $X$
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(_),
                east: Some(_),
                ..
            } => 4 * 16 + 6 + rand,

            //  
            // $X
            //  $
            Neighbors { 
                north: None, 
                south: Some(_),
                west: Some(_),
                east: None,
                ..
            } => 16 * 3 + 1 + rand * 2,

            //  
            //  X$
            //  $
            Neighbors { 
                north: None, 
                south: Some(_),
                west: None,
                east: Some(_),
                ..
            } => 16 * 3 + rand * 2,

            //  $
            // $X
            //
            Neighbors { 
                north: Some(_),
                south: None,
                west: Some(_),
                east: None,
                ..
            } => 16 * 4 + 1 + rand * 2,

            //  $
            //  X$
            //
            Neighbors { 
                north: Some(_),
                south: None,
                west: None,
                east: Some(_),
                ..
            } => 16 * 4 + rand * 2,

            //  $
            // $X
            //  $
            Neighbors { 
                north: Some(_),
                south: Some(_),
                west: Some(_),
                east: None,
                ..
            } => rand * 16 + 4,

            //  $
            //  X$
            //  $
            Neighbors { 
                north: Some(_),
                south: Some(_),
                west: None,
                east: Some(_),
                ..
            } => rand * 16,
        }
    }
}

impl Wall {
    pub fn get_sprite_index(neighbors: Neighbors<WallType>, wall_type: WallType) -> u32 {
        let rand: u32 = thread_rng().gen_range(0..3);

        get_wall_start_index(wall_type) + match neighbors {
            //  #
            // #X#

            //  #
            Neighbors { 
                north: Some(bt), 
                south: Some(bb), 
                west: Some(bl), 
                east: Some(br),
                ..
            } if bt == wall_type && bb == wall_type && bl == wall_type && br == wall_type => 13 + 1 + rand,
            
            //
            // X
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: None, 
                east: None,
                ..
            } => 13 * 3 + 9 + rand,

            // #
            // X
            //
            Neighbors { 
                north: Some(b), 
                south: None, 
                west: None, 
                east: None,
                ..
            } if b == wall_type => 13 * 2 + 1 + rand,

            //
            // X
            // #
            Neighbors { 
                north: None, 
                south: Some(b), 
                west: None, 
                east: None,
                ..
            } if b == wall_type => rand + 6,

            //  #
            //  X
            //  #
            Neighbors { 
                north: Some(bt), 
                south: Some(bb), 
                west: None, 
                east: None,
                ..
            } if bt == wall_type && bb == wall_type => rand * 13 + 5,

            //  #
            // #X#
            //
            Neighbors { 
                north: Some(bt), 
                south: None,
                west: Some(bl),
                east: Some(br),
                ..
            } if bt == wall_type && bl == wall_type && br == wall_type => 13 * 2 + 1 + rand,

            //  
            // #X#
            //  #
            Neighbors { 
                north: None, 
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                ..
            } if bb == wall_type && bl == wall_type && br == wall_type => 1 + rand,

            //  
            // #X#
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(bl),
                east: Some(br),
                ..
            } if bl == wall_type && br == wall_type => 13 * 4 + 6 + rand,

            //  
            // #X
            //  #
            Neighbors { 
                north: None, 
                south: Some(bb),
                west: Some(bl),
                east: None,
                ..
            } if bb == wall_type && bl == wall_type => 13 * 3 + 1 + rand * 2,

            //  
            //  X#
            //  #
            Neighbors { 
                north: None, 
                south: Some(bb),
                west: None,
                east: Some(br),
                ..
            } if bb == wall_type && br == wall_type => 13 * 3 + rand * 2,

            //  #
            // #X
            //
            Neighbors { 
                north: Some(bt),
                south: None,
                west: Some(bl),
                east: None,
                ..
            } if bt == wall_type && bl == wall_type => 13 * 4 + 1 + rand * 2,

            //  #
            //  X#
            //
            Neighbors { 
                north: Some(bt),
                south: None,
                west: None,
                east: Some(br),
                ..
            } if bt == wall_type && br == wall_type => 13 * 4 + rand * 2,

            //  #
            // #X
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: None,
                ..
            } if bt == wall_type && bb == wall_type && bl == wall_type => 13 * rand + 4,

            //  #
            //  X#
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: None,
                east: Some(br),
                ..
            } if bt == wall_type && bb == wall_type && br == wall_type => 13 * rand,

            _ => panic!()
        }
    }
}
