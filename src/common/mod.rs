use std::ops::Add;

use bevy::prelude::Component;

pub mod conditions;
pub mod extensions;
pub mod helpers;
pub mod lens;
pub mod math;
pub mod rect;
pub mod state;

#[derive(Debug, Clone, Copy)]
pub struct TextureAtlasPos {
    pub x: u32,
    pub y: u32
}

impl TextureAtlasPos {
    pub const ZERO: TextureAtlasPos = TextureAtlasPos::new(0, 0);

    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn to_block_index(self) -> u32 {
        self.to_2d_index(16)
    }
    
    #[inline(always)]
    pub const fn to_wall_index(self) -> u32 {
        self.to_2d_index(13)
    }

    #[inline(always)]
    pub const fn to_tree_index(self) -> u32 {
        self.to_2d_index(64)
    }

    #[inline(always)]
    pub const fn to_2d_index(self, width: u32) -> u32 {
        (self.y * width) + self.x
    }
}

impl Add<TextureAtlasPos> for TextureAtlasPos {
    type Output = TextureAtlasPos;

    fn add(self, rhs: TextureAtlasPos) -> Self::Output {
        TextureAtlasPos::new(self.x + rhs.x, self.y + rhs.y)
    }
}