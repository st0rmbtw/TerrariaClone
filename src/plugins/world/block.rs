use bevy::prelude::Component;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use rand::{thread_rng, Rng};

use crate::util::get_tile_start_index;

use super::{generator::BlockId, tree::Tree, Frame};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Dirt,
    Stone,
    Grass,
    Tree(Tree)
}

impl BlockType {
    pub const fn id(&self) -> BlockId {
        match self {
            BlockType::Dirt => 0,
            BlockType::Stone => 1,
            BlockType::Grass => 2,
            BlockType::Tree(_) => 5
        }
    }

    pub const fn frame(&self) -> Option<Frame> {
        match self {
            BlockType::Tree(tree) => Some(tree.frame),
            _ => None
        }
    }
    
    pub const fn max_health(&self) -> i32 {
        match self {
            BlockType::Dirt | BlockType::Grass => 50,
            BlockType::Stone => 100,
            BlockType::Tree(_) => 500,
        }
    }

    pub const fn dirt_mergable(&self) -> bool {
        match self {
            BlockType::Dirt | BlockType::Grass | BlockType::Tree(_) => false,
            BlockType::Stone => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct Block {
    pub block_type: BlockType,
    pub hp: i32
}

impl Block {
    #[inline(always)]
    pub const fn id(&self) -> BlockId {
        self.block_type.id()
    }

    #[inline(always)]
    pub const fn frame(&self) -> Option<Frame> {
        self.block_type.frame()
    }

    #[inline(always)]
    pub const fn max_health(&self) -> i32 {
        self.block_type.max_health()
    }
    
    #[inline(always)]
    pub const fn dirt_mergable(&self) -> bool {
        self.block_type.dirt_mergable()
    }
}

macro_rules! block {
    ($block_name: ident) => {
        pub const $block_name: Block = Block {
            block_type: BlockType::$block_name,
            hp: BlockType::$block_name.max_health()
        };
    };
}

#[allow(non_upper_case_globals)]
impl Block {
    block!(Dirt);
    block!(Grass);
    block!(Stone);
}

impl Block {
    pub fn get_sprite_index(neighbors: &Neighbors<BlockType>, block_type: BlockType) -> u32 {
        /*
         * "$" - Any block
         * "#" - Dirt
         * "X" - This block
        */

        let rand: u32 = thread_rng().gen_range(0..3);

        let mut index = Self::get_sprite_index_by_neighbors(neighbors, rand);

        if block_type.dirt_mergable() {
            if let Some(idx) = Self::get_sprite_index_by_dirt_connections(neighbors, rand) {
                index = idx;
            }
        }

        get_tile_start_index(block_type) + index
    }

    fn get_sprite_index_by_dirt_connections(neighbors: &Neighbors<BlockType>, rand: u32) -> Option<u32> {
        match neighbors {
            //  #
            // #X#
            //  #
            Neighbors { 
                north: Some(BlockType::Dirt | BlockType::Grass), 
                south: Some(BlockType::Dirt), 
                west: Some(BlockType::Dirt), 
                east: Some(BlockType::Dirt),
                .. 
            }
            => Some(16 * 11 + 6 + rand),

            //  #
            // $X$
            //  $
            Neighbors {
                north: Some(BlockType::Dirt | BlockType::Grass), 
                south: Some(bb), 
                west: Some(bl), 
                east: Some(br),
                ..
            } if *bb != BlockType::Dirt && *bl != BlockType::Dirt && *br != BlockType::Dirt
            => Some(16 * 6 + 8 + rand),

            //
            // #X
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: Some(BlockType::Dirt), 
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
                east: Some(BlockType::Dirt),
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
                east: Some(BlockType::Dirt),
                .. 
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bl != BlockType::Dirt && *bb != BlockType::Dirt
            => Some((7 + rand) * 16 + 8),

            //
            // X
            // #
            Neighbors { 
                north: None, 
                south: Some(BlockType::Dirt), 
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
                south: Some(BlockType::Dirt),
                west: Some(bl),
                east: None,
                .. 
            } if *bl != BlockType::Dirt && *bt != BlockType::Dirt
            => Some((5 + rand) * 16 + 5),

            // #
            // X$
            // $
            Neighbors { 
                north: Some(BlockType::Dirt | BlockType::Grass),
                south: Some(bb),
                west: None,
                east: Some(br),
                ..
            } if *br != BlockType::Dirt && *bb != BlockType::Dirt
            => Some((8 + rand) * 16 + 4),

            // $
            // X$
            // #
            Neighbors { 
                north: Some(bt),
                south: Some(BlockType::Dirt),
                west: None,
                east: Some(br),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *br != BlockType::Dirt
            => Some((5 + rand) * 16 + 4),

            //  #
            // $X
            //  $
            Neighbors { 
                north: Some(BlockType::Dirt | BlockType::Grass),
                south: Some(bb),
                west: Some(bl),
                east: None,
                ..
            } if *bb != BlockType::Dirt && *bl != BlockType::Dirt
            => Some((8 + rand) * 16 + 5),

            //  #
            //  X
            //
            Neighbors {
                north: Some(BlockType::Dirt | BlockType::Grass),
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
                north: Some(BlockType::Dirt | BlockType::Grass), 
                south: Some(bb),
                west: Some(BlockType::Dirt),
                east: Some(BlockType::Dirt),
                ..
            } if *bb != BlockType::Dirt
            => Some((5 + rand) * 16 + 11),

            //  $
            // #X#
            //  #
            Neighbors { 
                north: Some(bt), 
                south: Some(BlockType::Dirt),
                west: Some(BlockType::Dirt),
                east: Some(BlockType::Dirt),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass)
            => Some((8 + rand) * 16 + 11),

            // 
            // #X#
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(BlockType::Dirt),
                east: Some(BlockType::Dirt),
                ..
            }
            => Some(11 * 16 + 9 + rand),

            //  $
            // #X$
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(BlockType::Dirt),
                west: Some(BlockType::Dirt),
                east: Some(br),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *br != BlockType::Dirt
            => Some((6 + rand * 2) * 16 + 2),

            //  $
            // $X# 
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(BlockType::Dirt),
                west: Some(bl),
                east: Some(BlockType::Dirt),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bl != BlockType::Dirt
            => Some((6 + rand * 2) * 16 + 3),

            //  #
            // $X#
            //  $
            Neighbors { 
                north: Some(BlockType::Dirt | BlockType::Grass),
                south: Some(bb),
                west: Some(bl),
                east: Some(BlockType::Dirt),
                ..
            } if *bb != BlockType::Dirt && *bl != BlockType::Dirt
            => Some((5 + rand * 2) * 16 + 3),

            //  #
            // #X$
            //  $
            Neighbors { 
                north: Some(BlockType::Dirt | BlockType::Grass),
                south: Some(bb),
                west: Some(BlockType::Dirt),
                east: Some(br),
                ..
            } if *bb != BlockType::Dirt && *br != BlockType::Dirt
            => Some((5 + rand * 2) * 16 + 2),

            //  $
            // $X$
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(BlockType::Dirt),
                west: Some(bl),
                east: Some(br),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bl != BlockType::Dirt && *br != BlockType::Dirt
            => Some(5 * 16 + 8 + rand),

            //  #
            // $X$
            //
            Neighbors { 
                north: Some(BlockType::Dirt | BlockType::Grass),
                south: None,
                west: Some(bl),
                east: Some(br),
                ..
            } if *bl != BlockType::Dirt && *br != BlockType::Dirt
            => Some(16 + 13 + rand),

            //  
            // $X$
            //  #
            Neighbors { 
                north: None,
                south: Some(BlockType::Dirt),
                west: Some(bl),
                east: Some(br),
                ..
            } if *bl != BlockType::Dirt && *br != BlockType::Dirt
            => Some(13 + rand),

            //  #
            //  X
            //  $
            Neighbors { 
                north: Some(BlockType::Dirt | BlockType::Grass),
                south: Some(bb),
                west: None,
                east: None,
                ..
            } if *bb != BlockType::Dirt
            => Some((8 + rand) * 16 + 7),

            //  $
            //  X
            //  #
            Neighbors { 
                north: Some(bt),
                south: Some(BlockType::Dirt),
                west: None,
                east: None,
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass)
            => Some((5 + rand) * 16 + 7),

            // 
            // #X$
            // 
            Neighbors { 
                north: None,
                south: None,
                west: Some(BlockType::Dirt),
                east: Some(br),
                ..
            } if *br != BlockType::Dirt
            => Some(14 * 16 + rand),

            // 
            // $X#
            // 
            Neighbors { 
                north: None,
                south: None,
                west: Some(bl),
                east: Some(BlockType::Dirt),
                ..
            } if *bl != BlockType::Dirt
            => Some(14 * 16 + 3 + rand),

            //  #
            // $X$
            //  #
            Neighbors { 
                north: Some(BlockType::Dirt | BlockType::Grass),
                south: Some(BlockType::Dirt),
                west: Some(bl),
                east: Some(br),
                ..
            } if *bl != BlockType::Dirt && *br != BlockType::Dirt
            => Some(10 * 16 + 8 + rand),

            //  #
            // #X$
            //  #
            Neighbors { 
                north: Some(BlockType::Dirt | BlockType::Grass),
                south: Some(BlockType::Dirt),
                west: Some(BlockType::Dirt),
                east: Some(br),
                ..
            } if *br != BlockType::Dirt
            => Some((5 + rand) * 16 + 12),

            //  $
            // #X$
            //  $
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(BlockType::Dirt),
                east: Some(br),
                ..
            } if *bt != BlockType::Dirt && *bb != BlockType::Dirt && *br != BlockType::Dirt
            => Some((7 + rand) * 16 + 9),

            //  $
            // $X#
            //  $
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(BlockType::Dirt),
                ..
            } if *bt != BlockType::Dirt && *bb != BlockType::Dirt && *bl != BlockType::Dirt
            => Some((7 + rand) * 16 + 8),

            //  
            // #X$
            //  $
            Neighbors { 
                north: None,
                south: Some(bb),
                west: Some(BlockType::Dirt),
                east: Some(br),
                ..
            } if *bb != BlockType::Dirt && *br != BlockType::Dirt
            => Some(11 * 16 + rand),

            //  $
            // #X
            //  $
            Neighbors { 
                north: Some(bt),
                south: Some(bb),
                west: Some(BlockType::Dirt),
                east: None,
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bb != BlockType::Dirt
            => Some(3 * 16 + 13 + rand),

            //  #
            // $X#
            //  #
            Neighbors { 
                north: Some(BlockType::Dirt),
                south: Some(BlockType::Dirt),
                west: Some(bl),
                east: Some(BlockType::Dirt),
                ..
            } if *bl != BlockType::Dirt
            => Some((8 + rand) * 16 + 12),

            //  $
            // $X$
            //  $#
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                south_east: Some(BlockType::Dirt),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bb != BlockType::Dirt && *bl != BlockType::Dirt && *br != BlockType::Dirt
            => Some((5 + rand * 2) * 16),

            //  $#
            // $X$
            //  $
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                north_east: Some(BlockType::Dirt),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bb != BlockType::Dirt && *bl != BlockType::Dirt && *br != BlockType::Dirt
            => Some((6 + rand * 2) * 16),

            //  $
            // $X$
            // #$
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                south_west: Some(BlockType::Dirt),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bb != BlockType::Dirt && *bl != BlockType::Dirt && *br != BlockType::Dirt
            => Some((5 + rand * 2) * 16 + 1),

            // #$
            // $X$
            //  $
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(bl),
                east: Some(br),
                north_west: Some(BlockType::Dirt),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bb != BlockType::Dirt && *bl != BlockType::Dirt && *br != BlockType::Dirt
            => Some((6 + rand * 2) * 16 + 1),

            //  $
            // $X#
            //  
            Neighbors {
                north: Some(bt),
                south: None,
                west: Some(bl),
                east: Some(BlockType::Dirt),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bl != BlockType::Dirt
            => Some(12 * 16 + 3 + rand),

            //  
            // $X#
            //  $
            Neighbors {
                north: None,
                south: Some(bb),
                west: Some(bl),
                east: Some(BlockType::Dirt),
                ..
            } if *bb != BlockType::Dirt && *bl != BlockType::Dirt
            => Some(11 * 16 + 3 + rand),

            //  
            // #X$
            //  $
            Neighbors {
                north: None,
                south: Some(bb),
                west: Some(BlockType::Dirt),
                east: Some(br),
                ..
            } if *bb != BlockType::Dirt && *br != BlockType::Dirt
            => Some(11 * 16 + rand),

            //  $
            // #X$
            //  
            Neighbors {
                north: Some(bt),
                south: None,
                west: Some(BlockType::Dirt),
                east: Some(br),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *br != BlockType::Dirt
            => Some(12 * 16 + rand),

            //  $
            //  X#
            //  $
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: None,
                east: Some(BlockType::Dirt),
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bb != BlockType::Dirt
            => Some(2 * 16 + 13 + rand),

            //  $
            //  X#
            //  $
            Neighbors {
                north: Some(bt),
                south: Some(bb),
                west: Some(BlockType::Dirt),
                east: None,
                ..
            } if (*bt != BlockType::Dirt && *bt != BlockType::Grass) && *bb != BlockType::Dirt
            => Some(3 * 16 + 13 + rand),

            _ => None
        }
    }

    fn get_sprite_index_by_neighbors(neighbors: &Neighbors<BlockType>, rand: u32) -> u32 {
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
