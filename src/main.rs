use std::error::Error;

use bevy::{
    asset::AssetServerSettings,
    prelude::*,
    render::texture::ImageSettings,
    window::{PresentMode, WindowMode},
};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_hanabi::HanabiPlugin;
use game::{
    animation::TweeningPlugin,
    parallax::ParallaxPlugin,
    state::GameState, 
    plugins::{
        assets::AssetsPlugin, cursor::CursorPlugin, camera::CameraPlugin, background::BackgroundPlugin, 
        ui::PlayerUiPlugin, settings::SettingsPlugin, menu::MenuPlugin, world::WorldPlugin, 
        inventory::PlayerInventoryPlugin, fps::FpsPlugin
    }, 
    language::{load_language, Language},
};
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::seq::SliceRandom;

fn main() -> Result<(), Box<dyn Error>> {
    let language_content = load_language(Language::US)?;

    let title = language_content.titles.choose(&mut rand::thread_rng()).unwrap();

    let mut app = App::new();

    app
        .insert_resource(language_content.clone())
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: title.to_owned(),
            present_mode: PresentMode::Fifo,
            cursor_visible: false,
            position: WindowPosition::Centered(MonitorSelection::Current),
            mode: WindowMode::Windowed,
            ..default()
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::rgb(
            110. / 255.,
            151. / 255.,
            244. / 255.,
        )))
        .add_loopless_state(GameState::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(CursorPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(ParallaxPlugin { initial_speed: 0.2 })
        .add_plugin(HanabiPlugin)
        .add_plugin(BackgroundPlugin)
        .add_plugin(PlayerUiPlugin)
        .add_plugin(SettingsPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(PlayerInventoryPlugin)
        .add_plugin(FpsPlugin);

    #[cfg(not(feature = "free_camera"))] {
        use game::plugins::player::PlayerPlugin;
        app.add_plugin(PlayerPlugin);
    }

    #[cfg(feature = "debug")] {
        use game::plugins::debug::DebugPlugin;
        app.add_plugin(DebugPlugin);
    }

    app.run();

    Ok(())
}