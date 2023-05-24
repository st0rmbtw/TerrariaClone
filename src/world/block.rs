use bevy::prelude::Component;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use rand::{thread_rng, Rng};

use crate::{common::{helpers::get_tile_start_index, TextureAtlasPos}, items::Tool};

use super::{tree::{Tree, TreeFrameType}, TerrariaFrame};

pub(crate) type BlockId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect, bevy::reflect::FromReflect))]
pub(crate) enum BlockType {
    Dirt,
    Stone,
    Grass,
    Tree(Tree)
}

impl BlockType {
    pub(crate) const fn id(&self) -> BlockId {
        match self {
            BlockType::Dirt => 0,
            BlockType::Stone => 1,
            BlockType::Grass => 2,
            BlockType::Tree(_) => 5
        }
    }

    pub(crate) const fn frame(&self) -> Option<TerrariaFrame> {
        match self {
            BlockType::Tree(tree) => Some(tree.terraria_frame()),
            _ => None
        }
    }

    pub(crate) const fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Tree(_))
    }

    pub(crate) const fn dirt_mergable(&self) -> bool {
        match self {
            BlockType::Dirt | BlockType::Grass | BlockType::Tree(_) => false,
            BlockType::Stone => true,
        }
    }

    pub(crate) const fn check_required_tool(&self, tool: Tool) -> bool {
        match self {
            BlockType::Tree(_) => {
                matches!(tool, Tool::Axe(_))
            },
            _ => {
                matches!(tool, Tool::Pickaxe(_))
            }
        }
    }

    pub(crate) const fn max_health(&self) -> i32 {
        match self {
            BlockType::Dirt | BlockType::Grass => 50,
            BlockType::Stone => 100,
            BlockType::Tree(_) => 500,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(crate) struct Block {
    pub(crate) block_type: BlockType,
    pub(crate) hp: i32
}

impl From<BlockType> for Block {
    fn from(block_type: BlockType) -> Self {
        Block { block_type, hp: block_type.max_health() }
    }
}

impl Block {
    #[inline(always)]
    pub(crate) const fn id(&self) -> BlockId {
        self.block_type.id()
    }

    #[inline(always)]
    pub(crate) const fn frame(&self) -> Option<TerrariaFrame> {
        self.block_type.frame()
    }

    #[inline(always)]
    pub(crate) const fn max_health(&self) -> i32 {
        self.block_type.max_health()
    }
    
    #[inline(always)]
    pub(crate) const fn dirt_mergable(&self) -> bool {
        self.block_type.dirt_mergable()
    }

    #[inline(always)]
    pub(crate) const fn is_solid(&self) -> bool {
        self.block_type.is_solid()
    }

    #[inline(always)]
    pub(crate) const fn check_required_tool(&self, tool: Tool) -> bool {
        self.block_type.check_required_tool(tool)
    }
}

macro_rules! block {
    ($block_name: ident) => {
        pub(crate) const $block_name: Block = Block {
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
    pub(crate) fn get_sprite_index(neighbors: &Neighbors<BlockType>, block_type: BlockType) -> u32 {
        /*
         * "$" - Any block
         * "#" - Dirt
         * "X" - This block
        */

        if let BlockType::Tree(tree) = block_type {
            return get_tree_sprite_index(neighbors, tree).to_2d_index_from_block_type(block_type);
        }

        let variant: u32 = thread_rng().gen_range(0..3);

        let mut index = Self::get_sprite_index_by_neighbors(neighbors, variant);

        if block_type.dirt_mergable() {
            if let Some(idx) = Self::get_sprite_index_by_dirt_connections(neighbors, variant) {
                index = idx;
            }
        }

        if block_type == BlockType::Grass {
            if let Some(idx) = get_grass_sprite_index_by_dirt_connections(neighbors, variant) {
                index = idx;
            }
        }

        (get_tile_start_index(block_type) + index).to_block_index()
    }

    fn get_sprite_index_by_dirt_connections(neighbors: &Neighbors<BlockType>, variant: u32) -> Option<TextureAtlasPos> {
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
            => Some(TextureAtlasPos::new(6 + variant, 11)),

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
            => Some(TextureAtlasPos::new(8 + variant, 6)),

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
            => Some(TextureAtlasPos::new(variant, 13)),

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
            => Some(TextureAtlasPos::new(3 + variant, 13)),

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
            => Some(TextureAtlasPos::new(8, 7 + variant)),

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
            => Some(TextureAtlasPos::new(6, 5 + variant)),

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
            => Some(TextureAtlasPos::new(5, 5 + variant)),

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
            => Some(TextureAtlasPos::new(4, 8 + variant)),

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
            => Some(TextureAtlasPos::new(4, 5 + variant)),

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
            => Some(TextureAtlasPos::new(5, 8 + variant)),

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
            => Some(TextureAtlasPos::new(6, 8 + variant)),

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
            => Some(TextureAtlasPos::new(11, 5 + variant)),

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
            => Some(TextureAtlasPos::new(11, 8 + variant)),

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
            => Some(TextureAtlasPos::new(9 + variant, 11)),

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
            => Some(TextureAtlasPos::new(2, 6 + variant * 2)),

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
            => Some(TextureAtlasPos::new(3, 6 + variant * 2)),

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
            => Some( TextureAtlasPos::new(3, 5 + variant * 2)),

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
            => Some(TextureAtlasPos::new(2, 5 + variant * 2)),

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
            => Some(TextureAtlasPos::new(8 + variant, 5)),

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
            => Some(TextureAtlasPos::new(13 + variant, 1)),

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
            => Some(TextureAtlasPos::new(13 + variant, 0)),

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
            => Some(TextureAtlasPos::new(7, 8 + variant)),

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
            => Some(TextureAtlasPos::new(7, 5 + variant)),

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
            => Some(TextureAtlasPos::new(variant, 14)),

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
            => Some(TextureAtlasPos::new(3 + variant, 14)),

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
            => Some(TextureAtlasPos::new(8 + variant, 10)),

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
            => Some(TextureAtlasPos::new(12, 5 + variant)),

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
            => Some(TextureAtlasPos::new(9, 7 + variant)),

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
            => Some(TextureAtlasPos::new(8, 7 + variant)),

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
            => Some(TextureAtlasPos::new(variant, 11)),

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
            => Some(TextureAtlasPos::new(13 + variant, 3)),

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
            => Some(TextureAtlasPos::new(12, 8 + variant)),

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
            => Some(TextureAtlasPos::new(16, 5 + variant * 2)),

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
            => Some(TextureAtlasPos::new(0, 6 + variant * 2)),

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
            => Some(TextureAtlasPos::new(1, 5 + variant * 2)),

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
            => Some(TextureAtlasPos::new(1, 6 + variant * 2)),

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
            => Some(TextureAtlasPos::new(3 + variant, 12)),

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
            => Some(TextureAtlasPos::new(3 + variant, 11)),

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
            => Some(TextureAtlasPos::new(variant, 11)),

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
            => Some(TextureAtlasPos::new(variant, 12)),

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
            => Some(TextureAtlasPos::new(13 + variant, 2)),

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
            => Some(TextureAtlasPos::new(13 + variant, 3)),

            _ => None
        }
    }

    fn get_sprite_index_by_neighbors(neighbors: &Neighbors<BlockType>, variant: u32) -> TextureAtlasPos {
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

            // $
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
            // $
            Neighbors { 
                north: None, 
                south: Some(_), 
                west: None, 
                east: None,
                ..
            } => TextureAtlasPos::new(6 + variant, 0),

            //
            // $X
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: Some(_), 
                east: None,
                ..
            } => TextureAtlasPos::new(12, variant),

            //
            //  X$
            //
            Neighbors { 
                north: None, 
                south: None, 
                west: None, 
                east: Some(_),
                ..
            } => TextureAtlasPos::new(9, variant),

            //  $
            //  X
            //  $
            Neighbors { 
                north: Some(_), 
                south: Some(_), 
                west: None, 
                east: None,
                ..
            } => TextureAtlasPos::new(5, variant),

            //  $
            // $X$
            //
            Neighbors { 
                north: Some(_), 
                south: None,
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(1 + variant, 2),

            //  
            // $X$
            //  $
            Neighbors { 
                north: None, 
                south: Some(_),
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(1 + variant, 0),

            //  
            // $X$
            //
            Neighbors { 
                north: None, 
                south: None,
                west: Some(_),
                east: Some(_),
                ..
            } => TextureAtlasPos::new(6 + variant, 4),

            //  
            // $X
            //  $
            Neighbors { 
                north: None, 
                south: Some(_),
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(1 + variant * 2, 3),

            //  
            //  X$
            //  $
            Neighbors { 
                north: None, 
                south: Some(_),
                west: None,
                east: Some(_),
                ..
            } => TextureAtlasPos::new(variant * 2, 3),

            //  $
            // $X
            //
            Neighbors { 
                north: Some(_),
                south: None,
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(1 + variant * 2, 4),

            //  $
            //  X$
            //
            Neighbors { 
                north: Some(_),
                south: None,
                west: None,
                east: Some(_),
                ..
            } => TextureAtlasPos::new(variant * 2, 4),

            //  $
            // $X
            //  $
            Neighbors { 
                north: Some(_),
                south: Some(_),
                west: Some(_),
                east: None,
                ..
            } => TextureAtlasPos::new(4, variant),

            //  $
            //  X$
            //  $
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

fn get_grass_sprite_index_by_dirt_connections(neighbors: &Neighbors<BlockType>, variant: u32) -> Option<TextureAtlasPos> {
    match neighbors {
        Neighbors {
            north: Some(BlockType::Grass),
            south: Some(_),
            east: Some(BlockType::Grass),
            north_east: None,
            ..
        } => {
            Some(TextureAtlasPos::new(2, 6 + variant * 2))
        },

        Neighbors {
            north: Some(BlockType::Grass),
            south: Some(_),
            west: Some(BlockType::Grass),
            north_west: None,
            ..
        } => {
            Some(TextureAtlasPos::new(3, 6 + variant * 2))
        },

        Neighbors {
            north: Some(_),
            south: Some(BlockType::Grass),
            east: Some(BlockType::Grass),
            south_east: None,
            ..
        } => {
            Some(TextureAtlasPos::new(2, 6 + variant * 2 - 1))
        },

        Neighbors {
            north: Some(_),
            south: Some(BlockType::Grass),
            west: Some(BlockType::Grass),
            south_west: None,
            ..
        } => {
            Some(TextureAtlasPos::new(3, 6 + variant * 2 - 1))
        },

        //
        // #X#
        //  $
        Neighbors { 
            east: Some(BlockType::Dirt),
            west: Some(BlockType::Dirt),
            north: None,
            south: Some(_),
            ..
        } => Some(TextureAtlasPos::new(2 + variant, 15)),

        //
        // XX#
        //  $
        Neighbors { 
            east: Some(BlockType::Dirt),
            west: Some(BlockType::Grass),
            north: None,
            south: Some(_),
            ..
        } => Some(TextureAtlasPos::new(3 + variant, 11)),

        //
        // #XX
        //  $
        Neighbors { 
            east: Some(BlockType::Grass),
            west: Some(BlockType::Dirt),
            north: None,
            south: Some(_),
            ..
        } => Some(TextureAtlasPos::new(variant, 11)),
        

        //  $
        // #X#
        //
        Neighbors { 
            east: Some(BlockType::Dirt),
            west: Some(BlockType::Dirt),
            south: None,
            north: Some(_),
            ..
        } => Some(TextureAtlasPos::new(2 + variant, 16)),

        //  $
        // XX#
        //
        Neighbors { 
            east: Some(BlockType::Dirt),
            west: Some(BlockType::Grass),
            south: None,
            north: Some(_),
            ..
        } => Some(TextureAtlasPos::new(3 + variant, 12)),

        //  $
        // #XX
        //
        Neighbors { 
            east: Some(BlockType::Grass),
            west: Some(BlockType::Dirt),
            south: None,
            north: Some(_),
            ..
        } => Some(TextureAtlasPos::new(variant, 12)),
        _ => None
    }
}

fn get_tree_sprite_index(neighbors: &Neighbors<BlockType>, tree: Tree) -> TextureAtlasPos {
    match (tree.frame_type, neighbors) {
        (TreeFrameType::TrunkPlain, Neighbors {
            north: None,
            ..
        }) => TreeFrameType::TopBare.texture_atlas_pos(tree.tree_type, tree.variant),
        _ => tree.frame_type.texture_atlas_pos(tree.tree_type, tree.variant)
    }
}