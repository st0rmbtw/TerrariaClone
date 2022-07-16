use bevy::{prelude::*, window::PresentMode};
use bevy_rapier2d::plugin::{RapierPhysicsPlugin, NoUserData, RapierConfiguration};
use game::plugins::{SetupPlugin, PlayerPlugin, FpsPlugin, WorldPlugin, DebugPlugin};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Terraria".to_string(),
            present_mode: PresentMode::Fifo,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0., -2500.),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(SetupPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(FpsPlugin)
        .add_plugin(DebugPlugin)
        .add_plugin(PlayerPlugin)
        .run();
}