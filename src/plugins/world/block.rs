use bevy::prelude::Component;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use rand::{thread_rng, Rng};

use crate::util::{get_tile_start_index, TextureAtlasPos};

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
    pub fn get_sprite_index(neighbors: &Neighbors<BlockType>, block_type: BlockType) -> TextureAtlasPos {
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

        if block_type == BlockType::Grass {
            if let Some(idx) = get_grass_sprite_index_by_dirt_connections(neighbors, rand) {
                index = idx;
            }
        }

        get_tile_start_index(block_type) + index
    }

    fn get_sprite_index_by_dirt_connections(neighbors: &Neighbors<BlockType>, rand: u32) -> Option<TextureAtlasPos> {
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
            => Some(TextureAtlasPos::new(6 + rand, 11)),

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
            => Some(TextureAtlasPos::new(8 + rand, 6)),

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
            => Some(TextureAtlasPos::new(rand, 13)),

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
            => Some(TextureAtlasPos::new(3 + rand, 13)),

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
            => Some(TextureAtlasPos::new(8, 7 + rand)),

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
            => Some(TextureAtlasPos::new(6, 5 + rand)),

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
            => Some(TextureAtlasPos::new(5, 5 + rand)),

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
            => Some(TextureAtlasPos::new(4, 8 + rand)),

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
            => Some(TextureAtlasPos::new(4, 5 + rand)),

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
            => Some(TextureAtlasPos::new(5, 8 + rand)),

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
            => Some(TextureAtlasPos::new(6, 8 + rand)),

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
            => Some(TextureAtlasPos::new(11, 5 + rand)),

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
            => Some(TextureAtlasPos::new(11, 8 + rand)),

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
            => Some(TextureAtlasPos::new(9 + rand, 11)),

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
            => Some(TextureAtlasPos::new(2, 6 + rand * 2)),

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
            => Some(TextureAtlasPos::new(3, 6 + rand * 2)),

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
            => Some( TextureAtlasPos::new(3, 5 + rand * 2)),

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
            => Some(TextureAtlasPos::new(2, 5 + rand * 2)),

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
            => Some(TextureAtlasPos::new(8 + rand, 5)),

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
            => Some(TextureAtlasPos::new(13 + rand, 1)),

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
            => Some(TextureAtlasPos::new(13 + rand, 0)),

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
            => Some(TextureAtlasPos::new(7, 8 + rand)),

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
            => Some(TextureAtlasPos::new(7, 5 + rand)),

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
            => Some(TextureAtlasPos::new(rand, 14)),

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
            => Some(TextureAtlasPos::new(3 + rand, 14)),

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
            => Some(TextureAtlasPos::new(8 + rand, 10)),

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
            => Some(TextureAtlasPos::new(12, 5 + rand)),

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
            => Some(TextureAtlasPos::new(9, 7 + rand)),

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
            => Some(TextureAtlasPos::new(8, 7 + rand)),

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
            => Some(TextureAtlasPos::new(rand, 11)),

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
            => Some(TextureAtlasPos::new(13 + rand, 3)),

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
            => Some(TextureAtlasPos::new(12, 8 + rand)),

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
            => Some(TextureAtlasPos::new(16, 5 + rand * 2)),

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
            => Some(TextureAtlasPos::new(0, 6 + rand * 2)),

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
            => Some(TextureAtlasPos::new(1, 5 + rand * 2)),

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
            => Some(TextureAtlasPos::new(1, 6 + rand * 2)),

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
            => Some(TextureAtlasPos::new(3 + rand, 12)),

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
            => Some(TextureAtlasPos::new(3 + rand, 11)),

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
            => Some(TextureAtlasPos::new(rand, 11)),

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
            => Some(TextureAtlasPos::new(rand, 12)),

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
            => Some(TextureAtlasPos::new(13 + rand, 2)),

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
            => Some(TextureAtlasPos::new(13 + rand, 3)),

            _ => None
        }
    }

    fn get_sprite_index_by_neighbors(neighbors: &Neighbors<BlockType>, rand: u32) -> TextureAtlasPos {
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
            } => TextureAtlasPos::new(1 + rand, 1),
            
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

            // $
            // X
            //
            Neighbors { 
                north: Some(_), 
                south: None, 
                west: None, 
                east: None,
                ..
            } => TextureAtlasPos::new(6 + rand, 3),

            //
            // X
            // $
            Neighbors { 
                north: None, 
                south: Some(_), 
                west: None, 
                east: None,
                ..
            } => TextureAtlasPos::new(6 + rand, 0),

            //
            // $X
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: Some(_), 
                east: None,
                ..
            } => TextureAtlasPos::new(12, rand),

            //
            //  X$
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: None, 
                east: Some(_),
                ..
            } => TextureAtlasPos::new(9, rand),

            //  $
            //  X
            //  $
            Neighbors { 
                north: Some(_), 
                south: Some(_), 
                west: None, 
                east: None,
                ..
            } => TextureAtlasPos::new(5, rand),

            //  $
            // $X$
            //
            Neighbors { 
                north: Some(_), 
                south: None,
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(1 + rand, 2),

            //  
            // $X$
            //  $
            Neighbors { 
                north: None, 
                south: Some(_),
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(1 + rand, 0),

            //  
            // $X$
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(6 + rand, 4),

            //  
            // $X
            //  $
            Neighbors { 
                north: None, 
                south: Some(_),
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(1 + rand * 2, 3),

            //  
            //  X$
            //  $
            Neighbors { 
                north: None, 
                south: Some(_),
                west: None,
                east: Some(_),
                ..
            } => TextureAtlasPos::new(rand * 2, 3),

            //  $
            // $X
            //
            Neighbors { 
                north: Some(_),
                south: None,
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(1 + rand * 2, 4),

            //  $
            //  X$
            //
            Neighbors { 
                north: Some(_),
                south: None,
                west: None,
                east: Some(_),
                ..
            } => TextureAtlasPos::new(rand * 2, 4),

            //  $
            // $X
            //  $
            Neighbors { 
                north: Some(_),
                south: Some(_),
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(4, rand),

            //  $
            //  X$
            //  $
            Neighbors { 
                north: Some(_),
                south: Some(_),
                west: None,
                east: Some(_),
                ..
            } => TextureAtlasPos::new(0, rand),
        }
    }
}

fn get_grass_sprite_index_by_dirt_connections(neighbors: &Neighbors<BlockType>, rand: u32) -> Option<TextureAtlasPos> {
    match neighbors {
        //
        // #X#
        //  $
        Neighbors { 
            east: Some(BlockType::Dirt),
            west: Some(BlockType::Dirt),
            north: None,
            south: Some(_),
            ..
        } => Some(TextureAtlasPos::new(2 + rand, 15)),

        //
        // XX#
        //  $
        Neighbors { 
            east: Some(BlockType::Dirt),
            west: Some(BlockType::Grass),
            north: None,
            south: Some(_),
            ..
        } => Some(TextureAtlasPos::new(3 + rand, 11)),

        //
        // #XX
        //  $
        Neighbors { 
            east: Some(BlockType::Grass),
            west: Some(BlockType::Dirt),
            north: None,
            south: Some(_),
            ..
        } => Some(TextureAtlasPos::new(rand, 11)),
        

        //  $
        // #X#
        //
        Neighbors { 
            east: Some(BlockType::Dirt),
            west: Some(BlockType::Dirt),
            south: None,
            north: Some(_),
            ..
        } => Some(TextureAtlasPos::new(2 + rand, 16)),

        //  $
        // XX#
        //
        Neighbors { 
            east: Some(BlockType::Dirt),
            west: Some(BlockType::Grass),
            south: None,
            north: Some(_),
            ..
        } => Some(TextureAtlasPos::new(3 + rand, 12)),

        //  $
        // #XX
        //
        Neighbors { 
            east: Some(BlockType::Grass),
            west: Some(BlockType::Dirt),
            south: None,
            north: Some(_),
            ..
        } => Some(TextureAtlasPos::new(rand, 12)),
        _ => None
    }
}