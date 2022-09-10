use bevy::prelude::Component;

pub struct BlockData {
    name: &'static str,
}

#[derive(Debug, Component, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    Dirt,
    Stone,
    Grass,
}