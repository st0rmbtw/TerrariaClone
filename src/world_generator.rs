use autodefault::autodefault;
use bevy::prelude::{default, Entity};
use ndarray::prelude::*;
use noise::{NoiseFn, OpenSimplex, Seedable, SuperSimplex};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{block::Block, wall::Wall as WallType};

pub const WORLD_SIZE_X: usize = 1750;
pub const WORLD_SIZE_Y: usize = 900;

#[derive(Clone, Copy)]
pub struct Level {
    pub sky: (usize, usize),
    pub dirt: (usize, usize),
    pub stone: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Slope {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Slope {
    pub const NONE: Slope = Slope { top: false, bottom: false, left: false, right: false };
    pub const ALL: Slope = Slope { top: true, bottom: true, left: true, right: true };

    pub const TOP: Slope = Slope { top: true, ..Slope::NONE };
    pub const BOTTOM: Slope = Slope { bottom: true, ..Slope::NONE };
    pub const LEFT: Slope = Slope { left: true, ..Slope::NONE };
    pub const RIGHT: Slope = Slope { right: true, ..Slope::NONE };

    pub const TOP_BOTTOM: Slope = Slope { top: true, bottom: true, ..Slope::NONE };
    pub const LEFT_RIGHT: Slope = Slope { left: true, right: true, ..Slope::NONE };

    pub const TOP_LEFT: Slope = Slope { top: true, left: true, ..Slope::NONE };
    pub const TOP_RIGHT: Slope = Slope { top: true, right: true, ..Slope::NONE };
    pub const BOTTOM_LEFT: Slope = Slope { bottom: true, left: true, ..Slope::NONE };
    pub const BOTTOM_RIGHT: Slope = Slope { bottom: true, right: true, ..Slope::NONE };

    pub const TOP_BOTTOM_LEFT: Slope = Slope { top: true, bottom: true, left: true, ..Slope::NONE };
    pub const TOP_BOTTOM_RIGHT: Slope = Slope { top: true, bottom: true, right: true, ..Slope::NONE };
    pub const TOP_LEFT_RIGHT: Slope = Slope { top: true, left: true, right: true, ..Slope::NONE };
    pub const BOTTOM_LEFT_RIGHT: Slope = Slope { bottom: true, left: true, right: true, ..Slope::NONE };
}

impl Slope {
    pub fn is_all(&self) -> bool {
        self.top && self.bottom && self.left && self.right
    }

    pub fn is_none(&self) -> bool {
        !self.top && !self.bottom && !self.left && !self.right
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Wall {
    pub wall_type: WallType,
    pub slope: Slope,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    pub tile_type: Block,
    pub slope: Slope,
}
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Cell {
    pub tile: Option<Tile>,
    pub tile_entity: Option<Entity>,
    pub wall: Option<Wall>,
    pub wall_entity: Option<Entity>,
}

#[autodefault(except(Level, Tile, Wall))]
pub fn generate(seed: u32) -> Array2<Cell> {
    // region: Init world

    let mut rng = StdRng::seed_from_u64(seed as u64);

    println!("Initializing world...");

    let mut world = Array2::<Cell>::default((WORLD_SIZE_Y, WORLD_SIZE_X));
    world.fill(Cell {
        tile: Some(Tile {
            tile_type: Block::Stone,
            slope: Slope::default(),
        }),
        wall: Some(Wall {
            wall_type: WallType::DirtWall,
            slope: Slope::default(),
        }),
    });

    let level = Level {
        sky: (0, WORLD_SIZE_Y / 5),
        dirt: (WORLD_SIZE_Y / 5, WORLD_SIZE_Y / 3),
        stone: (WORLD_SIZE_Y / 3, world.nrows()),
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

    world.slice_collapse(s![level.sky.1.., ..]);

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

    set_tile_slope(&mut world);

    world
}

fn remove_extra_walls(world: &mut Array2<Cell>) {
    for x in 0..WORLD_SIZE_X {
        let mut y: usize = 0;

        loop {
            let cell = world.get((y, x));

            if let Some(cell) = cell {
                if let Some(tile) = cell.tile {
                    if tile.tile_type == Block::Grass {
                        for cell in world.slice_mut(s![..y + 1, x]).iter_mut() {
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

fn insert_specks<F: NoiseFn<[f64; 2]>>(
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
                world[[y, x]].tile = Some(Tile {
                    tile_type: speck_block,
                    slope: Slope::default(),
                });
            }
        }
    }
}

fn add_grass(world: &mut Array2<Cell>, level: Level) {
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
                .map(|tile| tile.tile_type);

            if let Some(Block::Dirt) = block {
                if prev_block.is_none() {
                    world[[y, x]].tile = Some(Tile {
                        tile_type: Block::Grass,
                        slope: Slope::default(),
                    });
                }
            }

            prev_block = block;
            y += 1;
        }
    }
}

fn make_surface_rough<F: NoiseFn<[f64; 2]>>(
    world: &mut Array2<Cell>,
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

fn make_epic_cave<F: NoiseFn<[f64; 2]>>(
    world: &mut Array2<Cell>,
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

fn make_caves<F: NoiseFn<[f64; 2]>>(
    world: &mut Array2<Cell>,
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

fn make_small_caves<F: NoiseFn<[f64; 2]>>(world: &mut Array2<Cell>, noise: F, max_level: usize) {
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
        if cell.tile.map(|tile| tile.tile_type) == replace {
            cell.tile = replacement.map(|block| Tile {
                tile_type: block,
                slope: Slope::default(),
            })
        }
    }
}

#[inline]
fn insert_stone_specks_into_dirt<F: NoiseFn<[f64; 2]>>(
    world: &mut Array2<Cell>,
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
fn insert_dirt_specks_into_stone<F: NoiseFn<[f64; 2]>>(
    world: &mut Array2<Cell>,
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

fn set_tile_slope(world: &mut Array2<Cell>) {
    for y in 0..WORLD_SIZE_Y {
        for x in 0..WORLD_SIZE_X {
            if let Some(cell) = world.get((y, x)).cloned() {
                let mut new_tile = cell.tile;
                let mut new_wall = cell.wall;

                let prev_y_option = y.checked_sub(1);
                let prev_x_option = x.checked_sub(1);
                let next_y_option = y.checked_add(1);
                let next_x_option = x.checked_add(1);

                let prev_y = prev_y_option.unwrap_or(y);
                let prev_x = prev_x_option.unwrap_or(x);
                let next_y = next_y_option.unwrap_or(y);
                let next_x = next_x_option.unwrap_or(x);

                if cell.tile.is_some() {
                    new_tile = Some(Tile {
                        slope: Slope {
                            left: prev_x_option
                                .and(world.get((y, prev_x)))
                                .and_then(|t| t.tile)
                                .is_some(),
                            right: next_x_option
                                .and(world.get((y, next_x)))
                                .and_then(|t| t.tile)
                                .is_some(),
                            top: prev_y_option
                                .and(world.get((prev_y, x)))
                                .and_then(|t| t.tile)
                                .is_some(),
                            bottom: next_y_option
                                .and(world.get((next_y, x)))
                                .and_then(|t| t.tile)
                                .is_some(),
                        },
                        ..cell.tile.unwrap()
                    });
                }

                if cell.wall.is_some() {
                    new_wall = Some(Wall {
                        slope: Slope {
                            left: prev_x_option
                                .and(world.get((y, prev_x)))
                                .and_then(|t| t.wall)
                                .is_some(),
                            right: next_x_option
                                .and(world.get((y, next_x)))
                                .and_then(|t| t.wall)
                                .is_some(),
                            top: prev_y_option
                                .and(world.get((prev_y, x)))
                                .and_then(|t| t.wall)
                                .is_some(),
                            bottom: next_y_option
                                .and(world.get((next_y, x)))
                                .and_then(|t| t.wall)
                                .is_some(),
                        },
                        ..cell.wall.unwrap()
                    });
                }

                world[[y, x]] = Cell {
                    tile: new_tile,
                    wall: new_wall,
                    ..default()
                };
            }
        }
    }
}
