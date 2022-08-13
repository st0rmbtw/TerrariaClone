use bevy::{prelude::*, window::PresentMode};
use bevy_rapier2d::plugin::{RapierPhysicsPlugin, NoUserData, RapierConfiguration};
use bevy_tweening::TweeningPlugin;
use game::plugins::{PlayerPlugin, FpsPlugin, WorldPlugin, DebugPlugin, AssetsPlugin, SetupPlugin};

fn main() {
    let mut app = App::new();

    app
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Terraria".to_string(),
            present_mode: PresentMode::Immediate,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0., -30.),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(SetupPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(FpsPlugin);
    
    if cfg!(debug_assertions) {
        app.add_plugin(DebugPlugin);
    }

    app.run();
}