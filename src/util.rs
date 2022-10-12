use std::ops::Mul;

use bevy::{
    ecs::system::EntityCommands,
    prelude::{default, Button, Changed, Component, Query, With, Vec2, Quat},
    ui::{Interaction, UiRect, Val},
};

pub trait Lerp<T> {
    fn lerp(self, other: T, t: f32) -> T;
}

impl Lerp<f32> for f32 {
    fn lerp(self, other: f32, t: f32) -> f32 {
        self * (1. - t) + other * t
    }
}

pub trait RectExtensions {
    fn horizontal(value: f32) -> Self;
    fn vertical(value: f32) -> Self;
    fn top(value: f32) -> Self;
}

impl RectExtensions for UiRect<Val> {
    fn horizontal(value: f32) -> Self {
        Self {
            left: Val::Px(value),
            right: Val::Px(value),
            ..default()
        }
    }

    fn vertical(value: f32) -> Self {
        Self {
            top: Val::Px(value),
            bottom: Val::Px(value),
            ..default()
        }
    }

    fn top(value: f32) -> Self {
        Self {
            top: Val::Px(value),
            ..default()
        }
    }
}

pub trait EntityCommandsExtensions<'w, 's, 'a> {
    fn insert_if<F>(
        &mut self,
        component: impl Component,
        predicate: F,
    ) -> &mut EntityCommands<'w, 's, 'a>
    where
        F: FnOnce() -> bool;
}

impl<'w, 's, 'a> EntityCommandsExtensions<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn insert_if<F>(
        &mut self,
        component: impl Component,
        predicate: F,
    ) -> &mut EntityCommands<'w, 's, 'a>
    where
        F: FnOnce() -> bool,
    {
        if predicate() {
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

use crate::{plugins::{world::TILE_SIZE, player::FaceDirection}, block::Block, wall::Wall};

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
    pub left: usize,
    pub right: usize,
    pub top: usize,
    pub bottom: usize,
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

pub fn inside_f(p: (f32, f32), rect: FRect) -> bool {
    p.0 < rect.bottom && p.0 > rect.top && p.1 > rect.left && p.1 < rect.right
}

pub fn get_tile_coords(world_coords: Vec2) -> Vec2 {
    (world_coords / TILE_SIZE).round()
}

pub fn get_rotation_by_direction(direction: FaceDirection) -> Quat {
    let start_rotation = match direction {
        FaceDirection::LEFT => -0.5,
        FaceDirection::RIGHT => 2.,
    };

    Quat::from_rotation_z(start_rotation)
}

pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        return target;
    }
    return current + (target - current).signum() * max_delta;
}

pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if a != b {
        return clamp01((value - a) / (b - a));
    } else {
        return 0.0;
    }
}

fn clamp01(value: f32) -> f32 {
    if value < 0. {
        return 0.;
    } else if value > 1. {
        return 1.;
    } else {
        return value;
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