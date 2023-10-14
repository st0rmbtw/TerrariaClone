use crate::world::wall::WallType;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ItemWall {
    Dirt,
    Stone,
}

impl From<WallType> for ItemWall {
    fn from(wall_type: WallType) -> Self {
        match wall_type {
            WallType::Dirt => Self::Dirt,
            WallType::Stone => Self::Stone,
        }
    }
}