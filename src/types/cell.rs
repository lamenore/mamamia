use image::{Rgba, RgbaImage};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_line_segment_mut, draw_polygon_mut},
    rect::Rect,
};
use num_enum::FromPrimitive;

use crate::{
    constants::{BTS_BREAKABLE_MASK_MASK, BTS_SLOPE_FLIP_MASK, BTS_SLOPE_TYPE_MASK, CELL_SIZE},
    shapes::polygon::Polygon,
    traits::Draw,
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
    QuarterAir,
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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Flip {
    None = 0x0,
    Horizontal,
    Vertical,
    Both,
}

impl From<u8> for Flip {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Flip::None,
            0x1 => Flip::Horizontal,
            0x2 => Flip::Vertical,
            0x3 => Flip::Both,
            _ => Flip::None,
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
    pub extra: u8,
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
            extra: 0,
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
        let mask = match self.bts & BTS_BREAKABLE_MASK_MASK {
            0x0 => BlockMask::OneByOne,
            0x1 => BlockMask::TwoByOne,
            0x2 => BlockMask::OneByTwo,
            0x3 => BlockMask::TwoByTwo,
            _ => BlockMask::OneByOne,
        };

        match self.block_type {
            BlockType::Shot | BlockType::Crumble => {
                let limit = if self.block_type == BlockType::Shot {
                    0x8
                } else {
                    0xE
                };
                if self.bts < limit {
                    mask
                } else {
                    BlockMask::OneByOne
                }
            }
            BlockType::AirBomb | BlockType::AirShot | BlockType::Bomb => mask,
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

    pub fn is_blue_door_cap(&self) -> bool {
        self.block_type == BlockType::Shot && self.bts >= 0x40 && self.bts < 0x45
    }
}

impl Draw for Cell {
    fn draw_to_img(&self, img: &mut RgbaImage) {
        let color = match self.treat_as_slope {
            TreatAsSlopeType::Solid => Rgba([0, 255, 0, 255]),
            TreatAsSlopeType::SlopeLeft => Rgba([255, 255, 0, 255]),
            TreatAsSlopeType::SlopeRight => Rgba([255, 0, 255, 255]),
            TreatAsSlopeType::SlopeProtectNegX => Rgba([255, 255, 0, 255]),
            TreatAsSlopeType::SlopeProtectPosX => Rgba([255, 0, 255, 255]),
        };

        match self.block_type {
            BlockType::Slope => {
                let slope_type = self.get_slope_type();
                let slope_flip = self.get_slope_flip();

                let mut shape = Polygon::from(slope_type);
                shape.flip(slope_flip);

                // add the shape into position
                shape.translate(
                    self.x as f32 * CELL_SIZE as f32,
                    self.y as f32 * CELL_SIZE as f32,
                );

                let points = shape
                    .points
                    .iter()
                    .map(|p| imageproc::point::Point::new(p.x, p.y))
                    .collect::<Vec<imageproc::point::Point<i32>>>();

                draw_polygon_mut(img, &points, color);
            }
            BlockType::Solid => match self.treat_as_slope {
                TreatAsSlopeType::Solid
                | TreatAsSlopeType::SlopeRight
                | TreatAsSlopeType::SlopeLeft => {
                    draw_filled_rect_mut(
                        img,
                        Rect::at((self.x * CELL_SIZE).into(), (self.y * CELL_SIZE).into())
                            .of_size(CELL_SIZE.into(), CELL_SIZE.into()),
                        color,
                    );
                }
                TreatAsSlopeType::SlopeProtectNegX => {
                    draw_filled_rect_mut(
                        img,
                        Rect::at((self.x * CELL_SIZE).into(), (self.y * CELL_SIZE).into())
                            .of_size(CELL_SIZE.into(), CELL_SIZE.into()),
                        color,
                    );
                    let start = ((self.x * CELL_SIZE).into(), (self.y * CELL_SIZE).into());
                    let end = (
                        (self.x * CELL_SIZE).into(),
                        (self.y * CELL_SIZE + CELL_SIZE - 1).into(),
                    );
                    draw_line_segment_mut(img, start, end, Rgba([0, 255, 0, 255]));
                }
                TreatAsSlopeType::SlopeProtectPosX => {
                    draw_filled_rect_mut(
                        img,
                        Rect::at((self.x * CELL_SIZE).into(), (self.y * CELL_SIZE).into())
                            .of_size(CELL_SIZE.into(), CELL_SIZE.into()),
                        color,
                    );
                    let start = (
                        (self.x * CELL_SIZE + CELL_SIZE - 1).into(),
                        (self.y * CELL_SIZE).into(),
                    );
                    let end = (
                        (self.x * CELL_SIZE + CELL_SIZE - 1).into(),
                        (self.y * CELL_SIZE + CELL_SIZE - 1).into(),
                    );
                    draw_line_segment_mut(img, start, end, Rgba([0, 255, 0, 255]));
                }
            },
            _ => {}
        }
    }
}
