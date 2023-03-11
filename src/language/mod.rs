use std::{io::BufReader, fs::File, error::Error};

use bevy::prelude::Resource;
use serde::Deserialize;

use crate::{items::{Pickaxe, Tool, Item, Axe}, plugins::world::BlockType};

pub enum Language {
    US
}

impl Language {
    pub fn file_name(&self) -> String {
        let name = match self {
            Language::US => "en_US",
        };

        name.to_string() + ".json"
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UI {
    pub items: String,
    pub inventory: String,
    pub settings: String,
    pub single_player: String,
    pub interface: String,
    pub video: String,
    pub tile_grid: String,
    pub cursor: String,
    pub exit: String,
    pub on: String,
    pub off: String,
    pub back: String,
    pub apply: String,
    pub resolution: String,
    pub full_screen: String,
    #[serde(rename = "VSync")]
    pub vsync: String,
    pub full_screen_resolution: String
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Items {
    pub copper_pickaxe: String,
    pub copper_axe: String,
    pub dirt_block: String,
    pub stone_block: String,
    pub dirt_wall: String,
    pub stone_wall: String
}

#[derive(Debug, Deserialize, Clone, Resource)]
#[serde(rename_all = "PascalCase")]
pub struct LanguageContent {
    #[serde(rename = "Title")]
    pub titles: Vec<String>,
    #[serde(rename = "UI")]
    pub ui: UI,
    pub items: Items
}

impl LanguageContent {
    pub fn name(&self, item: Item) -> String {
        match item {
            Item::Tool(tool) => self.tool_name(tool),
            Item::Block(block) => self.block_name(block.block_type),
        }
    }

    fn tool_name(&self, tool: Tool) -> String {
        match tool {
            Tool::Pickaxe(pickaxe) => self.pickaxe_name(pickaxe),
            Tool::Axe(axe) => self.axe_name(axe),
        }
    }

    fn pickaxe_name(&self, pickaxe: Pickaxe) -> String {
        match pickaxe {
            Pickaxe::CopperPickaxe => self.items.copper_pickaxe.clone(),
        }
    }

    fn block_name(&self, block_type: BlockType) -> String {
        match block_type {
            BlockType::Dirt => self.items.dirt_block.clone(),
            BlockType::Stone => self.items.stone_block.clone(),
            _ => panic!("No such item")
        }
    }

    fn axe_name(&self, axe: Axe) -> String {
        match axe {
            Axe::CopperAxe => self.items.copper_axe.clone(),
        }
    }
}

pub fn load_language(language: Language) -> Result<LanguageContent, Box<dyn Error>> {
    let reader = BufReader::new(
        File::open(format!("./assets/languages/{}", language.file_name()))?
    );
    let language_content: LanguageContent = serde_json::from_reader(reader)?;

    Ok(language_content)
}