#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Pickaxe {
    CopperPickaxe
}

impl Pickaxe {
    pub(crate) const fn power(&self) -> i32 {
        match self {
            Pickaxe::CopperPickaxe => 35,
        }
    }

    pub(crate) const fn use_cooldown(&self) -> u32 {
        match self {
            Pickaxe::CopperPickaxe => 15,
        }
    }

    pub(crate) const fn swing_cooldown(&self) -> u32 {
        match self {
            Pickaxe::CopperPickaxe => 23,
        }
    }
}