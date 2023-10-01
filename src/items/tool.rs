use super::{Pickaxe, Axe};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ItemTool {
    Pickaxe(Pickaxe),
    Axe(Axe)
}

impl ItemTool {
    pub(crate) const fn power(&self) -> i32 {
        match self {
            ItemTool::Pickaxe(pickaxe) => pickaxe.power(),
            ItemTool::Axe(axe) => axe.power(),
        }
    }

    pub(crate) const fn use_cooldown(&self) -> u32 {
        match self {
            ItemTool::Pickaxe(pickaxe) => pickaxe.use_cooldown(),
            ItemTool::Axe(axe) => axe.use_cooldown(),
        }
    }

    pub(crate) const fn swing_cooldown(&self) -> u32 {
        match self {
            ItemTool::Pickaxe(pickaxe) => pickaxe.swing_cooldown(),
            ItemTool::Axe(axe) => axe.swing_cooldown(),
        }
    }
}