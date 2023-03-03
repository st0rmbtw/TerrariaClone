use std::ops::Mul;

use bevy::{
    ecs::system::EntityCommands,
    prelude::{Button, Changed, Component, Query, With, Vec2, Camera, GlobalTransform},
    ui::Interaction,
};

use crate::{plugins::world::TILE_SIZE, wall::Wall, items::Block};

pub trait Lerp<T> {
    fn lerp(self, other: T, t: f32) -> T;
}

impl Lerp<f32> for f32 {
    fn lerp(self, other: f32, t: f32) -> f32 {
        self * (1. - t) + other * t
    }
}

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

pub fn map_range(from_range: (usize, usize), to_range: (usize, usize), s: usize) -> usize {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
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

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct FRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct URect {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct IRect {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl URect {
    pub fn to_frect(&self) -> FRect {
        FRect {
            left: self.left as f32,
            right: self.right as f32,
            top: self.top as f32,
            bottom: self.bottom as f32,
        }
    }
}

impl FRect {
    pub fn intersect(&self, rect: FRect) -> bool {
        self.left < rect.right
            && self.right > rect.left
            && self.bottom > rect.top
            && self.top > rect.bottom
    }

    
    pub fn inside(&self, point: (f32, f32)) -> bool {
        point.0 < self.bottom && point.0 > self.top && point.1 > self.left && point.1 < self.right
    }
}

impl Mul<f32> for FRect {
    type Output = FRect;

    fn mul(self, rhs: f32) -> Self::Output {
        FRect {
            left: self.left * rhs,
            right: self.right * rhs,
            top: self.top * rhs,
            bottom: self.bottom * rhs,
        }
    }
}

pub fn get_tile_coords(world_coords: Vec2) -> Vec2 {
    (world_coords / TILE_SIZE).round().abs()
}

pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        return target;
    }
    return current + (target - current).signum() * max_delta;
}

pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if a != b {
        return ((value - a) / (b - a)).clamp(0., 1.)
    } else {
        return 0.;
    }
}


pub fn get_tile_start_index(block: Block) -> u32 {
    match block {
        Block::Dirt => 0,
        Block::Stone => 16 * 15,
        Block::Grass => 16 * 30
    }
}

pub fn get_wall_start_index(wall: Wall) -> u32 {
    match wall {
        Wall::StoneWall => 0,
        Wall::DirtWall => 5 * 13,
    }
}

pub fn screen_to_world(screen_pos: Vec2, window_size: Vec2, camera: &Camera, transform: &GlobalTransform) -> Vec2 {
    // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

    // matrix for undoing the projection and camera transform
    let ndc_to_world = transform.compute_matrix() * camera.projection_matrix().inverse();

    // use it to convert ndc to world-space coordinates
    let world_pos = ndc_to_world.project_point3(ndc.extend(1.0));

    // reduce it to a 2D value
    world_pos.truncate()
}