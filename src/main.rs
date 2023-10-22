// #![windows_subsystem = "windows"]

use std::error::Error;

#[cfg(any(feature = "terraria_world", feature = "world_image"))]
use game::world::{WorldSize, generator, wall::WallType};

#[cfg(not(any(feature = "world_image", feature = "terraria_world")))]
fn main() -> Result<(), Box<dyn Error>> {
    game::create_app()?.run();
    Ok(())
}

#[cfg(all(feature = "terraria_world", not(feature = "world_image")))]
fn main() -> Result<(), Box<dyn Error>> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = std::env::args()
        .nth(1)
        .and_then(|seed| seed.parse::<u32>().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u32
        });

    println!("The seed of the world is {}", seed);

    generate_terraria_world_file(WorldSize::Tiny, seed, &seed.to_string())?;
    
    Ok(())
}

#[cfg(feature = "world_image")]
fn main() -> Result<(), Box<dyn Error>> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seed = std::env::args()
        .nth(1)
        .and_then(|seed| seed.parse::<u32>().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u32
        });

    let draw_layers = std::env::args()
        .nth(2)
        .and_then(|draw_layers| draw_layers.parse::<bool>().ok())
        .unwrap_or(false);

    println!("The seed of the world is {}", seed);
    println!("Draw world layers: {}", if draw_layers { "Enabled" } else { "Disabled" });

    generate_world_image(WorldSize::Tiny, seed, draw_layers)
}

#[cfg(feature = "terraria_world")]
pub fn generate_terraria_world_file(world_size: WorldSize, seed: u32, world_name: &str) -> std::io::Result<()> {
    generator::generate_world(seed, world_size).save_as_terraria_world(world_name)
}

#[cfg(feature = "world_image")]
pub fn generate_world_image(world_size: WorldSize, seed: u32, draw_layers: bool) -> Result<(), Box<dyn std::error::Error>> {
    use image::{RgbImage, ImageBuffer, GenericImageView, Pixel};

    fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
        return out_min + (((value - in_min) / (in_max - in_min)) * (out_max - out_min))
    }

    let world_data = generator::generate_world(seed, world_size);

    let size = world_size.size();

    let playable_area_min_x = world_data.playable_area.min.x;
    let playable_area_min_y = world_data.playable_area.min.y;

    println!("Saving as image...");

    let sky_image = image::io::Reader::open("assets/sprites/backgrounds/Background_0.png")?.decode()?;
    let sky_image_height = sky_image.height() as usize;
    let mut image: RgbImage = ImageBuffer::new(world_data.playable_area.width(), world_data.playable_area.height());

    // Draw walls
    for y in 0..world_data.playable_area.height() {
        for x in 0..world_data.playable_area.width() {
            if let Some(color) = world_data.get_wall_color((playable_area_min_x + x, playable_area_min_y + y)) {
                image.put_pixel(x as u32, y as u32, image::Rgb(color));
            } else {
                let sky_image_y = map_range(y as f32, 0., size.height as f32, 0., sky_image_height as f32);

                let color = sky_image.get_pixel(0, sky_image_y as u32);

                image.put_pixel(x as u32, y as u32, color.to_rgb());
            }
        }
    }

    // Draw background
    for y in world_data.layer.underground..world_data.playable_height() {
        for x in 0..world_data.playable_width() {
            let color = WallType::Dirt.color();
            image.put_pixel(x as u32, y as u32, image::Rgb(color));
        }
    }

    // Draw blocks
    for y in 0..world_data.playable_area.height() {
        for x in 0..world_data.playable_area.width() {
            if let Some(color) = world_data.get_block_color((playable_area_min_x + x, playable_area_min_y + y)) {
                image.put_pixel(x as u32, y as u32, image::Rgb(color));
            }
        }
    }

    // Draw layer borders
    if draw_layers {
        let surface_layer = world_data.layer.surface;
        let underground_layer = world_data.layer.underground;
        let cavern_layer = world_data.layer.cavern;

        for x in 0..world_data.playable_width() {
            image.put_pixel(x as u32, surface_layer as u32, image::Rgb([255, 0, 0]));
            image.put_pixel(x as u32, underground_layer as u32, image::Rgb([255, 0, 0]));
            image.put_pixel(x as u32, cavern_layer as u32, image::Rgb([255, 0, 0]));
        }
    }

    image.save("world.png")?;

    Ok(())
}