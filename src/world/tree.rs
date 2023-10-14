use crate::common::TextureAtlasPos;

use super::TerrariaFrame;

#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect))]
#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
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
    pub(crate) const fn terraria_frame(&self, tree_type: TreeType) -> [TerrariaFrame; 3] {
        match tree_type {
            TreeType::Forest => {
                match self {
                    Self::TrunkPlain => [TerrariaFrame::new(0, 0), TerrariaFrame::new(0, 22), TerrariaFrame::new(0, 44)],
                    Self::BasePlainLeft => [TerrariaFrame::new(44, 132), TerrariaFrame::new(44, 154), TerrariaFrame::new(44, 176)],
                    Self::BasePlainRight => [TerrariaFrame::new(22, 132), TerrariaFrame::new(22, 132), TerrariaFrame::new(22, 132)],
                    Self::BasePlainAD => [TerrariaFrame::new(88, 132), TerrariaFrame::new(88, 154), TerrariaFrame::new(88, 176)],
                    Self::BasePlainA => [TerrariaFrame::new(66, 132), TerrariaFrame::new(66, 154), TerrariaFrame::new(66, 176)],
                    Self::BasePlainD => [TerrariaFrame::new(0, 132), TerrariaFrame::new(0, 154), TerrariaFrame::new(0, 176)],
                    Self::BranchLeftBare => [TerrariaFrame::new(66, 0), TerrariaFrame::new(66, 22), TerrariaFrame::new(66, 44)],
                    Self::BranchRightBare => [TerrariaFrame::new(88, 66), TerrariaFrame::new(88, 88), TerrariaFrame::new(88, 110)],
                    Self::BranchLeftLeaves => [TerrariaFrame::new(44, 198), TerrariaFrame::new(44, 220), TerrariaFrame::new(44, 242)],
                    Self::BranchRightLeaves =>[TerrariaFrame::new(66, 198), TerrariaFrame::new(66, 220), TerrariaFrame::new(66, 242)],
                    Self::TopBare => [TerrariaFrame::new(110, 0), TerrariaFrame::new(110, 22), TerrariaFrame::new(110, 44)],
                    Self::TopLeaves => [TerrariaFrame::new(22, 198), TerrariaFrame::new(22, 220), TerrariaFrame::new(22, 242)],
                    Self::TopBareJagged => [TerrariaFrame::new(0, 198), TerrariaFrame::new(0, 220), TerrariaFrame::new(0, 242)]
                }
            }
        }
    }

    pub(crate) const fn texture_atlas_pos(&self, tree_type: TreeType, variant: u32) -> TextureAtlasPos {
        assert!(variant < 3, "Variant of texture must be in range of 0 to 3");

        match tree_type {
            TreeType::Forest => {
                match self {
                    Self::TrunkPlain => TextureAtlasPos::new(0, variant),
                    Self::BasePlainLeft => TextureAtlasPos::new(2, 6 + variant),
                    Self::BasePlainRight => TextureAtlasPos::new(1, 6 + variant),
                    Self::BasePlainAD => TextureAtlasPos::new(4, 6 + variant),
                    Self::BasePlainA => TextureAtlasPos::new(3, 6 + variant),
                    Self::BasePlainD => TextureAtlasPos::new(0, 6 + variant),
                    Self::BranchLeftBare => TextureAtlasPos::new(3, variant),
                    Self::BranchRightBare => TextureAtlasPos::new(4, 3 + variant),
                    Self::BranchLeftLeaves => TextureAtlasPos::new(0, variant),
                    Self::BranchRightLeaves => TextureAtlasPos::new(1, variant),
                    Self::TopBare => TextureAtlasPos::new(5, variant),
                    Self::TopLeaves => TextureAtlasPos::new(0, variant),
                    Self::TopBareJagged => TextureAtlasPos::new(0, 9 + variant)
                }
            }
        }
    }

    pub(crate) const fn is_trunk(&self) -> bool {
        matches!(self, Self::TrunkPlain | Self::BasePlainA | Self::BasePlainD | Self::BasePlainAD)
    }

    pub(crate) const fn texture_width(&self) -> u32 {
        match self {
            Self::BranchLeftLeaves | Self::BranchRightLeaves => 2,
            Self::TopLeaves => 3,
            _ => 64
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect))]
pub enum TreeType {
    Forest
}

#[derive(Debug, Clone, Copy, Eq)]
#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect))]
pub struct Tree {
    pub(crate) tree_type: TreeType,
    pub(crate) frame_type: TreeFrameType,
}

impl Tree {
    pub(crate) const fn new(tree_type: TreeType, frame_type: TreeFrameType) -> Self {
        Self { tree_type, frame_type }
    }

    pub(crate) const fn terraria_frame(&self, variant: u32) -> TerrariaFrame {
        self.frame_type.terraria_frame(self.tree_type)[variant as usize]
    }
    
    pub(crate) const fn texture_atlas_pos(&self, variant: u32) -> u32 {
        self.frame_type
            .texture_atlas_pos(self.tree_type, variant)
            .to_2d_index(self.frame_type.texture_width())
    }
}

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        self.tree_type == other.tree_type
    }
}