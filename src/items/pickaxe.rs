#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pickaxe {
    CopperPickaxe
}

impl Pickaxe {
    pub fn name(&self) -> &str {
        match self {
            Pickaxe::CopperPickaxe => "Copper Pickaxe",
        }
    }
}