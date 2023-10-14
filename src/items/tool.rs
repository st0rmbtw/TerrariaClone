#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ItemTool {
    Pickaxe(Pickaxe),
    Axe(Axe),
    Hammer(Hammer)
}

impl ItemTool {
    pub(crate) const fn power(&self) -> i32 {
        match self {
            Self::Pickaxe(pickaxe) => pickaxe.power(),
            Self::Axe(axe) => axe.power(),
            Self::Hammer(hammer) => hammer.power()
        }
    }

    pub(crate) const fn use_cooldown(&self) -> u32 {
        match self {
            Self::Pickaxe(pickaxe) => pickaxe.use_cooldown(),
            Self::Axe(axe) => axe.use_cooldown(),
            Self::Hammer(hammer) => hammer.use_cooldown()
        }
    }

    pub(crate) const fn swing_cooldown(&self) -> u32 {
        match self {
            Self::Pickaxe(pickaxe) => pickaxe.swing_cooldown(),
            Self::Axe(axe) => axe.swing_cooldown(),
            Self::Hammer(hammer) => hammer.swing_cooldown()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Axe {
    CopperAxe
}

impl Axe {
    pub(crate) const fn power(&self) -> i32 {
        match self {
            Self::CopperAxe => 35,
        }
    }

    pub(crate) const fn use_cooldown(&self) -> u32 {
        match self {
            Self::CopperAxe => 21,
        }
    }

    pub(crate) const fn swing_cooldown(&self) -> u32 {
        match self {
            Self::CopperAxe => 30,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Pickaxe {
    CopperPickaxe
}

impl Pickaxe {
    pub(crate) const fn power(&self) -> i32 {
        match self {
            Self::CopperPickaxe => 35,
        }
    }

    pub(crate) const fn use_cooldown(&self) -> u32 {
        match self {
            Self::CopperPickaxe => 15,
        }
    }

    pub(crate) const fn swing_cooldown(&self) -> u32 {
        match self {
            Self::CopperPickaxe => 23,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Hammer {
    CopperHammer
}

impl Hammer {
    pub(crate) const fn power(&self) -> i32 {
        match self {
            Self::CopperHammer => 35,
        }
    }

    pub(crate) const fn use_cooldown(&self) -> u32 {
        match self {
            Self::CopperHammer => 23,
        }
    }

    pub(crate) const fn swing_cooldown(&self) -> u32 {
        match self {
            Self::CopperHammer => 33,
        }
    }
}