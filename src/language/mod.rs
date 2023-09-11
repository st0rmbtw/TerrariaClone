use std::{io::BufReader, fs::File, error::Error};

use bevy::prelude::Resource;
use serde::Deserialize;

use crate::{items::{Pickaxe, Tool, Item, Axe, Seed}, world::block::BlockType};

pub(crate) enum Language {
    English
}

impl Language {
    pub(crate) fn file_name(&self) -> String {
        let mut file_name = String::with_capacity(5 + 5);

        let language = match self {
            Language::English => "en_US",
        };

        file_name.push_str(language);
        file_name.push_str(".json");

        file_name
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct UI {
    pub(crate) items: String,
    pub(crate) inventory: String,
    pub(crate) general: String,
    pub(crate) settings: String,
    pub(crate) settings_menu: String,
    pub(crate) single_player: String,
    pub(crate) interface: String,
    pub(crate) video: String,
    pub(crate) volume: String,
    pub(crate) tile_grid: String,
    pub(crate) cursor: String,
    pub(crate) exit: String,
    pub(crate) on: String,
    pub(crate) off: String,
    pub(crate) back: String,
    pub(crate) apply: String,
    pub(crate) resolution: String,
    pub(crate) full_screen: String,
    pub(crate) music: String,
    pub(crate) sound: String,
    pub(crate) language: String,
    #[serde(rename = "VSync")]
    pub(crate) vsync: String,
    pub(crate) full_screen_resolution: String,
    pub(crate) close_menu: String,
    pub(crate) save_and_exit: String,
    pub(crate) zoom: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Items {
    pub(crate) copper_pickaxe: String,
    pub(crate) copper_axe: String,
    pub(crate) dirt_block: String,
    pub(crate) stone_block: String,
    pub(crate) dirt_wall: String,
    pub(crate) stone_wall: String,
    pub(crate) grass_seed: String
}

#[derive(Deserialize, Resource)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct LanguageContent {
    #[serde(rename = "Title")]
    pub(crate) titles: Vec<String>,
    #[serde(rename = "UI")]
    pub(crate) ui: UI,
    pub(crate) items: Items
}

impl LanguageContent {
    pub(crate) fn item_name(&self, item: Item) -> &String {
        match item {
            Item::Tool(tool) => self.tool_name(tool),
            Item::Block(block) => self.block_name(block),
            Item::Seed(seed) => self.seed_name(seed)
        }
    }

    fn tool_name(&self, tool: Tool) -> &String {
        match tool {
            Tool::Pickaxe(pickaxe) => self.pickaxe_name(pickaxe),
            Tool::Axe(axe) => self.axe_name(axe),
        }
    }

    fn pickaxe_name(&self, pickaxe: Pickaxe) -> &String {
        match pickaxe {
            Pickaxe::CopperPickaxe => &self.items.copper_pickaxe,
        }
    }

    fn block_name(&self, block_type: BlockType) -> &String {
        match block_type {
            BlockType::Dirt => &self.items.dirt_block,
            BlockType::Stone => &self.items.stone_block,
            _ => panic!("No such item")
        }
    }

    fn seed_name(&self, seed: Seed) -> &String {
        match seed {
            Seed::Grass => &self.items.grass_seed,
        }
    }

    fn axe_name(&self, axe: Axe) -> &String {
        match axe {
            Axe::CopperAxe => &self.items.copper_axe,
        }
    }
}

pub(crate) fn load_language(language: Language) -> Result<LanguageContent, Box<dyn Error>> {
    let reader = BufReader::new(
        File::open(format!("./assets/languages/{}", language.file_name()))?
    );
    let language_content: LanguageContent = serde_json::from_reader(reader)?;

    Ok(language_content)
}