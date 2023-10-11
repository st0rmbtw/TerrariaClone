use std::{io::{BufWriter, Write}, fs::File};

use rand::{thread_rng, Rng};

use crate::world::WorldData;

impl WorldData {
    #[allow(unused)]
    pub(crate) fn save_as_terraria_world(&self, world_name: &str) -> std::io::Result<()> {
        println!("Saving as Terraria world...");

        let world_file = File::create(format!("{}.wld", world_name))?;
        let mut world_writer = BufWriter::new(world_file);

        self.save_world_header(world_name, &mut world_writer)?;

        self.save_world_tiles(&mut world_writer)?;

        save_world_chests(&mut world_writer)?;

        save_world_signs(&mut world_writer)?;

        save_world_npc(&mut world_writer)?;

        save_world_npc_names(&mut world_writer)?;

        // Validation
        write_true(&mut world_writer)?;

        write_string(world_name, &mut world_writer)?;
        write_i32(123, &mut world_writer);

        Ok(())
    }

    fn save_world_header<W: Write>(&self, world_name: &str, world_writer: &mut BufWriter<W>) -> std::io::Result<()> {
        // Terraria world version
        write_i32(71, world_writer)?;
        
        // World name
        write_string(world_name, world_writer)?;

        // World Id
        write_i32(thread_rng().gen(), world_writer)?;

        // Left world
        write_i32(0, world_writer)?;

        // Right world
        write_i32((self.width() * 16) as i32, world_writer)?;

        // Top world
        write_i32(0, world_writer)?;

        // Bottom world
        write_i32((self.height() * 16) as i32, world_writer)?;

        // World height
        write_i32(self.height() as i32, world_writer)?;

        // World width
        write_i32(self.width() as i32, world_writer)?;

        // Moon type
        write_i8(1, world_writer)?;

        // Tree x
        write_i32(0, world_writer)?;
        write_i32(0, world_writer)?;
        write_i32(0, world_writer)?;

        // Tree style
        write_i32(0, world_writer)?;
        write_i32(0, world_writer)?;
        write_i32(0, world_writer)?;
        write_i32(0, world_writer)?;

        // Caveback x
        write_i32((self.width() / 2) as i32, world_writer)?;
        write_i32(self.width() as i32, world_writer)?;
        write_i32(self.width() as i32, world_writer)?;

        // Tree style
        write_i32(0, world_writer)?;
        write_i32(1, world_writer)?;
        write_i32(2, world_writer)?;
        write_i32(3, world_writer)?;

        // Iceback style
        write_i32(0, world_writer)?;

        // Jungleback style
        write_i32(0, world_writer)?;

        // Hellback style
        write_i32(0, world_writer)?;

        // Spawn tile coords
        world_writer.write_all(&self.spawn_point.x.to_le_bytes())?;
        world_writer.write_all(&self.spawn_point.y.to_le_bytes())?;
        
        // Ground layer
        write_f64((self.layer.underground - 1) as f64, world_writer)?;

        // Rock layer
        write_f64(self.layer.cavern as f64, world_writer)?;

        // Time
        write_f64(25000., world_writer)?;

        // Day time
        write_true(world_writer)?;

        // Moon phase
        write_i32(0, world_writer)?;

        // Blood phase
        write_false(world_writer)?;

        // Eclipse
        write_false(world_writer)?;

        // Dungeon x
        write_i32(0, world_writer)?;

        // Dungeon y
        write_i32(0, world_writer)?;

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
        write_i8(100, world_writer)?;

        // Altar count
        write_i32(50, world_writer)?;

        // Hard mode
        write_false(world_writer)?;

        // Invasion delay
        write_i32(0, world_writer)?;

        // Invasion size
        write_i32(0, world_writer)?;

        // Invasion type
        write_i32(0, world_writer)?;

        // Invasion x
        write_i32(0, world_writer)?;

        // Is raining
        write_false(world_writer)?;

        // Rain time
        write_i32(0, world_writer)?;

        // Max rain
        write_f64(0., world_writer)?;

        // Ore tier 1
        write_i32(0, world_writer)?;

        // Ore tier 2
        write_i32(0, world_writer)?;

        // Ore tier 3
        write_i32(0, world_writer)?;

        // TreeBG
        write_u8(5, world_writer)?;
        
        // CorruptBG
        write_u8(0, world_writer)?;

        // JungleBG
        write_u8(0, world_writer)?;

        // SnowBG
        write_u8(0, world_writer)?;

        // HallowBG
        write_u8(0, world_writer)?;

        // CrimsonBG
        write_u8(0, world_writer)?;

        // DesertBG
        write_u8(0, world_writer)?;

        // OceanBG
        write_u8(0, world_writer)?;

        // Cloud BG Active
        write_i32(0, world_writer)?;

        // Number of clouds
        write_i16(200, world_writer)?;

        // Wind speed set
        write_f32(-0.2, world_writer)?;

        Ok(())
    }

    fn save_world_tiles<W: Write>(&self, writer: &mut BufWriter<W>) -> std::io::Result<()> {
        for x in 0..self.width() {
            for y in 0..self.height() {
                let block = self.blocks[(y, x)];
                let wall = self.walls[(y, x)];

                // Is active
                write_bool(block.is_some(), writer)?;

                if let Some(block) = block {                    
                    write_u8(block.id(), writer)?;
                    
                    if let Some(block_frame) = block.frame() {
                        writer.write_all(&block_frame.x.to_le_bytes())?;
                        writer.write_all(&block_frame.y.to_le_bytes())?;
                    }
                    
                    // Color
                    write_false(writer)?;
                }

                // Is wall
                write_bool(wall.is_some(), writer)?;

                if let Some(wall) = wall {
                    writer.write_all(&wall.id().to_le_bytes())?;

                    // Color
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
                write_u8(0, writer)?;

                write_false(writer)?;
                write_false(writer)?;

                write_i16(0, writer)?;
            }
        }

        Ok(())
    }
}

fn save_world_chests<W: Write>(writer: &mut BufWriter<W>) -> std::io::Result<()> {
    for _ in 0..1000 {
        write_false(writer)?;
    }

    Ok(())
}

fn save_world_signs<W: Write>(writer: &mut BufWriter<W>) -> std::io::Result<()> {
    for _ in 0..1000 {
        write_false(writer)?;
    }

    Ok(())
}

fn save_world_npc<W: Write>(writer: &mut BufWriter<W>) -> std::io::Result<()> {
    write_false(writer)
}

fn save_world_npc_names<W: Write>(writer: &mut BufWriter<W>) -> std::io::Result<()> {
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

fn write_string<W: Write>(string: &str, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    let length = string.len() as u8;

    writer.write_all(&length.to_le_bytes())?;
    writer.write_all(string.as_bytes())
}

fn write_bool<W: Write>(b: bool, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    use std::mem::transmute;
    unsafe {
        writer.write_all(&transmute::<bool, [u8; 1]>(b))
    }
}

#[inline(always)]
fn write_false<W: Write>(writer: &mut BufWriter<W>) -> std::io::Result<()> {
    write_bool(false, writer)
}

#[inline(always)]
fn write_true<W: Write>(writer: &mut BufWriter<W>) -> std::io::Result<()> {
    write_bool(true, writer)
}

#[inline(always)]
fn write_f64<W: Write>(value: f64, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

#[inline(always)]
fn write_f32<W: Write>(value: f32, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

#[inline(always)]
fn write_i32<W: Write>(value: i32, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

#[inline(always)]
fn write_i16<W: Write>(value: i16, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

#[inline(always)]
fn write_i8<W: Write>(value: i8, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

#[inline(always)]
fn write_u8<W: Write>(value: u8, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}