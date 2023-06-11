// #![windows_subsystem = "windows"]

use std::error::Error;

#[cfg(not(any(feature = "world_image", feature = "terraria_world")))]
fn main() -> Result<(), Box<dyn Error>> {
    game::create_app()?.run();
    Ok(())
}

#[cfg(all(feature = "terraria_world", not(feature = "world_image")))]
fn main() -> Result<(), Box<dyn Error>> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;

    println!("The seed of the world is {}", seed);

    game::generate_terraria_world_file(game::WorldSize::Tiny, seed, "LOOL")?;
    
    Ok(())
}

#[cfg(feature = "world_image")]
fn main() -> Result<(), Box<dyn Error>> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;

    println!("The seed of the world is {}", seed);

    game::generate_world_image(game::WorldSize::Tiny, seed, false)
}