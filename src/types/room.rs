use std::{fs::File, io::Write, path};

use image::{ImageError, Rgba};
use imageproc::{drawing::draw_hollow_rect_mut, rect::Rect};
use num_enum::FromPrimitive;

use crate::{
    constants::{CELL_SIZE, TILE_SIZE},
    shapes::{
        point::Point,
        vector::{Vector, constants::SlopeVectors},
    },
    traits::Draw,
    types::{
        cell::{BlockType, Cell, Flip, SlopeType, TreatAsSlopeType},
        disjointed_set::VectorDisjointedSet,
    },
};

#[derive(Default, Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
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

#[derive(Default)]
#[allow(dead_code)]
pub(crate) struct Room {
    pub(crate) room_id: u32,
    area_index: AreaIndex,
    room_index: u8,
    map_x: u8,
    map_y: u8,
    room_width: u8,
    room_height: u8,
    up_scroll: u8,
    down_scroll: u8,
    special_graphics_bitflag: u8,
    pub(crate) door_out_pointer: u16,
    unk3: u8,
    unk4: u8,
    unk5: u8,
    unk6: u8,
    pub(crate) cells: Vec<Cell>,
}

impl Room {
    pub(crate) fn from_bytes(room_id: u32, total_size: usize, bytes: &[u8]) -> Self {
        // separate header at offset 0x00 and length 0x0E and room data at offset 0x0E non-inclusive
        let header = &bytes[0x00..=0x0E];
        let mut room = Room::new_from_bytes(room_id, header);

        // get room width and height
        let room_width = room.get_width_cells();

        // [header][room_data][room_type_data][room_bts_data][unk_data]
        let raw_data = &bytes[0x0F..];
        let room_type_data = &raw_data[0x00..total_size];
        let room_bts_data = &raw_data[(total_size)..(total_size) + total_size / 2];

        // get block type, flip, and that's it for now
        for (i, byte_pair) in room_type_data.chunks_exact(2).enumerate() {
            let room_width = room_width as usize;

            // byte_pair[1] 0000        00      00
            //              ^block type ^flip   ^??
            let mut room_cell = Cell::new();
            room_cell.block_type = BlockType::from((byte_pair[1] & 0b11110000) >> 4);
            room_cell.flip = Flip::from((byte_pair[1] & 0b1100) >> 2);
            room_cell.extra = byte_pair[0];

            // get x and y
            room_cell.x = (i % room_width) as u16;
            room_cell.y = (i / room_width) as u16;

            room.cells.push(room_cell);
        }

        // get bts
        for (i, byte) in room_bts_data.iter().enumerate() {
            room.cells[i].bts = *byte;
        }

        room.set_treat_slope();

        room
    }

    fn new_from_bytes(room_id: u32, bytes: &[u8]) -> Self {
        Room {
            room_id,
            area_index: bytes[0x0].into(),
            room_index: bytes[0x1],
            map_x: bytes[0x2],
            map_y: bytes[0x3],
            room_width: bytes[0x4],
            room_height: bytes[0x5],
            up_scroll: bytes[0x6],
            down_scroll: bytes[0x7],
            special_graphics_bitflag: bytes[0x8],
            door_out_pointer: u16::from_le_bytes([bytes[0x9], bytes[0xA]]),
            unk3: bytes[0x0B],
            unk4: bytes[0x0C],
            unk5: bytes[0x0D],
            unk6: bytes[0x0E],
            cells: Vec::new(),
        }
    }

    pub(crate) fn get_width_cells(&self) -> u16 {
        (self.room_width * TILE_SIZE as u8) as u16
    }

    pub(crate) fn get_height_cells(&self) -> u16 {
        (self.room_height * TILE_SIZE as u8) as u16
    }

    fn set_treat_slope(&mut self) {
        let room_width = self.get_width_cells() as usize;
        let room_height = self.get_height_cells() as usize;

        for i in 0..(room_width * room_height) {
            if self.cells[i].block_type != BlockType::Slope
                || self.cells[i].get_slope_type() == SlopeType::Square
                || self.cells[i].treat_as_slope != TreatAsSlopeType::Solid
            {
                continue;
            }

            let slope_flip = self.cells[i].get_slope_flip();
            if slope_flip == Flip::None || slope_flip == Flip::Both {
                self.cells[i].treat_as_slope = TreatAsSlopeType::SlopeLeft;
            } else {
                self.cells[i].treat_as_slope = TreatAsSlopeType::SlopeRight;
            }

            let mut neighbors = get_4neighbors(i, room_width, room_height);

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

            let Some(right) = neighbors.right else {
                continue;
            };
            if !self.cells[right].is_square() {
                continue;
            }

            self.cells[right].treat_as_slope = TreatAsSlopeType::Solid;
            let right_neighbors = get_4neighbors(right, room_width, room_height);
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
                    if self.cells[l].block_type == BlockType::Slope
                        && self.cells[r].block_type == BlockType::Slope
                    {
                        self.cells[right].treat_as_slope = TreatAsSlopeType::SlopeLeft;
                    }

                    if l == i {
                        if self.cells[r].block_type == BlockType::Air {
                            self.cells[right].treat_as_slope = TreatAsSlopeType::SlopeProtectPosX;
                        } else {
                            self.cells[right].treat_as_slope = TreatAsSlopeType::SlopeRight;
                        }
                    }

                    if r == i {
                        if self.cells[l].block_type == BlockType::Air {
                            self.cells[right].treat_as_slope = TreatAsSlopeType::SlopeProtectNegX;
                        } else {
                            self.cells[right].treat_as_slope = TreatAsSlopeType::SlopeLeft;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn save_slopes(&self) -> Result<(), anyhow::Error> {
        let room_width = self.get_width_cells() as usize;
        let room_height = self.get_height_cells() as usize;

        let mut disjointed_set = VectorDisjointedSet::new(self.cells.len());

        let mut cells_sv: Vec<Vec<Vector>> = vec![Vec::new(); self.cells.len()];

        self.cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.block_type == BlockType::Slope)
            .for_each(|(i, cell)| {
                let slope_type = cell.get_slope_type();
                let slope_flip = cell.get_slope_flip();
                let vectors = SlopeVectors::from(slope_type);
                cells_sv[i].extend_from_slice(vectors);

                match slope_flip {
                    Flip::None => {}
                    Flip::Horizontal => {
                        for vector in &mut cells_sv[i] {
                            vector.start.x = (-vector.start.x) + CELL_SIZE as i32;
                            vector.end.x = (-vector.end.x) + CELL_SIZE as i32;
                        }
                    }
                    Flip::Vertical => {
                        for vector in &mut cells_sv[i] {
                            vector.start.y = (-vector.start.y) + CELL_SIZE as i32;
                            vector.end.y = (-vector.end.y) + CELL_SIZE as i32;
                        }
                    }
                    Flip::Both => {
                        for vector in &mut cells_sv[i] {
                            vector.start.x = (-vector.start.x) + CELL_SIZE as i32;
                            vector.end.x = (-vector.end.x) + CELL_SIZE as i32;
                            vector.start.y = (-vector.start.y) + CELL_SIZE as i32;
                            vector.end.y = (-vector.end.y) + CELL_SIZE as i32;
                        }
                    }
                }

                for j in 0..cells_sv[i].len() {
                    cells_sv[i][j].start.x += cell.x as i32 * CELL_SIZE as i32;
                    cells_sv[i][j].start.y += cell.y as i32 * CELL_SIZE as i32;
                    cells_sv[i][j].end.x += cell.x as i32 * CELL_SIZE as i32;
                    cells_sv[i][j].end.y += cell.y as i32 * CELL_SIZE as i32;
                }
            });

        for cell_i in 0..self.cells.len() {
            if self.cells[cell_i].block_type != BlockType::Slope {
                continue;
            }

            for vector_i in 0..cells_sv[cell_i].len() {
                let cell_x = self.cells[cell_i].x as usize;
                let cell_y = self.cells[cell_i].y as usize;

                let mut cell_parent;
                if !disjointed_set.exists(cell_i, vector_i) {
                    disjointed_set.init(cell_i, vector_i);
                }

                let offsets = [
                    (1, 1),  // x + 1, y + 1
                    (1, 0),  // x + 1, y
                    (0, 1),  // x, y + 1
                    (-1, 1), // x - 1, y + 1
                ];

                for offset in offsets {
                    let test_x = cell_x as i32 + offset.0;
                    let test_y = cell_y as i32 + offset.1;

                    if test_x < 0
                        || test_x >= room_width as i32
                        || test_y < 0
                        || test_y >= room_height as i32
                    {
                        continue;
                    }

                    let test_index = (test_x + test_y * room_width as i32) as usize;
                    if self.cells[test_index].block_type != BlockType::Slope {
                        continue;
                    }

                    cell_parent = disjointed_set.find(cell_i, vector_i);

                    for k in 0..cells_sv[test_index].len() {
                        if !disjointed_set.exists(test_index, k) {
                            disjointed_set.init(test_index, k);
                        }

                        let testp = disjointed_set.find(test_index, k);

                        let cellv = cells_sv[cell_parent.cell][cell_parent.vector];
                        let testv = cells_sv[testp.cell][testp.vector];

                        if !cellv.compare_direction(&testv) {
                            continue;
                        }
                        let ends_meet = cellv.end_points_meet(&testv);

                        let mut new_parent = cell_parent;
                        if ends_meet != 0 {
                            new_parent = disjointed_set.union(cell_i, vector_i, test_index, k);
                        }
                        if ends_meet == -1 {
                            if new_parent == cell_parent {
                                cells_sv[cell_parent.cell][cell_parent.vector].end = testv.end;
                            } else {
                                cells_sv[new_parent.cell][new_parent.vector].start = cellv.start;
                            }
                        }
                        if ends_meet == 1 {
                            if new_parent == cell_parent {
                                cells_sv[cell_parent.cell][cell_parent.vector].start = testv.start;
                            } else {
                                cells_sv[new_parent.cell][new_parent.vector].end = cellv.end;
                            }
                        }
                    }
                }
            }
        }

        let slope_info = self
            .cells
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                let parent0 = disjointed_set.find(*i, 0);
                let parent1 = disjointed_set.find(*i, 1);
                (parent0.cell == *i || parent1.cell == *i)
                    && self.cells[*i].block_type == BlockType::Slope
            })
            .collect::<Vec<_>>();

        if slope_info.is_empty() {
            return Ok(());
        }

        slope_info
            .iter()
            .filter(|(_, s)| s.get_slope_type() == SlopeType::Slope45)
            .filter(|(i, s)| {
                let offset = match s.get_slope_flip() {
                    Flip::None => (-1, 1),
                    Flip::Horizontal => (1, 1),
                    Flip::Vertical => (-1, -1),
                    Flip::Both => (1, -1),
                };

                let test_index = (self.cells[*i].x as i32
                    + offset.0
                    + (self.cells[*i].y as i32 + offset.1) * room_width as i32)
                    as usize;
                self.cells[test_index].block_type == BlockType::Air
                    || self.cells[test_index].block_type == BlockType::Bomb
                    || self.cells[test_index].block_type == BlockType::Crumble
                    || self.cells[test_index].block_type == BlockType::Shot
            })
            .for_each(|(i, s)| {
                for vector in cells_sv[*i].iter_mut() {
                    let offset = match s.get_slope_flip() {
                        Flip::None | Flip::Vertical => Point::new(2, 0),
                        Flip::Horizontal | Flip::Both => Point::new(-2, 0),
                    };

                    vector.start += offset;
                    vector.end += offset;
                }
            });

        let slope_not_vflip = slope_info
            .iter()
            .filter(|i| {
                i.1.get_slope_flip() == Flip::None || i.1.get_slope_flip() == Flip::Horizontal
            })
            .map(|(i, _)| *i)
            .collect::<Vec<_>>();

        let slope_vflip = slope_info
            .iter()
            .filter(|i| {
                i.1.get_slope_flip() == Flip::Vertical || i.1.get_slope_flip() == Flip::Both
            })
            .map(|(i, _)| *i)
            .collect::<Vec<_>>();

        let slope_filename = format!("output/{:X}/{:X}_slopes.txt", self.room_id, self.room_id);
        let slope_path = path::PathBuf::from(slope_filename.clone());

        // create folders if they don't exist
        if !slope_path.parent().unwrap().exists() {
            std::fs::create_dir_all(slope_path.parent().unwrap()).unwrap();
        }

        let mut file = File::create(slope_path).inspect_err(|e| {
            eprintln!("{e}: Failed to create file {slope_filename}");
        })?;

        for (slopes, label) in &[
            (slope_not_vflip, "// Ground slopes"),
            (slope_vflip, "// Vertical slopes"),
        ] {
            if !slopes.is_empty() {
                writeln!(file, "{}", label).inspect_err(|e| {
                    eprintln!("{e}: Failed to write label to file {slope_filename}");
                })?;
            }

            let vflip_str = if *label == "// Vertical slopes" {
                ", 0, 1, 1"
            } else {
                ""
            };

            for &i in slopes {
                for vector in &cells_sv[i] {
                    let start: crate::shapes::point::Point<i32> = vector.start;
                    let end = vector.end;

                    writeln!(
                        file,
                        "spawn_slope({}*2, {}*2, {}*2, {}*2{})",
                        start.x, start.y, end.x, end.y, vflip_str
                    )
                    .unwrap_or_else(|_| {
                        panic!(
                            "Failed to write slope to file output/{}/{}_slopes.txt",
                            self.room_id, self.room_id
                        )
                    });
                }
            }
        }

        Ok(())
    }
    pub(crate) fn save_image(&self) -> Result<(), ImageError> {
        // make a new image that is the size of the room
        let room_width = self.get_width_cells() as usize;
        let room_height = self.get_height_cells() as usize;
        let mut img = image::RgbaImage::new(
            (room_width * CELL_SIZE as usize) as u32,
            (room_height * CELL_SIZE as usize) as u32,
        );

        // draw the solid and slope cells
        self.cells
            .iter()
            .filter(|cell| {
                // draw solid and slope cells
                cell.block_type == BlockType::Solid || (cell.block_type == BlockType::Slope)
            })
            .for_each(|cell| {
                // draw the cell
                cell.draw_to_img(&mut img);
            });

        // remove every yellow and magenta pixel which has one transparent horizontal neighboring pixel
        // repeat 10 times because I don't know a better way to do it
        for _ in 0..10 {
            let img_copy = img.clone();
            for y in 0..img.height() {
                for x in 0..img.width() {
                    if x == 0 || x == 16 * room_width as u32 - 1 {
                        // ignore the first and last columns
                        continue;
                    }
                    if y == 0 || y == 16 * room_height as u32 - 1 {
                        // ignore the first and last rows
                        continue;
                    }

                    let pixel = img_copy.get_pixel(x, y);

                    if pixel.0 == [0, 255, 0, 255] {
                        // ignore yellow pixels
                        continue;
                    }

                    let left_pixel = img_copy.get_pixel(x - 1, y);
                    let right_pixel = img_copy.get_pixel(x + 1, y);
                    let up_pixel = img_copy.get_pixel(x, y - 1);
                    let down_pixel = img_copy.get_pixel(x, y + 1);

                    if (pixel.0 != left_pixel.0 || pixel.0 != right_pixel.0)
                        && (left_pixel.0 != [0, 255, 0, 255] || right_pixel.0 != [0, 255, 0, 255])
                    {
                        // if the pixel is not yellow and has a yellow or magenta neighbor, remove it
                        if up_pixel[3] == 0 && down_pixel.0 == [0, 255, 0, 255] {
                            img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                        }

                        if down_pixel[3] == 0 && up_pixel.0 == [0, 255, 0, 255] {
                            img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                        }
                    }

                    // if the pixel has a transparent neighbor, remove it
                    if left_pixel[3] == 0 || right_pixel[3] == 0 {
                        img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                    }
                }
            }
        }

        // draw the room outline
        draw_hollow_rect_mut(
            &mut img,
            Rect::at(0, 0).of_size(16 * room_width as u32, 16 * room_height as u32),
            Rgba([0, 255, 0, 255]),
        );

        // make the room folder if it doesn't exist
        let path = std::path::Path::new("./output");
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }

        let room_hex = format!("{:X}", self.room_id);
        let path = path.join(&room_hex);
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }

        // save the image
        img.save(format!(
            "./output/{:X}/{:X}_col.png",
            self.room_id, self.room_id
        ))?;

        Ok(())
    }

    pub(crate) fn save_breakables(&self) {
        let room_width = self.get_width_cells() as usize;
        let room_height = self.get_height_cells() as usize;

        let breakables = self
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| {
                let shot_not_door = cell.block_type == BlockType::Shot && !cell.is_blue_door_cap();

                let mt = matches!(
                    cell.block_type,
                    BlockType::Bomb
                        | BlockType::AirBomb
                        | BlockType::Shot
                        | BlockType::AirShot
                        | BlockType::Crumble
                );
                mt && shot_not_door
            })
            .collect::<Vec<_>>();

        let breakable_filename = format!(
            "./output/{:X}/{:X}_breakables.txt",
            self.room_id, self.room_id
        );
        let breakable_path = path::PathBuf::from(breakable_filename);

        // create folders if they don't exist
        if !breakable_path.parent().unwrap().exists() {
            std::fs::create_dir_all(&breakable_path.parent().unwrap()).unwrap();
        }

        if !breakables.is_empty() {
            let mut file = File::create(breakable_path)
                .unwrap_or_else(|e| panic!("Failed to create file {}", e));

            for &(breakable_index, _) in breakables.iter() {
                if self.cells[breakable_index].block_type == BlockType::Shot {
                    let breakable_neighbors =
                        get_4neighbors(breakable_index, room_width, room_height);
                    if breakable_neighbors.has_block_type(&self.cells, BlockType::Door) {
                        continue;
                    }
                }

                let pos = self.cells[breakable_index].get_canvas_pos();
                // let pos = (pos.0 - self.export_rect.x, pos.1 - self.export_rect.y);
                let breakable_mask = self.cells[breakable_index].get_breakable_mask();
                let breakable_by_type = self.cells[breakable_index].export_breakable_by_type();

                file.write_all(
                    format!(
                        "spawn_breakable({}*2, {}*2, {}, \"sprite_here\", 0, {})\n",
                        pos.0,
                        pos.1,
                        breakable_mask.export(),
                        breakable_by_type
                    )
                    .as_bytes(),
                )
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to write to file output/{}/{}_breakables.txt",
                        self.room_id, self.room_id
                    )
                })
            }
        }
    }

    pub(crate) fn get_cell(&self, x: u16, y: u16) -> &Cell {
        &self.cells[(y * self.get_width_cells() + x) as usize]
    }
}

pub(crate) struct CellNeighbors {
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
    pub(crate) up: Option<usize>,
    pub(crate) down: Option<usize>,
}

impl CellNeighbors {
    fn has_block_type(&self, cells: &[Cell], block_type: BlockType) -> bool {
        let mut has_block = false;

        if let Some(left_index) = self.left {
            has_block |= cells[left_index].block_type == block_type;
        }

        if let Some(right_index) = self.right {
            has_block |= cells[right_index].block_type == block_type;
        }

        if let Some(up_index) = self.up {
            has_block |= cells[up_index].block_type == block_type;
        }

        if let Some(down_index) = self.down {
            has_block |= cells[down_index].block_type == block_type;
        }

        has_block
    }
}

pub(crate) fn get_4neighbors(index: usize, room_width: usize, room_height: usize) -> CellNeighbors {
    let x = index % room_width;
    let y = index / room_width;

    CellNeighbors {
        left: if x > 0 { Some(index - 1) } else { None },
        right: if x < room_width - 1 {
            Some(index + 1)
        } else {
            None
        },
        up: if y > 0 {
            Some(index - room_width)
        } else {
            None
        },
        down: if y < room_height - 1 {
            Some(index + room_width)
        } else {
            None
        },
    }
}
