use bevy::{
    asset::AssetServerSettings,
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        texture::ImageSettings,
    },
    window::PresentMode,
};
use bevy_hanabi::HanabiPlugin;
use bevy_rapier2d::plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin};
use game::{
    animation::TweeningPlugin,
    parallax::ParallaxPlugin,
    plugins::{BackgroundPlugin, PlayerUiPlugin, SettingsPlugin, CursorPlugin},
};
use game::{
    plugins::{AssetsPlugin, FpsPlugin, MenuPlugin, PlayerPlugin, CameraPlugin, WorldPlugin},
    state::GameState,
};
use iyes_loopless::prelude::AppLooplessStateExt;

fn main() {
    let mut app = App::new();

    let mut settings = WgpuSettings::default();
    settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    app.insert_resource(settings)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Terraria".to_string(),
            present_mode: PresentMode::Fifo,
            cursor_visible: false,
            ..default()
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: false,
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::rgb(
            110. / 255.,
            151. / 255.,
            244. / 255.,
        )))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::Y * -0.,
            ..default()
        })
        .add_loopless_state(GameState::AssetLoading)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(2. * 16.))
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
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
        .add_plugin(PlayerPlugin)
        .add_plugin(FpsPlugin);

    #[cfg(debug_assertions)]
    app.add_plugin(game::plugins::DebugPlugin);

    app.run();
}
