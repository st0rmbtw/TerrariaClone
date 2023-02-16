use super::Pickaxe;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Pickaxe(Pickaxe)
}