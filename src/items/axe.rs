#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Axe {
    CopperAxe
}

impl Axe {
    pub(crate) const fn power(&self) -> i32 {
        match self {
            Axe::CopperAxe => 35,
        }
    }

    pub(crate) const fn use_cooldown(&self) -> u32 {
        match self {
            Axe::CopperAxe => 21,
        }
    }

    pub(crate) const fn swing_cooldown(&self) -> u32 {
        match self {
            Axe::CopperAxe => 30,
        }
    }
}