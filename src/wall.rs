use std::collections::HashMap;

pub struct WallData {
    name: &'static str,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Wall {
    DirtWall,
    StoneWall,
}

lazy_static! {
    pub static ref BLOCK_DATA: HashMap<Wall, WallData> = HashMap::from([
        (Wall::DirtWall, WallData { name: "Dirt Wall" }),
        (Wall::StoneWall, WallData { name: "Stone Wall" })
    ]);
}
