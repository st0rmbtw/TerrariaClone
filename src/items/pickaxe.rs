use crate::language::LanguageContent;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pickaxe {
    CopperPickaxe
}

impl Pickaxe {
    pub fn name(&self, language_content: LanguageContent) -> String {
        match self {
            Pickaxe::CopperPickaxe => language_content.items.copper_pickaxe,
        }
    }
}