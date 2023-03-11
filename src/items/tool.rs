use super::{Pickaxe, Axe};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Pickaxe(Pickaxe),
    Axe(Axe)
}

impl Tool {
    pub const fn power(&self) -> i32 {
        match self {
            Tool::Pickaxe(pickaxe) => pickaxe.power(),
            Tool::Axe(axe) => axe.power(),
        }
    }

    pub const fn cooldown(&self) -> u32 {
        match self {
            Tool::Pickaxe(pickaxe) => pickaxe.cooldown(),
            Tool::Axe(axe) => axe.cooldown(),
        }
    }
}