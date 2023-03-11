use crate::util::TextureAtlasPos;

use super::TerrariaFrame;

#[derive(Debug, Clone, Copy)]
pub enum TreeFrameType {
    // A trunk
    TrunkPlain,
    // A left base
    BasePlainLeft,
    // A right base
    BasePlainRight,
    // A center base when there are both a left and a right bases
    BasePlainAD,
    // A center base when there is only a left base
    BasePlainA,
    // A center base when there is only a right base
    BasePlainD,
    // A left side branch with no leaves 
    BranchLeftBare,
    // A right side branch with no leaves
    BranchRightBare,
    // A left side branch with leaves
    BranchLeftLeaves,
    // A right side branch with leaves
    BranchRightLeaves,
    // A top of a tree with no leaves
    TopBare,
    // A top of a tree with leaves
    TopLeaves,
    // A jagged top of a tree with no leaves
    TopBareJagged,
}

impl TreeFrameType {
    pub const fn terraria_frame(&self, tree_type: TreeType) -> [TerrariaFrame; 3] {
        match tree_type {
            TreeType::Forest => {
                match self {
                    TreeFrameType::TrunkPlain => [TerrariaFrame::new(0, 0), TerrariaFrame::new(0, 22), TerrariaFrame::new(0, 44)],
                    TreeFrameType::BasePlainLeft => [TerrariaFrame::new(44, 132), TerrariaFrame::new(44, 154), TerrariaFrame::new(44, 176)],
                    TreeFrameType::BasePlainRight => [TerrariaFrame::new(22, 132), TerrariaFrame::new(22, 132), TerrariaFrame::new(22, 132)],
                    TreeFrameType::BasePlainAD => [TerrariaFrame::new(88, 132), TerrariaFrame::new(88, 154), TerrariaFrame::new(88, 176)],
                    TreeFrameType::BasePlainA => [TerrariaFrame::new(66, 132), TerrariaFrame::new(66, 154), TerrariaFrame::new(66, 176)],
                    TreeFrameType::BasePlainD => [TerrariaFrame::new(0, 132), TerrariaFrame::new(0, 154), TerrariaFrame::new(0, 176)],
                    TreeFrameType::BranchLeftBare => [TerrariaFrame::new(66, 0), TerrariaFrame::new(66, 22), TerrariaFrame::new(66, 44)],
                    TreeFrameType::BranchRightBare => [TerrariaFrame::new(88, 66), TerrariaFrame::new(88, 88), TerrariaFrame::new(88, 110)],
                    TreeFrameType::BranchLeftLeaves => [TerrariaFrame::new(44, 198), TerrariaFrame::new(44, 220), TerrariaFrame::new(44, 242)],
                    TreeFrameType::BranchRightLeaves =>[TerrariaFrame::new(66, 198), TerrariaFrame::new(66, 220), TerrariaFrame::new(66, 242)],
                    TreeFrameType::TopBare => [TerrariaFrame::new(110, 0), TerrariaFrame::new(110, 22), TerrariaFrame::new(110, 44)],
                    TreeFrameType::TopLeaves => [TerrariaFrame::new(22, 198), TerrariaFrame::new(22, 220), TerrariaFrame::new(22, 242)],
                    TreeFrameType::TopBareJagged => [TerrariaFrame::new(0, 198), TerrariaFrame::new(0, 220), TerrariaFrame::new(0, 242)]
                }
            }
        }
    }

    pub const fn texture_atlas_pos(&self, tree_type: TreeType) -> [TextureAtlasPos; 3] {
        match tree_type {
            TreeType::Forest => {
                match self {
                    TreeFrameType::TrunkPlain => [
                        TextureAtlasPos::new(0, 0), TextureAtlasPos::new(0, 1), TextureAtlasPos::new(0, 2)
                    ],
                    TreeFrameType::BasePlainLeft => [
                        TextureAtlasPos::new(2, 6), TextureAtlasPos::new(2, 7), TextureAtlasPos::new(2, 8)
                    ],
                    TreeFrameType::BasePlainRight => [
                        TextureAtlasPos::new(1, 6), TextureAtlasPos::new(1, 6), TextureAtlasPos::new(1, 6)
                    ],
                    TreeFrameType::BasePlainAD => [
                        TextureAtlasPos::new(4, 6), TextureAtlasPos::new(4, 7), TextureAtlasPos::new(4, 8)
                    ],
                    TreeFrameType::BasePlainA => [
                        TextureAtlasPos::new(3, 6), TextureAtlasPos::new(3, 7), TextureAtlasPos::new(3, 8)
                    ],
                    TreeFrameType::BasePlainD => [
                        TextureAtlasPos::new(0, 6), TextureAtlasPos::new(0, 7), TextureAtlasPos::new(0, 8)
                    ],
                    TreeFrameType::BranchLeftBare => [
                        TextureAtlasPos::new(3, 0), TextureAtlasPos::new(3, 1), TextureAtlasPos::new(3, 2)
                    ],
                    TreeFrameType::BranchRightBare => [
                        TextureAtlasPos::new(4, 3), TextureAtlasPos::new(4, 4), TextureAtlasPos::new(4, 5)
                    ],
                    TreeFrameType::BranchLeftLeaves => [
                        TextureAtlasPos::new(0, 0), TextureAtlasPos::new(0, 1), TextureAtlasPos::new(0, 2)
                    ],
                    TreeFrameType::BranchRightLeaves =>[
                        TextureAtlasPos::new(1, 0), TextureAtlasPos::new(1, 1), TextureAtlasPos::new(1, 2)
                    ],
                    TreeFrameType::TopBare => [
                        TextureAtlasPos::new(5, 0), TextureAtlasPos::new(5, 1), TextureAtlasPos::new(5, 2)
                    ],
                    TreeFrameType::TopLeaves => [
                        TextureAtlasPos::new(0, 0), TextureAtlasPos::new(0, 1), TextureAtlasPos::new(0, 2)
                    ],
                    TreeFrameType::TopBareJagged => [
                        TextureAtlasPos::new(0, 9), TextureAtlasPos::new(0, 10), TextureAtlasPos::new(0, 11)
                    ]
                }
            }
        }
    }

    pub const fn texture_width(&self) -> u32 {
        match self {
            TreeFrameType::BranchLeftLeaves | TreeFrameType::BranchRightLeaves => 2,
            TreeFrameType::TopLeaves => 3,
            _ => 64
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeType {
    Forest
}

#[derive(Debug, Clone, Copy)]
pub struct Tree {
    pub tree_type: TreeType,
    pub frame_type: TreeFrameType,
    pub variant: usize
}

impl Tree {
    pub const fn new(tree_type: TreeType, frame_type: TreeFrameType, variant: usize) -> Self {
        Self { tree_type, frame_type, variant }
    }

    pub const fn terraria_frame(&self) -> TerrariaFrame {
        assert!(self.variant < 3, "Frame variant must be in range of 0 to 3");

        self.frame_type.terraria_frame(self.tree_type)[self.variant]
    }
    
    pub const fn texture_atlas_pos(&self) -> u32 {
        assert!(self.variant < 3, "Frame variant must be in range of 0 to 3");

        self.frame_type
            .texture_atlas_pos(self.tree_type)[self.variant]
            .to_2d_index(self.frame_type.texture_width())
    }
}

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        self.tree_type == other.tree_type
    }
}

macro_rules! tree {
    ($tree_type: path, $frame_type: ident, $variant: ident) => {
        Block {
            block_type: crate::plugins::world::block::BlockType::Tree(crate::plugins::world::tree::Tree::new($tree_type, $frame_type, $variant)),
            hp: crate::plugins::world::block::BlockType::Tree(crate::plugins::world::tree::Tree::new($tree_type, $frame_type, $variant)).max_health()
        }
    };
}

pub(crate) use tree;