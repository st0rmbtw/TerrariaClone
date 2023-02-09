use bevy::prelude::{Color, Vec2};
use bevy_ecs_tilemap::{tiles::TilePos, helpers::square_grid::neighbors::{Neighbors, SquareDirection}, prelude::SquarePos};
use block::Block;
use plugins::world::MAP_SIZE;
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

pub mod labels {
    use bevy::prelude::SystemLabel;

    #[derive(Debug, SystemLabel)]
    pub enum PlayerLabel {
        HorizontalMovement,
        Jump,
        Gravity,
        Collide,
        MovePlayer,
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