use bevy::prelude::Component;

#[derive(Debug, Component, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    Dirt,
    Stone,
    Grass,
}

impl Block {
    pub fn dirt_mergable(&self) -> bool {
        match self {
            Block::Grass => false,
            Block::Dirt => false,
            Block::Stone => true,
        }
    }
}