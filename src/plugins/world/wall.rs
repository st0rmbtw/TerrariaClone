use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use rand::{thread_rng, Rng};

use crate::common::{helpers::get_wall_start_index, TextureAtlasPos};

use super::generator::WallId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wall {
    Stone,
    Dirt,
}

impl Wall {
    pub const fn id(&self) -> WallId {
        match self {
            Wall::Stone => 1,
            Wall::Dirt => 2,
        }
    }
}

impl Wall {
    pub fn get_sprite_index(neighbors: Neighbors<Wall>, wall: Wall) -> TextureAtlasPos {
        let rand: u32 = thread_rng().gen_range(0..3);

        get_wall_start_index(wall) + match neighbors {
            //  #
            // #X#
            //  #
            Neighbors { 
                north: Some(bt), 
                south: Some(bb), 
                west: Some(bl), 
                east: Some(br),
                ..
            } if bt == wall && bb == wall && bl == wall && br == wall => TextureAtlasPos::new(1 + rand, 1),
            
            //
            // X
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: None, 
                east: None,
                ..
            } => TextureAtlasPos::new(9 + rand, 3),

            // #
            // X
            //
            Neighbors { 
                north: Some(b), 
                south: None, 
                west: None, 
                east: None,
                ..
            } if b == wall => TextureAtlasPos::new(1 + rand, 2),

            //
            // X
            // #
            Neighbors { 
                north: None, 
                south: Some(b), 
                west: None, 
                east: None,
                ..
            } if b == wall => TextureAtlasPos::new(6 + rand, 0),

            //
            // #X
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(b), 
                east: None,
                ..
            } if b == wall => TextureAtlasPos::new(13, rand),

            //
            // X#
            //
            Neighbors { 
                north: None, 
                south: None,
                west: None, 
                east: Some(b),
                ..
            } if b == wall => TextureAtlasPos::new(10, rand),

            //  #
            //  X
            //  #
            Neighbors { 
                north: Some(bt), 
                south: Some(bb), 
                west: None, 
                east: None,
                ..
            } if bt == wall && bb == wall => TextureAtlasPos::new(6, rand),

            //  #
            // #X#
            //
            Neighbors { 
                north: Some(bt), 
                south: None,
                west: Some(bl),
                east: Some(br),
                ..
            } if bt == wall && bl == wall && br == wall => TextureAtlasPos::new(1 + rand, 2),

            //  
            // #X#
            //  #
            Neighbors { 
                north: None, 
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                ..
            } if bb == wall && bl == wall && br == wall => TextureAtlasPos::new(1 + rand, 0),

            //  
            // #X#
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(bl),
                east: Some(br),
                ..
            } if bl == wall && br == wall => TextureAtlasPos::new(6 + rand, 4),

            //  
            // #X
            //  #
            Neighbors { 
                north: None, 
                south: Some(bb),
                west: Some(bl),
                east: None,
                ..
            } if bb == wall && bl == wall => TextureAtlasPos::new(1 + rand * 2, 3),

            //  
            //  X#
            //  #
            Neighbors { 
                north: None, 
                south: Some(bb),
                west: None,
                east: Some(br),
                ..
            } if bb == wall && br == wall => TextureAtlasPos::new(rand * 2, 3),

            //  #
            // #X
            //
            Neighbors { 
                north: Some(bt),
                south: None,
                west: Some(bl),
                east: None,
                ..
            } if bt == wall && bl == wall => TextureAtlasPos::new(1 + rand * 2, 4),

            //  #
            //  X#
            //
            Neighbors { 
                north: Some(bt),
                south: None,
                west: None,
                east: Some(br),
                ..
            } if bt == wall && br == wall => TextureAtlasPos::new(rand * 2, 4),

            //  #
            // #X
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: None,
                ..
            } if bt == wall && bb == wall && bl == wall => TextureAtlasPos::new(4, rand),

            //  #
            //  X#
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: None,
                east: Some(br),
                ..
            } if bt == wall && bb == wall && br == wall => TextureAtlasPos::new(0, rand),

            _ => {
                println!("Neighbors = {:#?}", neighbors);
                println!("Wall = {:#?}", wall);
                panic!();
            }
        }
    }
}