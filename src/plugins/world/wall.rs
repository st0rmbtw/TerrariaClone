use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use rand::{thread_rng, Rng};

use crate::util::get_wall_start_index;

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
    pub fn get_sprite_index(neighbors: Neighbors<Wall>, wall: Wall) -> u32 {
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
            } if bt == wall && bb == wall && bl == wall && br == wall => 13 + 1 + rand,
            
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
            } if b == wall => 13 * 2 + 1 + rand,

            //
            // X
            // #
            Neighbors { 
                north: None, 
                south: Some(b), 
                west: None, 
                east: None,
                ..
            } if b == wall => rand + 6,

            //  #
            //  X
            //  #
            Neighbors { 
                north: Some(bt), 
                south: Some(bb), 
                west: None, 
                east: None,
                ..
            } if bt == wall && bb == wall => rand * 13 + 5,

            //  #
            // #X#
            //
            Neighbors { 
                north: Some(bt), 
                south: None,
                west: Some(bl),
                east: Some(br),
                ..
            } if bt == wall && bl == wall && br == wall => 13 * 2 + 1 + rand,

            //  
            // #X#
            //  #
            Neighbors { 
                north: None, 
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                ..
            } if bb == wall && bl == wall && br == wall => 1 + rand,

            //  
            // #X#
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(bl),
                east: Some(br),
                ..
            } if bl == wall && br == wall => 13 * 4 + 6 + rand,

            //  
            // #X
            //  #
            Neighbors { 
                north: None, 
                south: Some(bb),
                west: Some(bl),
                east: None,
                ..
            } if bb == wall && bl == wall => 13 * 3 + 1 + rand * 2,

            //  
            //  X#
            //  #
            Neighbors { 
                north: None, 
                south: Some(bb),
                west: None,
                east: Some(br),
                ..
            } if bb == wall && br == wall => 13 * 3 + rand * 2,

            //  #
            // #X
            //
            Neighbors { 
                north: Some(bt),
                south: None,
                west: Some(bl),
                east: None,
                ..
            } if bt == wall && bl == wall => 13 * 4 + 1 + rand * 2,

            //  #
            //  X#
            //
            Neighbors { 
                north: Some(bt),
                south: None,
                west: None,
                east: Some(br),
                ..
            } if bt == wall && br == wall => 13 * 4 + rand * 2,

            //  #
            // #X
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: None,
                ..
            } if bt == wall && bb == wall && bl == wall => 13 * rand + 4,

            //  #
            //  X#
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: None,
                east: Some(br),
                ..
            } if bt == wall && bb == wall && br == wall => 13 * rand,

            _ => {
                println!("Neighbors = {:#?}", neighbors);
                println!("Wall = {:#?}", wall);
                panic!();
            }
        }
    }
}