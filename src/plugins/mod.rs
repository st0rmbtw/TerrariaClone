pub mod assets;
pub mod background;
pub mod cursor;
pub mod fps;
pub mod inventory;
pub mod menu;
#[cfg(not(feature = "free_camera"))]
pub mod player;
pub mod settings;
pub mod camera;
pub mod ui;
pub mod world;

#[cfg(feature = "debug")]
pub mod debug;