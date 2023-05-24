// #![windows_subsystem = "windows"]

use std::error::Error;

#[cfg(not(feature = "test-world-generator"))]
fn main() -> Result<(), Box<dyn Error>> {
    game::create_app()?.run();
    Ok(())
}

#[cfg(feature = "test-world-generator")]
fn main() -> Result<(), Box<dyn Error>> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;

    game::test_world_generator(game::WorldSize::Tiny, seed, true)?;

    Ok(())
}