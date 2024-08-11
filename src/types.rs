use rand::Rng;

use image::Rgba;
use imageproc::{
    drawing::{
        draw_filled_rect_mut, draw_hollow_rect_mut, draw_line_segment_mut, draw_polygon_mut,
    },
    rect::Rect,
};

use crate::{
    constants::{BTS_SLOPE_FLIP_MASK, BTS_SLOPE_TYPE_MASK, CELL_SIZE},
    shapes::{
        vectors::{SlopeVectors, Vector},
        Polygon,
    },
};

#[derive(Default, Debug, PartialEq)]
enum AreaIndex {
    #[default]
    Crateria = 0x0,
    Brinstar,
    Norfair,
    WreckedShip,
    Maridia,
    Tourian,
    Colony,
    Debug,
}

impl From<u8> for AreaIndex {
    fn from(value: u8) -> Self {
        match value {
            0x0 => AreaIndex::Crateria,
            0x1 => AreaIndex::Brinstar,
            0x2 => AreaIndex::Norfair,
            0x3 => AreaIndex::WreckedShip,
            0x4 => AreaIndex::Maridia,
            0x5 => AreaIndex::Tourian,
            0x6 => AreaIndex::Colony,
            0x7 => AreaIndex::Debug,
            _ => AreaIndex::Crateria,
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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BlockType {
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

impl From<u8> for BlockType {
    fn from(value: u8) -> Self {
        match value {
            0x0 => BlockType::Air,
            0x1 => BlockType::Slope,
            0x2 => BlockType::AirXray,
            0x3 => BlockType::Treadmill,
            0x4 => BlockType::AirShot,
            0x5 => BlockType::HCopy,
            0x6 => BlockType::Unused,
            0x7 => BlockType::AirBomb,
            0x8 => BlockType::Solid,
            0x9 => BlockType::Door,
            0xA => BlockType::Spike,
            0xB => BlockType::Crumble,
            0xC => BlockType::Shot,
            0xD => BlockType::VCopy,
            0xE => BlockType::Grapple,
            0xF => BlockType::Bomb,
            _ => BlockType::Air,
        }
    }
}

fn get_random_color() -> Rgba<u8> {
    Rgba([
        rand::thread_rng().gen_range(0..=255),
        rand::thread_rng().gen_range(0..=255),
        rand::thread_rng().gen_range(0..=255),
        255,
    ])
}

#[derive(Default)]
pub struct Room {
    pub room_id: String,
    area_index: AreaIndex,
    room_index: u8,
    map_x: u8,
    map_y: u8,
    room_width: u8,
    room_height: u8,
    up_scroll: u8,
    down_scroll: u8,
    special_graphics_bitflag: u8,
    door_out_pointer: u8,
    unk3: u8,
    unk4: u8,
    unk5: u8,
    unk6: u8,
    unk7: u8,
    pub cells: Vec<Cell>,
    pub slope_vectors: Vec<VisualDataState>,
    pub slope_union_sets: Vec<usize>,
}

struct VisualDataState {
    treat_as_slope: bool,
}

impl Room {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        // separate header at offset 0x00 and length 0x0E and room data at offset 0x0E non-inclusive
        let header = &bytes[0x00..=0x0E];
        let mut room = Room::new_from_bytes(header);

        // get room width and height
        let room_width = room.get_room_width_tiles();
        let room_height = room.get_room_height_tiles();
        let total_size = (room_width * room_height) as usize;

        // [header][room_data][room_type_data][room_bts_data][unk_data]
        let raw_data = &bytes[0x0F..];
        let room_type_data = &raw_data[0x00..total_size * 2];
        let room_bts_data = &raw_data[(total_size * 2)..];

        println!("room width: {}, room height: {}", room_width, room_height);

        // get block type, flip, and that's it for now
        for (i, byte_pair) in room_type_data.chunks_exact(2).enumerate() {
            let room_width = room_width as usize;
            let room_height = room_height as usize;

            if i == (room_width * room_height) {
                break;
            }

            // byte_pair[1] 0000        00      00
            //              ^block type ^flip   ^??
            let mut room_cell = Cell::new();
            room_cell.block_type = BlockType::from((byte_pair[1] & 0b11110000) >> 4);
            room_cell.flip = Flip::from((byte_pair[1] & 0b1100) >> 2);

            // get x and y
            room_cell.x = (i % room_width) as u16;
            room_cell.y = (i / room_width) as u16;

            room.cells.push(room_cell);
        }

        // get bts
        for (i, byte) in room_bts_data.iter().enumerate() {
            if i == total_size {
                break;
            }

            room.cells[i].bts = *byte;

            if room.cells[i].block_type == BlockType::Slope {
                let slope_type = room.cells[i].get_slope_type();
                let vectors = SlopeVectors::from(slope_type);
                room.cells[i].slope_vectors.extend_from_slice(vectors);
            }
        }

        room.set_data_visual();

        room
    }

    fn new_from_bytes(bytes: &[u8]) -> Self {
        Room {
            room_id: String::new(),
            area_index: bytes[0x0].into(),
            room_index: bytes[0x1],
            map_x: bytes[0x2],
            map_y: bytes[0x3],
            room_width: bytes[0x4],
            room_height: bytes[0x5],
            up_scroll: bytes[0x6],
            down_scroll: bytes[0x7],
            special_graphics_bitflag: bytes[0x8],
            door_out_pointer: bytes[0x9],
            unk3: bytes[0x0A],
            unk4: bytes[0x0B],
            unk5: bytes[0x0C],
            unk6: bytes[0x0D],
            unk7: bytes[0x0E],
            cells: Vec::new(),
            slope_vectors: Vec::new(),
            slope_union_sets: Vec::new(),
        }
    }

    pub fn get_room_width_tiles(&self) -> u16 {
        (self.room_width * CELL_SIZE as u8).into()
    }

    pub fn get_room_height_tiles(&self) -> u16 {
        (self.room_height * CELL_SIZE as u8).into()
    }

    fn set_data_visual(&mut self) -> SlopeType {
        let room_width = self.get_room_width_tiles() as usize;
        let room_height = self.get_room_height_tiles() as usize;

        for i in 0..(room_width * room_height) {
            if self.cells[i].block_type != BlockType::Slope
                || self.cells[i].get_slope_type() == SlopeType::Square
            {
                continue;
            }

            // (self.cells[i].block_type == BlockType::Slope && self.cells[i].get_slope_type() != SlopeType::Square)
            let slope_flip = self.cells[i].get_slope_flip();
            if slope_flip == Flip::None || slope_flip == Flip::Both {
                self.cells[i].treat_as_slope = TreatAsSlopeType::SlopeLeft;
            } else {
                self.cells[i].treat_as_slope = TreatAsSlopeType::SlopeRight;
            }

            let mut neighbors = get_neighbors(i, room_width, room_height);

            match slope_flip {
                Flip::None => {}
                Flip::Horizontal => std::mem::swap(&mut neighbors.left, &mut neighbors.right),
                Flip::Vertical => {
                    std::mem::swap(&mut neighbors.up, &mut neighbors.down);
                }
                Flip::Both => {
                    std::mem::swap(&mut neighbors.left, &mut neighbors.right);
                    std::mem::swap(&mut neighbors.up, &mut neighbors.down);
                }
            }

            if let Some(right) = neighbors.right {
                if self.cells[right].is_square() {
                    self.cells[right].treat_as_slope = TreatAsSlopeType::SlopeLeft;
                    let right_neighbors = get_neighbors(right, room_width, room_height);
                    match right_neighbors {
                        CellNeighbors {
                            left: _,
                            right: _,
                            up: Some(u),
                            down: Some(d),
                        } if self.cells[u].is_square() && self.cells[d].is_square() => {
                            self.cells[right].treat_as_slope = TreatAsSlopeType::Solid;
                        }
                        CellNeighbors {
                            left: Some(l),
                            right: Some(r),
                            up: _,
                            down: _,
                        } => {
                            if l == i {
                                self.cells[right].treat_as_slope =
                                    TreatAsSlopeType::SlopeProtectPosX;
                            } else if r == i {
                                self.cells[right].treat_as_slope =
                                    TreatAsSlopeType::SlopeProtectNegX;
                            } else {
                                self.cells[right].treat_as_slope = TreatAsSlopeType::Solid;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        SlopeType::None
    }

    pub fn save_image(self) {
        let room_width = self.get_room_width_tiles() as usize;
        let room_height = self.get_room_height_tiles() as usize;
        let mut img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(
            16 * room_width as u32,
            16 * room_height as u32,
        );

        // get slope vectors and draw a line
        self.cells
            .iter()
            .filter(|cell| {
                (cell.block_type == BlockType::Solid || (cell.block_type == BlockType::Slope))
            })
            .for_each(|cell| {
                let color = match cell.treat_as_slope {
                    TreatAsSlopeType::Solid => Rgba([0, 255, 0, 255]),
                    TreatAsSlopeType::SlopeLeft => Rgba([255, 255, 0, 255]),
                    TreatAsSlopeType::SlopeRight => Rgba([255, 0, 255, 255]),
                    TreatAsSlopeType::SlopeProtectNegX => Rgba([255, 255, 0, 255]),
                    TreatAsSlopeType::SlopeProtectPosX => Rgba([255, 0, 255, 255]),
                    _ => get_random_color(),
                };

                match cell.block_type {
                    BlockType::Slope => {
                        let slope_type = cell.get_slope_type();
                        let slope_flip = cell.get_slope_flip();

                        let mut shape = Polygon::from(slope_type);

                        match slope_flip {
                            Flip::None => {}
                            Flip::Horizontal => {
                                shape.mirror_x();
                            }
                            Flip::Vertical => {
                                shape.mirror_y();
                            }
                            Flip::Both => {
                                shape.mirror_x();
                                shape.mirror_y();
                            }
                        }

                        // add the shape into position
                        shape.translate(
                            cell.x as f32 * CELL_SIZE as f32,
                            cell.y as f32 * CELL_SIZE as f32,
                        );

                        draw_polygon_mut(&mut img, &shape.points, color);
                    }
                    BlockType::Solid => match cell.treat_as_slope {
                        TreatAsSlopeType::Solid => {
                            draw_filled_rect_mut(
                                &mut img,
                                Rect::at((cell.x * CELL_SIZE).into(), (cell.y * CELL_SIZE).into())
                                    .of_size(CELL_SIZE.into(), CELL_SIZE.into()),
                                color,
                            );
                        }
                        TreatAsSlopeType::SlopeRight | TreatAsSlopeType::SlopeLeft => {}
                        TreatAsSlopeType::SlopeProtectNegX => {
                            draw_filled_rect_mut(
                                &mut img,
                                Rect::at((cell.x * CELL_SIZE).into(), (cell.y * CELL_SIZE).into())
                                    .of_size(CELL_SIZE.into(), CELL_SIZE.into()),
                                color,
                            );
                            let start = ((cell.x * CELL_SIZE).into(), (cell.y * CELL_SIZE).into());
                            let end = (
                                (cell.x * CELL_SIZE).into(),
                                (cell.y * CELL_SIZE + CELL_SIZE - 1).into(),
                            );
                            draw_line_segment_mut(&mut img, start, end, Rgba([0, 255, 0, 255]));
                        }
                        TreatAsSlopeType::SlopeProtectPosX => {
                            draw_filled_rect_mut(
                                &mut img,
                                Rect::at((cell.x * CELL_SIZE).into(), (cell.y * CELL_SIZE).into())
                                    .of_size(CELL_SIZE.into(), CELL_SIZE.into()),
                                color,
                            );
                            let start = (
                                (cell.x * CELL_SIZE + CELL_SIZE - 1).into(),
                                (cell.y * CELL_SIZE).into(),
                            );
                            let end = (
                                (cell.x * CELL_SIZE + CELL_SIZE - 1).into(),
                                (cell.y * CELL_SIZE + CELL_SIZE - 1).into(),
                            );
                            draw_line_segment_mut(&mut img, start, end, Rgba([0, 255, 0, 255]));
                        }
                    },
                    _ => {}
                }
            });

        // remove every red pixel which has one transparent horizontal neighboring pixel, repeat 10 times
        for _ in 0..10 {
            let img_copy = img.clone();
            for y in 0..img.height() {
                for x in 0..img.width() {
                    if x == 0 || x == 16 * room_width as u32 - 1 {
                        continue;
                    }
                    if y == 0 || y == 16 * room_height as u32 - 1 {
                        continue;
                    }

                    let pixel = img_copy.get_pixel(x, y);

                    if pixel.0 == [0, 255, 0, 255] {
                        continue;
                    }

                    let left_pixel = img_copy.get_pixel(x - 1, y);
                    let right_pixel = img_copy.get_pixel(x + 1, y);
                    let up_pixel = img_copy.get_pixel(x, y - 1);
                    let down_pixel = img_copy.get_pixel(x, y + 1);

                    if (pixel.0 != left_pixel.0 || pixel.0 != right_pixel.0)
                        && (left_pixel.0 != [0, 255, 0, 255] || right_pixel.0 != [0, 255, 0, 255])
                    {
                        if up_pixel[3] == 0 && down_pixel.0 == [0, 255, 0, 255] {
                            img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                        }

                        if down_pixel[3] == 0 && up_pixel.0 == [0, 255, 0, 255] {
                            img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                        }
                    }

                    if left_pixel[3] == 0 || right_pixel[3] == 0 {
                        img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                    }
                }
            }
        }
        // draw room outline on borders
        draw_hollow_rect_mut(
            &mut img,
            Rect::at(0, 0).of_size(16 * room_width as u32, 16 * room_height as u32),
            Rgba([0, 255, 0, 255]),
        );

        // check if the folder exists
        let my_path = std::path::Path::new("./output");
        if !my_path.exists() {
            std::fs::create_dir(my_path).unwrap();
        }

        // save image
        img.save(format!("./output/{}.png", self.room_id)).unwrap();
    }
}

pub fn get_neighbors(index: usize, room_width: usize, room_height: usize) -> CellNeighbors {
    let mut neighbors: CellNeighbors = CellNeighbors {
        left: None,
        right: None,
        up: None,
        down: None,
    };

    let x = index % room_width;
    let y = index / room_width;

    // check left
    if x > 0 {
        neighbors.left = Some(index - 1);
    }

    // check right
    if x < room_width - 1 {
        neighbors.right = Some(index + 1);
    }

    // check up
    if y > 0 {
        neighbors.up = Some(index - room_width);
    }

    // check down
    if y < room_height - 1 {
        neighbors.down = Some(index + room_width);
    }

    neighbors
}

pub fn sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

#[derive(Debug, PartialEq)]
pub struct Cell {
    x: u16,
    y: u16,
    treat_as_slope: TreatAsSlopeType,
    pub block_type: BlockType,
    flip: Flip,
    sprite: u8,
    palette: u8,
    unk: u8,
    bts: u8,
    slope_vectors: Vec<Vector>,
}

impl Cell {
    fn new() -> Self {
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

    pub fn get_slope_flip(&self) -> Flip {
        match (self.bts & BTS_SLOPE_FLIP_MASK) >> 6 {
            0x0 => Flip::None,
            0x1 => Flip::Horizontal,
            0x2 => Flip::Vertical,
            0x3 => Flip::Both,
            _ => Flip::None,
        }
    }

    pub fn get_slope_type(&self) -> SlopeType {
        (self.bts & BTS_SLOPE_TYPE_MASK).into()
    }

    pub fn is_square(&self) -> bool {
        let slope_type = self.get_slope_type();
        self.block_type == BlockType::Solid
            || (self.block_type == BlockType::Slope
                && (slope_type == SlopeType::Square || slope_type == SlopeType::SquareDuplicate1))
    }
}

pub struct CellNeighbors {
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    pub down: Option<usize>,
}

impl CellNeighbors {}

#[derive(Debug, PartialEq, Copy, Clone)]
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
    None,
}

impl From<u8> for SlopeType {
    fn from(value: u8) -> Self {
        match value {
            0x0 => SlopeType::HalfSolidH,
            0x1 => SlopeType::HalfSolidV,
            0x2 => SlopeType::QuarterSolid,
            0x3 => SlopeType::StairBigSteps,
            0x4 => SlopeType::FullSolidUnused,
            0x5 => SlopeType::SmallTriangle,
            0x6 => SlopeType::BigTriangle,
            0x7 => SlopeType::HalfPlat,
            0x8 => SlopeType::SquareDuplicate1,
            0x9 => SlopeType::SquareDuplicate2,
            0xA => SlopeType::SquareDuplicate3,
            0xB => SlopeType::SquareDuplicate4,
            0xC => SlopeType::SquareDuplicate5,
            0xD => SlopeType::SquareDuplicate6,
            0xE => SlopeType::StairSmallSteps,
            0xF => SlopeType::ConcaveTriangle,
            0x10 => SlopeType::HorizontalLines,
            0x11 => SlopeType::VerticalLines,
            0x12 => SlopeType::Slope45,
            0x13 => SlopeType::Square,
            0x14 => SlopeType::HillPart1,
            0x15 => SlopeType::HillPart2,
            0x16 => SlopeType::SmoothHillPart1,
            0x17 => SlopeType::SmoothHillPart2,
            0x18 => SlopeType::SmootherHillPart1,
            0x19 => SlopeType::SmootherHillPart2,
            0x1A => SlopeType::SmootherHillPart3,
            0x1B => SlopeType::SteepHillPart1,
            0x1C => SlopeType::SteepHillPart2,
            0x1D => SlopeType::SteeperHillPart1,
            0x1E => SlopeType::SteeperHillPart2,
            0x1F => SlopeType::SteeperHillPart3,
            _ => SlopeType::None,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum TreatAsSlopeType {
    Solid = 0x0,
    SlopeRight,
    SlopeLeft,
    SlopeProtectNegX,
    SlopeProtectPosX,
}
