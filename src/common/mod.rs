use std::ops::Add;

use crate::world::block::BlockType;

pub(crate) mod conditions;
pub(crate) mod extensions;
pub(crate) mod helpers;
pub(crate) mod lens;
pub(crate) mod math;
pub(crate) mod rect;
pub(crate) mod state;
pub(crate) mod systems;
pub(crate) mod components;


pub(crate) trait BoolValue {
    fn value(&self) -> bool;
}

pub(crate) trait Toggle {
    fn toggle(&mut self);
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextureAtlasPos {
    pub(crate) x: u32,
    pub(crate) y: u32
}

impl TextureAtlasPos {
    pub(crate) const ZERO: TextureAtlasPos = TextureAtlasPos::new(0, 0);

    pub(crate) const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub(crate) const fn to_2d_index(self, width: u32) -> u32 {
        (self.y * width) + self.x
    }

    #[inline(always)]
    pub(crate) const fn to_block_index(self) -> u32 {
        self.to_2d_index(16)
    }
    
    #[inline(always)]
    pub(crate) const fn to_wall_index(self) -> u32 {
        self.to_2d_index(13)
    }
    
    pub(crate) const fn to_2d_index_from_block_type(self, block_type: BlockType) -> u32 {
        match block_type {
            BlockType::Tree(tree) => self.to_2d_index(tree.frame_type.texture_width()),
            _ => self.to_block_index()
        }
    }
}

impl Add<TextureAtlasPos> for TextureAtlasPos {
    type Output = TextureAtlasPos;

    fn add(self, rhs: TextureAtlasPos) -> Self::Output {
        TextureAtlasPos::new(self.x + rhs.x, self.y + rhs.y)
    }
}