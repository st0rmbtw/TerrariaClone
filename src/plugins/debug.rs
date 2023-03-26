use bevy::{prelude::{App, Plugin,IntoSystemConfig, OnUpdate, ResMut, Commands, TextBundle, Res, Color, IntoSystemAppConfig, OnEnter, Component, Query, Visibility, With, DetectChanges, Name, AppTypeRegistry}, utils::default, text::{Text, TextSection, TextStyle}, ui::{Style, UiRect, Val, PositionType}, sprite::TextureAtlasSprite, time::Time, reflect::{Reflect, ReflectMut}};
use bevy_inspector_egui::{bevy_egui::{EguiPlugin, egui, EguiContexts}, egui::Align2, quick::WorldInspectorPlugin, reflect_inspector};

use crate::{common::{state::GameState, helpers}, DebugConfiguration};
use bevy_prototype_debug_lines::DebugLinesPlugin;

use super::{cursor::CursorPosition, assets::FontAssets, inventory::{UseItemAnimationIndex, UseItemAnimationData}};

pub(crate) struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin);
        app.add_plugin(DebugLinesPlugin::default());
        app.add_plugin(WorldInspectorPlugin::new());

        app.register_type::<CursorPosition>();
        app.register_type::<TextureAtlasSprite>();
        app.register_type::<UseItemAnimationIndex>();
        app.register_type::<UseItemAnimationData>();

        app.add_system(debug_gui.in_set(OnUpdate(GameState::InGame)));
        app.add_system(spawn_free_camera_legend.in_schedule(OnEnter(GameState::InGame)));
        app.add_system(set_free_camera_legend_visibility.in_set(OnUpdate(GameState::InGame)));
    }
}

#[derive(Component)]
struct FreeCameraLegendText;

fn debug_gui(
    mut contexts: EguiContexts, 
    mut debug_config: ResMut<DebugConfiguration>,
    mut time: ResMut<Time>,
    type_registry: Res<AppTypeRegistry>,
) {
    let egui_context = contexts.ctx_mut();

    egui::Window::new("Debug Menu")
        .anchor(Align2::RIGHT_TOP, (-10., 10.))
        .resizable(false)
        .show(egui_context, |ui| {
            ui.checkbox(&mut debug_config.free_camera, "Free Camera");
            ui.checkbox(&mut debug_config.show_hitboxes, "Show Hitboxes");
            ui.checkbox(&mut debug_config.show_collisions, "Show Collisions");
            ui.checkbox(&mut debug_config.instant_break, "Break tiles instantly");

            if let ReflectMut::Struct(str) = time.reflect_mut() {
                ui.heading("Game Time");
                ui.separator();
                ui.columns(2, |columns| {
                    columns[0].label("Pause:");
                    reflect_inspector::ui_for_value(str.field_mut("paused").unwrap(), &mut columns[1], &type_registry.0.read());
                    columns[0].label("Speed:");
                    reflect_inspector::ui_for_value(str.field_mut("relative_speed").unwrap(), &mut columns[1], &type_registry.0.read());
                });
            }

            ui.heading("Player Speed");
            ui.separator();
            ui.columns(2, |columns| {
                columns[0].label("Horizontal:");
                reflect_inspector::ui_for_value_readonly(&debug_config.player_speed.x, &mut columns[1], &type_registry.0.read());
                columns[0].label("Vertical:");
                reflect_inspector::ui_for_value_readonly(&debug_config.player_speed.y, &mut columns[1], &type_registry.0.read());
            });
        });
}

fn set_free_camera_legend_visibility(
    mut query: Query<&mut Visibility, With<FreeCameraLegendText>>,
    debug_config: Res<DebugConfiguration>
) {
    if debug_config.is_changed() {
        let visibility = query.single_mut();
        helpers::set_visibility(visibility, debug_config.free_camera);
    }
}

fn spawn_free_camera_legend(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
) {
    let text_style = TextStyle {
        font: font_assets.andy_regular.clone_weak(),
        font_size: 22.,
        color: Color::WHITE,
    };

    commands.spawn((
        TextBundle {
            style: Style {
                position: UiRect::new(Val::Px(20.), Val::Undefined, Val::Undefined, Val::Px(50.)),
                position_type: PositionType::Absolute,
                ..default()
            },
            text: Text::from_sections([
                TextSection::new("Right Click to teleport\nWASD to pan, Shift for faster, Alt for slower", text_style),
            ]),
            visibility: Visibility::Hidden,
            ..default()
        },
        Name::new("Free Camera Legend Text"),
        FreeCameraLegendText
    ));
}