#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pickaxe {
    CopperPickaxe
}

impl Pickaxe {
    pub fn power(&self) -> i32 {
        match self {
            Pickaxe::CopperPickaxe => 35,
        }
    }

    pub fn cooldown(&self) -> u32 {
        match self {
            Pickaxe::CopperPickaxe => 15,
        }
    }
}