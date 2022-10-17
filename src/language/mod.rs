use std::{io::BufReader, fs::File, error::Error};

use serde::Deserialize;

use crate::{items::Pickaxe, block::Block};

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
    pub exit: String
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Items {
    pub copper_pickaxe: String,
    pub dirt_block: String,
    pub stone_block: String,
    pub dirt_wall: String,
    pub stone_wall: String
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LanguageContent {
    #[serde(rename = "Title")]
    pub titles: Vec<String>,
    #[serde(rename = "UI")]
    pub ui: UI,
    pub items: Items
}

impl LanguageContent {
    pub fn pickaxe_name(&self, pickaxe: Pickaxe) -> String {
        match pickaxe {
            Pickaxe::CopperPickaxe => self.items.copper_pickaxe.clone(),
        }
    }

    pub fn block_name(&self, block: Block) -> String {
        match block {
            Block::Dirt => self.items.dirt_block.clone(),
            Block::Stone => self.items.stone_block.clone(),
            _ => panic!("No such item")
        }
    }
}

pub fn load_language(language: Language) -> Result<LanguageContent, Box<dyn Error>> {
    let reader = BufReader::new(File::open(format!("./assets/languages/{}", language.file_name()))?);
    let language_content: LanguageContent = serde_json::from_reader(reader)?;

    Ok(language_content)
}