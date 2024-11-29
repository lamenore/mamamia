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
        cell::{BlockType, Cell, SlopeType, TreatAsSlopeType},
        disjointed_set::DisjointedSet,
        Flip,
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
struct RoomExportSize {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl RoomExportSize {
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

#[derive(Default)]
#[allow(dead_code)]
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
    export_size: RoomExportSize,
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
                let slope_flip = room.cells[i].get_slope_flip();
                let vectors = SlopeVectors::from(slope_type);
                room.cells[i].slope_vectors.extend_from_slice(vectors);

                match slope_flip {
                    Flip::None => {}
                    Flip::Horizontal => {
                        for vector in &mut room.cells[i].slope_vectors {
                            vector.start.x = (-vector.start.x) + CELL_SIZE as i32 - 1;
                            vector.end.x = (-vector.end.x) + CELL_SIZE as i32 - 1;
                        }
                    }
                    Flip::Vertical => {
                        for vector in &mut room.cells[i].slope_vectors {
                            vector.start.y = (-vector.start.y) + CELL_SIZE as i32 - 1;
                            vector.end.y = (-vector.end.y) + CELL_SIZE as i32 - 1;
                        }
                    }
                    Flip::Both => {
                        for vector in &mut room.cells[i].slope_vectors {
                            vector.start.x = (-vector.start.x) + CELL_SIZE as i32 - 1;
                            vector.end.x = (-vector.end.x) + CELL_SIZE as i32 - 1;
                            vector.start.y = (-vector.start.y) + CELL_SIZE as i32 - 1;
                            vector.end.y = (-vector.end.y) + CELL_SIZE as i32 - 1;
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
            export_size: RoomExportSize::new(),
        }
    }

    pub fn crop_room_export_size(&mut self) {
        let room_width = self.get_room_width_tiles();
        let room_height = self.get_room_height_tiles();

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

        self.export_size.update(x, y, width, height);
    }

    pub fn get_room_width_tiles(&self) -> u16 {
        (self.room_width * TILE_SIZE as u8).into()
    }

    pub fn get_room_height_tiles(&self) -> u16 {
        (self.room_height * TILE_SIZE as u8).into()
    }

    fn set_treat_slope(&mut self) {
        let room_width = self.get_room_width_tiles() as usize;
        let room_height = self.get_room_height_tiles() as usize;

        for i in 0..(room_width * room_height) {
            if self.cells[i].block_type != BlockType::Slope
                || self.cells[i].get_slope_type() == SlopeType::Square
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

            if let Some(right) = neighbors.right {
                if self.cells[right].is_square() {
                    self.cells[right].treat_as_slope = TreatAsSlopeType::SlopeLeft;
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
    }

    pub fn save_slopes(&mut self) {
        let room_width = self.get_room_width_tiles() as usize;
        let room_height = self.get_room_height_tiles() as usize;

        let mut disjointed_set = DisjointedSet::new(room_width * room_height);
        let mut has_apparent_slope = false;

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

                        let tp = disjointed_set.find(test_index, k);

                        let cv = self.cells[cell_parent.cell].slope_vectors[cell_parent.vector];
                        let tv = self.cells[tp.cell].slope_vectors[tp.vector];

                        if !cv.compare_direction(&tv) {
                            continue;
                        }
                        let ends_meet = cv.end_points_meet(&tv);

                        let mut new_parent = cell_parent;
                        if ends_meet != 0 {
                            new_parent = disjointed_set.union(cell_i, vector_i, test_index, k);
                        }
                        if ends_meet == -1 {
                            if new_parent == cell_parent {
                                self.cells[cell_parent.cell].slope_vectors[cell_parent.vector]
                                    .end = tv.end;
                            } else {
                                self.cells[new_parent.cell].slope_vectors[new_parent.vector]
                                    .start = cv.start;
                            }
                        }
                        if ends_meet == 1 {
                            if new_parent == cell_parent {
                                self.cells[cell_parent.cell].slope_vectors[cell_parent.vector]
                                    .start = tv.start;
                            } else {
                                self.cells[new_parent.cell].slope_vectors[new_parent.vector].end =
                                    cv.end;
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

        let slope_filename = format!("output/{}/{}_slopes.txt", self.room_id, self.room_id);
        let slope_path = path::Path::new(&slope_filename);
        if has_apparent_slope {
            let mut file = File::create(slope_path).unwrap_or_else(|_| {
                panic!(
                    "Failed to create file output/{}/{}_slopes.txt",
                    self.room_id, self.room_id
                )
            });

            for (i, _) in slope_info {
                for vector_i in 0..self.cells[i].slope_vectors.len() {
                    let start = self.cells[i].slope_vectors[vector_i].start;
                    let end = self.cells[i].slope_vectors[vector_i].end;

                    // Map start and end to the export size space
                    let mapped_start_x = start.x - self.export_size.x as i32;
                    let mapped_start_y = start.y - self.export_size.y as i32;
                    let mapped_end_x = end.x - self.export_size.x as i32;
                    let mapped_end_y = end.y - self.export_size.y as i32;

                    file.write_all(
                        format!(
                            "spawn_slope({}*2, {}*2, {}*2, {}*2)\n",
                            mapped_start_x, mapped_start_y, mapped_end_x, mapped_end_y
                        )
                        .as_bytes(),
                    )
                    .unwrap_or_else(|_| {
                        panic!(
                            "Failed to write to file output/{}/{}_slopes.txt",
                            self.room_id, self.room_id
                        )
                    });
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
        let room_width = self.get_room_width_tiles() as usize;
        let room_height = self.get_room_height_tiles() as usize;

        // let img_width = col_end - col_start;
        // let img_height = row_end - row_start;

        let mut img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::new(
            (room_width * CELL_SIZE as usize) as u32,
            (room_height * CELL_SIZE as usize) as u32,
        );

        // draw the rest of the cells
        self.cells
            .iter()
            .filter(|cell| {
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

        // crop the image to the correct export size
        let mut img = image::imageops::crop(
            &mut img,
            (self.export_size.x) as u32,
            (self.export_size.y) as u32,
            (self.export_size.width) as u32,
            (self.export_size.height) as u32,
        )
        .to_image();

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
        let path = path.join(&self.room_id);
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
        let room_width = self.get_room_width_tiles() as usize;
        let room_height = self.get_room_height_tiles() as usize;

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
                let pos = (pos.0 - self.export_size.x, pos.1 - self.export_size.x);
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
