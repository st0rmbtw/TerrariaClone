#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pickaxe {
    CopperPickaxe
}

impl Pickaxe {
    pub const fn power(&self) -> i32 {
        match self {
            Pickaxe::CopperPickaxe => 35,
        }
    }

    pub const fn cooldown(&self) -> u32 {
        match self {
            Pickaxe::CopperPickaxe => 15,
        }
    }
}