use crate::{items::{Item, Tool, Seed, Axe, Pickaxe}, world::block::BlockType};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LanguageStringKey {
    UI(UIStringKey),
    Items(ItemStringKey)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemStringKey {
    CopperPickaxe,
    CopperAxe,
    DirtBlock,
    StoneBlock,
    DirtWall,
    StoneWall,
    GrassSeeds
}

impl ItemStringKey {
    pub(crate) fn get_by_item(item: &Item) -> Self {
        match item {
            Item::Tool(tool) => match tool {
                Tool::Pickaxe(pickaxe) =>  match pickaxe {
                    Pickaxe::CopperPickaxe => ItemStringKey::CopperPickaxe,
                },
                Tool::Axe(axe) => match axe {
                    Axe::CopperAxe => ItemStringKey::CopperAxe,
                },
            },
            Item::Block(block) => match block {
                BlockType::Dirt => ItemStringKey::DirtBlock,
                BlockType::Stone => ItemStringKey::StoneBlock,
                BlockType::Grass => ItemStringKey::GrassSeeds,
                BlockType::Tree(_) => unreachable!(),
            },
            Item::Seed(seed) => match seed {
                Seed::Grass => ItemStringKey::GrassSeeds,
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum UIStringKey {
    Items,
    Inventory,
    General,
    Settings,
    SettingsMenu,
    SinglePlayer,
    Interface,
    Video,
    Volume,
    TileGrid,
    Cursor,
    Exit,
    On,
    Off,
    Back,
    Apply,
    Resolution,
    Fullscreen,
    Music,
    Sound,
    Language,
    Vsync,
    FullscreenResolution,
    LightSmoothness,
    CloseMenu,
    SaveAndExit,
    Zoom,
    Classic,
    Medium,
    High,
    Ultra,
}

impl From<UIStringKey> for LanguageStringKey {
    fn from(key: UIStringKey) -> Self {
        LanguageStringKey::UI(key)
    }
}

impl From<ItemStringKey> for LanguageStringKey {
    fn from(key: ItemStringKey) -> Self {
        LanguageStringKey::Items(key)
    }
}