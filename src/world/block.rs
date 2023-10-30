use std::{ops::Deref, fmt::Debug};

use bevy::prelude::Component;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use rand::{thread_rng, Rng};

use crate::{common::{helpers::get_tile_start_index, TextureAtlasPos}, items::{ItemTool, ItemBlock}};

use super::{tree::{Tree, TreeFrameType}, TerrariaFrame};

pub(crate) type BlockId = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect))]
pub enum BlockType {
    Dirt,
    Stone,
    Grass,
    Wood,
    Tree(Tree)
}

impl BlockType {
    pub const fn id(&self) -> BlockId {
        match self {
            BlockType::Dirt => 0,
            BlockType::Stone => 1,
            BlockType::Grass => 2,
            BlockType::Tree(_) => 5,
            BlockType::Wood => 30,
        }
    }

    pub(crate) const fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Tree(_))
    }

    pub(crate) const fn dirt_mergeable(&self) -> bool {
        match self {
            BlockType::Dirt | BlockType::Grass | BlockType::Tree(_) => false,
            BlockType::Stone => true,
            BlockType::Wood => true,
        }
    }

    pub(crate) const fn check_required_tool(&self, tool: ItemTool) -> bool {
        match self {
            BlockType::Tree(_) => {
                matches!(tool, ItemTool::Axe(_))
            },
            _ => {
                matches!(tool, ItemTool::Pickaxe(_))
            }
        }
    }

    pub(crate) const fn max_hp(&self) -> i32 {
        match self {
            BlockType::Dirt | BlockType::Grass => 50,
            BlockType::Stone | BlockType::Wood => 100,
            BlockType::Tree(_) => 500,
        }
    }

    pub(crate) const fn dusty(&self) -> bool {
        match self {
            BlockType::Dirt => true,
            _ => false
        }
    }

    pub(crate) const fn cracks(&self) -> bool {
        match self {
            BlockType::Tree(tree) => match tree.frame_type {
                TreeFrameType::TrunkPlain => true,
                TreeFrameType::BasePlainA => true,
                TreeFrameType::BasePlainD  => true,
                TreeFrameType::BasePlainAD => true,
                TreeFrameType::TopBare => true,
                TreeFrameType::TopLeaves => true,
                TreeFrameType::TopBareJagged => true,
                _ => false
            },
            _ => true
        }
    }

    pub(crate) const fn color(&self) -> [u8; 3] {
        match self {
            BlockType::Dirt =>  [151, 107, 75],
            BlockType::Stone => [128, 128, 128],
            BlockType::Grass => [28, 216, 94],
            BlockType::Wood => [170, 120, 84],
            BlockType::Tree(_) => [151, 107, 75],
        }
    }
}

impl From<ItemBlock> for BlockType {
    fn from(item: ItemBlock) -> Self {
        match item {
            ItemBlock::Dirt => BlockType::Dirt,
            ItemBlock::Stone => BlockType::Stone,
            ItemBlock::Wood => BlockType::Wood
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct Block {
    pub(crate) block_type: BlockType,
    pub(crate) hp: i32,
    pub(crate) variant: u32,
    pub(crate) cracks_index: Option<u32>
}

impl From<BlockType> for Block {
    fn from(block_type: BlockType) -> Self {
        let mut rng = thread_rng();
        Block::new(block_type, rng.gen_range(0..3))
    }
}

impl Deref for Block {
    type Target = BlockType;

    fn deref(&self) -> &Self::Target {
        &self.block_type
    }
}

impl Block {
    #[inline(always)]
    pub(crate) fn new(block_type: BlockType, variant: u32) -> Block {
        debug_assert!((0..3).contains(&variant));

        Self {
            block_type,
            variant,
            hp: block_type.max_hp(),
            cracks_index: None
        }
    }

    pub(crate) const fn frame(&self) -> Option<TerrariaFrame> {
        match self.block_type {
            BlockType::Tree(tree) => Some(tree.terraria_frame(self.variant)),
            _ => None
        }
    }
}

impl Block {
    pub(crate) fn get_sprite_index(neighbors: &Neighbors<BlockType>, block: &Block) -> u32 {
        /*
         * "$" - Any block
         * "#" - Dirt
         * "X" - This block
        */

        if let BlockType::Tree(tree) = block.block_type {
            return get_tree_sprite_index(neighbors, tree, block.variant).to_2d_index_from_block_type(block.block_type);
        }

        let mut index = get_sprite_index_by_neighbors(neighbors, block.variant);

        if block.dirt_mergeable() {
            if let Some(idx) = get_sprite_index_by_dirt_connections(neighbors, block.variant) {
                index = idx;
            }
        }

        if block.block_type == BlockType::Grass {
            if let Some(idx) = get_grass_sprite_index_by_dirt_connections(neighbors, block.variant) {
                index = idx;
            }
        }

        (get_tile_start_index(block.block_type) + index).to_block_index()
    }
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
        } => Some(TextureAtlasPos::new(6 + variant, 11)),

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
        } => Some(TextureAtlasPos::new(variant, 13)),

        //
        // X#
        //
        Neighbors { 
            north: None, 
            south: None, 
            west: None, 
            east: Some(BlockType::Dirt),
            .. 
        } => Some(TextureAtlasPos::new(3 + variant, 13)),

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
        } => Some(TextureAtlasPos::new(6, 5 + variant)),

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
        } => Some(TextureAtlasPos::new(6, 8 + variant)),

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
        } => Some(TextureAtlasPos::new(9 + variant, 11)),

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
        => Some(TextureAtlasPos::new(0, 5 + variant * 2)),

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

fn get_grass_sprite_index_by_dirt_connections(neighbors: &Neighbors<BlockType>, variant: u32) -> Option<TextureAtlasPos> {
    match *neighbors {
        Neighbors {
            north: None,
            south: Some(BlockType::Grass),
            east: None,
            west: Some(BlockType::Grass),
            south_west: None,
            ..
        } => Some(TextureAtlasPos::new(8 + variant, 15)),

        Neighbors {
            north: Some(BlockType::Grass),
            south: Some(BlockType::Dirt),
            east: Some(BlockType::Grass),
            west: None,
            ..
        } => Some(TextureAtlasPos::new(4, 5 + variant)),

        Neighbors {
            north: Some(BlockType::Grass),
            south: None,
            east: Some(BlockType::Grass),
            west: None,
            ..
        } => Some(TextureAtlasPos::new(5 + variant, 16)),

        Neighbors {
            north: None,
            south: Some(BlockType::Grass),
            east: Some(BlockType::Grass),
            west: None,
            south_east: None,
            ..
        } => Some(TextureAtlasPos::new(5 + variant, 15)),

        Neighbors {
            north: Some(BlockType::Grass),
            south: Some(BlockType::Dirt),
            east: Some(BlockType::Grass),
            west: Some(BlockType::Dirt),

            north_east: None,
            ..
        } => Some(TextureAtlasPos::new(2, 6 + variant * 2)),

        Neighbors {
            north: Some(BlockType::Grass),
            south: Some(_),
            west: Some(BlockType::Grass),
            north_west: None,
            ..
        } => Some(TextureAtlasPos::new(3, 6 + variant * 2)),

        Neighbors {
            north: Some(bt),
            south: Some(BlockType::Grass),
            east: Some(BlockType::Grass),
            west: Some(BlockType::Dirt),
            south_east: None,
            ..
        } if bt != BlockType::Grass => Some(TextureAtlasPos::new(2, 6 + variant * 2 - 1)),

        Neighbors {
            north: Some(_),
            south: Some(BlockType::Grass),
            west: Some(BlockType::Grass),
            south_west: None,
            ..
        } => Some(TextureAtlasPos::new(3, 6 + variant * 2 - 1)),

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

        //  X
        // #XX
        //  X
        Neighbors { 
            east: Some(BlockType::Grass),
            west: Some(BlockType::Dirt),
            north: Some(BlockType::Grass),
            south: Some(BlockType::Grass),
            ..
        } => Some(TextureAtlasPos::new(11, variant)),
        

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

fn get_tree_sprite_index(neighbors: &Neighbors<BlockType>, tree: Tree, variant: u32) -> TextureAtlasPos {
    match (tree.frame_type, neighbors) {
        (TreeFrameType::TrunkPlain, Neighbors {
            north: None,
            ..
        }) => TreeFrameType::TopBare.texture_atlas_pos(tree.tree_type, variant),

        (TreeFrameType::BasePlainA | TreeFrameType::BasePlainD | TreeFrameType::BasePlainAD, Neighbors {
            west: None,
            east: Some(BlockType::Tree(_)),
            ..
        }) => TreeFrameType::BasePlainD.texture_atlas_pos(tree.tree_type, variant),

        (TreeFrameType::BasePlainA | TreeFrameType::BasePlainD | TreeFrameType::BasePlainAD, Neighbors {
            west: Some(BlockType::Tree(_)),
            east: None,
            ..
        }) => TreeFrameType::BasePlainA.texture_atlas_pos(tree.tree_type, variant),

        (TreeFrameType::BasePlainA | TreeFrameType::BasePlainD | TreeFrameType::BasePlainAD, Neighbors {
            west: None,
            east: None,
            ..
        }) => TreeFrameType::TrunkPlain.texture_atlas_pos(tree.tree_type, variant),

        _ => tree.frame_type.texture_atlas_pos(tree.tree_type, variant)
    }
}
