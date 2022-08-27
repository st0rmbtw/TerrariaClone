mod setup;
mod player;
mod fps;
mod world;
#[cfg(debug_assertions)]
mod debug;
mod inventory;
mod assets;
mod cursor;
mod menu;
mod background;
mod ui;
mod settings;

pub use cursor::*;
pub use setup::*;
pub use player::*;
pub use fps::*;
pub use world::*;
#[cfg(debug_assertions)]
pub use debug::*;
pub use inventory::*;
pub use assets::*;
pub use menu::*;
pub use background::*;
pub use ui::*;
pub use settings::*;