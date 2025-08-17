pub mod constants;
pub mod shapes;
pub mod traits;

use std::collections::HashMap;
use std::io::Read;

use constants::CELL_SIZE;
mod types;
use std::io::Write;
use types::address::LoRom;
use types::cell::BlockType;
use types::rom::{DoorDirection, DoorHeader, Rom};
use types::room::Room;

use crate::constants::TILE_SIZE;

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
        return Err(anyhow::anyhow!("Directory not found"));
    }

    let rom_path = std::path::PathBuf::from("./rom/rom.bin");
    let rom = Rom::open(rom_path);

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
            eprintln!("Failed to read file: {}", err);
            continue;
        }

        // Extract room ID from file name
        let room_id_str = file_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|name| name.split("_Room_").last())
            .unwrap_or_default();

        let room_id = u32::from_str_radix(room_id_str, 16).unwrap();

        // println!("Room ID: {:X}", room_id);
        // check if room has events
        let mut curr_event_offset = 11;
        let mut standard1_pointer = 0;
        loop {
            let ind = room_id as usize + curr_event_offset;
            if ind >= rom.len() {
                break;
            }

            let possible_event = u16::from_le_bytes([rom[ind], rom[ind + 1]]);
            // println!(
            //     "Room {:X} has event at offset {:X}: {:X}",
            //     room_id, ind, possible_event
            // );

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

        // println!(
        //     "Room {:X} has level data at offset {:X}",
        //     room_id, level_data_pointer
        // );

        let level_data_pointer = LoRom::from_hex(level_data_pointer).unwrap().to_pc().0 as usize;
        let level_data_size = u16::from_le_bytes([
            rom[level_data_pointer + 1 as usize],
            rom[level_data_pointer + 2 as usize],
        ]);

        // println!(
        //     "Room {:X} has level data at offset {:X} with size {:X}",
        //     room_id, level_data_pointer, level_data_size
        // );

        let room = Room::from_bytes(room_id, level_data_size as usize, &file_content);

        rooms.push(room);
    }

    rooms.iter().for_each(|room| {
        if should_save_images {
            // println!("Saving image for: {}", room.room_id);
            if let Err(err) = room.save_image() {
                eprintln!("Error saving image for room {:X}: {}", room.room_id, err);
            }
        }
        if should_save_slopes {
            // println!("Saving slopes for: {:X}", room.room_id);
            if let Err(err) = room.save_slopes() {
                eprintln!("Error saving slopes for room {:X}: {}", room.room_id, err);
            }
        }
        if should_save_breakables {
            //println!("Saving breakables for: {}", room.room_id);
            room.save_breakables();
        }
    });

    if should_save_doors {
        save_doors(&rom, rooms)?;
    }
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct DoorStuff {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

fn save_doors(rom: &Rom, rooms: Vec<Room>) -> Result<(), anyhow::Error> {
    let mut door_pos: HashMap<(u32, u8), DoorStuff> = HashMap::new(); // (room_id, bts)
    let mut door_cap_link_to_door: HashMap<(u32, u8), u16> = HashMap::new(); // (room_id, door_bts), door_cap_index
    let mut door_output_info: HashMap<u32, Vec<DoorOutputInfo>> = HashMap::new();
    rooms.iter().for_each(|room| {
        let room_id = room.room_id;

        room.cells
            .iter()
            .filter(|cell| cell.block_type == BlockType::Door && cell.extra != 0xFF)
            .for_each(|cell| {
                let key = (room_id, cell.bts); // Initial position of the door cap

                door_pos
                    .entry(key)
                    .and_modify(|door| {
                        // As we are walking the cells linearly and the cells are positioned on a grid
                        // we can check for neighbours to the left and top, modify the width and height
                        // accordingly, positive y is down and positive x is right

                        if cell.x > door.x {
                            door.width += 1;
                        }
                        if cell.y > door.y {
                            door.height += 1;
                        }

                        if room_id == 0x791F8 {
                            println!(
                                "Bts: {} Door: x: {:X}, y: {:X}, width: {}, height: {}",
                                cell.bts, door.x, door.y, door.width, door.height
                            );

                            // println!("Cell: x: {:X}, y: {:X}", cell.x, cell.y);
                        }
                    })
                    .or_insert(DoorStuff {
                        x: cell.x,
                        y: cell.y,
                        width: 1,
                        height: 1,
                    });
            });

        room.cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.is_blue_door_cap())
            .for_each(|(i, cell)| {
                // link door caps to door blocks
                // walk to the opposite direction of flip

                // doors face left by default (flip none)
                let flip = cell.bts & 0b11;
                let dir_walking = match flip {
                    0 => (1, 0),
                    1 => (-1, 0),
                    2 => (0, 1),
                    3 => (0, -1),
                    _ => unreachable!("Invalid door direction"),
                };
                let mut x = cell.x as i32;
                let mut y = cell.y as i32;
                loop {
                    x = x + dir_walking.0;
                    y = y + dir_walking.1;

                    if x < 0
                        || y < 0
                        || x >= room.get_width_cells() as i32
                        || y >= room.get_height_cells() as i32
                    {
                        eprintln!(
                            "Room {:X} Door cap at {:X}:{:X} has no matching door block",
                            room.room_id, cell.x, cell.y
                        );
                        break;
                    }

                    let x = x as u16;
                    let y = y as u16;

                    let test_cell = room.get_cell(x, y);
                    if test_cell.block_type == BlockType::Door {
                        let key = (room_id, test_cell.bts);
                        door_cap_link_to_door.insert(key, i as u16);
                        break;
                    }
                }
            });
    });

    rooms.iter().for_each(|room| {
        let lorom_hex = 0x8F0000 + (room.door_out_pointer as u32); // Compute LoRom hex address
        let dop_pc = match LoRom::from_hex(lorom_hex) {
            Err(e) => {
                eprintln!(
                    "{}\nRoom {} has no valid door out pointer: {:X}",
                    e, room.room_id, room.door_out_pointer
                );
                return;
            }
            Ok(dop_pc) => dop_pc.to_pc(),
        };

        let mut door_headers: Vec<Option<DoorHeader>> = Vec::with_capacity(4); // pessimistic capacity
        let mut it = 0;
        loop {
            let current_pc = (dop_pc.0 + it * 2) as usize;
            if let [byte1, byte2] = &rom[current_pc..current_pc + 2] {
                let door_addrs = u16::from_le_bytes([*byte1, *byte2]);
                if door_addrs < 0x8000 {
                    break; // End of door addresses
                }
                let door_hex = 0x830000 + door_addrs as u32; // door address are located at bank 0x83
                let dh_pc = match LoRom::from_hex(door_hex) {
                    Err(e) => {
                        eprintln!("{}\nRoom {:X} has no door header", e, room.room_id);
                        break;
                    }
                    Ok(dh_pc) => dh_pc.to_pc(),
                };

                let dh_usize = dh_pc.0 as usize;
                let dh_array = &rom[dh_usize..dh_usize + DoorHeader::SIZE];
                if dh_array[0..2] == [0x00, 0x00] {
                    // println!("Elevator (start) in room {:X}", room.room_id);
                    it += 1;
                    continue;
                }

                // Parse the door header and add it to the list
                let door_header = DoorHeader::from_bytes(dh_array);

                if door_header.is_elevator() {
                    // println!("Elevator in room {:X}", room.room_id);
                    it += 1;
                    continue;
                }
                if door_header.door_cap_x == 0 && door_header.door_cap_y == 0 {
                    // eprintln!("Room {:X} door {it} has door cap at (0,0)", room.room_id);
                    // it += 1;
                    // continue;
                }
                let bts = it as u8;
                if !door_pos.contains_key(&(room.room_id, bts)) {
                    eprintln!("Room {:X} has no door with bts {}", room.room_id, bts);
                    it += 1;
                    continue;
                }

                // Add the door header to the door_headers vector on the bts index,
                // check for length and reserve
                if door_headers.len() <= bts as usize {
                    door_headers.resize(bts as usize + 1, None);
                }

                door_headers[bts as usize] = Some(door_header);
            }
            it += 1;
        }

        door_headers
            .iter()
            .enumerate()
            .filter(|(_, dh)| dh.is_some())
            .for_each(|(bts, dh)| {
                let dh = dh.unwrap();
                let bts = bts as u8;
                let door_size = match door_pos.get(&(room.room_id, bts)) {
                    None => {
                        eprintln!("Room {:X} has no door with bts {}", room.room_id, bts);
                        return;
                    }
                    Some(door_size) => door_size,
                };
                let mut output = DoorOutputInfo::new();

                let linked_door_cap_idx = door_cap_link_to_door.get(&(room.room_id, bts));
                output.hatch = if let None = linked_door_cap_idx {
                    println!("Room {:X} bts {bts} has no door cap", room.room_id);
                    9
                } else {
                    3
                };
                output.troom = 0x70000 + dh.troom_id as u32;
                output.room_id = room.room_id;
                output.bts = bts;

                let dir = match dh.get_door_direction() {
                    DoorDirection::Right => -1,
                    DoorDirection::Left => 1,
                    DoorDirection::Up => -1,
                    DoorDirection::Down => 1,
                };
                output.dir = dir;

                let angle = match dh.get_door_direction() {
                    DoorDirection::Right | DoorDirection::Left => 0,
                    DoorDirection::Up | DoorDirection::Down => 90,
                };
                output.angle = if angle == 0 { None } else { Some(angle) };

                let (door_x, door_y) = match dh.get_door_direction() {
                    DoorDirection::Right => (door_size.x, door_size.y),
                    DoorDirection::Left => (door_size.x + 1, door_size.y),
                    DoorDirection::Up => (door_size.x, door_size.y),
                    DoorDirection::Down => (door_size.x + 1, door_size.y + 1),
                };

                output.xpos = door_x * CELL_SIZE as u16;
                output.ypos = door_y * CELL_SIZE as u16;

                // Distance from door to door cap in 8 units (each cell is 16 units) increments multiplier
                let distance = match linked_door_cap_idx {
                    None => None,
                    Some(idx) => {
                        // from room index to x and y
                        let door_cap_cell = &room.cells[*idx as usize];
                        let distance = match dh.get_door_direction() {
                            DoorDirection::Right | DoorDirection::Left => {
                                door_size.x as i32 - door_cap_cell.x as i32
                            }
                            DoorDirection::Up | DoorDirection::Down => {
                                door_size.y as i32 - door_cap_cell.y as i32
                            }
                        };

                        let mut dabs = distance.abs() as u8;
                        dabs = ((dabs - 1) * 2) + 1;

                        Some(dabs)
                    }
                };
                output.distance = distance;

                // if let Some(dst) = distance
                //     && dst != 1
                //     && dst != 3
                // {
                //     // debug
                //     println!(
                //         "Room {:X} door {} has distance {:X}",
                //         room.room_id, bts, dst
                //     );
                // }

                output.transition_hdt = match door_size.height {
                    4 => None,
                    _ => Some(door_size.height as i8),
                };

                output.transition_wdt = match door_size.width {
                    1 => None,
                    _ => Some(door_size.width as i8),
                };

                let mut target_pos = (
                    dh.screen_x as i32 * (TILE_SIZE as i32),
                    dh.screen_y as i32 * (TILE_SIZE as i32),
                );

                let mut targetpos_offset = dh.distance_to_spawn;
                if targetpos_offset == 0x8000 {
                    targetpos_offset = 0x0080;
                }
                targetpos_offset /= 4;
                let targetpos_offset = (targetpos_offset / 16) as i32;

                match dh.get_door_direction() {
                    DoorDirection::Right => {
                        target_pos.0 = target_pos.0 + targetpos_offset;
                        target_pos.1 = target_pos.1 + 10;
                    }
                    DoorDirection::Left => {
                        target_pos.0 = (target_pos.0 + (TILE_SIZE as i32) - 1) - targetpos_offset;
                        target_pos.1 = target_pos.1 + 10;
                    }
                    DoorDirection::Up => {
                        target_pos.0 = target_pos.0 + (TILE_SIZE as i32 / 2);
                        target_pos.1 = target_pos.1 + (TILE_SIZE as i32) - targetpos_offset;
                    }
                    DoorDirection::Down => {
                        target_pos.0 = target_pos.0 + (TILE_SIZE as i32 / 2);
                        target_pos.1 = target_pos.1 + targetpos_offset + 1 as i32;
                    }
                };
                target_pos = (
                    target_pos.0 * CELL_SIZE as i32,
                    target_pos.1 * CELL_SIZE as i32,
                );

                // if room.room_id == 0x791F8 {
                //     println!(
                //         "Target pos: ({}, {}) for room {:X} and bts {} leading to room 7{:X}, distance {}",
                //         target_pos.0, target_pos.1, room.room_id, bts, dh.troom_id, distance.unwrap_or(1)
                //     );
                // }
                output.target_pos = [target_pos.0 as i32, target_pos.1 as i32];

                // door id left default

                door_output_info
                    .entry(room.room_id)
                    .and_modify(|v| {
                        v.push(output.clone());
                    })
                    .or_insert(vec![output.clone()]);
            });
    });

    for (room_id, v) in door_output_info {
        let file_door_path =
            std::path::PathBuf::from(format!("output/{:X}/{:X}_doors.txt", room_id, room_id));

        // create folders if they don't exist
        if !file_door_path.parent().unwrap().exists() {
            std::fs::create_dir_all(file_door_path.parent().unwrap()).unwrap();
        }

        let mut file = std::fs::File::create(file_door_path)?;
        for door_info in v {
            let mut line = format!(
                "{}, {}*2, {}*2, {}, {:X}, [{}*2,{}*2]",
                door_info.hatch,
                door_info.xpos,
                door_info.ypos,
                door_info.dir,
                door_info.troom,
                door_info.target_pos[0],
                door_info.target_pos[1],
            );
            // option variables are optional arguments in the output, so if they are not set, they are not added to the output
            // however if one is set, every preceding option must be set
            // Order matters — list optional args here
            let optional_args: Vec<(Option<String>, String)> = vec![
                (
                    door_info.distance.map(|d| {
                        if d > 5 {
                            format!("[door failed, probable cause: overlapping rooms]",)
                        } else {
                            d.to_string()
                        }
                    }),
                    "1".to_string(),
                ),
                (door_info.door_id.map(|d| d.to_string()), "-1".to_string()),
                (door_info.angle.map(|a| a.to_string()), "0".to_string()),
                (
                    door_info.transition_wdt.map(|w| w.to_string()),
                    "-1".to_string(),
                ),
                (
                    door_info.transition_hdt.map(|h| h.to_string()),
                    "4".to_string(),
                ),
            ];

            // Find the last set optional argument
            if let Some(last) = optional_args.iter().rposition(|(opt, _)| opt.is_some()) {
                for (opt, default) in optional_args.iter().take(last + 1) {
                    match opt {
                        Some(val) => line.push_str(&format!(", {}", val)),
                        None => line.push_str(&format!(", {}", default)),
                    }
                }
            }

            let func_str = format!("spawn_door({});\n", line);
            file.write_all(func_str.as_bytes())?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct DoorOutputInfo {
    room_id: u32,
    bts: u8,
    hatch: u8,
    xpos: u16,
    ypos: u16,
    dir: i8,
    troom: u32,
    target_pos: [i32; 2],
    distance: Option<u8>,
    door_id: Option<i32>,
    angle: Option<u8>,
    transition_wdt: Option<i8>,
    transition_hdt: Option<i8>,
}

impl DoorOutputInfo {
    fn new() -> Self {
        Self {
            room_id: 0,
            hatch: 3,
            dir: 0,
            bts: 0,
            xpos: 0,
            ypos: 0,
            troom: 0,
            target_pos: [0, 0],
            distance: None,
            door_id: None,
            angle: None,
            transition_wdt: None,
            transition_hdt: None,
        }
    }
}

// sanity check first Unused, for later
// let's check if every door cap has a neighbor hcopy to the right
// print both cases
// for i in 0..room.cells.len() {
//     let cell = &room.cells[i];
//     if cell.is_blue_door_cap() && (cell.bts == 0x40 || cell.bts == 0x41) {
//         let pos = (cell.x, cell.y);
//         let mut found = false;
//         let offsets = [(1, 0), (-1, 0), (0, 1), (0, -1)];
//         for offset in offsets {
//             let neighbor_pos = (pos.0 as i32 + offset.0, pos.1 as i32 + offset.1);
//             if neighbor_pos.0 < 0
//                 || neighbor_pos.1 < 0
//                 || neighbor_pos.0 as usize >= room.get_room_width_tiles() as usize
//                 || neighbor_pos.1 as usize >= room.get_room_height_tiles() as usize
//             {
//                 continue;
//             }
//             let neighbor_cell = room.get_cell(neighbor_pos.0 as u16, neighbor_pos.1 as u16);
//             if neighbor_cell.block_type == BlockType::VCopy && neighbor_cell.bts & 0xF0 != 0
//             {
//                 match offset {
//                     (1, 0) => {
//                         found = true;
//                         println!("Room {:X} has neighbor to the right", room.room_id);
//                     }
//                     (-1, 0) => {
//                         found = true;
//                         println!("Room {:X} has neighbor to the left", room.room_id);
//                     }
//                     (0, 1) => {
//                         found = true;
//                         println!("Room {:X} has neighbor above", room.room_id);
//                     }
//                     (0, -1) => {
//                         found = true;
//                         println!("Room {:X} has neighbor below", room.room_id);
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         if !found {
//             println!("Room {} has no neighbor to the right", room.room_id);
//         }
//     }
// }
