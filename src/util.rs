use bevy::{prelude::{default, Component, Query, Changed, With, Button}, ui::{UiRect, Val, Interaction}, ecs::system::EntityCommands};

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
    fn insert_if<F>(&mut self, component: impl Component, predicate: F) -> &mut EntityCommands<'w, 's, 'a>
    where 
        F: FnOnce() -> bool;
}

impl<'w, 's, 'a> EntityCommandsExtensions<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn insert_if<F>(&mut self, component: impl Component, predicate: F) -> &mut EntityCommands<'w, 's, 'a>
    where 
    F: FnOnce() -> bool 
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
            // This is purely an example—not a good one.
            fn handles(&self) -> Vec<&$field_type> {
                vec![$(&self.$field_name),*]
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

pub fn inside_f(p: (f32, f32), rect: FRect) -> bool {
    p.0 < rect.bottom && p.0 > rect.top && p.1 > rect.left && p.1 < rect.right
}