use bevy::prelude::Plugin;


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // app.add_startup_system(spawn_hotbar.label(SPAWN_PLAYER_UI_LABEL));
    }
}
