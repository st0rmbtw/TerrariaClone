use std::{fs::OpenOptions, io::{BufReader, BufWriter}, error::Error};

use bevy::{prelude::{Plugin, App, IntoSystemConfigs, in_state, Update}, text::Text};
use serde::{Deserialize, Serialize};

use crate::{common::{state::GameState, systems::{animate_button_scale, play_sound_on_hover, set_visibility}}, animation::{component_animator_system, AnimationSystemSet}};

mod components;
mod systems;
mod resources;

use components::*;
pub(crate) use systems::*;
pub(crate) use resources::*;

use super::ui::ExtraUiVisibility;

const SETTINGS_FILENAME: &str = "settings.json";

pub(super) const RESOLUTIONS: [Resolution; 16] = [
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Settings {
    pub(crate) full_screen: bool,
    pub(crate) show_tile_grid: bool,
    #[serde(rename = "VSync")]
    pub(crate) vsync: bool,
    pub(crate) resolution: Resolution,
    pub(crate) cursor_color: CursorColor,
    pub(crate) sound_volume: f32,
    pub(crate) music_volume: f32
}


impl Default for Settings {
    fn default() -> Self {
        Self { 
            full_screen: true,
            show_tile_grid: false,
            vsync: true,
            resolution: Resolution::new(1920., 1080.),
            cursor_color: CursorColor::default(),
            sound_volume: 100.,
            music_volume: 100.
        }
    }
}

pub(crate) struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = load_settings().unwrap_or_default();

        app.insert_resource(FullScreen(settings.full_screen));
        app.insert_resource(ShowTileGrid(settings.show_tile_grid));
        app.insert_resource(VSync(settings.vsync));
        app.insert_resource(MusicVolume::from_slider_value(settings.music_volume));
        app.insert_resource(SoundVolume::from_slider_value(settings.sound_volume));
        app.insert_resource(settings.cursor_color);
        app.insert_resource(settings.resolution);

        app.add_systems(
            Update,
            (
                animate_button_scale::<SettingsButton>,
                play_sound_on_hover::<SettingsButton>,
                set_visibility::<SettingsButtonContainer, ExtraUiVisibility>,
                component_animator_system::<Text>.in_set(AnimationSystemSet::AnimationUpdate)
            )
            .run_if(in_state(GameState::InGame))
        );
    }
}

fn load_settings() -> Result<Settings, Box<dyn Error>> {
    let file = OpenOptions::new()
        .read(true)
        .open(SETTINGS_FILENAME)?;

    let reader = BufReader::new(file);

    let settings: Settings = serde_json::from_reader(reader)?;

    Ok(settings)
}

pub(super) fn save_settings(settings: Settings) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(SETTINGS_FILENAME)
        .unwrap();

    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &settings).unwrap();
}