use std::{fs::OpenOptions, io::{BufReader, Write}};

use bevy::{prelude::{Plugin, App, OnUpdate, IntoSystemConfigs, IntoSystemConfig}, text::Text};
use serde::{Deserialize, Serialize};

use crate::{state::GameState, animation::{component_animator_system, AnimationSystemSet}};

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
    pub cursor_color: CursorColor
}


impl Default for Settings {
    fn default() -> Self {
        Self { 
            full_screen: false,
            show_tile_grid: false,
            vsync: true,
            resolution: Resolution::default(),
            cursor_color: CursorColor::default()
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

        app.add_systems(
            (
                update,
                set_btn_visibility
            )
            .chain()
            .in_set(OnUpdate(GameState::InGame))
        );

        app.add_system(
            component_animator_system::<Text>
                .in_set(OnUpdate(GameState::InGame))
                .in_set(AnimationSystemSet::AnimationUpdate)
        );

        app.insert_resource(FullScreen(settings.full_screen));
        app.insert_resource(ShowTileGrid(settings.show_tile_grid));
        app.insert_resource(VSync(settings.vsync));
        app.insert_resource(settings.cursor_color);
        app.insert_resource(settings.resolution);
    }
}