use autodefault::autodefault;
use bevy::prelude::default;
use ndarray::prelude::*;
use noise::{NoiseFn, OpenSimplex, Seedable, SuperSimplex};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{block::Block, wall::Wall as WallType};

pub const WORLD_SIZE_X: usize = 1750;
pub const WORLD_SIZE_Y: usize = 900;

pub type CellArray = Array2<Cell>;

#[derive(Clone, Copy)]
pub struct Level {
    pub sky: (usize, usize),
    pub dirt: (usize, usize),
    pub stone: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Neighbors {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Neighbors {
    pub const NONE: Neighbors = Neighbors { top: false, bottom: false, left: false, right: false };
    pub const ALL: Neighbors = Neighbors { top: true, bottom: true, left: true, right: true };

    pub const TOP: Neighbors = Neighbors { top: true, ..Neighbors::NONE };
    pub const BOTTOM: Neighbors = Neighbors { bottom: true, ..Neighbors::NONE };
    pub const LEFT: Neighbors = Neighbors { left: true, ..Neighbors::NONE };
    pub const RIGHT: Neighbors = Neighbors { right: true, ..Neighbors::NONE };

    pub const TOP_BOTTOM: Neighbors = Neighbors { top: true, bottom: true, ..Neighbors::NONE };
    pub const LEFT_RIGHT: Neighbors = Neighbors { left: true, right: true, ..Neighbors::NONE };

    pub const TOP_LEFT: Neighbors = Neighbors { top: true, left: true, ..Neighbors::NONE };
    pub const TOP_RIGHT: Neighbors = Neighbors { top: true, right: true, ..Neighbors::NONE };
    pub const BOTTOM_LEFT: Neighbors = Neighbors { bottom: true, left: true, ..Neighbors::NONE };
    pub const BOTTOM_RIGHT: Neighbors = Neighbors { bottom: true, right: true, ..Neighbors::NONE };

    pub const TOP_BOTTOM_LEFT: Neighbors = Neighbors { top: true, bottom: true, left: true, ..Neighbors::NONE };
    pub const TOP_BOTTOM_RIGHT: Neighbors = Neighbors { top: true, bottom: true, right: true, ..Neighbors::NONE };
    pub const TOP_LEFT_RIGHT: Neighbors = Neighbors { top: true, left: true, right: true, ..Neighbors::NONE };
    pub const BOTTOM_LEFT_RIGHT: Neighbors = Neighbors { bottom: true, left: true, right: true, ..Neighbors::NONE };

    pub fn is_all(&self) -> bool {
        self.top && self.bottom && self.left && self.right
    }

    pub fn is_none(&self) -> bool {
        !self.top && !self.bottom && !self.left && !self.right
    }

    pub fn or(&self, neighbors: Neighbors) -> Neighbors {
        Neighbors { 
            top: self.top || neighbors.top, 
            bottom: self.bottom || neighbors.bottom, 
            left: self.left || neighbors.left, 
            right: self.right || neighbors.right 
        }
    }

    pub fn not(&self) -> Neighbors {
        Neighbors { 
            top: !self.top, 
            bottom: !self.bottom, 
            left: !self.left, 
            right: !self.right 
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum DirtConnection {
    Connected,
    // Is connected with the same tile
    NotConnected(bool)
}

impl Default for DirtConnection {
    fn default() -> Self {
        Self::NotConnected(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DirtConnections {
    pub top: DirtConnection,
    pub bottom: DirtConnection,
    pub left: DirtConnection,
    pub right: DirtConnection,
}

impl DirtConnections {
//     pub const NONE: DirtConnections = DirtConnections { top: DirtConnection::NotConnected(false), bottom: DirtConnection::NotConnected(false), left: DirtConnection::NotConnected(false), right: DirtConnection::NotConnected(false) };
//     pub const ALL: DirtConnections = DirtConnections { top: true, bottom: true, left: true, right: true };

//     pub const TOP: DirtConnections = DirtConnections { top: true, ..DirtConnections::NONE };
//     pub const BOTTOM: DirtConnections = DirtConnections { bottom: true, ..DirtConnections::NONE };
//     pub const LEFT: DirtConnections = DirtConnections { left: true, ..DirtConnections::NONE };
//     pub const RIGHT: DirtConnections = DirtConnections { right: true, ..DirtConnections::NONE };

//     pub const TOP_BOTTOM: DirtConnections = DirtConnections { top: true, bottom: true, ..DirtConnections::NONE };
//     pub const LEFT_RIGHT: DirtConnections = DirtConnections { left: true, right: true, ..DirtConnections::NONE };

//     pub const TOP_LEFT: DirtConnections = DirtConnections { top: true, left: true, ..DirtConnections::NONE };
//     pub const TOP_RIGHT: DirtConnections = DirtConnections { top: true, right: true, ..DirtConnections::NONE };
//     pub const BOTTOM_LEFT: DirtConnections = DirtConnections { bottom: true, left: true, ..DirtConnections::NONE };
//     pub const BOTTOM_RIGHT: DirtConnections = DirtConnections { bottom: true, right: true, ..DirtConnections::NONE };

//     pub const TOP_BOTTOM_LEFT: DirtConnections = DirtConnections { top: true, bottom: true, left: true, ..DirtConnections::NONE };
//     pub const TOP_BOTTOM_RIGHT: DirtConnections = DirtConnections { top: true, bottom: true, right: true, ..DirtConnections::NONE };
//     pub const TOP_LEFT_RIGHT: DirtConnections = DirtConnections { top: true, left: true, right: true, ..DirtConnections::NONE };
//     pub const BOTTOM_LEFT_RIGHT: DirtConnections = DirtConnections { bottom: true, left: true, right: true, ..DirtConnections::NONE };

    pub fn any(&self) -> bool {
        self.left == DirtConnection::Connected || 
        self.right == DirtConnection::Connected || 
        self.top == DirtConnection::Connected || 
        self.bottom == DirtConnection::Connected
    }
}

#[derive(Clone, Copy)]
pub struct Wall {
    pub wall_type: WallType,
    pub neighbors: Neighbors,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub tile_type: Block,
    pub neighbors: Neighbors,
    pub dirt_connections: DirtConnections
}

#[derive(Clone, Copy, Default)]
pub struct Cell {
    pub tile: Option<Tile>,
    pub wall: Option<Wall>,
}

#[autodefault(except(Level, Tile, Wall))]
pub fn generate(seed: u32) -> CellArray {
    // region: Init world

    let mut rng = StdRng::seed_from_u64(seed as u64);

    println!("Initializing world...");

    let mut world = Array2::<Cell>::default((WORLD_SIZE_Y, WORLD_SIZE_X));
    world.fill(Cell {
        tile: Some(Tile {
            tile_type: Block::Stone,
            neighbors: Neighbors::default(),
            dirt_connections: DirtConnections::default()
        }),
    });

    for cell in world.slice_mut(s![.., 0..WORLD_SIZE_X]).iter_mut() {
        cell.wall = Some(Wall { wall_type: WallType::DirtWall, neighbors: Neighbors::default() });
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

    set_tile_neighbors(&mut world);

    set_tile_dirt_connections(&mut world);

    world
}

fn remove_extra_walls(world: &mut CellArray) {
    for x in 0..WORLD_SIZE_X {
        let mut y: usize = 0;

        loop {
            let cell = world.get((y, x));

            if let Some(cell) = cell {
                if let Some(tile) = cell.tile {
                    if tile.tile_type == Block::Grass {
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
                world[[y, x]].tile = Some(Tile {
                    tile_type: speck_block,
                    neighbors: Neighbors::default(),
                    dirt_connections: DirtConnections::default(),
                });
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
                .map(|tile| tile.tile_type);

            if let Some(Block::Dirt) = block {
                if prev_block.is_none() {
                    world[[y, x]].tile = Some(Tile {
                        tile_type: Block::Grass,
                        neighbors: Neighbors::default(),
                        dirt_connections: DirtConnections::default(),
                    });
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
        if cell.tile.map(|tile| tile.tile_type) == replace {
            cell.tile = replacement.map(|block| Tile {
                tile_type: block,
                neighbors: Neighbors::default(),
                dirt_connections: DirtConnections::default(),
            })
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

fn set_tile_neighbors(world: &mut CellArray) {
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
                        neighbors: Neighbors {
                            left: x == 0 || world.get((y, prev_x))
                                .and_then(|t| t.tile)
                                .is_some(),
                            right: x == (WORLD_SIZE_X - 1) || world.get((y, next_x))
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
                        neighbors: Neighbors {
                            left: x == 0 || world.get((y, prev_x))
                                .and_then(|t| t.wall)
                                .is_some(),
                            right: x == (WORLD_SIZE_X - 1) || world.get((y, next_x))
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

fn set_tile_dirt_connections(world: &mut CellArray) {
    for y in 0..WORLD_SIZE_Y {
        for x in 0..WORLD_SIZE_X {
            if let Some(cell) = world.get((y, x)).cloned() {
                let mut new_tile = cell.tile;

                if let Some(tile) = &mut new_tile {
                    if tile.tile_type.merge_with_dirt() {
                        tile.dirt_connections = get_dirt_connections((y, x), tile.tile_type, world);
                    }
                } 

                world[[y, x]] = Cell {
                    tile: new_tile,
                    ..cell
                };
            }
        }
    }
}

pub fn get_dirt_connections(tile_pos: (usize, usize), block: Block, world: &CellArray) -> DirtConnections {
    let y = tile_pos.0;
    let x = tile_pos.1;

    let mut dirt_connections = DirtConnections::default();

    if x != 0 {
        let tile = world.get((y, x - 1)).and_then(|cell| cell.tile);

        dirt_connections.left = if let Some(tile) = tile {
            if tile.tile_type == Block::Dirt {
                DirtConnection::Connected
            } else {
                DirtConnection::NotConnected(tile.tile_type == block)
            }
        } else {
            DirtConnection::NotConnected(false)
        }
    }

    if x != WORLD_SIZE_X - 1 {
        let tile = world.get((y, x + 1)).and_then(|cell| cell.tile);

        dirt_connections.right = if let Some(tile) = tile {
            if tile.tile_type == Block::Dirt {
                DirtConnection::Connected
            } else {
                DirtConnection::NotConnected(tile.tile_type == block)
            }
        } else {
            DirtConnection::NotConnected(false)
        }
    }

    if y != 0 { 
        let tile = world.get((y - 1, x)).and_then(|cell| cell.tile);

        dirt_connections.top = if let Some(tile) = tile {
            if tile.tile_type == Block::Dirt {
                DirtConnection::Connected
            } else {
                DirtConnection::NotConnected(tile.tile_type == block)
            }
        } else {
            DirtConnection::NotConnected(false)
        }
    }

    if y != WORLD_SIZE_Y - 1 {
        let tile =  world.get((y + 1, x)).and_then(|cell| cell.tile);

        dirt_connections.bottom = if let Some(tile) = tile {
            if tile.tile_type == Block::Dirt {
                DirtConnection::Connected
            } else {
                DirtConnection::NotConnected(tile.tile_type == block)
            }
        } else {
            DirtConnection::NotConnected(false)
        }
    }

    dirt_connections
}