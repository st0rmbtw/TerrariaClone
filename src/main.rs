// #![windows_subsystem = "windows"]

use std::error::Error;

#[cfg(not(feature = "world_image"))]
fn main() -> Result<(), Box<dyn Error>> {
    game::create_app()?.run();
    Ok(())
}

#[cfg(feature = "world_image")]
fn main() -> Result<(), Box<dyn Error>> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;

    game::test_world_generator(game::WorldSize::Tiny, seed, false)?;

    Ok(())
}