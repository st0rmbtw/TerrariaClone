use std::ops::Add;

use bevy::{
    ecs::system::EntityCommands,
    prelude::{Button, Changed, Component, Query, With, Vec2},
    ui::Interaction,
};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::plugins::world::{TILE_SIZE, Wall, BlockType};

pub trait EntityCommandsExtensions<'w, 's, 'a> {
    fn insert_if(
        &mut self,
        component: impl Component,
        insert: bool,
    ) -> &mut EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> EntityCommandsExtensions<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn insert_if(
        &mut self,
        component: impl Component,
        insert: bool,
    ) -> &mut EntityCommands<'w, 's, 'a> {
        if insert {
            self.insert(component);
        }

        self
    }
}

macro_rules! handles{
    (
     $field_type:ty,
     // meta data about struct
     $(#[$meta:meta])*
     $vis:vis struct $struct_name:ident {
        $(
        // meta data about field
        $(#[$field_meta:meta])*
        $field_vis:vis $field_name:ident : $field_t:ty
        ),*$(,)+
    }
    ) => {
        $(#[$meta])*
        pub struct $struct_name {
            $(
            $(#[$field_meta])*
            pub $field_name : $field_type,
            )*
        }

        impl $struct_name {
            pub fn handles(&self) -> Vec<$field_type> {
                vec![$(self.$field_name.clone()),*]
            }
        }
    }
}

pub(crate) use handles;

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

pub fn on_btn_clicked<B: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<B>)>,
) -> bool {
    for interaction in query.iter() {
        if *interaction == Interaction::Clicked {
            return true;
        }
    }

    false
}

pub fn map_range_usize(from_range: (usize, usize), to_range: (usize, usize), s: usize) -> usize {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

pub fn map_range_f32(in_min: f32, in_max: f32, out_min: f32, out_max: f32, value: f32) -> f32 {
    out_min + (((value - in_min) / (in_max - in_min)) * (out_max - out_min))
}

pub fn get_tile_coords(world_coords: Vec2) -> Vec2 {
    (world_coords / TILE_SIZE).round().abs()
}

pub fn tile_to_world_coords(tile_pos: TilePos) -> Vec2 {
    Vec2::new(tile_pos.x as f32 * TILE_SIZE, -(tile_pos.y as f32) * TILE_SIZE)
}

pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        return target;
    }
    current + (target - current).signum() * max_delta
}

pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if a != b {
        ((value - a) / (b - a)).clamp(0., 1.)
    } else {
        0.
    }
}


pub fn get_tile_start_index(block: BlockType) -> TextureAtlasPos {
    match block {
        BlockType::Dirt => TextureAtlasPos::ZERO,
        BlockType::Stone => TextureAtlasPos::new(0, 15),
        BlockType::Grass => TextureAtlasPos::new(0, 30),
        BlockType::Tree(_) => panic!(),
    }
}

pub fn get_wall_start_index(wall: Wall) -> TextureAtlasPos {
    match wall {
        Wall::Stone => TextureAtlasPos::ZERO,
        Wall::Dirt => TextureAtlasPos::new(0, 5),
    }
}