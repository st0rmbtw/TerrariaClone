use bevy::{prelude::{default, Component}, ui::{UiRect, Val}, ecs::system::EntityCommands};

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