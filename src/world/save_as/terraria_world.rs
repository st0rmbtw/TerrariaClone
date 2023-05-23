use std::{io::{BufWriter, Write}, fs::File};

use crate::world::{WorldData, generator::{DIRT_HILL_HEIGHT, STONE_HILL_HEIGHT}};

use super::DefaultResult;

impl WorldData {
    #[allow(unused)]
    pub(crate) fn save_as_terraria_world(&self, file_name: &str) -> DefaultResult {
        println!("Saving as Terraria world...");

        let world_file = File::create(file_name)?;
        let mut world_writer = BufWriter::new(world_file);

        let world_name = "LOOOL";

        self.save_world_header(world_name, &mut world_writer)?;

        self.save_world_tiles(&mut world_writer)?;

        save_world_chests(&mut world_writer)?;

        save_world_signs(&mut world_writer)?;

        save_world_npc(&mut world_writer)?;

        save_world_npc_names(&mut world_writer)?;

        // Validation
        write_true(&mut world_writer)?;

        write_string(world_name, &mut world_writer)?;
        world_writer.write_all(&123_i32.to_le_bytes())?;

        Ok(())
    }

    fn save_world_header<W: Write>(&self, world_name: &str, world_writer: &mut BufWriter<W>) -> DefaultResult {
        // Terraria world version
        world_writer.write_all(&71_i32.to_le_bytes())?;
        
        write_string(world_name, world_writer)?;

        // World Id
        {
            let world_id = 123_i32;
            world_writer.write_all(&world_id.to_le_bytes())?;
        }

        // Left world
        world_writer.write_all(&0i32.to_le_bytes())?;

        // Right world
        {
            let right_world = (self.size.width * 16) as i32;
            world_writer.write_all(&right_world.to_le_bytes())?;
        }

        // Top world
        world_writer.write_all(&0i32.to_le_bytes())?;

        // Bottom world
        {
            let bottom_world = (self.size.height * 16) as i32;
            world_writer.write_all(&bottom_world.to_le_bytes())?;
        }

        // World height
        {
            let world_height = self.size.height as i32;
            world_writer.write_all(&world_height.to_le_bytes())?;
        }

        // World width
        {
            let world_width = self.size.width as i32;
            world_writer.write_all(&world_width.to_le_bytes())?;
        }

        // Moon type
        {
            let moon_type = 1_i8;
            world_writer.write_all(&moon_type.to_le_bytes())?;
        }

        // Tree x
        {    
            let tree_x_0 = 0_i32;
            let tree_x_1 = 0_i32;
            let tree_x_2 = 0_i32;

            world_writer.write_all(&tree_x_0.to_le_bytes())?;
            world_writer.write_all(&tree_x_1.to_le_bytes())?;
            world_writer.write_all(&tree_x_2.to_le_bytes())?;
        }

        // Tree style
        {
            let tree_style_0 = 0_i32;
            let tree_style_1 = 0_i32;
            let tree_style_2 = 0_i32;
            let tree_style_3 = 0_i32;

            world_writer.write_all(&tree_style_0.to_le_bytes())?;
            world_writer.write_all(&tree_style_1.to_le_bytes())?;
            world_writer.write_all(&tree_style_2.to_le_bytes())?;
            world_writer.write_all(&tree_style_3.to_le_bytes())?;
        }

        // Caveback x
        {
            let caveback_x_0 = (self.size.width / 2) as i32;
            let caveback_x_1 = self.size.width as i32;
            let caveback_x_2 = self.size.width as i32;

            world_writer.write_all(&caveback_x_0.to_le_bytes())?;
            world_writer.write_all(&caveback_x_1.to_le_bytes())?;
            world_writer.write_all(&caveback_x_2.to_le_bytes())?;
        }

        // Tree style
        {
            let caveback_style_0 = 0_i32;
            let caveback_style_1 = 1_i32;
            let caveback_style_2 = 2_i32;
            let caveback_style_3 = 3_i32;

            world_writer.write_all(&caveback_style_0.to_le_bytes())?;
            world_writer.write_all(&caveback_style_1.to_le_bytes())?;
            world_writer.write_all(&caveback_style_2.to_le_bytes())?;
            world_writer.write_all(&caveback_style_3.to_le_bytes())?;
        }

        // Iceback style
        {
            let iceback_style = 0i32;
            world_writer.write_all(&iceback_style.to_le_bytes())?;
        }

        // Jungleback style
        {
            let jungleback_style = 0i32;
            world_writer.write_all(&jungleback_style.to_le_bytes())?;
        }

        // Hellback style
        {
            let hellback_style = 0i32;
            world_writer.write_all(&hellback_style.to_le_bytes())?;
        }

        // Spawn tile coords
        {
            world_writer.write_all(&self.spawn_point.x.to_le_bytes())?;
            world_writer.write_all(&self.spawn_point.y.to_le_bytes())?;
        }
        
        // World surface
        {
            let world_surface = (self.layer.surface + DIRT_HILL_HEIGHT as usize) as f64;
            world_writer.write_all(&world_surface.to_le_bytes())?;
        }

        // Rock layer
        {
            let spawn_tile_y = (self.layer.surface + STONE_HILL_HEIGHT as usize) as f64;
            world_writer.write_all(&spawn_tile_y.to_le_bytes())?;
        }

        // Time
        {
            let time = 0f64;
            world_writer.write_all(&time.to_le_bytes())?;
        }

        // Day time
        {
            write_true(world_writer)?;
        }

        // Moon phase
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Blood phase
        write_false(world_writer)?;

        // Eclipse
        write_false(world_writer)?;

        // Dungeon x
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Dungeon y
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Downed boss 1
        write_false(world_writer)?;

        // Downed boss 2
        write_false(world_writer)?;

        // Downed boss 3
        write_false(world_writer)?;

        // Downed boss queen bee
        write_false(world_writer)?;

        // Downed mech boss 1
        write_false(world_writer)?;

        // Downed mech boss 2
        write_false(world_writer)?;

        // Downed mech boss 3
        write_false(world_writer)?;

        // Downed mech boss any
        write_false(world_writer)?;

        // Downed plant boss
        write_false(world_writer)?;

        // Downed golem
        write_false(world_writer)?;

        // Saved goblin
        write_false(world_writer)?;

        // Saved wizard
        write_false(world_writer)?;

        // Saved mech
        write_false(world_writer)?;

        // Downed goblins
        write_false(world_writer)?;

        // Downed clown
        write_false(world_writer)?;

        // Downed frost
        write_false(world_writer)?;

        // Downed pirates
        write_false(world_writer)?;

        // Shadow orb smashed
        write_false(world_writer)?;

        // Spawn meteor
        write_false(world_writer)?;

        // Downed golem
        write_false(world_writer)?;

        // Shadow orb count
        {
            let value = 100i8;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Altar count
        {
            let value = 50i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Hard mode
        write_false(world_writer)?;

        // Invasion delay
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Invasion size
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Invasion type
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Invasion x
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Is raining
        {
            write_false(world_writer)?;
        }

        // Rain time
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Max rain
        {
            let value = 0f64;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Ore tier 1
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Ore tier 2
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Ore tier 3
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // TreeBG
        {
            let value = 0u8;
            world_writer.write_all(&value.to_le_bytes())?;
        }
        
        // CorruptBG
        {
            let value = 0u8;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // JungleBG
        {
            let value = 0u8;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // SnowBG
        {
            let value = 0u8;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // HallowBG
        {
            let value = 0u8;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // CrimsonBG
        {
            let value = 0u8;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // DesertBG
        {
            let value = 0u8;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // OceanBG
        {
            let value = 0u8;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Cloud BG Active
        {
            let value = 0i32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Number of clouds
        {
            let value = 200i16;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        // Wind speed set
        {
            let value = -0.2f32;
            world_writer.write_all(&value.to_le_bytes())?;
        }

        Ok(())
    }

    fn save_world_tiles<W: Write>(&self, writer: &mut BufWriter<W>) -> DefaultResult {
        for x in 0..self.size.width {
            for y in 0..self.size.height {
                let block = self.blocks[(y, x)];
                let wall = self.walls[(y, x)];

                if let Some(block) = block {
                    write_true(writer)?;
                    
                    writer.write_all(&block.id().to_le_bytes())?;
                    
                    if let Some(block_frame) = block.frame() {
                        writer.write_all(&block_frame.x.to_le_bytes())?;
                        writer.write_all(&block_frame.y.to_le_bytes())?;
                    }

                    write_false(writer)?;
                } else {
                    write_false(writer)?;
                }

                if let Some(wall) = wall {
                    write_true(writer)?;
                    writer.write_all(&wall.id().to_le_bytes())?;

                    // Color
                    write_false(writer)?;
                } else {
                    write_false(writer)?;
                }

                // Is liquid
                write_false(writer)?;

                // Is wire (red, green, blue)
                write_false(writer)?;
                write_false(writer)?;
                write_false(writer)?;

                // Half brick
                write_false(writer)?;

                // Slope
                writer.write_all(&0u8.to_le_bytes())?;

                write_false(writer)?;
                write_false(writer)?;

                writer.write_all(&0_i16.to_le_bytes())?;
            }
        }

        Ok(())
    }
}

fn save_world_chests<W: Write>(writer: &mut BufWriter<W>) -> DefaultResult {
    for _ in 0..1000 {
        write_false(writer)?;
    }

    Ok(())
}

fn save_world_signs<W: Write>(writer: &mut BufWriter<W>) -> DefaultResult {
    for _ in 0..1000 {
        write_false(writer)?;
    }

    Ok(())
}

fn save_world_npc<W: Write>(writer: &mut BufWriter<W>) -> DefaultResult {
    write_false(writer)?;

    Ok(())
}

fn save_world_npc_names<W: Write>(writer: &mut BufWriter<W>) -> DefaultResult {
    write_string("NPC 1", writer)?;
    write_string("NPC 2", writer)?;
    write_string("NPC 3", writer)?;
    write_string("NPC 4", writer)?;
    write_string("NPC 5", writer)?;
    write_string("NPC 6", writer)?;
    write_string("NPC 7", writer)?;
    write_string("NPC 8", writer)?;
    write_string("NPC 9", writer)?;
    write_string("NPC 10", writer)?;
    write_string("NPC 11", writer)?;
    write_string("NPC 12", writer)?;
    write_string("NPC 13", writer)?;
    write_string("NPC 14", writer)?;
    write_string("NPC 15", writer)?;
    write_string("NPC 16", writer)?;
    write_string("NPC 17", writer)?;
    write_string("NPC 18", writer)?;

    Ok(())
}

fn write_string<W: Write>(string: &str, writer: &mut BufWriter<W>) -> DefaultResult {
    let length = string.len() as u8;

    writer.write_all(&length.to_le_bytes())?;
    writer.write_all(string.as_bytes())?;

    Ok(())
}

fn write_bool<W: Write>(b: bool, writer: &mut BufWriter<W>) -> DefaultResult {
    use std::mem::transmute;
    unsafe {
        writer.write_all(&transmute::<bool, [u8; 1]>(b))?;
    }

    Ok(())
}

#[inline(always)]
fn write_false<W: Write>(writer: &mut BufWriter<W>) -> DefaultResult {
    return write_bool(false, writer);
}

#[inline(always)]
fn write_true<W: Write>(writer: &mut BufWriter<W>) -> DefaultResult {
    return write_bool(true, writer);
}
