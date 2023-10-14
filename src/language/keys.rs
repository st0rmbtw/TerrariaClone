use crate::items::{Item, ItemTool, ItemSeed, Axe, Pickaxe, ItemBlock, Hammer, ItemWall};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LanguageStringKey {
    UI(UIStringKey),
    Items(ItemStringKey)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemStringKey {
    CopperPickaxe,
    CopperAxe,
    CopperHammer,
    DirtBlock,
    StoneBlock,
    DirtWall,
    StoneWall,
    GrassSeeds,
    Wood
}

impl ItemStringKey {
    pub(crate) const fn get_by_item(item: Item) -> Self {
        match item {
            Item::Tool(tool) => match tool {
                ItemTool::Pickaxe(pickaxe) => match pickaxe {
                    Pickaxe::CopperPickaxe => ItemStringKey::CopperPickaxe,
                },
                ItemTool::Axe(axe) => match axe {
                    Axe::CopperAxe => ItemStringKey::CopperAxe,
                },
                ItemTool::Hammer(hammer) => match hammer {
                    Hammer::CopperHammer => ItemStringKey::CopperHammer,
                }
            },
            Item::Block(block) => match block {
                ItemBlock::Dirt => ItemStringKey::DirtBlock,
                ItemBlock::Stone => ItemStringKey::StoneBlock,
                ItemBlock::Wood => ItemStringKey::Wood,
            },
            Item::Wall(wall) => match wall {
                ItemWall::Dirt => ItemStringKey::DirtWall,
                ItemWall::Stone => ItemStringKey::StoneWall,
            }
            Item::Seed(seed) => match seed {
                ItemSeed::Grass => ItemStringKey::GrassSeeds,
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