#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axe {
    CopperAxe
}

impl Axe {
    pub const fn power(&self) -> i32 {
        match self {
            Axe::CopperAxe => 35,
        }
    }

    pub const fn cooldown(&self) -> u32 {
        match self {
            Axe::CopperAxe => 21,
        }
    }
}