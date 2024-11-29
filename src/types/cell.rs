use num_enum::FromPrimitive;

use crate::{
    constants::{BTS_BREAKABLE_MASK_MASK, BTS_SLOPE_FLIP_MASK, BTS_SLOPE_TYPE_MASK, CELL_SIZE},
    shapes::vector::Vector,
    types::Flip,
};

#[derive(Debug, PartialEq, Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum BlockType {
    #[default]
    Air = 0x0,
    Slope,
    AirXray,
    Treadmill,
    AirShot,
    HCopy,
    Unused,
    AirBomb,
    Solid,
    Door,
    Spike,
    Crumble,
    Shot,
    VCopy,
    Grapple,
    Bomb,
}

#[derive(Debug, PartialEq, Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum SlopeType {
    HalfSolidH = 0x0,
    HalfSolidV,
    QuarterSolid,
    StairBigSteps,
    FullSolidUnused,
    SmallTriangle,
    BigTriangle,
    HalfPlat,
    SquareDuplicate1,
    SquareDuplicate2,
    SquareDuplicate3,
    SquareDuplicate4,
    SquareDuplicate5,
    SquareDuplicate6,
    StairSmallSteps,
    ConcaveTriangle,
    HorizontalLines,
    VerticalLines,
    Slope45,
    Square,
    HillPart1,
    HillPart2,
    SmoothHillPart1,
    SmoothHillPart2,
    SmootherHillPart1,
    SmootherHillPart2,
    SmootherHillPart3,
    SteepHillPart1,
    SteepHillPart2,
    SteeperHillPart1,
    SteeperHillPart2,
    SteeperHillPart3,
    #[default]
    None,
}

#[derive(Debug, PartialEq, Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum TreatAsSlopeType {
    #[default]
    Solid = 0x0,
    SlopeRight,
    SlopeLeft,
    SlopeProtectNegX,
    SlopeProtectPosX,
}

#[derive(Debug, PartialEq, Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum BlockMask {
    #[default]
    OneByOne = 0,
    TwoByOne,
    OneByTwo,
    TwoByTwo,
}

impl BlockMask {
    pub fn export(&self) -> u16 {
        match self {
            BlockMask::OneByOne => 3,
            BlockMask::TwoByOne => 1,
            BlockMask::OneByTwo => 2,
            BlockMask::TwoByTwo => 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Cell {
    pub x: u16,
    pub y: u16,
    pub treat_as_slope: TreatAsSlopeType,
    pub block_type: BlockType,
    pub flip: Flip,
    pub sprite: u8,
    pub palette: u8,
    pub unk: u8,
    pub bts: u8,
    pub slope_vectors: Vec<Vector>,
}

impl Cell {
    pub fn new() -> Self {
        Cell {
            x: 0,
            y: 0,
            treat_as_slope: TreatAsSlopeType::Solid,
            block_type: BlockType::Air,
            flip: Flip::None,
            sprite: 0,
            palette: 0,
            unk: 0,
            bts: 0,
            slope_vectors: Vec::new(),
        }
    }

    pub fn get_canvas_pos(&self) -> (u16, u16) {
        (self.x * CELL_SIZE, self.y * CELL_SIZE)
    }

    pub fn get_slope_flip(&self) -> Flip {
        match (self.bts & BTS_SLOPE_FLIP_MASK) >> 6 {
            0x0 => Flip::None,
            0x1 => Flip::Horizontal,
            0x2 => Flip::Vertical,
            0x3 => Flip::Both,
            _ => Flip::None,
        }
    }

    #[inline(always)]
    pub fn get_slope_type(&self) -> SlopeType {
        (self.bts & BTS_SLOPE_TYPE_MASK).into()
    }

    pub fn is_square(&self) -> bool {
        let slope_type = self.get_slope_type();
        self.block_type == BlockType::Solid
            || (self.block_type == BlockType::Slope
                && (slope_type == SlopeType::Square || slope_type == SlopeType::SquareDuplicate1))
    }

    pub fn get_breakable_mask(&self) -> BlockMask {
        match self.bts & BTS_BREAKABLE_MASK_MASK {
            0x0 => BlockMask::OneByOne,
            0x1 => BlockMask::TwoByOne,
            0x2 => BlockMask::OneByTwo,
            0x3 => BlockMask::TwoByTwo,
            _ => BlockMask::OneByOne,
        }
    }

    pub fn export_breakable_by_type(&self) -> u8 {
        match self.block_type {
            BlockType::Bomb => 1,
            BlockType::AirBomb => 1,
            BlockType::Shot => {
                if self.bts < 0x8 {
                    2
                } else if self.bts < 0xA {
                    4
                } else {
                    3
                }
            }
            BlockType::Crumble => {
                if self.bts < 0xE {
                    6
                } else {
                    5
                }
            }
            _ => 0,
        }
    }
}
