use std::ops::Deref;

use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use rand::{thread_rng, Rng};

use crate::{common::{helpers::get_wall_start_index, TextureAtlasPos}, items::ItemWall};

pub(crate) type WallId = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallType {
    Stone,
    Dirt,
}

impl WallType {
    pub fn max_hp(&self) -> i32 {
        70
    }

    pub fn id(&self) -> WallId {
        match self {
            WallType::Stone => 1,
            WallType::Dirt => 2,
        }
    }

    pub fn color(&self) -> [u8; 3] {
        match self {
            WallType::Stone => [52, 52, 52],
            WallType::Dirt => [88, 61, 46],
        }
    }
}

impl From<ItemWall> for WallType {
    fn from(item: ItemWall) -> Self {
        match item {
            ItemWall::Dirt => WallType::Dirt,
            ItemWall::Stone => WallType::Stone,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wall {
    pub wall_type: WallType,
    pub hp: i32,
    pub cracks_index: Option<u32>,
    pub variant: u32
}

impl From<WallType> for Wall {
    fn from(block_type: WallType) -> Self {
        let mut rng = thread_rng();
        Wall::new(block_type, rng.gen_range(0..3))
    }
}

impl Deref for Wall {
    type Target = WallType;

    fn deref(&self) -> &Self::Target {
        &self.wall_type
    }
}

impl Wall {
    #[inline(always)]
    pub(crate) fn new(wall_type: WallType, variant: u32) -> Wall {
        Self {
            wall_type,
            variant,
            hp: wall_type.max_hp(),
            cracks_index: None
        }
    }
}

impl Wall {
    pub(crate) fn get_sprite_index(neighbors: &Neighbors<WallType>, wall: &Wall) -> TextureAtlasPos {
        let variant = wall.variant;

        get_wall_start_index(wall.wall_type) + match *neighbors {
            //  #
            // #X#
            //  #
            Neighbors {
                north: Some(_),
                south: Some(_),
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(1 + variant, 1),
            
            //
            // X
            //
            Neighbors {
                north: None,
                south: None,
                west: None,
                east: None,
                ..
            } => TextureAtlasPos::new(9 + variant, 3),

            // #
            // X
            //
            Neighbors {
                north: Some(_),
                south: None,
                west: None,
                east: None,
                ..
            } => TextureAtlasPos::new(6 + variant, 3),

            //
            // X
            // #
            Neighbors {
                north: None,
                south: Some(_),
                west: None,
                east: None,
                ..
            } => TextureAtlasPos::new(6 + variant, 0),

            //
            // #X
            //
            Neighbors {
                north: None,
                south: None,
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(12, variant),

            //
            // X#
            //
            Neighbors {
                north: None,
                south: None,
                west: None,
                east: Some(_),
                ..
            } => TextureAtlasPos::new(9, variant),

            //  #
            //  X
            //  #
            Neighbors {
                north: Some(_),
                south: Some(_),
                west: None,
                east: None,
                ..
            } => TextureAtlasPos::new(5, variant),

            //  #
            // #X#
            //
            Neighbors {
                north: Some(_),
                south: None,
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(1 + variant, 2),

            //  
            // #X#
            //  #
            Neighbors {
                north: None,
                south: Some(_),
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(1 + variant, 0),

            //  
            // #X#
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(6 + variant, 4),

            //  
            // #X
            //  #
            Neighbors { 
                north: None, 
                south: Some(_),
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(1 + variant * 2, 3),

            //  
            //  X#
            //  #
            Neighbors { 
                north: None, 
                south: Some(_),
                west: None,
                east: Some(_),
                ..
            } => TextureAtlasPos::new(variant * 2, 3),

            //  #
            // #X
            //
            Neighbors { 
                north: Some(_),
                south: None,
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(1 + variant * 2, 4),

            //  #
            //  X#
            //
            Neighbors { 
                north: Some(_),
                south: None,
                west: None,
                east: Some(_),
                ..
            } => TextureAtlasPos::new(variant * 2, 4),

            //  #
            // #X
            //  #
            Neighbors { 
                north: Some(_),
                south: Some(_),
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(4, variant),

            //  #
            //  X#
            //  #
            Neighbors { 
                north: Some(_),
                south: Some(_),
                west: None,
                east: Some(_),
                ..
            } => TextureAtlasPos::new(0, variant),
        }
    }
}