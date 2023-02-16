use std::{fs::OpenOptions, io::{BufReader, Write}};

use bevy::{prelude::{Plugin, App, Color}, text::Text};
use iyes_loopless::prelude::{ConditionSet, IntoConditionalSystem};
use serde::{Deserialize, Serialize};

use crate::{state::GameState, animation::{component_animator_system, AnimationSystem}};

mod components;
mod systems;
mod resources;

pub use components::*;
pub use systems::*;
pub use resources::*;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
    pub full_screen: bool,
    pub show_tile_grid: bool,
    #[serde(rename = "VSync")]
    pub vsync: bool,
    pub resolution: Resolution,
    pub cursor_color: Color
}


impl Default for Settings {
    fn default() -> Self {
        Self { 
            full_screen: false,
            show_tile_grid: false,
            vsync: true,
            resolution: RESOLUTIONS[16],
            cursor_color: Color::PINK
        }
    }
}

fn load_settings() -> Settings {
    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(false)
        .open("terra_settings.json")
        .unwrap();

    let reader = BufReader::new(file);

    serde_json::from_reader(reader).unwrap_or_default()
}

pub fn save_settings(settings: Settings) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("terra_settings.json")
        .unwrap();

    file.write_all(
        serde_json::to_string(&settings).unwrap().as_bytes()
    ).unwrap();
}

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = load_settings();

        app
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(update)
                    .with_system(set_btn_visibility)
                    .into(),
            )
            .add_system(
                component_animator_system::<Text>
                    .run_in_state(GameState::InGame)
                    .label(AnimationSystem::AnimationUpdate)
            )

            .insert_resource(FullScreen(settings.full_screen))
            .insert_resource(ShowTileGrid(settings.show_tile_grid))
            .insert_resource(VSync(settings.vsync))
            .insert_resource(CursorColor(settings.cursor_color))
            .insert_resource(settings.resolution);
    }
}

lazy_static! {
    pub static ref RESOLUTIONS: Vec<Resolution> = vec![
        Resolution {
            width: 800.,
            height: 600.,
            index: 0
        },
        Resolution {
            width: 1024.,
            height: 768.,
            index: 1
        },
        Resolution {
            width: 1152.,
            height: 864.,
            index: 2
        },
        Resolution {
            width: 1176.,
            height: 664.,
            index: 3
        },
        Resolution {
            width: 1280.,
            height: 720.,
            index: 4
        },
        Resolution {
            width: 1280.,
            height: 768.,
            index: 5
        },
        Resolution {
            width: 1280.,
            height: 800.,
            index: 6
        },
        Resolution {
            width: 1280.,
            height: 960.,
            index: 7
        },
        Resolution {
            width: 1280.,
            height: 1024.,
            index: 8
        },
        Resolution {
            width: 1360.,
            height: 768.,
            index: 9
        },
        Resolution {
            width: 1366.,
            height: 768.,
            index: 10
        },
        Resolution {
            width: 1440.,
            height: 900.,
            index: 11
        },
        Resolution {
            width: 1600.,
            height: 900.,
            index: 12
        },
        Resolution {
            width: 1600.,
            height: 1024.,
            index: 13
        },
        Resolution {
            width: 1680.,
            height: 1050.,
            index: 14
        },
        Resolution {
            width: 1680.,
            height: 1050.,
            index: 15
        },
        Resolution {
            width: 1920.,
            height: 1080.,
            index: 16
        }
    ];
}