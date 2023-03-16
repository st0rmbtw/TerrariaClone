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
            resolution: Resolution::new(1920., 1080.),
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

pub const RESOLUTIONS: [Resolution; 16] = [
    Resolution::new(800., 600.),
    Resolution::new(1024., 768.),
    Resolution::new(1152., 864.),
    Resolution::new(1176., 664.),
    Resolution::new(1280., 720.),
    Resolution::new(1280., 768.),
    Resolution::new(1280., 800.),
    Resolution::new(1280., 960.),
    Resolution::new(1280., 1024.),
    Resolution::new(1360., 768.),
    Resolution::new(1366., 768.),
    Resolution::new(1440., 900.),
    Resolution::new(1600., 900.),
    Resolution::new(1600., 1024.),
    Resolution::new(1680., 1050.),
    Resolution::new(1920., 1080.),
];