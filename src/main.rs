// #![windows_subsystem = "windows"]

use std::error::Error;

use game::GameApp;

fn main() -> Result<(), Box<dyn Error>> {
    GameApp::new()?.run();

    Ok(())
}