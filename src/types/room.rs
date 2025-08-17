use std::{fs::File, io::Write, path};

use image::{ImageError, Rgba};
use imageproc::{
    drawing::{
        draw_filled_rect_mut, draw_hollow_rect_mut, draw_line_segment_mut, draw_polygon_mut,
    },
    rect::Rect,
};
use num_enum::FromPrimitive;

use crate::{
    constants::{CELL_SIZE, TILE_SIZE},
    shapes::{polygon::Polygon, vector::constants::SlopeVectors},
    types::{
        cell::Flip,
        cell::{BlockType, Cell, SlopeType, TreatAsSlopeType},
        disjointed_set::DisjointedSet,
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
struct RoomExportRect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl RoomExportRect {
    fn new() -> Self {
        Default::default()
    }

    fn update(&mut self, x: u16, y: u16, width: u16, height: u16) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }
}

#[derive(Default, Clone, Debug)]
struct GrowingRect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl GrowingRect {
    fn new() -> Self {
        Default::default()
    }

    fn with_cell(cell: &Cell) -> Self {
        let mut growing_rect = GrowingRect::new();
        growing_rect.update(cell.x, cell.y, 1, 1);
        growing_rect
    }

    fn update(&mut self, x: u16, y: u16, width: u16, height: u16) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }

    /*************  ✨ Codeium Command 🌟  *************/
    fn merge(&self, nb_index: &GrowingRect) -> GrowingRect {
        let mut merged = self.clone();

        merged.x = merged.x.min(nb_index.x);
        merged.y = merged.y.min(nb_index.y);
        merged.width = (self.x + self.width).max(nb_index.x + nb_index.width) - merged.x;
        merged.height = (self.y + self.height).max(nb_index.y + nb_index.height) - merged.y;

        merged
    }
    /******  aee7c7d9-198c-4469-91c5-2a828366efec  *******/

    fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[derive(Default)]
#[allow(dead_code)]
pub struct Room {
    pub room_id: u32,
    area_index: AreaIndex,
    room_index: u8,
    map_x: u8,
    map_y: u8,
    room_width: u8,
    room_height: u8,
    up_scroll: u8,
    down_scroll: u8,
    special_graphics_bitflag: u8,
    pub door_out_pointer: u16,
    unk3: u8,
    unk4: u8,
    unk5: u8,
    unk6: u8,
    pub cells: Vec<Cell>,
    export_rect: RoomExportRect,
}

impl Room {
    pub fn from_bytes(room_id: u32, total_size: usize, bytes: &[u8]) -> Self {
        // separate header at offset 0x00 and length 0x0E and room data at offset 0x0E non-inclusive
        let header = &bytes[0x00..=0x0E];
        let mut room = Room::new_from_bytes(room_id, header);

        // get room width and height
        let room_width = room.get_width_cells();
        let room_height = room.get_height_cells();

        // [header][room_data][room_type_data][room_bts_data][unk_data]
        let raw_data = &bytes[0x0F..];
        let room_type_data = &raw_data[0x00..total_size];
        let room_bts_data = &raw_data[(total_size)..(total_size) + total_size/2];

        // get block type, flip, and that's it for now
        for (i, byte_pair) in room_type_data.chunks_exact(2).enumerate() {
            let room_width = room_width as usize;

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
        let mut j = 0;
        for (i, byte) in room_bts_data.iter().enumerate() {

            room.cells[i].bts = *byte;
            // 0x11FF
            if room.cells[i].block_type == BlockType::Slope {
                let slope_type = room.cells[i].get_slope_type();
                let slope_flip = room.cells[i].get_slope_flip();
                let vectors = SlopeVectors::from(slope_type);
                room.cells[i].slope_vectors.extend_from_slice(vectors);

                match slope_flip {
                    Flip::None => {}
                    Flip::Horizontal => {
                        for vector in &mut room.cells[i].slope_vectors {
                            vector.start.x = (-vector.start.x) + CELL_SIZE as i32;
                            vector.end.x = (-vector.end.x) + CELL_SIZE as i32;
                        }
                    }
                    Flip::Vertical => {
                        for vector in &mut room.cells[i].slope_vectors {
                            vector.start.y = (-vector.start.y) + CELL_SIZE as i32;
                            vector.end.y = (-vector.end.y) + CELL_SIZE as i32;
                        }
                    }
                    Flip::Both => {
                        for vector in &mut room.cells[i].slope_vectors {
                            vector.start.x = (-vector.start.x) + CELL_SIZE as i32;
                            vector.end.x = (-vector.end.x) + CELL_SIZE as i32;
                            vector.start.y = (-vector.start.y) + CELL_SIZE as i32;
                            vector.end.y = (-vector.end.y) + CELL_SIZE as i32;
                        }
                    }
                }

                for j in 0..room.cells[i].slope_vectors.len() {
                    room.cells[i].slope_vectors[j].start.x +=
                        room.cells[i].x as i32 * CELL_SIZE as i32;
                    room.cells[i].slope_vectors[j].start.y +=
                        room.cells[i].y as i32 * CELL_SIZE as i32;
                    room.cells[i].slope_vectors[j].end.x +=
                        room.cells[i].x as i32 * CELL_SIZE as i32;
                    room.cells[i].slope_vectors[j].end.y +=
                        room.cells[i].y as i32 * CELL_SIZE as i32;
                }
            }
        }

        room.set_treat_slope();
        room.crop_room_export_size();

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
            export_rect: RoomExportRect::new(),
        }
    }

    pub fn crop_room_export_size(&mut self) {
        let room_width = self.get_width_cells();
        let room_height = self.get_height_cells();

        let mut row_start = 0;
        // increase row_start if entire rows of cells that are only solid, top first
        for j in 0..room_height {
            if (0..room_width)
                .all(|i| self.cells[(i + j * room_width) as usize].block_type == BlockType::Solid)
            {
                row_start += 1;
            } else {
                if row_start != 0 {
                    row_start -= 1;
                }
                break;
            }
        }

        let mut row_end = room_height;
        // bottom first
        for j in (0..room_height).rev() {
            if (0..room_width)
                .all(|i| self.cells[(i + j * room_width) as usize].block_type == BlockType::Solid)
            {
                row_end -= 1;
            } else {
                if row_end != room_height {
                    row_end += 1;
                }
                break;
            }
        }

        let mut col_start = 0;
        // increase col_start if entire columns of cells that are only solid, left first
        for i in 0..room_width {
            if (0..room_height)
                .all(|j| self.cells[(i + j * room_width) as usize].block_type == BlockType::Solid)
            {
                col_start += 1;
            } else {
                if col_start != 0 {
                    col_start -= 1;
                }
                break;
            }
        }

        let mut col_end = room_width;
        // right first
        for i in (0..room_width).rev() {
            if (0..room_height)
                .all(|j| self.cells[(i + j * room_width) as usize].block_type == BlockType::Solid)
            {
                col_end -= 1;
            } else {
                if col_end != room_width {
                    col_end += 1;
                }
                break;
            }
        }

        let x = col_start * CELL_SIZE;
        let y = row_start * CELL_SIZE;
        let width = (col_end - col_start) * CELL_SIZE;
        let height = (row_end - row_start) * CELL_SIZE;

        self.export_rect.update(x, y, width, height);
    }

    pub fn get_width_cells(&self) -> u16 {
        (self.room_width * TILE_SIZE as u8) as u16
    }

    pub fn get_height_cells(&self) -> u16 {
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

    pub fn save_slopes(&self) -> Result<(), anyhow::Error> {
        let room_width = self.get_width_cells() as usize;
        let room_height = self.get_height_cells() as usize;

        let mut disjointed_set = VectorDisjointedSet::new(self.cells.len());
        let mut has_apparent_slope: bool = false;

        for cell_i in 0..self.cells.len() {
            if self.cells[cell_i].block_type != BlockType::Slope {
                continue;
            }

            for vector_i in 0..self.cells[cell_i].slope_vectors.len() {
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

                    for k in 0..self.cells[test_index].slope_vectors.len() {
                        if !disjointed_set.exists(test_index, k) {
                            disjointed_set.init(test_index, k);
                        }

                        let testp = disjointed_set.find(test_index, k);

                        let cellv = self.cells[cell_parent.cell].slope_vectors[cell_parent.vector];
                        let testv = self.cells[testp.cell].slope_vectors[testp.vector];

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
                                self.cells[cell_parent.cell].slope_vectors[cell_parent.vector]
                                    .end = testv.end;
                            } else {
                                self.cells[new_parent.cell].slope_vectors[new_parent.vector]
                                    .start = cellv.start;
                            }
                        }
                        if ends_meet == 1 {
                            if new_parent == cell_parent {
                                self.cells[cell_parent.cell].slope_vectors[cell_parent.vector]
                                    .start = testv.start;
                            } else {
                                self.cells[new_parent.cell].slope_vectors[new_parent.vector].end =
                                    cellv.end;
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

        if !slope_info.is_empty() {
            has_apparent_slope = true;
        }

        let slope_not_vflip = slope_info
            .iter()
            .filter(|i| i.1.get_slope_flip() != Flip::Vertical)
            .map(|(i, _)| *i)
            .collect::<Vec<_>>();

        let slope_vflip = slope_info
            .iter()
            .filter(|i| i.1.get_slope_flip() == Flip::Vertical)
            .map(|(i, _)| *i)
            .collect::<Vec<_>>();

        let slope_filename = format!("output/{}/{}_slopes.txt", self.room_id, self.room_id);
        let slope_path = path::Path::new(&slope_filename);
        if has_apparent_slope {
            let mut file = File::create(slope_path).unwrap_or_else(|_| {
                panic!(
                    "Failed to create file output/{}/{}_slopes.txt",
                    self.room_id, self.room_id
                )
            });

            for (slopes, label) in &[
                (slope_not_vflip, "// Ground slopes"),
                (slope_vflip, "// Vertical slopes"),
            ] {
                if !slopes.is_empty() {
                    writeln!(file, "{}", label).unwrap_or_else(|_| {
                        panic!(
                            "Failed to write slopes label to file output/{}/{}_slopes.txt",
                            self.room_id, self.room_id
                        )
                    });
                }

                let vflip_str = if *label == "// Vertical slopes" {
                    ", 0, 1, 1"
                } else {
                    ""
                };

                for &i in slopes {
                    for vector in &self.cells[i].slope_vectors {
                        let start = vector.start;
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
        } else if slope_path.exists() {
            std::fs::remove_file(slope_path).unwrap_or_else(|_| {
                panic!(
                    "Failed to delete file output/{}/{}_slopes.txt",
                    self.room_id, self.room_id
                )
            });
        }
    }
    pub fn save_image(&self) -> Result<(), ImageError> {
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
                let color = match cell.treat_as_slope {
                    TreatAsSlopeType::Solid => Rgba([0, 255, 0, 255]),
                    TreatAsSlopeType::SlopeLeft => Rgba([255, 255, 0, 255]),
                    TreatAsSlopeType::SlopeRight => Rgba([255, 0, 255, 255]),
                    TreatAsSlopeType::SlopeProtectNegX => Rgba([255, 255, 0, 255]),
                    TreatAsSlopeType::SlopeProtectPosX => Rgba([255, 0, 255, 255]),
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
                        TreatAsSlopeType::Solid
                        | TreatAsSlopeType::SlopeRight
                        | TreatAsSlopeType::SlopeLeft => {
                            draw_filled_rect_mut(
                                &mut img,
                                Rect::at((cell.x * CELL_SIZE).into(), (cell.y * CELL_SIZE).into())
                                    .of_size(CELL_SIZE.into(), CELL_SIZE.into()),
                                color,
                            );
                        }
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

        // remove every yellow and magenta pixel which has one transparent horizontal neighboring pixel, repeat 10 times
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

        // crop the image to the correct export size
        // let mut img = image::imageops::crop(
        //     &mut img,
        //     (self.export_rect.x) as u32,
        //     (self.export_rect.y) as u32,
        //     (self.export_rect.width) as u32,
        //     (self.export_rect.height) as u32,
        // )
        // .to_image();

        // draw room outline on borders
        draw_hollow_rect_mut(
            &mut img,
            Rect::at(0, 0).of_size(16 * room_width as u32, 16 * room_height as u32),
            Rgba([0, 255, 0, 255]),
        );

        // check if the folder exists
        let path = std::path::Path::new("./output");
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }

        // make room folder
        let room_hex = format!("{:X}", self.room_id);
        let path = path.join(&room_hex);
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }

        // save image
        img.save(format!(
            "./output/{}/{}_col.png",
            self.room_id, self.room_id
        ))?;

        Ok(())
    }

    pub fn save_breakables(&self) {
        let room_width = self.get_width_cells() as usize;
        let room_height = self.get_height_cells() as usize;

        let breakables = self
            .cells
            .iter()
            .enumerate()
            .filter(|(i, cell)| {
                let shot_not_door = if cell.block_type == BlockType::Shot {
                    let breakable_neighbors = get_4neighbors(*i, room_width, room_height);
                    !breakable_neighbors.has_block_type(&self.cells, BlockType::Door)
                } else {
                    true
                };

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

        let breakable_filename = format!("output/{}/{}_breakables.txt", self.room_id, self.room_id);
        let breakable_path = path::Path::new(&breakable_filename);

        if !breakables.is_empty() {
            let mut file = File::create(breakable_path).unwrap_or_else(|_| {
                panic!(
                    "Failed to create file output/{}/{}_breakables.txt",
                    self.room_id, self.room_id
                )
            });

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
        } else if breakable_path.exists() {
            std::fs::remove_file(breakable_path).unwrap_or_else(|_| {
                panic!(
                    "Failed to delete file output/{}/{}_breakables.txt",
                    self.room_id, self.room_id
                )
            });
        }
    }

    pub fn save_doors(&self) {
        let room_width = self.get_width_cells() as usize;
        let room_height = self.get_height_cells() as usize;

        // spawn_door(xpos, ypos, dir, hatch, troom, targetpos, door_id = -1, angle = 0;)
        let doors_indexes = self
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.block_type == BlockType::Door)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        // let mut disjointed_set = DisjointedSet::new(self.cells.len());
        // let mut doors_rects: Vec<GrowingRect> = Vec::with_capacity(self.cells.len());
        // let mut p = 0;
        // doors_rects.resize_with(self.cells.len(), || {
        //     p += 1;
        //     GrowingRect::with_cell(&self.cells[p - 1])
        // });

        // doors_indexes.iter().for_each(|&i| {
        //     if disjointed_set.find(i) != i {
        //         return;
        //     }

        //     let neighbors = get_4neighbors(i, room_width, room_height);
        //     let nb_doors = neighbors.get_neighbors_of_type(&self.cells, BlockType::Door);

        //     nb_doors.iter().for_each(|&nb_index| {
        //         if disjointed_set.find(i) == disjointed_set.find(nb_index) {
        //             return;
        //         }

        //         // union is determined as cell1 as parent of cell2
        //         // cell1 should be the one with lowest index (because we want always top left)
        //         disjointed_set.union(i, nb_index);
        //         let merged = doors_rects[i].merge(&doors_rects[nb_index]);
        //         //println!("{:?}", merged);
        //         doors_rects[i] = merged;
        //         //println!("{:?}", doors_rects[nb_index]);
        //     })
        // });

        println!("room index: {}", self.room_id);

        // doors_indexes
        //     .iter()
        //     .filter(|&i| disjointed_set.find(*i) == *i)
        //     .for_each(|&i| {
        //         println!("{:?}", doors_rects[i]);
        //     })
    }
}

pub struct CellNeighbors {
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    pub down: Option<usize>,
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

    fn get_neighbors_of_type(&self, cells: &[Cell], block_type: BlockType) -> Vec<usize> {
        let mut neighbors = Vec::new();

        if let Some(left_index) = self.left {
            if cells[left_index].block_type == block_type {
                neighbors.push(left_index);
            }
        }

        if let Some(right_index) = self.right {
            if cells[right_index].block_type == block_type {
                neighbors.push(right_index);
            }
        }
        if let Some(up_index) = self.up {
            if cells[up_index].block_type == block_type {
                neighbors.push(up_index);
            }
        }

        if let Some(down_index) = self.down {
            if cells[down_index].block_type == block_type {
                neighbors.push(down_index);
            }
        }

        neighbors
    }
}

pub fn get_4neighbors(index: usize, room_width: usize, room_height: usize) -> CellNeighbors {
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
