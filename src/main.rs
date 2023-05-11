// #![windows_subsystem = "windows"]

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(not(feature = "test-world-generator"))]
    game::create_app()?.run();

    #[cfg(feature = "test-world-generator")] {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        game::test_world_generator(game::WorldSize::Tiny, seed)?;
    }

    Ok(())
}