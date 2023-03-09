use super::Frame;

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
    pub const fn frame(&self, tree_type: TreeType) -> [Frame; 3] {
        match tree_type {
            TreeType::Forest => {
                match self {
                    TreeFrameType::TrunkPlain => [Frame::new(0, 0), Frame::new(0, 22), Frame::new(0, 44)],
                    TreeFrameType::BasePlainLeft => [Frame::new(44, 132), Frame::new(44, 154), Frame::new(44, 176)],
                    TreeFrameType::BasePlainRight => [Frame::new(22, 132), Frame::new(22, 132), Frame::new(22, 132)],
                    TreeFrameType::BasePlainAD => [Frame::new(88, 132), Frame::new(88, 154), Frame::new(88, 176)],
                    TreeFrameType::BasePlainA => [Frame::new(66, 132), Frame::new(66, 154), Frame::new(66, 176)],
                    TreeFrameType::BasePlainD => [Frame::new(0, 132), Frame::new(0, 154), Frame::new(0, 176)],
                    TreeFrameType::BranchLeftBare => [Frame::new(66, 0), Frame::new(66, 22), Frame::new(66, 44)],
                    TreeFrameType::BranchRightBare => [Frame::new(88, 66), Frame::new(88, 88), Frame::new(88, 110)],
                    TreeFrameType::BranchLeftLeaves => [Frame::new(44, 198), Frame::new(44, 220), Frame::new(44, 242)],
                    TreeFrameType::BranchRightLeaves =>[Frame::new(66, 198), Frame::new(66, 220), Frame::new(66, 242)],
                    TreeFrameType::TopBare => [Frame::new(110, 0), Frame::new(110, 22), Frame::new(110, 44)],
                    TreeFrameType::TopLeaves => [Frame::new(22, 198), Frame::new(22, 220), Frame::new(22, 242)],
                    TreeFrameType::TopBareJagged => [Frame::new(0, 198), Frame::new(0, 220), Frame::new(0, 242)]
                }
            }
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
    pub frame: Frame
}

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        self.tree_type == other.tree_type
    }
}

macro_rules! tree {
    ($tree_type: path, $frame: ident) => {
        Block {
            block_type: crate::plugins::world::block::BlockType::Tree(crate::plugins::world::tree::Tree {
                tree_type: $tree_type,
                frame: $frame
            }),
            hp: crate::plugins::world::block::BlockType::Tree(crate::plugins::world::tree::Tree {
                tree_type: $tree_type,
                frame: $frame
            }).max_health()
        }
    };
}

pub(crate) use tree;