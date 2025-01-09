use std::collections::HashMap;
use std::io::Read;

use constants::CELL_SIZE;
use image::error;
use log::error;
pub mod constants;
pub mod shapes;
mod types;
use anyhow::anyhow;
use std::io::Write;
use types::address::LoRom;
use types::cell::BlockType;
use types::rom::{DoorDirection, DoorHeader, Rom};
use types::room::{self, Room};

fn main() -> Result<(), anyhow::Error> {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    let should_save_images = args.contains(&"-i".to_string());
    let should_save_slopes = args.contains(&"-s".to_string());
    let should_save_breakables = args.contains(&"-b".to_string());
    let should_save_doors = args.contains(&"-d".to_string());

    let mut rooms: Vec<Room> = Vec::new();

    // Retrieve all files in the directory
    let directory_path = std::path::Path::new("./bins");
    if !directory_path.exists() {
        println!(
            "Directory not found: {}",
            std::path::absolute(directory_path).unwrap().display()
        );
        return Err(anyhow!("Directory not found"));
    }

    let rom_path = std::path::PathBuf::from("./rom/rom.bin");
    let rom = Rom::open(rom_path);

    let mut first = true;
    let directory_entries = std::fs::read_dir(directory_path)?;
    for entry in directory_entries {
        let entry = entry?;
        let file_path = entry.path();

        // Check if the file is a .room file
        if !file_path.is_file()
            || file_path.extension().and_then(|ext| ext.to_str()) != Some("room")
        {
            continue;
        }

        // Open the file
        let mut file = std::fs::File::open(&file_path)?;
        let mut file_content = Vec::new();
        if let Err(err) = file.read_to_end(&mut file_content) {
            error!("Failed to read file: {}", err);
            continue;
        }

        // Extract room ID from file name
        let room_id = file_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|name| name.split("_Room_").last())
            .unwrap_or_default();

        let room_id = u32::from_str_radix(room_id, 16).unwrap();

        // println!("Room ID: {:X}", room_id);
        // check if room has events
        let mut curr_event_offset = 11;
        let mut standard1_pointer = 0;
        loop {
            let ind = room_id as usize + curr_event_offset;
            let possible_event = u16::from_le_bytes([rom[ind], rom[ind + 1]]);
            // println!("Room {:X} has event at offset {:X}: {:X}", room_id, ind, possible_event);

            match possible_event {
                0xE5E6 => {
                    standard1_pointer = ind;
                    break;
                }
                0xE612 | 0xE629 => curr_event_offset += 5,
                0xE5FF | 0xE652 | 0xE669 => curr_event_offset += 4,
                _ => {
                    println!("Unknown event of room {:X}: {:X}", room_id, possible_event);
                    break;
                }
            }
        }
        // println!(
        //     "Room {:X} has standard1 pointer at offset {:X}",
        //     room_id,
        //     standard1_pointer - (room_id as usize)
        // );

        if standard1_pointer == 0 {
            println!("Room {} has no standard pointer which is required", room_id);
            continue;
        }

        let level_data_pointer = u32::from_le_bytes([
            rom[standard1_pointer + 2],
            rom[standard1_pointer + 3],
            rom[standard1_pointer + 4],
            0,
        ]);

        // println!("Room {:X} has level data at offset {:X}", room_id, level_data_pointer);

        let level_data_pointer = LoRom::from_hex(level_data_pointer).unwrap().to_pc() as usize;
        let level_data_size = u16::from_le_bytes([
            rom[level_data_pointer + 1 + first as usize],
            rom[level_data_pointer + 2 + first as usize],
        ]);

        // println!("Room {:X} has level data at offset {:X} with size {:X}", room_id, level_data_pointer, level_data_size);

        let room = Room::from_bytes(room_id, level_data_size as usize, &file_content);

        rooms.push(room);
        first = false;
    }

    for room in rooms.iter_mut() {
        if should_save_images {
            //println!("Saving image for: {}", room.room_id);
            room.save_image()?;
        }
        if should_save_slopes {
            //println!("Saving slopes for: {}", room.room_id);
            room.save_slopes();
        }
        if should_save_breakables {
            //println!("Saving breakables for: {}", room.room_id);
            room.save_breakables();
        }
    }

    if should_save_doors {
        save_doors(&rom, rooms);
    }
    Ok(())
}

fn save_doors(rom: &Rom, rooms: Vec<Room>) {
    //println!("Saving doors");
    let mut room_doors_map: HashMap<u32, Vec<(DoorHeader, u32, u32)>> = HashMap::new();
    // hash key is room id
    let mut supsup: HashMap<u32, HashMap<(i32, i32), u32>> = HashMap::new();

    // this variable is used to get the room id from the door header, it's a bit hacky. value is the (room id, index)
    // let mut room_doors_inv_map: HashMap<u32, > = HashMap::new();

    for room in rooms.iter() {
        let mut max_bts = -1;

        room.cells
            .iter()
            .filter(|cell| cell.block_type == BlockType::Shot && (cell.bts & 0x40 != 0))
            .enumerate()
            .for_each(|(_, cell)| {
                let room_id = room.room_id;
                let room_id = room_id & 0xFFFF;

                // find the first door block opposite of the shot block/door_hatch_direction
                // or we reach the room walls
                let door_cap_pos = (cell.x as i32, cell.y as i32);
                let mut pos = door_cap_pos;

                // 0 facing left, 1 facing right, 2 facing up, 3 facing down
                loop {
                    let (dx, dy) = match cell.bts & 0x3 {
                        0 => (1, 0),
                        1 => (-1, 0),
                        2 => (0, 1),
                        3 => (0, -1),
                        _ => unreachable!("Invalid door direction"),
                    };
                    pos.0 += dx;
                    pos.1 += dy;
                    if pos.0 < 0
                        || pos.0 >= room.get_room_width_tiles() as i32
                        || pos.1 < 0
                        || pos.1 >= room.get_room_height_tiles() as i32
                    {
                        break;
                    }

                    let index =
                        pos.0 as usize + room.get_room_width_tiles() as usize * pos.1 as usize;
                    let cell = &room.cells[index];
                    if cell.block_type == BlockType::Door {
                        let door_bts = cell.bts;
                        if door_bts as i8 > max_bts {
                            max_bts = door_bts as i8;
                        }

                        let entry = supsup
                            .entry(room_id)
                            .or_default()
                            .entry(door_cap_pos)
                            .or_default();
                        *entry = door_bts as u32;
                        break;
                    }
                }
            });

        if max_bts == -1 {
            continue;
        }

        let room_id = room.room_id & 0xFFFF;
        room_doors_map
            .entry(room_id)
            .or_default()
            .resize((max_bts + 1) as usize, (DoorHeader::default(), 0, 0));
    }

    for room in rooms.iter() {
        let lorom_hex = 0x8F0000 + (room.door_out_pointer as u32);

        let dop_pc = LoRom::from_hex(lorom_hex).unwrap();

        let dop_pc = dop_pc.to_pc() as usize;
        //println!("door1 maybe: {:X}{:X}", rom[dop_pc], rom[dop_pc + 1]);
        let mut door_headers = Vec::new();

        let mut it = 0;
        loop {
            let current_pc = dop_pc + it * 2;
            let door_addrs = u16::from_le_bytes([rom[current_pc], rom[current_pc + 1]]);
            if door_addrs < 0x8000 {
                break;
            }
            let door_hex = 0x830000 + door_addrs as u32;
            let door_header_pc = LoRom::from_hex(door_hex).unwrap().to_pc() as usize;

            let door_header = DoorHeader::from_bytes(&rom[door_header_pc..door_header_pc + 12]);
            door_headers.push(door_header);

            it += 1;
        }

        door_headers
            .iter()
            .filter(|door_header| door_header.room_id != 0)
            .enumerate()
            .for_each(|(i, door_header)| {
                if door_header.room_id == 0 {
                    return;
                }

                let rd_map_vec = room_doors_map
                    .entry(door_header.room_id as u32)
                    .or_default();

                let room_id = room.room_id & 0xFFFF;

                let pos = (door_header.door_cap_x as i32, door_header.door_cap_y as i32);
                if pos == (0, 0) {
                    return;
                }

                let entry_op = supsup
                    .entry(door_header.room_id as u32)
                    .or_default()
                    .get(&pos);

                let entry = if let Some(entry_op) = entry_op {
                    entry_op
                } else {
                    // println!(
                    //     "door_header.room_id: {:X}, entry_op: None, pos: {:?}",
                    //     door_header.room_id, pos
                    // );
                    return;
                };

                rd_map_vec[*entry as usize] = (*door_header, room_id, i as u32);
            });
    }

    for (addrs, vec_door_headers) in &room_doors_map {
        let res_room_id = LoRom::from_hex(0x8F0000 + *addrs);
        let room_id = if let Ok(room_id) = res_room_id {
            room_id.to_pc()
        } else {
            continue;
        };
        let file_door_path =
            std::path::PathBuf::from(format!("output/{:X}/{:X}_doors.txt", room_id, room_id));

        let mut file = std::fs::File::create(file_door_path).unwrap();
        for entry_dh in vec_door_headers {
            let (door_header, from_room_id, from_door_index) = entry_dh;
            let door_dir = door_header.get_door_direction();
            let dir: i32 = match door_dir {
                DoorDirection::Right => 1,
                DoorDirection::Left => -1,
                DoorDirection::Up => 1,
                DoorDirection::Down => -1,
            };
            let hatch = if door_header.should_close_behind() {
                3
            } else {
                7
            };
            let troom = format!("0x{:X}", from_room_id);

            let mut targetpos_offset = door_header.distance_to_spawn;
            if targetpos_offset == 0x8000 {
                targetpos_offset = 0x0080;
            }
            targetpos_offset >>= 1;

            let acs_room_vec = if let Some(acs_room_vec) = room_doors_map.get(from_room_id) {
                acs_room_vec
            } else {
                //println!("acs_room not found for room id: {:X}", from_room_id);
                continue;
            };
            let acs_door_header =
                if let Some(acs_door_header) = acs_room_vec.get(*from_door_index as usize) {
                    acs_door_header.0
                } else {
                    println!(
                        "acs_door_header not found for room id: {:X} and index {}",
                        from_room_id, from_door_index
                    );
                    continue;
                };

            let mut pos = (door_header.door_cap_x as i32, door_header.door_cap_y as i32);
            if door_dir == DoorDirection::Left {
                pos.0 += 1;
            }
            if door_dir == DoorDirection::Up {
                pos.1 += 1;
            }

            let targetpos = if acs_door_header.door_cap_x == 0 && acs_door_header.door_cap_y == 0 {
                (0, 0)
            } else {
                let temp_pos = (
                    acs_door_header.door_cap_x as i32,
                    acs_door_header.door_cap_y as i32,
                );
                match door_dir {
                    DoorDirection::Right | DoorDirection::Left => (
                        temp_pos.0 * CELL_SIZE as i32 + targetpos_offset as i32,
                        (temp_pos.1 + 4) * CELL_SIZE as i32,
                    ),
                    DoorDirection::Up | DoorDirection::Down => (
                        (temp_pos.0 + 4) * CELL_SIZE as i32,
                        temp_pos.1 * CELL_SIZE as i32 + targetpos_offset as i32,
                    ),
                }
            };

            let door_id = -1;
            let angle = match door_dir {
                DoorDirection::Right | DoorDirection::Left => 0,
                DoorDirection::Up | DoorDirection::Down => 90,
            };
            let angle_str = if angle == 0 {
                String::new()
            } else {
                format!(", {angle}")
            };
            let id_str = if door_id == -1 {
                String::new()
            } else {
                format!(", {door_id}")
            };

            let target_pos_str = if targetpos.0 == 0 && targetpos.1 == 0 {
                "[failed]".to_string()
            } else {
                format!("[{}, {}]", targetpos.0, targetpos.1)
            };
            let spawn_door = format!("spawn_door({}*2, {}*2, {dir}, {hatch}, {troom}, {target_pos_str}{id_str}{angle_str});\n", 
                                                (pos.0 as u16)*CELL_SIZE,
                                                (pos.1 as u16)*CELL_SIZE);

            file.write_all(spawn_door.as_bytes()).unwrap();
        }
    }
}
