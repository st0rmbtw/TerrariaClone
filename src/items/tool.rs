use super::{Pickaxe, Axe};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Tool {
    Pickaxe(Pickaxe),
    Axe(Axe)
}

impl Tool {
    pub(crate) const fn power(&self) -> i32 {
        match self {
            Tool::Pickaxe(pickaxe) => pickaxe.power(),
            Tool::Axe(axe) => axe.power(),
        }
    }

    pub(crate) const fn use_cooldown(&self) -> u32 {
        match self {
            Tool::Pickaxe(pickaxe) => pickaxe.use_cooldown(),
            Tool::Axe(axe) => axe.use_cooldown(),
        }
    }

    pub(crate) const fn swing_cooldown(&self) -> u32 {
        match self {
            Tool::Pickaxe(pickaxe) => pickaxe.swing_cooldown(),
            Tool::Axe(axe) => axe.swing_cooldown(),
        }
    }
}