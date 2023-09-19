pub(crate) mod keys;
pub(crate) mod plugin;

use std::{io::BufReader, fs::File, error::Error, sync::Arc};

use bevy::prelude::{Resource, Component};
use dyn_fmt::AsStrFormatExt;
use serde::Deserialize;

use self::keys::{LanguageStringKey, UIStringKey, ItemStringKey};

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
struct UI {
    items: String,
    inventory: String,
    general: String,
    settings: String,
    settings_menu: String,
    single_player: String,
    interface: String,
    video: String,
    volume: String,
    tile_grid: String,
    cursor: String,
    exit: String,
    on: String,
    off: String,
    back: String,
    apply: String,
    resolution: String,
    #[serde(rename = "FullScreen")]
    fullscreen: String,
    music: String,
    sound: String,
    language: String,
    #[serde(rename = "VSync")]
    vsync: String,
    #[serde(rename = "FullScreenResolution")]
    fullscreen_resolution: String,
    light_smoothness: String,
    close_menu: String,
    save_and_exit: String,
    zoom: String,
    classic: String,
    medium: String,
    high: String,
    ultra: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Items {
    copper_pickaxe: String,
    copper_axe: String,
    dirt_block: String,
    stone_block: String,
    dirt_wall: String,
    stone_wall: String,
    grass_seed: String
}

#[derive(Deserialize, Resource)]
#[serde(rename_all = "PascalCase")]
pub(super) struct LanguageContent {
    #[serde(rename = "Title")]
    pub(super) titles: Vec<String>,
    #[serde(rename = "UI")]
    ui: UI,
    items: Items
}

impl LanguageContent {
    pub(super) fn get_by_key(&self, key: LanguageStringKey) -> &str {
        match key {
            LanguageStringKey::UI(ui_key) => match ui_key {
                keys::UIStringKey::Items => &self.ui.items,
                keys::UIStringKey::Inventory => &self.ui.inventory,
                keys::UIStringKey::General => &self.ui.general,
                keys::UIStringKey::Settings => &self.ui.settings,
                keys::UIStringKey::SettingsMenu => &self.ui.settings_menu,
                keys::UIStringKey::SinglePlayer => &self.ui.single_player,
                keys::UIStringKey::Interface => &self.ui.interface,
                keys::UIStringKey::Video => &self.ui.video,
                keys::UIStringKey::Volume => &self.ui.volume,
                keys::UIStringKey::TileGrid => &self.ui.tile_grid,
                keys::UIStringKey::Cursor => &self.ui.cursor,
                keys::UIStringKey::Exit => &self.ui.exit,
                keys::UIStringKey::On => &self.ui.on,
                keys::UIStringKey::Off => &self.ui.off,
                keys::UIStringKey::Back => &self.ui.back,
                keys::UIStringKey::Apply => &self.ui.apply,
                keys::UIStringKey::Resolution => &self.ui.resolution,
                keys::UIStringKey::Fullscreen => &self.ui.fullscreen,
                keys::UIStringKey::Music => &self.ui.music,
                keys::UIStringKey::Sound => &self.ui.sound,
                keys::UIStringKey::Language => &self.ui.language,
                keys::UIStringKey::Vsync => &self.ui.vsync,
                keys::UIStringKey::FullscreenResolution => &self.ui.fullscreen_resolution,
                keys::UIStringKey::LightSmoothness => &self.ui.light_smoothness,
                keys::UIStringKey::CloseMenu => &self.ui.close_menu,
                keys::UIStringKey::SaveAndExit => &self.ui.save_and_exit,
                keys::UIStringKey::Zoom => &self.ui.zoom,
                keys::UIStringKey::Classic => &self.ui.classic,
                keys::UIStringKey::Medium => &self.ui.medium,
                keys::UIStringKey::High => &self.ui.high,
                keys::UIStringKey::Ultra => &self.ui.ultra,
            },
            LanguageStringKey::Items(item_key) => match item_key {
                keys::ItemStringKey::CopperPickaxe => &self.items.copper_pickaxe,
                keys::ItemStringKey::CopperAxe => &self.items.copper_axe,
                keys::ItemStringKey::DirtBlock => &self.items.dirt_block,
                keys::ItemStringKey::StoneBlock => &self.items.stone_block,
                keys::ItemStringKey::DirtWall => &self.items.dirt_wall,
                keys::ItemStringKey::StoneWall => &self.items.stone_wall,
                keys::ItemStringKey::GrassSeeds => &self.items.grass_seed,
            },
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

pub(crate) trait Localize: Sync + Send {
    fn localize(&self, language_content: &LanguageContent) -> Box<str>;
}

impl<T: Into<LanguageStringKey> + Send + Sync + Clone> Localize for T {
    fn localize(&self, language_content: &LanguageContent) -> Box<str> {
        Box::from(language_content.get_by_key(self.clone().into()))
    }
}

impl Localize for &str {
    fn localize(&self, _: &LanguageContent) -> Box<str> {
        Box::from(*self)
    }
}

impl Localize for f32 {
    fn localize(&self, _: &LanguageContent) -> Box<str> {
        Box::from(self.to_string())
    }
}

impl Localize for u16 {
    fn localize(&self, _: &LanguageContent) -> Box<str> {
        Box::from(self.to_string())
    }
}

#[derive(Component)]
pub(crate) struct LocalizedText {
    pub(crate) key: LanguageStringKey,
    pub(crate) format: Option<String>,
    pub(crate) args: Option<Arc<[Box<dyn Localize>]>>
}

impl LocalizedText {
    pub(crate) fn new(key: impl Into<LanguageStringKey>, format: impl Into<String>, args: Arc<[Box<dyn Localize>]>) -> Self {
        Self { key: key.into(), format: Some(format.into()), args: Some(args) }
    }

    pub(super) fn format(&self, language_content: &LanguageContent) -> String {
        let key_str = Box::from(language_content.get_by_key(self.key));

        match (&self.format, &self.args) {
            (Some(format), Some(args)) => {
                let localized_args = args
                    .iter()
                    .map(|key| key.localize(language_content));

                let mut args = Vec::with_capacity(localized_args.len() + 1);
                args.push(key_str);
                args.extend(localized_args);

                format.format(&args)
            },
            _ => key_str.to_string()
        }
    }
}

impl From<LanguageStringKey> for LocalizedText {
    fn from(key: LanguageStringKey) -> Self {
        Self { key, format: None, args: None }       
    }
}

impl From<UIStringKey> for LocalizedText {
    fn from(key: UIStringKey) -> Self {
        Self { key: key.into(), format: None, args: None }       
    }
}

impl From<ItemStringKey> for LocalizedText {
    fn from(key: ItemStringKey) -> Self {
        Self { key: key.into(), format: None, args: None }       
    }
}

macro_rules! args {
    [$($i:expr),+] => {
        std::sync::Arc::new([$(std::boxed::Box::new($i)),+])
    };
}

pub(crate) use args;