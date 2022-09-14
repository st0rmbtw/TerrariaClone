use bevy::prelude::Component;

#[derive(Debug, Component, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    Dirt,
    Stone,
    Grass,
}

impl Block {
    pub fn name(&self) -> &str {
        match self {
            Block::Dirt => "Dirt Block",
            Block::Stone => "Stone Block",
            Block::Grass => "Grass Block",
        }
    }
}