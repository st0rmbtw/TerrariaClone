use bevy::prelude::{Color, Vec2};
use bevy_ecs_tilemap::{tiles::TilePos, helpers::square_grid::neighbors::{Neighbors, SquareDirection}, prelude::SquarePos};
use block::Block;
use plugins::{world::MAP_SIZE, player::Collisions};
use wall::Wall as WallType;
use world_generator::{Tile, Cell, CellArray, Wall};

#[macro_use]
extern crate lazy_static;

pub mod animation;
pub mod block;
pub mod items;
pub mod lens;
pub mod parallax;
pub mod plugins;
pub mod state;
pub mod util;
pub mod wall;
pub mod world_generator;
pub mod language;

pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);
pub const TEXT_COLOR: Color = Color::rgb(156. / 255., 156. / 255., 156. / 255.);

pub type Velocity = Vec2;

#[derive(Clone, Copy)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2
}

#[derive(Clone, Copy)]
pub struct Rect {
    // Center x
    cx: f32,
    // Center y
    cy: f32,
    // X velocity
    vx: f32,
    // Y velocity
    vy: f32,
    width: f32,
    height: f32,
}

impl Rect {
    pub fn new(cx: f32, cy: f32, vx: f32, vy: f32, width: f32, height: f32) -> Self {
        Self { cx, cy, vx, vy, width, height }
    }

    fn is_colliding(&self, other: Rect) -> bool {
        let self_x = self.cx - self.width / 2.;
        let self_y = self.cy - self.height / 2.;

        let other_x = other.cx - other.width / 2.;
        let other_y = other.cy - other.height / 2.;

        let left = other_x - (self_x + self.width);
        let top = (other_y + other.height) - self_y;
        let right = (other_x + other.width) - self_x;
        let bottom = other_y - (self_y + self.height);

        return !(left > 0. || right < 0. || top < 0. || bottom > 0.);
    }

    fn get_swept_broadphase_rect(&self) -> Rect {
        let cx = if self.vx > 0. { self.cx } else { self.cx + self.vx };
        let cy = if self.vy > 0. { self.cy } else { self.cy + self.vy };
        let w = self.width + self.vx.abs();
        let h = self.height + self.vy.abs();

        return Rect::new(cx, cy, 0., 0., w, h);
    }

    pub fn swept_aabb(&self, other: Rect) -> (f32, Collisions) {
        let mut collisions = Collisions::default();
        let rect = self.get_swept_broadphase_rect();

        if !rect.is_colliding(other) {
            return (1., collisions);
        }

        let self_x = self.cx - self.width / 2.;
        let self_y = self.cy - self.height / 2.;

        let other_x = other.cx - other.width / 2.;
        let other_y = other.cy - other.height / 2.;

        let dx_entry: f32;
        let dx_exit: f32;
        let tx_entry: f32;
        let tx_exit: f32;

        let dy_entry: f32;
        let dy_exit: f32;
        let ty_entry: f32;
        let ty_exit: f32;

        if other.vx > 0. {
            dx_entry = other_x - (self_x + self.width);
            dx_exit = (other_x + other.width) - self_x;
        } else {
            dx_entry = (other_x + other.width) - self_x;
            dx_exit = other_x - (self_x + self.width);
        }

        if other.vy > 0. {
            dy_entry = other_y - (self_y + self.height);
            dy_exit = (other_y + other.height) - self_y;
        } else {
            dy_entry = (other_y + other.height) - self_y;
            dy_exit = other_y - (self_y + self.height);
        }

        if self.vx == 0. {
            tx_entry = -f32::INFINITY;
            tx_exit = f32::INFINITY;
        } else {
            tx_entry = dx_entry / self.vx;
            tx_exit = dx_exit / self.vx;
        }

        if self.vy == 0. {
            ty_entry = -f32::INFINITY;
            ty_exit = f32::INFINITY;
        } else {
            ty_entry = dy_entry / self.vy;
            ty_exit = dy_exit / self.vy;
        }

        let entry_time = tx_entry.max(ty_entry);
        let exit_time = tx_exit.min(ty_exit);

        if entry_time > exit_time || (tx_entry < 0. && ty_entry < 0.) || tx_entry > 1. || ty_entry > 1. {
            return (1., collisions);
        }

        if tx_entry > ty_entry {
            if dx_entry > 0. {
                collisions.right = true;
            } else {
                collisions.left = true;
            }
        } else {
            if dy_entry > 0. {
                collisions.top = true;
            } else {
                collisions.bottom = true;
            }
        }

        return (entry_time, collisions);
    }
}

pub mod labels {
    use bevy::prelude::SystemLabel;

    #[derive(Debug, SystemLabel)]
    pub enum PlayerLabel {
        HorizontalMovement,
        Jump,
        Update
    }
}

pub trait To2dArrayIndex {
    fn to_2d_array_index(&self) -> (usize, usize);
}

impl To2dArrayIndex for TilePos {
    fn to_2d_array_index(&self) -> (usize, usize) {
        (self.y as usize, self.x as usize)
    }
}

pub trait CellArrayExtensions {
    fn get_cell(&self, pos: TilePos) -> Option<&Cell>;
    fn get_cell_mut(&mut self, pos: TilePos) -> Option<&mut Cell>;

    fn get_tile(&self, pos: TilePos) -> Option<&Tile>;
    fn get_tile_mut(&mut self, pos: TilePos) -> Option<&mut Tile>;

    fn get_wall(&self, pos: TilePos) -> Option<&Wall>;

    fn tile_exists(&self, pos: TilePos) -> bool;

    fn get_tile_neighbors(&self, pos: TilePos) -> Neighbors<Block>;
    fn get_wall_neighbors(&self, pos: TilePos) -> Neighbors<WallType>;

    fn set_cell(&mut self, pos: TilePos, cell: Cell);
    fn set_tile(&mut self, pos: TilePos, tile: Option<Tile>);
}

impl CellArrayExtensions for CellArray {
    fn get_cell(&self, pos: TilePos) -> Option<&Cell> {
        self.get(pos.to_2d_array_index())
    }

    fn get_cell_mut(&mut self, pos: TilePos) -> Option<&mut Cell> {
        self.get_mut(pos.to_2d_array_index())
    }

    fn get_tile(&self, pos: TilePos) -> Option<&Tile> {
        self.get(pos.to_2d_array_index()).and_then(|cell| cell.tile.as_ref())
    }

    fn get_tile_mut(&mut self, pos: TilePos) -> Option<&mut Tile> {
        self.get_mut(pos.to_2d_array_index()).and_then(|cell| cell.tile.as_mut())
    }

    fn get_wall(&self, pos: TilePos) -> Option<&Wall> {
        self.get(pos.to_2d_array_index()).and_then(|cell| cell.wall.as_ref())
    }

    fn tile_exists(&self, pos: TilePos) -> bool {
        self.get(pos.to_2d_array_index()).and_then(|cell| cell.tile).is_some()
    }

    fn get_tile_neighbors(&self, pos: TilePos) -> Neighbors<Block> {
        Neighbors {
            west: SquarePos::from(&pos).offset(&SquareDirection::West).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_tile(pos))
                .flatten()
                .map(|tile| tile.block),

            east: SquarePos::from(&pos).offset(&SquareDirection::East).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_tile(pos))
                .flatten()
                .map(|tile| tile.block),

            north: SquarePos::from(&pos).offset(&SquareDirection::South).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_tile(pos))
                .flatten()
                .map(|tile| tile.block),

            south: SquarePos::from(&pos).offset(&SquareDirection::North).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_tile(pos))
                .flatten()
                .map(|tile| tile.block),

            north_west: SquarePos::from(&pos).offset(&SquareDirection::SouthWest).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_tile(pos))
                .flatten()
                .map(|tile| tile.block),

            south_west: SquarePos::from(&pos).offset(&SquareDirection::NorthWest).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_tile(pos))
                .flatten()
                .map(|tile| tile.block),

            south_east: SquarePos::from(&pos).offset(&SquareDirection::NorthEast).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_tile(pos))
                .flatten()
                .map(|tile| tile.block),
            
            north_east: SquarePos::from(&pos).offset(&SquareDirection::SouthEast).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_tile(pos))
                .flatten()
                .map(|tile| tile.block),
                
        }
    }

    fn get_wall_neighbors(&self, pos: TilePos) -> Neighbors<WallType> {
        Neighbors {
            west: SquarePos::from(&pos).offset(&SquareDirection::West).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| wall.wall_type),

            east: SquarePos::from(&pos).offset(&SquareDirection::East).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| wall.wall_type),

            north: SquarePos::from(&pos).offset(&SquareDirection::South).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| wall.wall_type),

            south: SquarePos::from(&pos).offset(&SquareDirection::North).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| wall.wall_type),

            north_west: SquarePos::from(&pos).offset(&SquareDirection::SouthWest).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| wall.wall_type),

            south_west: SquarePos::from(&pos).offset(&SquareDirection::NorthWest).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| wall.wall_type),

            south_east: SquarePos::from(&pos).offset(&SquareDirection::NorthEast).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| wall.wall_type),
            
            north_east: SquarePos::from(&pos).offset(&SquareDirection::SouthEast).as_tile_pos(&MAP_SIZE)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| wall.wall_type),
        }
    }

    fn set_cell(&mut self, pos: TilePos, cell: Cell) {
        self[[pos.y as usize, pos.x as usize]] = cell;
    }

    fn set_tile(&mut self, pos: TilePos, tile: Option<Tile>) {
        if let Some(cell) = self.get_cell_mut(pos) {
            cell.tile = tile;
        } 
    }
}