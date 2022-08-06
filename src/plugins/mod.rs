mod setup;
mod player;
mod fps;
mod world;
#[cfg(debug_assertions)]
mod debug;
mod inventory;
mod assets;
mod cursor;

pub use cursor::*;
pub use setup::*;
pub use player::*;
pub use fps::*;
pub use world::*;
#[cfg(debug_assertions)]
pub use debug::*;
pub use inventory::*;
pub use assets::*;