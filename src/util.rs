use bevy::{reflect::Reflect, prelude::{default, Component, Vec2}, ui::UiRect, ecs::system::EntityCommands};

pub trait Lerp<T> {
    fn lerp(self, other: T, t: f32) -> T;
}


impl Lerp<f32> for f32 {
    fn lerp(self, other: f32, t: f32) -> f32 {
        self * (1. - t) + other * t
    }
}


pub trait RectExtensions<T: Reflect + PartialEq> {
    fn horizontal(value: T) -> Self;
    fn vertical(value: T) -> Self;
    fn top(value: T) -> Self;
}

impl<T: Reflect + PartialEq + Default + Clone> RectExtensions<T> for UiRect<T> {
    fn horizontal(value: T) -> Self {
        Self {
            left: value.clone(),
            right: value,
            ..default()
        }
    }

    fn vertical(value: T) -> Self {
        Self {
            top: value.clone(),
            bottom: value,
            ..default()
        }
    }

    fn top(value: T) -> Self {
        Self {
            top: value,
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
    where F: FnOnce() -> bool {
        if predicate() {
            self.insert(component);
        }

        self
    }
}

pub trait VectorBetween {
    fn is_between(&self, p1: Vec2, p2: Vec2) -> bool;
}

impl VectorBetween for Vec2 {
    fn is_between(&self, p1: Vec2, p2: Vec2) -> bool {
        (p1.y * self.x - p1.x * self.y) * (p1.y * p2.x - p1.x * p2.y) < 0.
    }
}